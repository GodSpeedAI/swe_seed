use serde::{Deserialize, Serialize};

use super::events::CanonicalHookEvent;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeHookMap {
    pub runtime: String,
    pub events: Vec<CanonicalHookEvent>,
}
