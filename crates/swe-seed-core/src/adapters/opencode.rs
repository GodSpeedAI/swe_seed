use crate::hooks::{CanonicalHookEvent, ALL_CANONICAL_HOOK_EVENTS};

use super::capabilities::{FailureMode, HostCapabilities};
use super::host::{AdapterSupport, HostAdapter, HostId};
use super::marker;
use super::projection::{ProjectedFile, ProjectionPlan};

pub struct OpenCodeAdapter;

const FULL: &[(CanonicalHookEvent, &str)] = &[
    (CanonicalHookEvent::PreToolUse, "tool.execute.before"),
    (CanonicalHookEvent::PostToolUse, "tool.execute.after"),
    (CanonicalHookEvent::PermissionRequest, "permission.asked"),
    (CanonicalHookEvent::Stop, "session.idle"),
    (CanonicalHookEvent::StopFailure, "session.error"),
    (CanonicalHookEvent::PostCompact, "session.compacted"),
    (CanonicalHookEvent::FileChanged, "file.edited"),
];

impl HostAdapter for OpenCodeAdapter {
    fn host_id(&self) -> HostId {
        HostId::OpenCode
    }

    fn capabilities(&self) -> HostCapabilities {
        HostCapabilities {
            can_block_pre_tool: true,
            can_rewrite_tool_input: true,
            can_modify_tool_result: true,
            can_inject_prompt_context: false,
            can_continue_on_stop: false,
            has_subagent_hooks: false,
            has_file_watch_hooks: true,
            pre_tool_failure_mode: FailureMode::RuntimeSpecific,
        }
    }

    fn support_for(&self, event: CanonicalHookEvent) -> AdapterSupport {
        if FULL.iter().any(|(candidate, _)| *candidate == event) {
            AdapterSupport::Full
        } else {
            AdapterSupport::Partial {
                reason: "OpenCode plugin projection has no verified native mapping for this event"
                    .into(),
            }
        }
    }

    fn project(&self) -> ProjectionPlan {
        let mappings = FULL
            .iter()
            .map(|(event, native)| format!("  ['{}', '{}'],", native, event.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        let body = format!(
            "const mappings = new Map<string, string>([\n{mappings}\n]);\nexport default async function sweSeed(event: string, payload: unknown) {{\n  const canonical = mappings.get(event);\n  if (!canonical) return;\n  // PreToolUse enforces the routing gate (blocks unrouted tool use on an active trace).\n  if (canonical === 'PreToolUse') {{ await $`swe-seed agent-hooks route-gate`; return; }}\n  await $`swe-seed agent-hooks capture ${{canonical}}`;\n}}\n"
        );
        ProjectionPlan {
            host_id: self.host_id(),
            files: vec![ProjectedFile {
                rel_path: ".opencode/plugins/swe_seed.ts".into(),
                content: marker::block("opencode-plugin", &body),
                owned_whole_file: true,
            }],
            support: super::support_entries(ALL_CANONICAL_HOOK_EVENTS, |event| {
                self.support_for(event)
            }),
        }
    }
}
