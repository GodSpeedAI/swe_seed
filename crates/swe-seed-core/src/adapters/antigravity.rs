use serde_json::json;

use crate::hooks::{CanonicalHookEvent, ALL_CANONICAL_HOOK_EVENTS};

use super::capabilities::HostCapabilities;
use super::host::{AdapterSupport, HostAdapter, HostId};
use super::projection::{ProjectedFile, ProjectionPlan};

pub struct AntigravityAdapter;

impl HostAdapter for AntigravityAdapter {
    fn host_id(&self) -> HostId {
        HostId::Antigravity
    }

    fn capabilities(&self) -> HostCapabilities {
        HostCapabilities::command_hooks()
    }

    fn support_for(&self, event: CanonicalHookEvent) -> AdapterSupport {
        match event {
            CanonicalHookEvent::PreToolUse => AdapterSupport::Full,
            CanonicalHookEvent::Stop => AdapterSupport::Partial {
                reason: "Antigravity hook schema is not stable enough to enforce Stop hooks".into(),
            },
            _ => AdapterSupport::Unsupported {
                reason: "Antigravity projection starts with the verified PreToolUse minimum".into(),
            },
        }
    }

    fn project(&self) -> ProjectionPlan {
        let value = json!({
            "swe_seed_managed": {"version": 1, "stable_id": "antigravity-hooks"},
            "hooks": [{
                "event": "PreToolUse",
                "matchers": ["run_command", "shell", "bash", "write", "edit"],
                "command": "swe-seed agent-hooks capture PreToolUse"
            }]
        });
        ProjectionPlan {
            host_id: self.host_id(),
            files: vec![ProjectedFile {
                rel_path: ".agents/hooks.json".into(),
                content: serde_json::to_string_pretty(&value).unwrap_or_default(),
                owned_whole_file: false,
            }],
            support: super::support_entries(ALL_CANONICAL_HOOK_EVENTS, |event| {
                self.support_for(event)
            }),
        }
    }
}
