//! pretooluse_gate_projection: the host adapters project the ENFORCING
//! route-gate as the PreToolUse hook (not the logging capture), so tool use on
//! an active, unrouted trace is blocked at the host. Other events still capture.

use swe_seed_core::adapters::{hook_command_for, project_host, CanonicalHookEvent, HostId};

#[test]
fn pretooluse_projects_the_enforcing_route_gate() {
    assert_eq!(
        hook_command_for(CanonicalHookEvent::PreToolUse),
        "swe-seed agent-hooks route-gate",
        "PreToolUse must project the enforcing gate, not the logging capture"
    );
    // Other events still capture (logging).
    for event in [
        CanonicalHookEvent::SessionStart,
        CanonicalHookEvent::PostToolUse,
        CanonicalHookEvent::Stop,
    ] {
        let cmd = hook_command_for(event);
        assert!(
            cmd.starts_with("swe-seed agent-hooks capture "),
            "{event:?} should capture, got {cmd}"
        );
    }
}

#[test]
fn claude_projection_contains_route_gate_for_pretooluse() {
    // The projected Claude settings JSON must carry the gate command on PreToolUse.
    let plan = project_host(HostId::Claude);
    let blob = serde_json::to_string(&plan).unwrap_or_default();
    assert!(
        blob.contains("swe-seed agent-hooks route-gate"),
        "claude projection missing route-gate: {blob}"
    );
}

#[test]
fn antigravity_projection_uses_route_gate() {
    let plan = project_host(HostId::Antigravity);
    let blob = serde_json::to_string(&plan).unwrap_or_default();
    assert!(
        blob.contains("swe-seed agent-hooks route-gate"),
        "antigravity projection missing route-gate: {blob}"
    );
}
