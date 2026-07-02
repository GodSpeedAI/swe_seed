//! hooks_runtime: JSONL log + SQLite index + redaction + OTEL/JUnit export +
//! compaction. Falsification: a secret-pattern value must NOT survive in the
//! written log (spec 0005).

use std::fs;
use std::path::PathBuf;

use serde_json::json;

use swe_seed_core::hooks::{
    export::{export_junit, export_otel},
    index::{count_rows, rebuild_index},
    runtime::{append_event, compact_logs, iter_events},
    RedactionConfig,
};

fn temp_log_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "swe-seed-hooks-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn redaction_cfg() -> RedactionConfig {
    RedactionConfig {
        key_substrings: vec![
            "secret".into(),
            "token".into(),
            "password".into(),
            "api_key".into(),
            "authorization".into(),
            "cookie".into(),
        ],
        value_patterns: vec![r"sk-[a-zA-Z0-9]{20,}".into(), r"ghp_[a-zA-Z0-9]{36}".into()],
    }
}

#[test]
fn secrets_do_not_survive_in_the_log() {
    let root = temp_log_root();
    let log_dir = root.join("logs");

    // The envelope carries a secret three ways: a key-matched value, a
    // value-pattern in a normal field, and a nested object.
    let mut env = json!({
        "event": "PreToolUse",
        "hook_id": "h",
        "status": "ok",
        "trace_id": "t",
        "session_id": "s",
        "attributes": {
            "api_key": "should-be-redacted-by-key",
            "note": "leaked sk-abcdef01234567890123456789 in prose",
            "nested": { "password": "p" }
        },
    });
    let (log, _id) = append_event(&log_dir, &mut env, &redaction_cfg()).unwrap();
    let written = fs::read_to_string(&log).unwrap();

    assert!(
        !written.contains("should-be-redacted-by-key"),
        "key-matched secret leaked: {written}"
    );
    assert!(
        !written.contains("sk-abcdef01234567890123456789"),
        "value-pattern secret leaked: {written}"
    );
    assert!(written.contains("[REDACTED]"), "redaction marker missing");

    // Reading back: the envelope is redacted at every level.
    let events = iter_events(&log_dir).unwrap();
    assert_eq!(events.len(), 1);
    let attrs = &events[0].0["attributes"];
    assert_eq!(attrs["api_key"], "[REDACTED]");
    assert_eq!(attrs["nested"]["password"], "[REDACTED]");
    assert!(
        !attrs["note"]
            .as_str()
            .unwrap()
            .contains("sk-abcdef01234567890123456789"),
        "value-pattern survived readback"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn index_export_and_compaction_work() {
    let root = temp_log_root();
    let log_dir = root.join("logs");
    let index_db = root.join("index").join("hooks.rusql");

    let mut a = json!({"event":"SessionStart","hook_id":"h","status":"ok","trace_id":"t","session_id":"s","event_id":"e1"});
    let mut b = json!({"event":"PostToolUse","hook_id":"h","status":"error","trace_id":"t","session_id":"s","event_id":"e2"});
    append_event(&log_dir, &mut a, &redaction_cfg()).unwrap();
    append_event(&log_dir, &mut b, &redaction_cfg()).unwrap();

    // SQLite index reflects both events.
    let n = rebuild_index(&index_db, &log_dir).unwrap();
    assert_eq!(n, 2);
    assert_eq!(count_rows(&index_db).unwrap(), 2);

    // OTEL export carries both log records.
    let otel = export_otel(&log_dir, &root).unwrap();
    let records = otel["scopeLogs"][0]["logRecords"].as_array().unwrap();
    assert_eq!(records.len(), 2);

    // JUnit export: the error event is a failure.
    let junit = export_junit(&log_dir).unwrap();
    assert!(junit.contains("testsuite"));
    assert!(junit.contains("tests=\"2\""));
    assert!(junit.contains("failures=\"1\""));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn compaction_gzips_oversized_logs() {
    let root = temp_log_root();
    let log_dir = root.join("logs");
    // Make a large event line and force a non-today filename so compaction sees it.
    let big = "x".repeat(4096);
    let mut env =
        json!({"event":"PostToolUse","hook_id":"h","status":"ok","event_id":"e1","payload": big});
    append_event(&log_dir, &mut env, &redaction_cfg()).unwrap();
    // Rename today's log to a past date so compact_logs (which skips today) processes it.
    let mut files: Vec<_> = fs::read_dir(&log_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);
    let today_path = files.remove(0).path();
    let old_path = log_dir.join("events-2000-01-01.jsonl");
    fs::rename(&today_path, &old_path).unwrap();

    let compacted = compact_logs(&log_dir, 128).unwrap();
    assert_eq!(compacted.len(), 1, "oversized log should be compacted");
    assert!(compacted[0].to_string_lossy().ends_with(".jsonl.gz"));
    // Original removed; gz exists.
    assert!(!old_path.exists());
    assert!(compacted[0].exists());

    let _ = fs::remove_dir_all(&root);
}
