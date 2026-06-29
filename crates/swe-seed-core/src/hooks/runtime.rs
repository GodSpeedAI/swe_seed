//! Hook event log runtime (spec 0005). Appends redacted envelopes to a dated
//! JSONL log, iterates events back, and compacts oversized logs.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

use super::redact::{redact_value, RedactionConfig};
use crate::util::{slugify, utc_now};

fn today_log(log_dir: &Path) -> PathBuf {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    log_dir.join(format!("events-{date}.jsonl"))
}

/// Append an event envelope to today's JSONL log, applying redaction first.
/// Returns `(log_path, event_id)`. Assigns `event_id`/`timestamp` if missing.
pub fn append_event(
    log_dir: &Path,
    envelope: &mut Value,
    cfg: &RedactionConfig,
) -> Result<(PathBuf, String)> {
    redact_value(envelope, cfg);

    let event_id = envelope
        .get("event_id")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            let id = format!(
                "evt-{}-{}",
                chrono::Utc::now().timestamp(),
                slugify(
                    envelope
                        .get("event")
                        .and_then(Value::as_str)
                        .unwrap_or("event"),
                )
            );
            if let Some(obj) = envelope.as_object_mut() {
                obj.insert("event_id".into(), Value::String(id.clone()));
            }
            id
        });
    if envelope.get("timestamp").is_none() {
        if let Some(obj) = envelope.as_object_mut() {
            obj.insert("timestamp".into(), Value::String(utc_now()));
        }
    }

    std::fs::create_dir_all(log_dir).with_context(|| format!("create {}", log_dir.display()))?;
    let path = today_log(log_dir);
    let line = format!("{}\n", serde_json::to_string(envelope)?);
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()))
        .with_context(|| format!("write {}", path.display()))?;
    Ok((path, event_id))
}

/// Iterate every event envelope across all `events-*.jsonl` logs (newest first
/// by filename). Returns `(envelope, log_path)` pairs.
pub fn iter_events(log_dir: &Path) -> Result<Vec<(Value, PathBuf)>> {
    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(log_dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("events-") && (n.ends_with(".jsonl")))
            {
                files.push(p);
            }
        }
    }
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(&f)?;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                out.push((v, f.clone()));
            }
        }
    }
    Ok(out)
}

/// Compact logs: gzip any `events-*.jsonl` (except today's) whose size exceeds
/// `min_bytes`, then remove the original. Returns the compacted `.jsonl.gz` paths.
pub fn compact_logs(log_dir: &Path, min_bytes: u64) -> Result<Vec<PathBuf>> {
    use std::io::Read;
    let today = today_log(log_dir);
    let mut compacted = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(log_dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("events-") && n.ends_with(".jsonl"))
            {
                files.push(p);
            }
        }
    }
    files.sort();
    for f in files {
        if f == today {
            continue;
        }
        let size = std::fs::metadata(&f).map(|m| m.len()).unwrap_or(0);
        if size < min_bytes {
            continue;
        }
        let gz = f.with_extension("jsonl.gz");
        let mut input = std::fs::File::open(&f)?;
        let output = std::fs::File::create(&gz)?;
        let mut encoder = flate2::write::GzEncoder::new(output, flate2::Compression::default());
        let mut buf = Vec::new();
        input.read_to_end(&mut buf)?;
        std::io::Write::write_all(&mut encoder, &buf)?;
        encoder.finish()?;
        std::fs::remove_file(&f)?;
        compacted.push(gz);
    }
    Ok(compacted)
}
