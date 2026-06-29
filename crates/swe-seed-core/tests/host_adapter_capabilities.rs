mod host_test_support;

use std::collections::BTreeMap;

use swe_seed_core::adapters::{
    adapter_for, all_host_ids, AdapterSupport, CanonicalHookEvent, FailureMode, HostId,
};

const ALL_EVENTS: &[CanonicalHookEvent] = &[
    CanonicalHookEvent::SessionStart,
    CanonicalHookEvent::SessionEnd,
    CanonicalHookEvent::UserPromptSubmit,
    CanonicalHookEvent::PreToolUse,
    CanonicalHookEvent::PermissionRequest,
    CanonicalHookEvent::PostToolUse,
    CanonicalHookEvent::ToolError,
    CanonicalHookEvent::SubagentStart,
    CanonicalHookEvent::SubagentStop,
    CanonicalHookEvent::Stop,
    CanonicalHookEvent::StopFailure,
    CanonicalHookEvent::PreCompact,
    CanonicalHookEvent::PostCompact,
    CanonicalHookEvent::Notification,
    CanonicalHookEvent::FileChanged,
    CanonicalHookEvent::CwdChanged,
];

#[test]
fn every_phase_9_host_exposes_capability_matrix() {
    // Given: the six host runtimes required by the phase 9 plan.
    let hosts = all_host_ids();

    // When: their capability matrices are queried.
    // Then: every required host is present with explicit pre-tool failure semantics.
    assert_eq!(
        hosts,
        vec![
            HostId::Claude,
            HostId::Codex,
            HostId::OpenCode,
            HostId::GithubCopilot,
            HostId::Antigravity,
            HostId::Ci,
        ]
    );
    for host in hosts {
        let adapter = adapter_for(host);
        assert_ne!(
            adapter.capabilities().pre_tool_failure_mode,
            FailureMode::Unknown,
            "{host} must not hide pre-tool failure behavior"
        );
    }
}

#[test]
fn antigravity_is_conservative_and_claude_is_richest() {
    // Given: the conservative and richest phase 9 adapters.
    let antigravity = adapter_for(HostId::Antigravity);
    let claude = adapter_for(HostId::Claude);

    // When: core hook support is inspected.
    let antigravity_caps = antigravity.capabilities();
    let claude_caps = claude.capabilities();

    // Then: Antigravity only claims the verified minimum; Claude claims the richer surface.
    assert!(antigravity_caps.can_block_pre_tool);
    assert!(!antigravity_caps.can_rewrite_tool_input);
    assert!(!antigravity_caps.has_subagent_hooks);
    assert!(claude_caps.can_block_pre_tool);
    assert!(claude_caps.can_inject_prompt_context);
    assert!(claude_caps.has_subagent_hooks);
    assert!(claude
        .support_for(CanonicalHookEvent::SubagentStart)
        .is_full());
    assert!(!antigravity
        .support_for(CanonicalHookEvent::SubagentStart)
        .is_full());
}

#[test]
fn projection_support_matches_adapter_support_for_all_supported_events() {
    // Given: every adapter and every canonical event.
    for host in all_host_ids() {
        let adapter = adapter_for(host);
        let plan = adapter.project();
        let projected: BTreeMap<&str, &AdapterSupport> = plan
            .support
            .iter()
            .map(|entry| (entry.event.as_str(), &entry.support))
            .collect();

        // When: the adapter reports Full or Partial support for an event.
        for event in ALL_EVENTS {
            let direct = adapter.support_for(*event);

            // Then: the projection metadata reports the same support class.
            match direct {
                AdapterSupport::Full | AdapterSupport::Partial { .. } => {
                    let projected_support = projected.get(event.as_str()).unwrap_or_else(|| {
                        panic!("{} missing projected support for {}", host, event.as_str())
                    });
                    assert_eq!(
                        std::mem::discriminant(*projected_support),
                        std::mem::discriminant(&direct),
                        "{} support mismatch for {}",
                        host,
                        event.as_str()
                    );
                }
                AdapterSupport::Unsupported { .. } => {}
            }
        }
    }
}

#[test]
fn host_id_serializes_with_runtime_wire_values() {
    // Given: HostId is serialized into public JSON reports and snapshots.
    let cases = [
        (HostId::Claude, "\"claude\""),
        (HostId::Codex, "\"codex\""),
        (HostId::OpenCode, "\"opencode\""),
        (HostId::GithubCopilot, "\"github-copilot\""),
        (HostId::Antigravity, "\"antigravity\""),
        (HostId::Ci, "\"ci\""),
    ];

    // When: each host id is serialized and deserialized.
    for (host, expected_json) in cases {
        let encoded = serde_json::to_string(&host).expect("serialize host id");
        let decoded: HostId = serde_json::from_str(expected_json).expect("deserialize host id");

        // Then: serde uses the same wire value as Display / FromStr.
        assert_eq!(encoded, expected_json);
        assert_eq!(decoded, host);
        assert_eq!(host.to_string(), expected_json.trim_matches('"'));
    }
}
