//! `HookPolicy` / `PermissionPolicy` (harness.baml) + the permission gate, and
//! the 5 required v0.1 lifecycle events (spec 0005).

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};

/// The five lifecycle events v0.1 supports end-to-end (spec 0005). Phase 9
/// projects these into host hook config.
pub const REQUIRED_LIFECYCLE_EVENTS: &[&str] = &[
    "SessionStart",
    "ContextBuild",
    "PreToolUse",
    "PostToolUse",
    "TaskEnd",
];

/// harness.baml `ValidationRequirement` (hooks-local copy).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookValidation {
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub blocking: bool,
    #[serde(default)]
    pub evidence: String,
}

/// harness.baml `HookPolicy`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPolicy {
    pub id: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub blocking_rules: Vec<String>,
    #[serde(default)]
    pub advisory_rules: Vec<String>,
    #[serde(default)]
    pub payload_fields: Vec<String>,
    #[serde(default)]
    pub failure_behavior: String,
    #[serde(default)]
    pub validation: Vec<HookValidation>,
}

impl BamlParity for HookPolicy {
    fn baml_name() -> &'static str {
        "HookPolicy"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "events",
                "blocking_rules",
                "advisory_rules",
                "payload_fields",
                "failure_behavior",
                "validation",
            ],
            field_types: vec![
                "string",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string",
                "ValidationRequirement[]",
            ],
        }
    }
}

/// harness.baml `PermissionPolicy`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionPolicy {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub allowed_actions: Vec<String>,
    #[serde(default)]
    pub approval_gated_actions: Vec<String>,
    #[serde(default)]
    pub forbidden_actions: Vec<String>,
    #[serde(default)]
    pub validation: Vec<HookValidation>,
}

impl BamlParity for PermissionPolicy {
    fn baml_name() -> &'static str {
        "PermissionPolicy"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "allowed_actions",
                "approval_gated_actions",
                "forbidden_actions",
                "validation",
            ],
            field_types: vec![
                "string",
                "string[]",
                "string[]",
                "string[]",
                "ValidationRequirement[]",
            ],
        }
    }
}

/// The gate decision for an action under a `PermissionPolicy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionGate {
    /// Explicitly allowed (or not listed) → proceed.
    Allowed,
    /// Listed as approval-gated → must not auto-execute without approval.
    ApprovalGated,
    /// Listed as forbidden → blocked.
    Forbidden,
}

/// Decide whether `action` may proceed under `policy`. Forbidden takes
/// precedence, then approval-gated, then allowed (default for unlisted actions).
pub fn gate_action(policy: &PermissionPolicy, action: &str) -> ActionGate {
    if policy.forbidden_actions.iter().any(|a| a == action) {
        return ActionGate::Forbidden;
    }
    if policy.approval_gated_actions.iter().any(|a| a == action) {
        return ActionGate::ApprovalGated;
    }
    ActionGate::Allowed
}
