use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookDecision {
    Allow,
    Deny { reason: String },
    Advisory { reason: String },
}
