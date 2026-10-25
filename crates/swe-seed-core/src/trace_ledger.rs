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
//! - **Truncation (head anchor):** every append also updates a per-trace head
//!   anchor (`{length, head}`) in a sidecar file next to the SQLite ledger
//!   (`ledger.anchors.json`). Verification compares the chain's entry count and
//!   head hash to that anchor, so deleting trailing rows is detected. A trace
//!   with no anchor is reported as `unanchored` and fails closed unless the
//!   caller explicitly opts in with [`AnchorPolicy::AllowUnanchored`].
//! - **Two-phase anchor (crash safety):** an append first writes the anchor's
//!   `pending` state (the post-append length and head) before COMMIT, then
//!   promotes it to the committed state after COMMIT. A crash in either window
//!   leaves a state that verification accepts and the next append (or finalize)
//!   resolves. Truncation of a finalized trace is still detected.
//! - **Signed anchor file:** the anchor file carries the public key and an
//!   Ed25519 signature over a canonical encoding of the anchors. The private key
//!   lives outside the ledger directory (see [`anchor_key_path`]). A signed file
//!   that does not verify fails closed. An unsigned (legacy) file fails closed
//!   under [`AnchorPolicy::Required`], and appends refuse it. It is signed only
//!   by the explicit operator step [`adopt_legacy_anchors`] (trust-on-first-use),
//!   which checks every anchored chain first. It is never re-signed automatically.
//! - **Chain root** (`chain_root`): the head hash — a single value that covers
//!   the whole chain. This is the value that will be Ed25519-signed in Phase B
//!   and carried in the SEA envelope's `trace_chain_root`.
//!
//! Threat-model limits of the anchor: a writer who can modify the ledger
//! directory but cannot read the anchor signing key cannot forge an anchor, so
//! rewriting rows without the key is detected. A process running as the key's
//! uid can read the key and forge anchors, so it is not defended against. The
//! key is not a hardware or OS-keyring secret. Signing is not a substitute for
//! off-host anchoring. Between a crash and the next append, a trailing row that
//! was committed but not yet finalized can be removed without detection, because
//! the pending state matches the shorter chain. The window closes on the next
//! append.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ulid::Ulid;

/// The zero hash that seeds the genesis entry's `prev_hash`.
pub const GENESIS_PREV: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Environment variable that overrides the anchor signing key path.
pub const ANCHOR_KEY_ENV: &str = "SWE_SEED_ANCHOR_KEY";

/// Anchor file format tag for the signed document.
const ANCHOR_FORMAT: &str = "swe-seed-anchors/2";

/// Default ledger location.
pub fn default_db_path(root: &Path) -> std::path::PathBuf {
    root.join(".agent-harness")
        .join("traces")
        .join("ledger.sqlite3")
}

/// Where the anchor signing key lives: `$SWE_SEED_ANCHOR_KEY` if set, else
/// `$XDG_CONFIG_HOME/swe-seed/anchor-ed25519.key`, else
/// `$HOME/.config/swe-seed/anchor-ed25519.key`. Deliberately outside the ledger
/// directory.
pub fn anchor_key_path() -> Result<PathBuf> {
    if let Some(p) = std::env::var_os(ANCHOR_KEY_ENV).filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(p));
    }
    let config_home = match std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
        Some(dir) => PathBuf::from(dir),
        None => {
            let home = std::env::var_os("HOME").filter(|v| !v.is_empty()).ok_or_else(|| {
                anyhow!(
                    "cannot locate the anchor signing key: set {ANCHOR_KEY_ENV}, XDG_CONFIG_HOME, or HOME (fail closed)"
                )
            })?;
            PathBuf::from(home).join(".config")
        }
    };
    Ok(config_home.join("swe-seed").join("anchor-ed25519.key"))
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

/// How verification treats a trace that has no head anchor, and whether an
/// unsigned (legacy) anchor file is accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorPolicy {
    /// Fail closed: an unanchored trace, or an unsigned anchor file, cannot
    /// prove it was not truncated.
    Required,
    /// Explicit legacy opt-in: accept an unanchored trace (structure only) and
    /// an unsigned anchor file. A trace that *has* an anchor is still checked
    /// against it.
    AllowUnanchored,
}

/// A chain state: entry count and head hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadState {
    /// Number of chain entries.
    pub length: i64,
    /// Hash of the last entry.
    pub head: String,
}

/// The expected state of one trace's chain. `length`/`head` is the committed
/// state. `pending` is written before COMMIT and promoted after it, so an
/// interrupted append leaves a state that is either committed or pending.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadAnchor {
    /// Number of chain entries the writer last committed.
    pub length: i64,
    /// Hash of the last committed entry.
    pub head: String,
    /// The state an in-flight append is about to commit. Absent once finalized,
    /// and always absent in legacy anchor files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<HeadState>,
}

impl HeadAnchor {
    fn committed_matches(&self, length: i64, head: &str) -> bool {
        self.length == length && self.head == head
    }

    fn pending_matches(&self, length: i64, head: &str) -> bool {
        self.pending
            .as_ref()
            .is_some_and(|p| p.length == length && p.head == head)
    }

    fn matches_either(&self, length: i64, head: &str) -> bool {
        self.committed_matches(length, head) || self.pending_matches(length, head)
    }
}

/// Sidecar anchor file for a ledger DB: `ledger.sqlite3` -> `ledger.anchors.json`.
/// `None` for connections without a file path (in-memory).
fn anchor_path(conn: &Connection) -> Option<PathBuf> {
    conn.path()
        .filter(|p| !p.is_empty())
        .map(|p| Path::new(p).with_extension("anchors.json"))
}

/// On-disk signed anchor document.
#[derive(Serialize, Deserialize)]
struct SignedAnchorFile {
    format: String,
    /// Base64 of the 32-byte Ed25519 public key that signed the document.
    public_key: String,
    anchors: BTreeMap<String, HeadAnchor>,
    /// Base64 of the 64-byte Ed25519 signature over [`signing_bytes`].
    signature: String,
}

/// Canonical signed body. Field order is fixed, `anchors` is a `BTreeMap`
/// (sorted keys), and `serde_json::to_vec` emits compact output, so the same
/// anchors always produce the same bytes.
#[derive(Serialize)]
struct AnchorSigningBody<'a> {
    format: &'a str,
    public_key: &'a str,
    anchors: &'a BTreeMap<String, HeadAnchor>,
}

fn signing_bytes(public_key: &str, anchors: &BTreeMap<String, HeadAnchor>) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(&AnchorSigningBody {
        format: ANCHOR_FORMAT,
        public_key,
        anchors,
    })?)
}

fn encode_signed_file(anchors: &BTreeMap<String, HeadAnchor>, key: &SigningKey) -> Result<String> {
    let public_key = B64.encode(key.verifying_key().to_bytes());
    let signature = key.sign(&signing_bytes(&public_key, anchors)?);
    let file = SignedAnchorFile {
        format: ANCHOR_FORMAT.to_string(),
        public_key,
        anchors: anchors.clone(),
        signature: B64.encode(signature.to_bytes()),
    };
    Ok(serde_json::to_string_pretty(&file)?)
}

fn verify_signed_file(
    file: SignedAnchorFile,
    expected: &VerifyingKey,
) -> Result<BTreeMap<String, HeadAnchor>> {
    if file.format != ANCHOR_FORMAT {
        bail!(
            "unsupported anchor file format '{}' (fail closed)",
            file.format
        );
    }
    if file.public_key != B64.encode(expected.to_bytes()) {
        bail!("anchor file was signed by a different key than the configured anchor key (fail closed)");
    }
    let sig_bytes = B64
        .decode(file.signature.as_bytes())
        .map_err(|_| anyhow!("anchor signature is not valid base64 (fail closed)"))?;
    let sig = Signature::from_slice(&sig_bytes)
        .map_err(|_| anyhow!("anchor signature is malformed (fail closed)"))?;
    let body = signing_bytes(&file.public_key, &file.anchors)?;
    expected.verify(&body, &sig).map_err(|_| {
        anyhow!("anchor signature does not verify: the anchor file was modified (fail closed)")
    })?;
    Ok(file.anchors)
}

fn decode_anchor_key(text: &str, path: &Path) -> Result<SigningKey> {
    let bytes = B64
        .decode(text.trim().as_bytes())
        .with_context(|| format!("anchor key {} is not base64", path.display()))?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("anchor key {} must be 32 bytes", path.display()))?;
    Ok(SigningKey::from_bytes(&arr))
}

/// Load the existing anchor key. Verification never creates a key.
fn read_anchor_key(path: &Path) -> Result<SigningKey> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        anyhow!(
            "anchor signing key {} unavailable ({e}); cannot verify signed anchors (fail closed)",
            path.display()
        )
    })?;
    decode_anchor_key(&text, path)
}

/// Load the anchor key, creating it on first use (0600, outside the ledger).
fn load_or_create_anchor_key(path: &Path) -> Result<SigningKey> {
    match std::fs::read_to_string(path) {
        Ok(text) => decode_anchor_key(&text, path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("create anchor key dir {}", parent.display()))?;
            }
            // Write the complete key to a private temp file, then hard-link it
            // into place. The link is atomic and fails if the key already exists,
            // so a concurrent first use never sees a partly written key.
            let key = SigningKey::generate(&mut rand_core::OsRng);
            let tmp = path.with_extension(format!("tmp-{}", Ulid::new()));
            let mut opts = std::fs::OpenOptions::new();
            opts.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                opts.mode(0o600);
            }
            {
                let mut f = opts
                    .open(&tmp)
                    .with_context(|| format!("create anchor key {}", tmp.display()))?;
                f.write_all(B64.encode(key.to_bytes()).as_bytes())
                    .with_context(|| format!("write anchor key {}", tmp.display()))?;
                f.sync_all()?;
            }
            let linked = std::fs::hard_link(&tmp, path);
            std::fs::remove_file(&tmp).ok();
            match linked {
                Ok(()) => Ok(key),
                // Another process or thread created it first; use theirs.
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let text = std::fs::read_to_string(path)?;
                    decode_anchor_key(&text, path)
                }
                Err(e) => Err(e).with_context(|| format!("create anchor key {}", path.display())),
            }
        }
        Err(e) => Err(e).with_context(|| format!("read anchor key {}", path.display())),
    }
}

/// The anchor file as read from disk.
enum AnchorFileState {
    /// No anchor file.
    Absent,
    /// Pre-signature format: a bare `{trace_id: {length, head}}` map.
    Legacy,
    /// Signed document whose signature verified against the configured key.
    Signed,
}

struct LoadedAnchors {
    state: AnchorFileState,
    anchors: BTreeMap<String, HeadAnchor>,
}

/// Read the anchor file. A signed document must verify with the configured
/// key, or this fails. A bare legacy map is returned as `Legacy` and not
/// vouched for.
fn load_anchors(path: &Path) -> Result<LoadedAnchors> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(LoadedAnchors {
                state: AnchorFileState::Absent,
                anchors: BTreeMap::new(),
            })
        }
        Err(e) => return Err(e).with_context(|| format!("read head anchors {}", path.display())),
    };
    let value: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("parse head anchors {} (fail closed)", path.display()))?;
    let signed = value.get("signature").is_some() || value.get("format").is_some();
    if !signed {
        let anchors = serde_json::from_value(value)
            .with_context(|| format!("parse head anchors {} (fail closed)", path.display()))?;
        return Ok(LoadedAnchors {
            state: AnchorFileState::Legacy,
            anchors,
        });
    }
    let file: SignedAnchorFile = serde_json::from_value(value)
        .with_context(|| format!("parse signed head anchors {} (fail closed)", path.display()))?;
    let key = read_anchor_key(&anchor_key_path()?)?;
    let anchors = verify_signed_file(file, &key.verifying_key())?;
    Ok(LoadedAnchors {
        state: AnchorFileState::Signed,
        anchors,
    })
}

/// Write the anchor file as a signed document (tmp + rename, fsynced).
fn save_anchors(path: &Path, anchors: &BTreeMap<String, HeadAnchor>) -> Result<()> {
    let key = load_or_create_anchor_key(&anchor_key_path()?)?;
    let text = encode_signed_file(anchors, &key)?;
    let tmp = path.with_extension("anchors.json.tmp");
    {
        let mut f = std::fs::File::create(&tmp)
            .with_context(|| format!("write head anchors {}", tmp.display()))?;
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)
        .with_context(|| format!("replace head anchors {}", path.display()))?;
    Ok(())
}

/// The chain state of `trace_id` after verifying every stored hash:
/// `(entry count, head hash)`.
fn chain_state(conn: &Connection, trace_id: &str) -> std::result::Result<(i64, String), String> {
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
    Ok((count, last_hash))
}

/// Decide what the anchor file must say before the append's COMMIT.
///
/// Returns `None` when the file need not be written, or `Some((anchors, anchored))`
/// where `anchored` says whether this trace's pending state must be finalized
/// after COMMIT. The existing committed/pending state is reconciled with the
/// pre-append chain (`seq` entries ending in `prev`): a committed or pending
/// match is promoted, and any other state refuses the append.
fn plan_anchor_update(
    loaded: &LoadedAnchors,
    trace_id: &str,
    seq: i64,
    prev: &str,
    hash: &str,
) -> Result<Option<(BTreeMap<String, HeadAnchor>, bool)>> {
    // Never sign an unsigned file automatically. An attacker who can write the
    // ledger directory could otherwise swap in a legacy file that matches a
    // truncated chain, and the append would bless it. Adoption is an explicit,
    // operator-run step (`adopt_legacy_anchors`).
    if matches!(loaded.state, AnchorFileState::Legacy) {
        bail!(
            "anchor file is unsigned (legacy); refusing to append. Run `swe-seed trace adopt-anchors` once after upgrade to sign it (trust-on-first-use), then append again"
        );
    }
    let mut anchors = loaded.anchors.clone();

    let anchored = match anchors.get_mut(trace_id) {
        Some(a) => {
            if a.matches_either(seq, prev) {
                // Promote a pending state that a crash left behind, or discard a
                // stale pending state for an entry that never committed.
                a.length = seq;
                a.head = prev.to_string();
                a.pending = None;
            } else if seq == 0 {
                bail!(
                    "head anchor mismatch before append: trace '{trace_id}' has a head anchor but no chain entries (truncated to empty)"
                );
            } else {
                bail!(
                    "head anchor mismatch before append: trace '{trace_id}' anchor records {} entries ending in {}, chain has {} entries ending in {} (entries removed outside the writer); refusing to extend",
                    a.length,
                    a.head,
                    seq,
                    prev
                );
            }
            true
        }
        None if seq == 0 => {
            // First entry for a new trace: committed is the empty chain.
            anchors.insert(
                trace_id.to_string(),
                HeadAnchor {
                    length: 0,
                    head: GENESIS_PREV.to_string(),
                    pending: None,
                },
            );
            true
        }
        // A legacy trace with rows but no anchor stays unanchored. Appending
        // must not vouch for rows written before the anchor existed.
        None => false,
    };

    if anchored {
        let a = anchors.get_mut(trace_id).expect("anchored trace present");
        a.pending = Some(HeadState {
            length: seq + 1,
            head: hash.to_string(),
        });
        Ok(Some((anchors, true)))
    } else {
        Ok(None)
    }
}

/// Operator trust-on-first-use: sign a legacy (unsigned) anchor file once after
/// upgrade. Every anchored chain must match its committed state, or nothing is
/// signed. A signed file is refused, since it has nothing to adopt. Returns
/// `(trace_id, length)` for each adopted trace.
pub fn adopt_legacy_anchors(conn: &Connection) -> Result<Vec<(String, i64)>> {
    let path = anchor_path(conn).ok_or_else(|| anyhow!("in-memory ledger has no anchor file"))?;
    conn.execute_batch("BEGIN IMMEDIATE TRANSACTION")?;
    match adopt_locked(conn, &path) {
        Ok(adopted) => {
            if let Err(e) = conn.execute_batch("COMMIT") {
                conn.execute_batch("ROLLBACK").ok();
                return Err(e.into());
            }
            Ok(adopted)
        }
        Err(e) => {
            conn.execute_batch("ROLLBACK").ok();
            Err(e)
        }
    }
}

fn adopt_locked(conn: &Connection, path: &Path) -> Result<Vec<(String, i64)>> {
    let loaded = load_anchors(path)?;
    match loaded.state {
        AnchorFileState::Absent => bail!("no anchor file at {}; nothing to adopt", path.display()),
        AnchorFileState::Signed => bail!(
            "anchor file {} is already signed; nothing to adopt",
            path.display()
        ),
        AnchorFileState::Legacy => {}
    }
    let mut adopted = Vec::new();
    for (trace_id, anchor) in &loaded.anchors {
        if anchor.pending.is_some() {
            bail!("refusing to adopt legacy anchors: trace '{trace_id}' has a pending state");
        }
        let (count, last) = chain_state(conn, trace_id)
            .map_err(|e| anyhow!("refusing to adopt legacy anchors: trace '{trace_id}' ({e})"))?;
        if !anchor.committed_matches(count, &last) {
            bail!(
                "refusing to adopt legacy anchors: trace '{trace_id}' does not match its anchor (head anchor mismatch); nothing was signed"
            );
        }
        adopted.push((trace_id.clone(), count));
    }
    save_anchors(path, &loaded.anchors)?;
    Ok(adopted)
}

/// Promote `trace_id`'s pending anchor to committed, if the ledger matches the
/// pending state. Runs in its own short transaction so it serializes with other
/// writers. If the ledger does not match, it is a no-op and the next append
/// reconciles.
fn finalize_anchor(conn: &Connection, path: &Path, trace_id: &str) -> Result<()> {
    conn.execute_batch("BEGIN IMMEDIATE TRANSACTION")?;
    let result = finalize_locked(conn, path, trace_id);
    match result {
        Ok(()) => {
            if let Err(e) = conn.execute_batch("COMMIT") {
                conn.execute_batch("ROLLBACK").ok();
                return Err(e.into());
            }
            Ok(())
        }
        Err(e) => {
            conn.execute_batch("ROLLBACK").ok();
            Err(e)
        }
    }
}

fn finalize_locked(conn: &Connection, path: &Path, trace_id: &str) -> Result<()> {
    let (count, last) = chain_state(conn, trace_id).map_err(|e| anyhow!("{e}"))?;
    let mut loaded = load_anchors(path)?;
    if let Some(a) = loaded.anchors.get_mut(trace_id) {
        if a.pending_matches(count, &last) {
            a.length = count;
            a.head = last;
            a.pending = None;
            save_anchors(path, &loaded.anchors)?;
        }
    }
    Ok(())
}

/// A point at which a simulated crash interrupts [`append_with_crash_point`].
/// Test hook: lets tests exercise both windows without killing the process.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashPoint {
    /// The pending anchor is on disk, COMMIT never runs (the row rolls back).
    BeforeCommit,
    /// COMMIT runs, the anchor is never finalized.
    BeforeFinalize,
}

/// Append an entry to a trace's chain. The first append for a `trace_id`
/// becomes the genesis (seq 0, prev_hash = GENESIS_PREV). A fresh ULID is
/// minted for `event_id`.
pub fn append(conn: &Connection, entry: LedgerEntry<'_>) -> Result<Appended> {
    append_impl(conn, entry, None)
}

/// [`append`] that simulates a crash at `point` and returns an error. Test hook.
#[doc(hidden)]
pub fn append_with_crash_point(
    conn: &Connection,
    entry: LedgerEntry<'_>,
    point: CrashPoint,
) -> Result<Appended> {
    append_impl(conn, entry, Some(point))
}

fn append_impl(
    conn: &Connection,
    entry: LedgerEntry<'_>,
    crash: Option<CrashPoint>,
) -> Result<Appended> {
    let event_id = Ulid::new().to_string();
    conn.execute_batch("BEGIN IMMEDIATE TRANSACTION")?;
    let staged = match stage_append(conn, &entry, &event_id, crash) {
        Ok(staged) => staged,
        Err(e) => {
            conn.execute_batch("ROLLBACK").ok();
            return Err(e);
        }
    };
    if let Err(e) = conn.execute_batch("COMMIT") {
        conn.execute_batch("ROLLBACK").ok();
        return Err(e.into());
    }
    if crash == Some(CrashPoint::BeforeFinalize) {
        bail!("simulated crash before anchor finalize");
    }
    if let Some(path) = staged.finalize_path {
        if let Err(e) = finalize_anchor(conn, &path, entry.trace_id) {
            // The row is durable. The pending anchor verifies, and the next
            // append finalizes it.
            eprintln!(
                "warning: ledger entry committed but its head anchor was not finalized ({e:#}); the next append will finalize it"
            );
        }
    }
    Ok(staged.appended)
}

struct Staged {
    appended: Appended,
    /// Anchor file to finalize after COMMIT, if this trace is anchored.
    finalize_path: Option<PathBuf>,
}

/// Insert the row and write the pending anchor, inside the open transaction.
fn stage_append(
    conn: &Connection,
    entry: &LedgerEntry<'_>,
    event_id: &str,
    crash: Option<CrashPoint>,
) -> Result<Staged> {
    let (max_seq, prev_hash): (i64, String) = match conn.query_row(
        "SELECT seq, hash FROM chain_entries WHERE trace_id = ?1 ORDER BY seq DESC LIMIT 1",
        params![entry.trace_id],
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
    ) {
        Ok(row) => row,
        Err(rusqlite::Error::QueryReturnedNoRows) => (-1, GENESIS_PREV.to_string()),
        Err(e) => return Err(e.into()),
    };
    let seq = max_seq + 1;
    let prev = if seq == 0 {
        GENESIS_PREV.to_string()
    } else {
        prev_hash
    };
    let hash = hash_of(entry.event_type, entry.payload_json, &prev);

    // Decide the anchor update against the pre-append chain, before inserting.
    let anchor_file = match anchor_path(conn) {
        Some(path) => {
            let loaded = load_anchors(&path)?;
            plan_anchor_update(&loaded, entry.trace_id, seq, &prev, &hash)?.map(|plan| (path, plan))
        }
        None => None,
    };

    conn.execute(
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
    )?;

    let mut finalize_path = None;
    if let Some((path, (anchors, anchored))) = anchor_file {
        save_anchors(&path, &anchors)?;
        if anchored {
            finalize_path = Some(path);
        }
    }
    if crash == Some(CrashPoint::BeforeCommit) {
        bail!("simulated crash before COMMIT");
    }
    Ok(Staged {
        appended: Appended {
            seq,
            event_id: event_id.to_string(),
            hash,
        },
        finalize_path,
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
/// The chain matches if it equals the committed state, or the pending state
/// left by an append interrupted after COMMIT.
///
/// Returns the verified root, or `Err(reason)` if any entry is broken, the
/// chain disagrees with its anchor (`head anchor mismatch`, e.g. truncation),
/// the anchor file is unsigned or fails its signature check, or the trace is
/// unanchored under `AnchorPolicy::Required` (`unanchored`).
pub fn verify_chain_with(
    conn: &Connection,
    trace_id: &str,
    policy: AnchorPolicy,
) -> std::result::Result<String, String> {
    let (count, last_hash) = chain_state(conn, trace_id)?;

    let Some(path) = anchor_path(conn) else {
        return match policy {
            AnchorPolicy::Required => Err(format!(
                "trace '{trace_id}' is unanchored: in-memory ledger has no head anchor"
            )),
            AnchorPolicy::AllowUnanchored => Ok(last_hash),
        };
    };
    let loaded = load_anchors(&path).map_err(|e| format!("{e:#}"))?;
    if matches!(loaded.state, AnchorFileState::Legacy) && policy == AnchorPolicy::Required {
        return Err(format!(
            "anchor file {} is unsigned (legacy); truncation cannot be ruled out until an operator runs `swe-seed trace adopt-anchors` once to sign it (trust-on-first-use)",
            path.display()
        ));
    }
    match loaded.anchors.get(trace_id) {
        None => match policy {
            AnchorPolicy::Required => Err(format!(
                "trace '{trace_id}' is unanchored: no head anchor in {} (legacy ledger; truncation cannot be ruled out)",
                path.display()
            )),
            AnchorPolicy::AllowUnanchored => Ok(last_hash),
        },
        Some(anchor) => {
            if !anchor.matches_either(count, &last_hash) {
                return Err(format!(
                    "head anchor mismatch: anchor records {} entries ending in {}, ledger has {} entries ending in {} (entries removed or appended outside the writer)",
                    anchor.length, anchor.head, count, last_hash
                ));
            }
            Ok(last_hash)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signing_key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn sample_anchors() -> BTreeMap<String, HeadAnchor> {
        let mut m = BTreeMap::new();
        m.insert(
            "T".to_string(),
            HeadAnchor {
                length: 2,
                head: "abc".to_string(),
                pending: Some(HeadState {
                    length: 3,
                    head: "def".to_string(),
                }),
            },
        );
        m
    }

    #[test]
    fn signed_document_round_trips() {
        let key = signing_key(1);
        let text = encode_signed_file(&sample_anchors(), &key).unwrap();
        let file: SignedAnchorFile = serde_json::from_str(&text).unwrap();
        let back = verify_signed_file(file, &key.verifying_key()).unwrap();
        assert_eq!(back, sample_anchors());
    }

    #[test]
    fn wrong_key_fails_closed() {
        let text = encode_signed_file(&sample_anchors(), &signing_key(1)).unwrap();
        let file: SignedAnchorFile = serde_json::from_str(&text).unwrap();
        let err = verify_signed_file(file, &signing_key(2).verifying_key()).unwrap_err();
        assert!(format!("{err:#}").contains("different key"), "{err:#}");
    }

    #[test]
    fn edited_body_with_stale_signature_fails_closed() {
        let key = signing_key(1);
        let text = encode_signed_file(&sample_anchors(), &key).unwrap();
        let mut file: SignedAnchorFile = serde_json::from_str(&text).unwrap();
        file.anchors.get_mut("T").unwrap().length = 1;
        let err = verify_signed_file(file, &key.verifying_key()).unwrap_err();
        assert!(format!("{err:#}").contains("does not verify"), "{err:#}");
    }

    #[test]
    fn pending_is_omitted_when_finalized() {
        let key = signing_key(1);
        let mut anchors = sample_anchors();
        anchors.get_mut("T").unwrap().pending = None;
        let text = encode_signed_file(&anchors, &key).unwrap();
        assert!(!text.contains("pending"), "{text}");
    }
}
