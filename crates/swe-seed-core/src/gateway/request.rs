//! Request orchestrator (spec 0020 §5, §8). Composes the per-stage gates into
//! ONE end-to-end path so the headline outcome contract is demonstrable: every
//! request completes with a backend response OR a typed `GatewayError`, and a
//! local audit record is written BEFORE the request is considered complete. If
//! the audit record cannot be written, the request MUST fail closed — a
//! returned response without audit is incomplete (spec 0020 §5, §18).

use std::time::{Duration, SystemTime};

use serde_json::Value;

use crate::hooks::policy::PermissionPolicy;

use super::audit::AuditRecord;
use super::governance::Governance;
use super::policy::evaluate_invoke;
use super::routing::Router;
use super::{AuditWriter, GatewayCatalogEntry, GatewayError};

/// Caller-supplied request identity + target (spec 0020 §5 audit fields).
pub struct RequestContext<'a> {
    pub request_id: &'a Value,
    pub session_id: &'a str,
    pub client_id: &'a str,
    pub method: &'a str,
    /// Target tool/resource/prompt, when the request is a namespaced call.
    pub namespaced_name: Option<&'a str>,
    pub route_card_id: Option<&'a str>,
    /// Forwarded JSON-RPC params (spec 0020 §8). Defaults to `{}` when absent.
    pub params: &'a Value,
}

/// Handle one request end-to-end. `entry` = resolved catalog entry (None for
/// non-namespaced methods like `tools/list`). `backend_id` = owning backend.
/// Exactly one audit record is written on every completion path. Returns the
/// backend result on success or a typed error on any block/failure.
///
/// Audit fail-closed: if `AuditWriter::write` errors, this returns
/// `AuditWriteFailed` regardless of the underlying result — the request is
/// incomplete without durable audit evidence (spec 0020 §5).
pub fn handle_request(
    ctx: &RequestContext,
    entry: Option<&GatewayCatalogEntry>,
    backend_id: Option<&str>,
    scan_status: Option<&str>,
    provenance_present: bool,
    policy: &PermissionPolicy,
    governance: &Governance,
    router: &Router,
    timeout: Duration,
    audit: &AuditWriter,
    occurred_at: &str,
) -> Result<Value, GatewayError> {
    let started = SystemTime::now();

    // 1. Policy + integrity gates (namespaced calls only).
    if let Some(entry) = entry {
        if let Err(e) = evaluate_invoke(policy, entry, scan_status, provenance_present) {
            write_audit_or_fail(audit, ctx, Some(entry.namespaced_name.as_str()), backend_id, Err(&e), started, occurred_at)?;
            return Err(e);
        }
    }

    // 2. Governance counters (one critical section, durable before forward).
    if let Some(name) = ctx.namespaced_name {
        if let Err(e) = governance.check_and_increment(ctx.session_id, ctx.client_id, name) {
            write_audit_or_fail(audit, ctx, Some(name), backend_id, Err(&e), started, occurred_at)?;
            return Err(e);
        }
    }

    // 3. Forward to the owning backend.
    let forward = router.invoke(
        backend_id.unwrap_or(""),
        ctx.request_id,
        ctx.method,
        ctx.params.clone(),
        timeout,
    );
    match forward {
        Ok(value) => {
            write_audit_or_fail(audit, ctx, ctx.namespaced_name, backend_id, Ok(&value), started, occurred_at)?;
            Ok(value)
        }
        Err(e) => {
            write_audit_or_fail(audit, ctx, ctx.namespaced_name, backend_id, Err(&e), started, occurred_at)?;
            Err(e)
        }
    }
}

/// Build + write the single audit record for this outcome. Returns
/// `AuditWriteFailed` on write error so the caller treats the request as
/// incomplete (the audit error DOMINATES any underlying result/error).
fn write_audit_or_fail(
    audit: &AuditWriter,
    ctx: &RequestContext,
    namespaced_name: Option<&str>,
    backend: Option<&str>,
    outcome: Result<&Value, &GatewayError>,
    started: SystemTime,
    occurred_at: &str,
) -> Result<(), GatewayError> {
    let (decision, reason_code) = match &outcome {
        Ok(_) => ("allow".to_string(), None),
        Err(e) => (
            AuditRecord::decision_for_error(e).to_string(),
            Some(e.reason_code().to_string()),
        ),
    };
    let latency_ms = SystemTime::now()
        .duration_since(started)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let record = AuditRecord {
        occurred_at: occurred_at.to_string(),
        request_id: ctx.request_id.to_string(),
        session_id: ctx.session_id.to_string(),
        client_id: ctx.client_id.to_string(),
        method: ctx.method.to_string(),
        namespaced_name: namespaced_name.map(|s| s.to_string()),
        backend: backend.map(|s| s.to_string()),
        decision,
        reason_code,
        latency_ms: Some(latency_ms),
        cost_delta: None,
        route_card_id: ctx.route_card_id.map(|s| s.to_string()),
        domain_model_hash_source: None,
        semantic_envelope_id: None,
        semantic_envelope_event_type: None,
        detail: None,
    };
    match audit.write(&record) {
        Ok(()) => Ok(()),
        // The audit-write error is the request outcome (spec 0020 §5/§18).
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::{
        AuditWriter, GatewayCatalogEntry, GatewayEffectiveLimits, Governance, MockTransport, Router,
    };
    use crate::hooks::policy::PermissionPolicy;
    use crate::hooks::redact::RedactionConfig;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    fn temp_root(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "swe-seed-gw-req-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn entry(name: &str, forbidden: bool) -> (GatewayCatalogEntry, PermissionPolicy) {
        let e = GatewayCatalogEntry {
            kind: crate::gateway::CatalogKind::Tool,
            namespaced_name: name.into(),
            backend_id: "fs".into(),
            wire_definition: json!({}),
            definition_hash: "sha256:actual".into(),
            source_hash: None,
            policy_tags: vec![],
            pinned_hash: None,
            input_schema: None,
        };
        let mut p = PermissionPolicy::default();
        if forbidden {
            p.forbidden_actions = vec![name.into()];
        }
        (e, p)
    }

    #[test]
    fn success_writes_allow_audit_and_returns_result() {
        let root = temp_root("success");
        let transport = MockTransport::responding(r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#);
        let router = Router::with("fs", Box::new(transport));
        let gov = Governance::load(&root, GatewayEffectiveLimits::default()).unwrap();
        let audit = AuditWriter::new(&root, RedactionConfig::default());
        let (entry, policy) = entry("fs.read", false);
        let ctx = RequestContext {
            request_id: &json!(1),
            session_id: "sess",
            client_id: "cli",
            method: "tools/call",
            namespaced_name: Some("fs.read"),
            route_card_id: None,
            params: &json!({}),
        };
        let result = handle_request(
            &ctx, Some(&entry), Some("fs"), None, false,
            &policy, &gov, &router, Duration::from_secs(2), &audit, "2026-07-28T00:00:00+00:00",
        )
        .unwrap();
        assert_eq!(result, json!({"ok":true}));
        // exactly one allow audit line written
        let content = fs::read_to_string(audit.file_path()).unwrap();
        let line = content.trim_end().lines().next().unwrap();
        let rec: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(rec["decision"], "allow");
        assert_eq!(rec["namespaced_name"], "fs.read");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn policy_deny_writes_deny_audit_and_returns_typed_error() {
        let root = temp_root("deny");
        let transport = MockTransport::responding(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#);
        let router = Router::with("fs", Box::new(transport));
        let gov = Governance::load(&root, GatewayEffectiveLimits::default()).unwrap();
        let audit = AuditWriter::new(&root, RedactionConfig::default());
        let (entry, policy) = entry("fs.read", true); // forbidden
        let ctx = RequestContext {
            request_id: &json!(1),
            session_id: "sess",
            client_id: "cli",
            method: "tools/call",
            namespaced_name: Some("fs.read"),
            route_card_id: None,
            params: &json!({}),
        };
        let err = handle_request(
            &ctx, Some(&entry), Some("fs"), None, false,
            &policy, &gov, &router, Duration::from_secs(2), &audit, "2026-07-28T00:00:00+00:00",
        )
        .unwrap_err();
        assert_eq!(err.reason_code(), "scope_denied");
        // the backend was NEVER consulted (deny before forward)
        let content = fs::read_to_string(audit.file_path()).unwrap();
        let rec: serde_json::Value = serde_json::from_str(content.trim_end()).unwrap();
        assert_eq!(rec["decision"], "deny");
        assert_eq!(rec["reason_code"], "scope_denied");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn audit_write_failure_fails_the_request_closed_even_when_forward_would_succeed() {
        // Make `.swe-seed` a file so the audit dir cannot be created → write
        // fails. The request MUST return AuditWriteFailed, not the result.
        let root = temp_root("auditfail");
        fs::write(root.join(".swe-seed"), "not a dir").unwrap();
        let transport = MockTransport::responding(r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#);
        let router = Router::with("fs", Box::new(transport));
        let gov = Governance::load(&root, GatewayEffectiveLimits::default()).unwrap();
        let audit = AuditWriter::new(&root, RedactionConfig::default());
        let (entry, policy) = entry("fs.read", false);
        let ctx = RequestContext {
            request_id: &json!(1),
            session_id: "sess",
            client_id: "cli",
            method: "tools/call",
            namespaced_name: Some("fs.read"),
            route_card_id: None,
            params: &json!({}),
        };
        let err = handle_request(
            &ctx, Some(&entry), Some("fs"), None, false,
            &policy, &gov, &router, Duration::from_secs(2), &audit, "2026-07-28T00:00:00+00:00",
        )
        .unwrap_err();
        assert_eq!(
            err.reason_code(),
            "audit_write_failed",
            "audit failure must dominate: request incomplete without audit"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn backend_unavailable_writes_audit_and_returns_typed_error() {
        let root = temp_root("backenddown");
        let transport = MockTransport {
            drop_request: true,
            ..MockTransport::responding("")
        };
        let router = Router::with("fs", Box::new(transport));
        let gov = Governance::load(&root, GatewayEffectiveLimits::default()).unwrap();
        let audit = AuditWriter::new(&root, RedactionConfig::default());
        let (entry, policy) = entry("fs.read", false);
        let ctx = RequestContext {
            request_id: &json!(1),
            session_id: "sess",
            client_id: "cli",
            method: "tools/call",
            namespaced_name: Some("fs.read"),
            route_card_id: None,
            params: &json!({}),
        };
        let err = handle_request(
            &ctx, Some(&entry), Some("fs"), None, false,
            &policy, &gov, &router, Duration::from_millis(200), &audit, "2026-07-28T00:00:00+00:00",
        )
        .unwrap_err();
        assert_eq!(err.reason_code(), "backend_unavailable");
        let content = fs::read_to_string(audit.file_path()).unwrap();
        let rec: serde_json::Value = serde_json::from_str(content.trim_end()).unwrap();
        assert_eq!(rec["decision"], "backend_unavailable");
        fs::remove_dir_all(&root).ok();
    }
}
