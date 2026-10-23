//! Permission, approval, scan, provenance, and call-time hash gates
//! (spec 0020 §8, §10). These run BEFORE any backend forwarding (stage 5) and
//! reuse the Harness `PermissionPolicy` deny-overrides-allow semantics so the
//! gateway is a faithful `PreToolUse` fallback (§10). Every blocked path returns
//! a typed `GatewayError` carrying a stable `reason_code` for audit (§5).
//!
//! Gate order is fail-closed for risky capabilities (§10): a dangerous,
//! unscanned or unproven tool is blocked before an approval- or hash-level
//! outcome is considered.

use crate::hooks::policy::{gate_action, ActionGate, PermissionPolicy};

use super::{GatewayCatalogEntry, GatewayError};

/// The non-denial outcome of an invocation dry-run: clean allow, or
/// approval-required (the caller must supply approval before forwarding).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvokeDecision {
    Allow,
    ApprovalRequired { namespaced_name: String },
}

/// Tags that mark a capability risky enough to require scan + provenance
/// fail-closed (spec 0020 §10).
/// ponytail: fixed set; upgrade path is a config-driven `risky_tags` list if
/// operators need per-project risk taxonomy.
const RISKY_TAGS: &[&str] = &[
    "dangerous",
    "destructive",
    "write",
    "exec",
    "shell",
    "network",
    "untrusted",
];

pub fn is_risky(policy_tags: &[String]) -> bool {
    policy_tags
        .iter()
        .any(|t| RISKY_TAGS.contains(&t.as_str()))
}

/// True iff `scan_status` is present and `clear` (mirrors the
/// `ProvenanceRecord.license_status` vocabulary).
pub fn scan_is_clear(scan_status: Option<&str>) -> bool {
    scan_status == Some("clear")
}

/// Evaluate an invocation against policy + integrity gates WITHOUT forwarding.
/// This is the `swe_seed_invoke` dry-run path (spec 0020 §8). Denials are
/// returned as typed `GatewayError`s.
///
/// `scan_status` = the backend's scan status (`Some("clear")` to pass).
/// `provenance_present` = whether provenance exists for this capability.
pub fn evaluate_invoke(
    policy: &PermissionPolicy,
    entry: &GatewayCatalogEntry,
    scan_status: Option<&str>,
    provenance_present: bool,
) -> Result<InvokeDecision, GatewayError> {
    // 1. PermissionPolicy — deny overrides allow. The action is the namespaced
    //    name; any policy_tag that is forbidden also blocks (tag-level deny).
    if gate_action(policy, &entry.namespaced_name) == ActionGate::Forbidden {
        return Err(GatewayError::ScopeDenied {
            scope: entry.namespaced_name.clone(),
        });
    }
    for tag in &entry.policy_tags {
        if gate_action(policy, tag) == ActionGate::Forbidden {
            return Err(GatewayError::ScopeDenied {
                scope: format!("{} (tag:{tag})", entry.namespaced_name),
            });
        }
    }

    // 2. Scan + provenance fail-closed for risky capabilities (spec 0020 §10).
    if is_risky(&entry.policy_tags) {
        if !scan_is_clear(scan_status) {
            return Err(GatewayError::ScanBlocked {
                namespaced_name: entry.namespaced_name.clone(),
                reason: "risky capability not scanned clear".into(),
            });
        }
        if !provenance_present {
            return Err(GatewayError::ProvenanceMissing {
                namespaced_name: entry.namespaced_name.clone(),
            });
        }
    }

    // 3. Call-time pinned-hash verification (spec 0020 §7, §19).
    if let Some(pinned) = &entry.pinned_hash {
        if pinned != &entry.definition_hash {
            return Err(GatewayError::CapabilityHashMismatch {
                namespaced_name: entry.namespaced_name.clone(),
                expected: pinned.clone(),
                got: entry.definition_hash.clone(),
            });
        }
    }

    // 4. Approval gate (spec 0020 §10): dangerous tool groups require explicit
    //    approval. Approval is necessary-but-not-sufficient — a later scan or
    //    hash block still wins (checked above).
    if gate_action(policy, &entry.namespaced_name) == ActionGate::ApprovalGated
        || entry
            .policy_tags
            .iter()
            .any(|t| gate_action(policy, t) == ActionGate::ApprovalGated)
    {
        return Ok(InvokeDecision::ApprovalRequired {
            namespaced_name: entry.namespaced_name.clone(),
        });
    }

    Ok(InvokeDecision::Allow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::CatalogKind;
    use serde_json::json;

    fn entry(name: &str, tags: &[&str], pinned: Option<&str>) -> GatewayCatalogEntry {
        GatewayCatalogEntry {
            kind: CatalogKind::Tool,
            namespaced_name: name.into(),
            backend_id: "fs".into(),
            wire_definition: json!({}),
            definition_hash: "sha256:actual".into(),
            source_hash: None,
            policy_tags: tags.iter().map(|s| s.to_string()).collect(),
            pinned_hash: pinned.map(|p| p.to_string()),
            input_schema: None,
        }
    }

    fn policy() -> PermissionPolicy {
        PermissionPolicy::default()
    }

    #[test]
    fn readonly_tool_allows() {
        let e = entry("fs.read", &["readonly"], None);
        assert_eq!(
            evaluate_invoke(&policy(), &e, None, false).unwrap(),
            InvokeDecision::Allow
        );
    }

    #[test]
    fn forbidden_action_denies_with_scope_denied() {
        let mut p = policy();
        p.forbidden_actions = vec!["fs.delete".into()];
        let e = entry("fs.delete", &["destructive"], None);
        let err = evaluate_invoke(&p, &e, Some("clear"), true).unwrap_err();
        assert_eq!(err.reason_code(), "scope_denied");
    }

    #[test]
    fn forbidden_tag_denies() {
        let mut p = policy();
        p.forbidden_actions = vec!["dangerous".into()];
        let e = entry("fs.x", &["dangerous"], None);
        let err = evaluate_invoke(&p, &e, Some("clear"), true).unwrap_err();
        assert_eq!(err.reason_code(), "scope_denied");
        assert!(err.to_string().contains("tag:dangerous"));
    }

    #[test]
    fn risky_unscanned_fail_closed() {
        let e = entry("fs.write", &["write"], None);
        let err = evaluate_invoke(&policy(), &e, None, true).unwrap_err();
        assert_eq!(err.reason_code(), "scan_blocked");
    }

    #[test]
    fn risky_scanned_but_unproven_fail_closed() {
        let e = entry("fs.write", &["write"], None);
        let err = evaluate_invoke(&policy(), &e, Some("clear"), false).unwrap_err();
        assert_eq!(err.reason_code(), "provenance_missing");
    }

    #[test]
    fn risky_scanned_and_proven_allows() {
        let e = entry("fs.write", &["write"], None);
        assert_eq!(
            evaluate_invoke(&policy(), &e, Some("clear"), true).unwrap(),
            InvokeDecision::Allow
        );
    }

    #[test]
    fn pinned_hash_mismatch_blocks() {
        let e = entry("fs.read", &["readonly"], Some("sha256:different"));
        let err = evaluate_invoke(&policy(), &e, None, false).unwrap_err();
        assert_eq!(err.reason_code(), "capability_hash_mismatch");
    }

    #[test]
    fn pinned_hash_match_allows() {
        let e = entry("fs.read", &["readonly"], Some("sha256:actual"));
        assert_eq!(
            evaluate_invoke(&policy(), &e, None, false).unwrap(),
            InvokeDecision::Allow
        );
    }

    #[test]
    fn approval_gated_returns_approval_required() {
        let mut p = policy();
        p.approval_gated_actions = vec!["fs.deploy".into()];
        let e = entry("fs.deploy", &["deploy"], None);
        assert_eq!(
            evaluate_invoke(&p, &e, Some("clear"), true).unwrap(),
            InvokeDecision::ApprovalRequired {
                namespaced_name: "fs.deploy".into()
            }
        );
    }

    #[test]
    fn deny_overrides_approval_tag() {
        // A tag both approval-gated (via tag) and forbidden: forbidden wins.
        let mut p = policy();
        p.forbidden_actions = vec!["dangerous".into()];
        p.approval_gated_actions = vec!["dangerous".into()];
        let e = entry("fs.x", &["dangerous"], None);
        let err = evaluate_invoke(&p, &e, Some("clear"), true).unwrap_err();
        assert_eq!(err.reason_code(), "scope_denied");
    }
}
