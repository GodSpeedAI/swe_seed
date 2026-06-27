//! `ContextBudget` — bounds a route's context intake (spec 0015 / harness.baml).

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};

/// harness.baml `ValidationRequirement` (context-local copy so this module is
/// self-contained; not registered for parity separately here).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextValidation {
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub blocking: bool,
    #[serde(default)]
    pub evidence: String,
}

/// harness.baml `ContextBudget`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBudget {
    pub id: String,
    pub max_context_size: String,
    pub required_files: Vec<String>,
    #[serde(default)]
    pub optional_files: Vec<String>,
    #[serde(default)]
    pub excluded_files: Vec<String>,
    #[serde(default)]
    pub freshness_requirements: Vec<String>,
    #[serde(default)]
    pub relevance_rules: Vec<String>,
    #[serde(default)]
    pub summarization_rules: Vec<String>,
    #[serde(default)]
    pub validation: Vec<ContextValidation>,
}

impl BamlParity for ContextBudget {
    fn baml_name() -> &'static str {
        "ContextBudget"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "max_context_size",
                "required_files",
                "optional_files",
                "excluded_files",
                "freshness_requirements",
                "relevance_rules",
                "summarization_rules",
                "validation",
            ],
            field_types: vec![
                "string",
                "string",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "ValidationRequirement[]",
            ],
        }
    }
}
