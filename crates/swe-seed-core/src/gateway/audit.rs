//! Local gateway audit (spec 0020 §5, §18). Every request outcome — allow, deny,
//! backend-unavailable, hash mismatch, session/budget block, invalid request —
//! writes exactly one redacted JSONL record to `.swe-seed/gateway/audit/audit.jsonl`
//! BEFORE the request is considered complete. If the audit record cannot be
//! written, the request MUST fail closed (`GatewayError::AuditWriteFailed`);
//! audit is part of request completion, not a side effect (§5, §18).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::hooks::redact::{redact_value, RedactionConfig};

use super::GatewayError;

/// Default audit log location.
pub const AUDIT_DIR_REL: &str = ".swe-seed/gateway/audit";
pub const AUDIT_FILE: &str = "audit.jsonl";

/// One audit record per request outcome (spec 0020 §5). Fixed schema; the
/// optional `detail` carries freeform context (e.g. a backend error reason) and
/// IS redacted before write, since backend echoes are the realistic leak vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub occurred_at: String,
    pub request_id: String,
    pub session_id: String,
    pub client_id: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespaced_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    /// `allow` | `approval_required` | `deny` | `backend_unavailable` |
    /// `backend_error` | `session_block` | `budget_block` | `invalid_request`.
    pub decision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_delta: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_card_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_model_hash_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_envelope_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_envelope_event_type: Option<String>,
    /// Freeform context (redacted before write).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl AuditRecord {
    /// The high-level decision bucket for a typed gateway error.
    pub fn decision_for_error(err: &GatewayError) -> &'static str {
        use GatewayError::*;
        match err {
            BackendUnavailable { .. } => "backend_unavailable",
            BackendError { .. } => "backend_error",
            BudgetExceeded { .. } => "budget_block",
            SessionLimitExceeded { .. } => "session_block",
            InvalidRequest { .. } => "invalid_request",
            // All other denials are policy/integrity denials.
            _ => "deny",
        }
    }
}

/// Append-only, redacting audit writer.
pub struct AuditWriter {
    dir: PathBuf,
    redaction: RedactionConfig,
    /// Serializes appends. `writeln!` on an unbuffered file is two writes (record, then newline), so
    /// concurrent callers could otherwise interleave and fuse two records into one line.
    append: std::sync::Mutex<()>,
}

impl AuditWriter {
    pub fn new(root: &Path, redaction: RedactionConfig) -> Self {
        Self {
            dir: root.join(AUDIT_DIR_REL),
            redaction,
            append: std::sync::Mutex::new(()),
        }
    }

    /// The resolved audit file path (for inspection/tests).
    pub fn file_path(&self) -> PathBuf {
        self.dir.join(AUDIT_FILE)
    }

    /// Serialize + redact + append one record. Fail-closed: any I/O or
    /// serialization error becomes `GatewayError::AuditWriteFailed` so the
    /// caller treats the request as incomplete (spec 0020 §5, §18).
    pub fn write(&self, record: &AuditRecord) -> Result<(), GatewayError> {
        let mut value = serde_json::to_value(record)
            .map_err(|e| GatewayError::AuditWriteFailed {
                reason: format!("serialize: {e}"),
            })?;
        redact_value(&mut value, &self.redaction);
        let line = serde_json::to_string(&value)
            .map_err(|e| GatewayError::AuditWriteFailed {
                reason: format!("stringify: {e}"),
            })?;
        std::fs::create_dir_all(&self.dir).map_err(|e| GatewayError::AuditWriteFailed {
            reason: format!("create {}: {e}", self.dir.display()),
        })?;
        let path = self.file_path();
        // One buffer, one write: the record and its newline can never be split by another appender.
        let mut bytes = line.into_bytes();
        bytes.push(b'\n');
        let _guard = self.append.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| GatewayError::AuditWriteFailed {
                reason: format!("open {}: {e}", path.display()),
            })?;
        use std::io::Write;
        let mut file = file;
        file.write_all(&bytes).map_err(|e| GatewayError::AuditWriteFailed {
            reason: format!("write {}: {e}", path.display()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_buckets_match_spec() {
        assert_eq!(
            AuditRecord::decision_for_error(&GatewayError::BackendUnavailable {
                backend: "b".into()
            }),
            "backend_unavailable"
        );
        assert_eq!(
            AuditRecord::decision_for_error(&GatewayError::BudgetExceeded { limit: 1 }),
            "budget_block"
        );
        assert_eq!(
            AuditRecord::decision_for_error(&GatewayError::SessionLimitExceeded {
                session_id: "s".into()
            }),
            "session_block"
        );
        assert_eq!(
            AuditRecord::decision_for_error(&GatewayError::InvalidRequest { reason: "x".into() }),
            "invalid_request"
        );
        assert_eq!(
            AuditRecord::decision_for_error(&GatewayError::CapabilityHashMismatch {
                namespaced_name: "n".into(),
                expected: "e".into(),
                got: "g".into()
            }),
            "deny"
        );
    }

    /// Concurrent appends must each stay one whole line. Before the single-write fix, the record and its
    /// newline were separate writes, so two threads could fuse records and drop one.
    #[test]
    fn concurrent_appends_never_fuse_or_drop_records() {
        let root = std::env::temp_dir().join(format!("swe-seed-audit-race-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let writer = std::sync::Arc::new(AuditWriter::new(
            &root,
            crate::gateway::redaction::builtin_redaction(),
        ));
        let (threads, each) = (8, 100);
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let w = writer.clone();
                std::thread::spawn(move || {
                    for n in 0..each {
                        let record: AuditRecord = serde_json::from_value(serde_json::json!({
                            "occurred_at": "2026-10-05T00:00:00+00:00",
                            "request_id": format!("{t}-{n}"),
                            "session_id": "s",
                            "client_id": "c",
                            "method": "tools/call",
                            "decision": "allow",
                        }))
                        .unwrap();
                        w.write(&record).unwrap();
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let content = std::fs::read_to_string(writer.file_path()).unwrap();
        let ids: std::collections::BTreeSet<String> = content
            .lines()
            .map(|l| {
                let v: serde_json::Value =
                    serde_json::from_str(l).unwrap_or_else(|e| panic!("fused or torn line: {e}: {l}"));
                v["request_id"].as_str().unwrap().to_string()
            })
            .collect();
        assert_eq!(ids.len(), threads * each, "an audit record was dropped");
        std::fs::remove_dir_all(&root).ok();
    }
}
