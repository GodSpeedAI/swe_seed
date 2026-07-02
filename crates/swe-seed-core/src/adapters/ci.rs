use crate::hooks::{CanonicalHookEvent, ALL_CANONICAL_HOOK_EVENTS};

use super::capabilities::{FailureMode, HostCapabilities};
use super::host::{AdapterSupport, HostAdapter, HostId};
use super::marker;
use super::projection::{ProjectedFile, ProjectionPlan};

pub struct CiAdapter;

impl HostAdapter for CiAdapter {
    fn host_id(&self) -> HostId {
        HostId::Ci
    }

    fn capabilities(&self) -> HostCapabilities {
        HostCapabilities {
            can_block_pre_tool: true,
            can_rewrite_tool_input: false,
            can_modify_tool_result: false,
            can_inject_prompt_context: false,
            can_continue_on_stop: false,
            has_subagent_hooks: false,
            has_file_watch_hooks: false,
            pre_tool_failure_mode: FailureMode::FailClosed,
        }
    }

    fn support_for(&self, event: CanonicalHookEvent) -> AdapterSupport {
        match event {
            CanonicalHookEvent::PreToolUse | CanonicalHookEvent::PostToolUse => {
                AdapterSupport::Partial {
                    reason: "CI enforces policy after the fact, not inside the interactive runtime"
                        .into(),
                }
            }
            _ => AdapterSupport::Unsupported {
                reason: "CI is a non-interactive enforcement backstop".into(),
            },
        }
    }

    fn project(&self) -> ProjectionPlan {
        let sh = marker::block(
            "ci-shell-policy",
            "set -eu\nswe-seed doctor --host all\nswe-seed harness\n",
        );
        let ps1 = marker::block(
            "ci-powershell-policy",
            "$ErrorActionPreference = 'Stop'\nswe-seed doctor --host all\nswe-seed harness\n",
        );
        let workflow = marker::block(
            "ci-github-workflow",
            "name: swe-seed-policy\non: [push, pull_request]\njobs:\n  swe-seed-policy:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: bash .agents/ci/swe-seed-policy-check.sh\n",
        );
        ProjectionPlan {
            host_id: self.host_id(),
            files: vec![
                ProjectedFile {
                    rel_path: ".agents/ci/swe-seed-policy-check.sh".into(),
                    content: sh,
                    owned_whole_file: true,
                },
                ProjectedFile {
                    rel_path: ".agents/ci/swe-seed-policy-check.ps1".into(),
                    content: ps1,
                    owned_whole_file: true,
                },
                ProjectedFile {
                    rel_path: ".github/workflows/swe-seed-policy.yml".into(),
                    content: workflow,
                    owned_whole_file: true,
                },
            ],
            support: super::support_entries(ALL_CANONICAL_HOOK_EVENTS, |event| {
                self.support_for(event)
            }),
        }
    }
}
