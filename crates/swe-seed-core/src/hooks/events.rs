use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CanonicalHookEvent {
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    PreToolUse,
    PermissionRequest,
    PostToolUse,
    ToolError,
    SubagentStart,
    SubagentStop,
    Stop,
    StopFailure,
    PreCompact,
    PostCompact,
    Notification,
    FileChanged,
    CwdChanged,
}

pub const ALL_CANONICAL_HOOK_EVENTS: &[CanonicalHookEvent] = &[
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

impl CanonicalHookEvent {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionStart => "SessionStart",
            Self::SessionEnd => "SessionEnd",
            Self::UserPromptSubmit => "UserPromptSubmit",
            Self::PreToolUse => "PreToolUse",
            Self::PermissionRequest => "PermissionRequest",
            Self::PostToolUse => "PostToolUse",
            Self::ToolError => "ToolError",
            Self::SubagentStart => "SubagentStart",
            Self::SubagentStop => "SubagentStop",
            Self::Stop => "Stop",
            Self::StopFailure => "StopFailure",
            Self::PreCompact => "PreCompact",
            Self::PostCompact => "PostCompact",
            Self::Notification => "Notification",
            Self::FileChanged => "FileChanged",
            Self::CwdChanged => "CwdChanged",
        }
    }
}

pub fn event_names(events: &[CanonicalHookEvent]) -> Vec<&'static str> {
    events.iter().map(|event| event.as_str()).collect()
}
