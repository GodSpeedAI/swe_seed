use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureMode {
    FailOpen,
    FailClosed,
    RuntimeSpecific,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostCapabilities {
    pub can_block_pre_tool: bool,
    pub can_rewrite_tool_input: bool,
    pub can_modify_tool_result: bool,
    pub can_inject_prompt_context: bool,
    pub can_continue_on_stop: bool,
    pub has_subagent_hooks: bool,
    pub has_file_watch_hooks: bool,
    pub pre_tool_failure_mode: FailureMode,
}

impl HostCapabilities {
    pub const fn command_hooks() -> Self {
        Self {
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
}
