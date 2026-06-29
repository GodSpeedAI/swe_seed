use serde::{Deserialize, Serialize};

use super::events::CanonicalHookEvent;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookPortMapping {
    pub canonical_event: CanonicalHookEvent,
    pub native_event: String,
    pub command: String,
}
