//! SQLite index over the hook event log (spec 0005). Rebuilds an event table
//! keyed by `event_id` from the JSONL logs via `rusqlite`.

use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

use super::runtime::iter_events;

/// Rebuild the SQLite index at `index_db` from all `events-*.jsonl` logs under
/// `log_dir`. Returns the number of indexed events.
pub fn rebuild_index(index_db: &Path, log_dir: &Path) -> Result<usize> {
    if let Some(parent) = index_db.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(index_db)
        .with_context(|| format!("open sqlite {}", index_db.display()))?;
    conn.execute_batch("DROP TABLE IF EXISTS events;")?;
    conn.execute_batch(
        "CREATE TABLE events (
            event_id      TEXT PRIMARY KEY,
            trace_id      TEXT NOT NULL,
            session_id    TEXT NOT NULL,
            event         TEXT NOT NULL,
            hook_id       TEXT NOT NULL,
            status        TEXT NOT NULL,
            timestamp     TEXT NOT NULL,
            log_path      TEXT NOT NULL,
            raw_event_json TEXT NOT NULL
        );",
    )?;

    let mut count = 0usize;
    let events = iter_events(log_dir)?;
    for (envelope, log_path) in events {
        let event_id = envelope
            .get("event_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let trace_id = envelope
            .get("trace_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let session_id = envelope
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let event = envelope.get("event").and_then(|v| v.as_str()).unwrap_or("");
        let hook_id = envelope
            .get("hook_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let status = envelope
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("ok");
        let timestamp = envelope
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let raw = serde_json::to_string(&envelope).unwrap_or_default();
        conn.execute(
            "INSERT OR REPLACE INTO events
                (event_id, trace_id, session_id, event, hook_id, status, timestamp, log_path, raw_event_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                event_id,
                trace_id,
                session_id,
                event,
                hook_id,
                status,
                timestamp,
                log_path.to_string_lossy(),
                raw,
            ],
        )?;
        count += 1;
    }
    Ok(count)
}

/// Count indexed rows (for tests).
pub fn count_rows(index_db: &Path) -> Result<i64> {
    if !index_db.exists() {
        return Ok(0);
    }
    let conn = Connection::open(index_db)?;
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))?;
    Ok(n)
}
