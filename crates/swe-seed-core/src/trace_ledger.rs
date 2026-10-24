//! Tamper-evident trace ledger (Phase A of the routing-enforcement design).
//!
//! A hash-chained append-only log in SQLite. Each trace's events chain:
//!   `entry.hash = SHA256(entry.event_type || entry.payload_json || entry.prev_hash)`
//! The **genesis entry (seq 0) is the route-decision** — so the fact that a
//! task was routed is recorded in a tamper-evident chain by construction.
//!
//! Properties:
//! - **Tamper-evidence (partial edit):** altering any entry's payload breaks
//!   the hash of every later entry; `verify_chain` detects it.
//! - **Truncation (head anchor):** every append also writes a per-trace head
//!   anchor (`{length, head}`) to a sidecar file next to the SQLite ledger
//!   (`ledger.anchors.json`). Verification compares the chain's entry count and
//!   head hash to that anchor, so deleting trailing rows is detected. A trace
//!   with no anchor is reported as `unanchored` and fails closed unless the
//!   caller explicitly opts in with [`AnchorPolicy::AllowUnanchored`].
//! - **Chain root** (`chain_root`): the head hash — a single value that covers
//!   the whole chain. This is the value that will be Ed25519-signed in Phase B
//!   and carried in the SEA envelope's `trace_chain_root`.
//!
//! Threat-model limits of the anchor: the anchor is NOT stored in the SQLite
//! file and is not signed. Anyone with write access to the `.agent-harness/
//! traces/` directory can rewrite the chain rows and the anchor together, or
//! delete the anchor file (which downgrades the trace to `unanchored`). The
//! anchor therefore only defeats truncation by a writer who edits the rows but
//! not the sidecar. Full tamper-resistance arrives in Phase B when the chain
//! root is signed with a key the writer cannot forge, and the signed head is
//! kept outside the writer's reach.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ulid::Ulid;

/// The zero hash that seeds the genesis entry's `prev_hash`.
pub const GENESIS_PREV: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Default ledger location.
pub fn default_db_path(root: &Path) -> std::path::PathBuf {
    root.join(".agent-harness")
        .join("traces")
        .join("ledger.sqlite3")
}

fn hash_of(event_type: &str, payload_json: &str, prev_hash: &str) -> String {
    let mut h = Sha256::new();
    h.update(event_type.as_bytes());
    h.update(b"\n");
    h.update(payload_json.as_bytes());
    h.update(b"\n");
    h.update(prev_hash.as_bytes());
    format!("{:x}", h.finalize())
}

/// Open (or create) the ledger DB at `path`, ensuring the schema exists.
pub fn open(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(path).with_context(|| format!("open ledger {}", path.display()))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS chain_entries (
            trace_id    TEXT NOT NULL,
            seq         INTEGER NOT NULL,
            event_id    TEXT NOT NULL,
            event_type  TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            prev_hash   TEXT NOT NULL,
            hash        TEXT NOT NULL,
            occurred_at TEXT NOT NULL,
            PRIMARY KEY (trace_id, seq)
        );
        CREATE INDEX IF NOT EXISTS idx_chain_entries_trace ON chain_entries(trace_id, seq);",
    )?;
    Ok(conn)
}

/// A ledger entry to append.
#[derive(Debug, Clone)]
pub struct LedgerEntry<'a> {
    pub trace_id: &'a str,
    pub event_type: &'a str,
    pub payload_json: &'a str,
    pub occurred_at: &'a str,
}

/// The result of appending an entry: its sequence number, event id, and hash.
#[derive(Debug, Clone)]
pub struct Appended {
    pub seq: i64,
    pub event_id: String,
    pub hash: String,
}

/// How verification treats a trace that has no head anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorPolicy {
    /// Fail closed: an unanchored trace cannot prove it was not truncated.
    Required,
    /// Explicit legacy opt-in: accept an unanchored trace (structure only).
    /// A trace that *has* an anchor is still checked against it.
    AllowUnanchored,
}

/// The expected state of one trace's chain, written on every append.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadAnchor {
    /// Number of chain entries the writer last committed.
    pub length: i64,
    /// Hash of the last committed entry.
    pub head: String,
}

/// Sidecar anchor file for a ledger DB: `ledger.sqlite3` -> `ledger.anchors.json`.
/// `None` for connections without a file path (in-memory).
fn anchor_path(conn: &Connection) -> Option<PathBuf> {
    conn.path()
        .filter(|p| !p.is_empty())
        .map(|p| Path::new(p).with_extension("anchors.json"))
}

fn read_anchors(path: &Path) -> Result<BTreeMap<String, HeadAnchor>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read head anchors {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("parse head anchors {} (fail closed)", path.display()))
}

fn write_anchors(path: &Path, anchors: &BTreeMap<String, HeadAnchor>) -> Result<()> {
    let tmp = path.with_extension("anchors.json.tmp");
    let text = serde_json::to_string_pretty(anchors)?;
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp)
            .with_context(|| format!("write head anchors {}", tmp.display()))?;
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)
        .with_context(|| format!("replace head anchors {}", path.display()))?;
    Ok(())
}

/// Record the new head for `trace_id`. Runs inside the append transaction, so
/// the anchor and the row commit or roll back together (modulo a crash between
/// the anchor write and COMMIT, which surfaces later as a head mismatch).
///
/// A trace that already has rows but no anchor (a legacy ledger) is left
/// unanchored: appending must not silently vouch for rows written before the
/// anchor existed. Migrating such a trace requires an explicit operation.
fn update_anchor(conn: &Connection, trace_id: &str, seq: i64, hash: &str) -> Result<()> {
    let Some(path) = anchor_path(conn) else {
        return Ok(());
    };
    let mut anchors = read_anchors(&path)?;
    if seq == 0 {
        if anchors.contains_key(trace_id) {
            return Err(anyhow!(
                "trace '{trace_id}' has a head anchor but no chain entries (truncated to empty)"
            ));
        }
    } else if !anchors.contains_key(trace_id) {
        return Ok(());
    }
    anchors.insert(
        trace_id.to_string(),
        HeadAnchor {
            length: seq + 1,
            head: hash.to_string(),
        },
    );
    write_anchors(&path, &anchors)
}

/// Append an entry to a trace's chain. The first append for a `trace_id`
/// becomes the genesis (seq 0, prev_hash = GENESIS_PREV). A fresh ULID is
/// minted for `event_id`.
pub fn append(conn: &Connection, entry: LedgerEntry<'_>) -> Result<Appended> {
    let event_id = Ulid::new().to_string();
    conn.execute_batch("BEGIN IMMEDIATE TRANSACTION")?;
    let mut stmt = conn.prepare(
        "SELECT seq, hash FROM chain_entries WHERE trace_id = ?1 ORDER BY seq DESC LIMIT 1",
    )?;
    let (max_seq, prev_hash): (i64, String) = match stmt.query_row(params![entry.trace_id], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
    }) {
        Ok(row) => row,
        Err(rusqlite::Error::QueryReturnedNoRows) => (-1, GENESIS_PREV.to_string()),
        Err(e) => {
            drop(stmt);
            conn.execute_batch("ROLLBACK").ok();
            return Err(e.into());
        }
    };
    drop(stmt);
    let seq = max_seq + 1;
    let prev = if seq == 0 {
        GENESIS_PREV.to_string()
    } else {
        prev_hash
    };
    let hash = hash_of(entry.event_type, entry.payload_json, &prev);
    if let Err(e) = conn.execute(
        "INSERT INTO chain_entries (trace_id, seq, event_id, event_type, payload_json, prev_hash, hash, occurred_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            entry.trace_id,
            seq,
            event_id,
            entry.event_type,
            entry.payload_json,
            prev,
            hash,
            entry.occurred_at,
        ],
    ) {
        conn.execute_batch("ROLLBACK").ok();
        return Err(e.into());
    }
    if let Err(e) = update_anchor(conn, entry.trace_id, seq, &hash) {
        conn.execute_batch("ROLLBACK").ok();
        return Err(e);
    }
    conn.execute_batch("COMMIT")?;
    Ok(Appended {
        seq,
        event_id,
        hash,
    })
}

/// Does this trace have a genesis (`RouteSelected`) entry? The routing gate
/// (spec 0011 / routing enforcement) blocks work on a trace that was never
/// routed — i.e. has no genesis in the tamper-evident chain.
pub fn has_genesis(conn: &Connection, trace_id: &str) -> bool {
    conn.query_row(
        "SELECT 1 FROM chain_entries WHERE trace_id = ?1 AND seq = 0 AND event_type = 'RouteSelected' LIMIT 1",
        params![trace_id],
        |_| Ok(()),
    )
    .is_ok()
}

/// The head hash of a trace's chain (None if the trace has no entries).
/// This is the value to sign + carry in the envelope's `trace_chain_root`.
pub fn chain_root(conn: &Connection, trace_id: &str) -> Option<String> {
    conn.query_row(
        "SELECT hash FROM chain_entries WHERE trace_id = ?1 ORDER BY seq DESC LIMIT 1",
        params![trace_id],
        |r| r.get::<_, String>(0),
    )
    .ok()
}

/// Verify a trace with [`AnchorPolicy::Required`]. See [`verify_chain_with`].
pub fn verify_chain(conn: &Connection, trace_id: &str) -> std::result::Result<String, String> {
    verify_chain_with(conn, trace_id, AnchorPolicy::Required)
}

/// Recompute the chain from stored payloads, confirm every stored hash matches,
/// and compare the entry count and head against the trace's head anchor.
/// Returns the verified root, or `Err(reason)` if any entry is broken, the
/// chain disagrees with its anchor (`head anchor mismatch`, e.g. truncation),
/// or the trace is unanchored under `AnchorPolicy::Required` (`unanchored`).
pub fn verify_chain_with(
    conn: &Connection,
    trace_id: &str,
    policy: AnchorPolicy,
) -> std::result::Result<String, String> {
    let mut stmt = conn
        .prepare("SELECT seq, event_type, payload_json, prev_hash, hash FROM chain_entries WHERE trace_id = ?1 ORDER BY seq")
        .map_err(|e| e.to_string())?;
    let rows: Vec<(i64, String, String, String, String)> = stmt
        .query_map(params![trace_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    if rows.is_empty() {
        return Err("no chain entries".into());
    }
    let count = rows.len() as i64;
    let mut prev = GENESIS_PREV.to_string();
    let mut last_hash = String::new();
    for (seq, event_type, payload, stored_prev, stored_hash) in rows {
        if seq == 0 && stored_prev != GENESIS_PREV {
            return Err(format!("genesis prev_hash must be {GENESIS_PREV}"));
        }
        if seq > 0 && stored_prev != prev {
            return Err(format!(
                "entry {seq}: prev_hash broken (chain links severed)"
            ));
        }
        let recomputed = hash_of(&event_type, &payload, &stored_prev);
        if recomputed != stored_hash {
            return Err(format!(
                "entry {seq}: event or payload tampered (hash mismatch)"
            ));
        }
        prev = stored_hash.clone();
        last_hash = stored_hash;
    }

    let Some(path) = anchor_path(conn) else {
        return match policy {
            AnchorPolicy::Required => Err(format!(
                "trace '{trace_id}' is unanchored: in-memory ledger has no head anchor"
            )),
            AnchorPolicy::AllowUnanchored => Ok(last_hash),
        };
    };
    let anchors = read_anchors(&path).map_err(|e| format!("{e:#}"))?;
    match anchors.get(trace_id) {
        None => match policy {
            AnchorPolicy::Required => Err(format!(
                "trace '{trace_id}' is unanchored: no head anchor in {} (legacy ledger; truncation cannot be ruled out)",
                path.display()
            )),
            AnchorPolicy::AllowUnanchored => Ok(last_hash),
        },
        Some(anchor) => {
            if anchor.length != count || anchor.head != last_hash {
                return Err(format!(
                    "head anchor mismatch: anchor records {} entries ending in {}, ledger has {} entries ending in {} (entries removed or appended outside the writer)",
                    anchor.length, anchor.head, count, last_hash
                ));
            }
            Ok(last_hash)
        }
    }
}
