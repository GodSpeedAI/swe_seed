//! permission_gate: a forbidden action is blocked; an approval-gated action
//! requires approval; forbidden wins over approval-gated (spec 0005).

use swe_seed_core::hooks::{gate_action, ActionGate, PermissionPolicy, REQUIRED_LIFECYCLE_EVENTS};

fn policy(allowed: &[&str], gated: &[&str], forbidden: &[&str]) -> PermissionPolicy {
    PermissionPolicy {
        id: "p".into(),
        allowed_actions: allowed.iter().map(|s| s.to_string()).collect(),
        approval_gated_actions: gated.iter().map(|s| s.to_string()).collect(),
        forbidden_actions: forbidden.iter().map(|s| s.to_string()).collect(),
        validation: vec![],
    }
}

#[test]
fn forbidden_action_is_blocked() {
    let p = policy(&[], &[], &["force-push"]);
    assert_eq!(gate_action(&p, "force-push"), ActionGate::Forbidden);
}

#[test]
fn approval_gated_action_requires_approval() {
    let p = policy(&[], &["deploy"], &[]);
    assert_eq!(gate_action(&p, "deploy"), ActionGate::ApprovalGated);
}

#[test]
fn unlisted_action_is_allowed_by_default() {
    let p = policy(&["read"], &[], &[]);
    assert_eq!(gate_action(&p, "read"), ActionGate::Allowed);
    // Not listed anywhere → allowed (per spec, unlisted defaults to allowed).
    assert_eq!(gate_action(&p, "something-new"), ActionGate::Allowed);
}

#[test]
fn forbidden_takes_precedence_over_approval_gated() {
    // An action in both lists must be blocked, not approval-gated.
    let p = policy(&[], &["x"], &["x"]);
    assert_eq!(gate_action(&p, "x"), ActionGate::Forbidden);
}

#[test]
fn five_required_lifecycle_events_present() {
    // The 5 v0.1 lifecycle events (spec 0005) that Phase 9 projects to hosts.
    assert_eq!(REQUIRED_LIFECYCLE_EVENTS.len(), 5);
    for ev in ["SessionStart", "ContextBuild", "PreToolUse", "PostToolUse", "TaskEnd"] {
        assert!(
            REQUIRED_LIFECYCLE_EVENTS.iter().any(|e| *e == ev),
            "missing required lifecycle event {ev}"
        );
    }
}
