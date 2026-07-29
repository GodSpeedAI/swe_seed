//! Gateway stage-4 proof (spec 0020 §5, §18): redacted JSONL audit writer,
//! one record per decision, fail-closed on write error.
//! Run: `cargo test gateway_audit`.

use std::fs;
use std::io::Read;

use swe_seed_core::gateway::{AuditRecord, AuditWriter, GatewayError};
use swe_seed_core::hooks::redact::RedactionConfig;

fn temp_root(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-gw-audit-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn allow_record() -> AuditRecord {
    AuditRecord {
        occurred_at: "2026-07-28T00:00:00+00:00".into(),
        request_id: "req-1".into(),
        session_id: "sess-1".into(),
        client_id: "cli-1".into(),
        method: "tools/call".into(),
        namespaced_name: Some("fs.read".into()),
        backend: Some("fs".into()),
        decision: "allow".into(),
        reason_code: None,
        latency_ms: Some(12),
        cost_delta: None,
        route_card_id: None,
        domain_model_hash_source: None,
        semantic_envelope_id: None,
        semantic_envelope_event_type: None,
        detail: None,
    }
}

#[test]
fn writes_one_jsonl_line_per_record_and_round_trips() {
    let root = temp_root("basic");
    let writer = AuditWriter::new(&root, RedactionConfig::default());
    writer.write(&allow_record()).unwrap();
    writer
        .write(&AuditRecord {
            decision: "deny".into(),
            reason_code: Some("scope_denied".into()),
            ..allow_record()
        })
        .unwrap();

    let content = fs::read_to_string(writer.file_path()).unwrap();
    let lines: Vec<&str> = content.trim_end().lines().collect();
    assert_eq!(lines.len(), 2, "one line per record");
    let rec: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(rec["decision"], "deny");
    assert_eq!(rec["reason_code"], "scope_denied");
    assert_eq!(rec["session_id"], "sess-1");
    fs::remove_dir_all(&root).ok();
}

#[test]
fn redacts_secret_detail_using_hook_rules() {
    let root = temp_root("redact");
    let mut redaction = RedactionConfig::default();
    // value_patterns redact secret substrings inside freeform fields like
    // `detail` (the realistic leak vector: a backend error echoing a credential).
    redaction.value_patterns = vec![r#"Bearer [^"]+"#.into()];
    // key_substrings redact any whole value whose key looks secret-bearing.
    redaction.key_substrings = vec!["token".into()];
    let writer = AuditWriter::new(&root, redaction);
    let mut rec = allow_record();
    rec.detail = Some("auth=Bearer super-secret-value".into());
    writer.write(&rec).unwrap();

    let content = fs::read_to_string(writer.file_path()).unwrap();
    assert!(
        !content.contains("super-secret-value"),
        "secret leaked into audit: {content}"
    );
    assert!(content.contains("[REDACTED]"));
    fs::remove_dir_all(&root).ok();
}

#[test]
fn write_fails_closed_when_audit_dir_cannot_be_created() {
    // Make `<root>/.swe-seed` a FILE so `.swe-seed/gateway/audit` cannot be
    // created → the writer must return AuditWriteFailed, not silently drop.
    let root = temp_root("failclosed");
    fs::write(root.join(".swe-seed"), "i am a file, not a dir").unwrap();
    let writer = AuditWriter::new(&root, RedactionConfig::default());
    let err = writer.write(&allow_record()).unwrap_err();
    assert_eq!(err.reason_code(), "audit_write_failed");
    assert!(matches!(err, GatewayError::AuditWriteFailed { .. }));
    // and no audit file exists
    assert!(!writer.file_path().exists());
    fs::remove_dir_all(&root).ok();
}

#[test]
fn skip_serializing_none_keeps_audit_lines_compact() {
    let root = temp_root("compact");
    let writer = AuditWriter::new(&root, RedactionConfig::default());
    writer.write(&allow_record()).unwrap();
    let content = fs::read_to_string(writer.file_path()).unwrap();
    // optional unset fields are omitted, not null
    assert!(!content.contains("\"reason_code\""));
    assert!(!content.contains("\"detail\""));
    fs::remove_dir_all(&root).ok();
}

#[test]
fn appends_across_multiple_writers_same_path() {
    let root = temp_root("append");
    let w1 = AuditWriter::new(&root, RedactionConfig::default());
    let w2 = AuditWriter::new(&root, RedactionConfig::default());
    w1.write(&allow_record()).unwrap();
    w2
        .write(&AuditRecord {
            request_id: "req-2".into(),
            ..allow_record()
        })
        .unwrap();
    let content = fs::read_to_string(w1.file_path()).unwrap();
    assert_eq!(content.trim_end().lines().count(), 2);
    // sanity: ensure no truncation occurred (append, not overwrite)
    let mut f = fs::File::open(w1.file_path()).unwrap();
    let mut before = String::new();
    f.read_to_string(&mut before).unwrap();
    assert!(before.contains("req-1") && before.contains("req-2"));
    fs::remove_dir_all(&root).ok();
}
