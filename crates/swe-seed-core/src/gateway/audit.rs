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
}

impl AuditWriter {
    pub fn new(root: &Path, redaction: RedactionConfig) -> Self {
        Self {
            dir: root.join(AUDIT_DIR_REL),
            redaction,
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
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| GatewayError::AuditWriteFailed {
                reason: format!("open {}: {e}", path.display()),
            })?;
        use std::io::Write;
        let mut file = file;
        writeln!(file, "{line}").map_err(|e| GatewayError::AuditWriteFailed {
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
}
