use serde_json::json;

use crate::hooks::{CanonicalHookEvent, ALL_CANONICAL_HOOK_EVENTS};

use super::capabilities::{FailureMode, HostCapabilities};
use super::host::{AdapterSupport, HostAdapter, HostId};
use super::projection::{ProjectedFile, ProjectionPlan};

pub struct GithubCopilotAdapter;

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
    CanonicalHookEvent::Notification,
];

impl HostAdapter for GithubCopilotAdapter {
    fn host_id(&self) -> HostId {
        HostId::GithubCopilot
    }

    fn capabilities(&self) -> HostCapabilities {
        HostCapabilities {
            can_block_pre_tool: true,
            can_rewrite_tool_input: false,
            can_modify_tool_result: false,
            can_inject_prompt_context: true,
            can_continue_on_stop: false,
            has_subagent_hooks: true,
            has_file_watch_hooks: false,
            pre_tool_failure_mode: FailureMode::RuntimeSpecific,
        }
    }

    fn support_for(&self, event: CanonicalHookEvent) -> AdapterSupport {
        if FULL.contains(&event) {
            AdapterSupport::Full
        } else {
            AdapterSupport::Partial {
                reason:
                    "GitHub Copilot cloud agents can weaken local filesystem and permission hooks"
                        .into(),
            }
        }
    }

    fn project(&self) -> ProjectionPlan {
        let value = json!({
            "swe_seed_managed": {"version": 1, "stable_id": "github-copilot-hooks"},
            "events": FULL.iter().map(|event| event.as_str()).collect::<Vec<_>>(),
            "cloud_agent_constraints": ["cloud execution may make local permissions partial"],
        });
        ProjectionPlan {
            host_id: self.host_id(),
            files: vec![ProjectedFile {
                rel_path: ".github/hooks/swe-seed.json".into(),
                content: serde_json::to_string_pretty(&value).unwrap_or_default(),
                owned_whole_file: false,
            }],
            support: super::support_entries(ALL_CANONICAL_HOOK_EVENTS, |event| {
                self.support_for(event)
            }),
        }
    }
}
