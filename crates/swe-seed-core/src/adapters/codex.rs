use serde_json::json;

use crate::hooks::{CanonicalHookEvent, ALL_CANONICAL_HOOK_EVENTS};

use super::capabilities::HostCapabilities;
use super::host::{AdapterSupport, HostAdapter, HostId};
use super::projection::{ProjectedFile, ProjectionPlan};

pub struct CodexAdapter;

const FULL: &[CanonicalHookEvent] = &[
    CanonicalHookEvent::SessionStart,
    CanonicalHookEvent::PreToolUse,
    CanonicalHookEvent::PermissionRequest,
    CanonicalHookEvent::PostToolUse,
    CanonicalHookEvent::SubagentStart,
    CanonicalHookEvent::SubagentStop,
    CanonicalHookEvent::Stop,
    CanonicalHookEvent::PreCompact,
    CanonicalHookEvent::PostCompact,
];

impl HostAdapter for CodexAdapter {
    fn host_id(&self) -> HostId {
        HostId::Codex
    }

    fn capabilities(&self) -> HostCapabilities {
        HostCapabilities::command_hooks()
    }

    fn support_for(&self, event: CanonicalHookEvent) -> AdapterSupport {
        if FULL.contains(&event) {
            AdapterSupport::Full
        } else {
            AdapterSupport::Partial {
                reason: "Codex CLI projection is limited to command hooks".into(),
            }
        }
    }

    fn project(&self) -> ProjectionPlan {
        let partial = ["UserPromptSubmit: partial command-hook coverage only"];
        let value = json!({
            "swe_seed_managed": {"version": 1, "stable_id": "codex-hooks"},
            "hooks": FULL.iter().map(|event| json!({
                "event": event.as_str(),
                "command": format!("swe-seed agent-hooks capture {}", event.as_str())
            })).collect::<Vec<_>>(),
            "partial_support": partial,
        });
        ProjectionPlan {
            host_id: self.host_id(),
            files: vec![ProjectedFile {
                rel_path: ".codex/hooks.json".into(),
                content: serde_json::to_string_pretty(&value).unwrap_or_default(),
                owned_whole_file: false,
            }],
            support: super::support_entries(ALL_CANONICAL_HOOK_EVENTS, |event| {
                self.support_for(event)
            }),
        }
    }
}
