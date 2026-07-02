use serde_json::json;

use crate::hooks::CanonicalHookEvent;

use super::capabilities::{FailureMode, HostCapabilities};
use super::host::{AdapterSupport, HostAdapter, HostId};
use super::projection::{EventSupport, ProjectedFile, ProjectionPlan};

pub struct ClaudeAdapter;

const FULL: &[CanonicalHookEvent] = &[
    CanonicalHookEvent::SessionStart,
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

impl HostAdapter for ClaudeAdapter {
    fn host_id(&self) -> HostId {
        HostId::Claude
    }

    fn capabilities(&self) -> HostCapabilities {
        HostCapabilities {
            can_block_pre_tool: true,
            can_rewrite_tool_input: true,
            can_modify_tool_result: true,
            can_inject_prompt_context: true,
            can_continue_on_stop: true,
            has_subagent_hooks: true,
            has_file_watch_hooks: true,
            pre_tool_failure_mode: FailureMode::FailClosed,
        }
    }

    fn support_for(&self, event: CanonicalHookEvent) -> AdapterSupport {
        if FULL.contains(&event) {
            AdapterSupport::Full
        } else {
            AdapterSupport::Unsupported {
                reason: "Claude projection has no native mapping for this canonical event".into(),
            }
        }
    }

    fn project(&self) -> ProjectionPlan {
        let settings = json!({
            "swe_seed_managed": {"version": 1, "stable_id": "claude-hooks"},
            "hooks": FULL.iter().map(|event| json!({
                "event": event.as_str(),
                "command": super::hook_command_for(*event)
            })).collect::<Vec<_>>()
        });
        let local = json!({
            "swe_seed_managed": {"version": 1, "stable_id": "claude-local-hooks"},
            "local_overrides": {"enabled": true}
        });
        ProjectionPlan {
            host_id: self.host_id(),
            files: vec![
                ProjectedFile {
                    rel_path: ".claude/settings.json".into(),
                    content: serde_json::to_string_pretty(&settings).unwrap_or_default(),
                    owned_whole_file: false,
                },
                ProjectedFile {
                    rel_path: ".claude/settings.local.json".into(),
                    content: serde_json::to_string_pretty(&local).unwrap_or_default(),
                    owned_whole_file: false,
                },
            ],
            support: FULL
                .iter()
                .map(|event| EventSupport {
                    event: event.as_str().into(),
                    support: AdapterSupport::Full,
                })
                .collect(),
        }
    }
}
