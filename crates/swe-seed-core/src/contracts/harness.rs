//! Shared harness-layer contract types (spec 0019 / `harness.baml`). These are
//! the canonical Rust forms of the harness `.baml` types that were previously
//! duplicated as module-local aliases (`HookValidation`, `ContextValidation`,
//! `RouteValidationEntry`, `SkillValidation`, `TraceSchemaValidation`, …).
//! Defining them once here lets `baml_parity` assert the contract for real
//! (registered types match `.baml`) instead of "registered types match".

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};

/// harness.baml `SourceRef`.
#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct SourceRef {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub summary: String,
}

impl BamlParity for SourceRef {
    fn baml_name() -> &'static str {
        "SourceRef"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["path", "summary"],
            field_types: vec!["string", "string"],
        }
    }
}

/// harness.baml `ArtifactStatus`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ArtifactStatus {
    Draft,
    Candidate,
    Active,
    Deprecated,
    Rejected,
}

impl BamlParity for ArtifactStatus {
    fn baml_name() -> &'static str {
        "ArtifactStatus"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Enum {
            variants: vec!["Draft", "Candidate", "Active", "Deprecated", "Rejected"],
        }
    }
}

/// harness.baml `ProofDisposition`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ProofDisposition {
    Required,
    Optional,
    SkippedWithReason,
}

impl BamlParity for ProofDisposition {
    fn baml_name() -> &'static str {
        "ProofDisposition"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Enum {
            variants: vec!["Required", "Optional", "SkippedWithReason"],
        }
    }
}

/// harness.baml `ValidationRequirement` — the canonical validation-requirement
/// type used across every harness class that carries a `validation` field.
/// Module-local aliases MUST be replaced by this type.
#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct ValidationRequirement {
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub blocking: bool,
    #[serde(default)]
    pub evidence: String,
}

impl BamlParity for ValidationRequirement {
    fn baml_name() -> &'static str {
        "ValidationRequirement"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["check", "blocking", "evidence"],
            field_types: vec!["string", "bool", "string"],
        }
    }
}

/// harness.baml `HarnessNeed`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HarnessNeed {
    pub request: String,
    pub desired_outcome: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub non_goals: Vec<String>,
    #[serde(default)]
    pub proof_requirements: Vec<String>,
}

impl BamlParity for HarnessNeed {
    fn baml_name() -> &'static str {
        "HarnessNeed"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "request",
                "desired_outcome",
                "constraints",
                "non_goals",
                "proof_requirements",
            ],
            field_types: vec!["string", "string", "string[]", "string[]", "string[]"],
        }
    }
}

/// harness.baml `HarnessADR`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HarnessADR {
    pub id: String,
    pub title: String,
    pub status: ArtifactStatus,
    pub context: String,
    pub decision: String,
    #[serde(default)]
    pub alternatives: Vec<String>,
    #[serde(default)]
    pub consequences: Vec<String>,
    pub rollback_plan: String,
    #[serde(default)]
    pub validation: Vec<ValidationRequirement>,
    #[serde(default)]
    pub sources: Vec<SourceRef>,
}

impl BamlParity for HarnessADR {
    fn baml_name() -> &'static str {
        "HarnessADR"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "title",
                "status",
                "context",
                "decision",
                "alternatives",
                "consequences",
                "rollback_plan",
                "validation",
                "sources",
            ],
            field_types: vec![
                "string",
                "string",
                "ArtifactStatus",
                "string",
                "string",
                "string[]",
                "string[]",
                "string",
                "ValidationRequirement[]",
                "SourceRef[]",
            ],
        }
    }
}

/// harness.baml `RegenerationInput`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegenerationInput {
    pub harness_spec_path: String,
    #[serde(default)]
    pub approved_route_cards: Vec<String>,
    #[serde(default)]
    pub approved_skill_ir_files: Vec<String>,
    #[serde(default)]
    pub eval_specs: Vec<String>,
    #[serde(default)]
    pub hook_policies: Vec<String>,
    #[serde(default)]
    pub approved_learning_records: Vec<String>,
    #[serde(default)]
    pub regression_cases: Vec<String>,
}

impl BamlParity for RegenerationInput {
    fn baml_name() -> &'static str {
        "RegenerationInput"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "harness_spec_path",
                "approved_route_cards",
                "approved_skill_ir_files",
                "eval_specs",
                "hook_policies",
                "approved_learning_records",
                "regression_cases",
            ],
            field_types: vec![
                "string", "string[]", "string[]", "string[]", "string[]", "string[]", "string[]",
            ],
        }
    }
}

/// harness.baml `RegenerationPlan`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegenerationPlan {
    #[serde(default)]
    pub source_artifacts: Vec<SourceRef>,
    #[serde(default)]
    pub render_targets: Vec<String>,
    #[serde(default)]
    pub preserved_decisions: Vec<String>,
    #[serde(default)]
    pub blocking_diffs: Vec<String>,
    #[serde(default)]
    pub validation: Vec<ValidationRequirement>,
}

impl BamlParity for RegenerationPlan {
    fn baml_name() -> &'static str {
        "RegenerationPlan"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "source_artifacts",
                "render_targets",
                "preserved_decisions",
                "blocking_diffs",
                "validation",
            ],
            field_types: vec![
                "SourceRef[]",
                "string[]",
                "string[]",
                "string[]",
                "ValidationRequirement[]",
            ],
        }
    }
}
