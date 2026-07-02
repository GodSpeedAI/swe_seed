//! Fabricator chain artifact types — baml-faithful (spec 0017 / fabricator.baml).
//! Contracts-as-data: no LLM runtime. These structs ARE the chain; the binary
//! validates, links, and renders them.

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};

// --------------------------------- enums ----------------------------------

/// fabricator.baml `FabricatorArtifactStatus`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum FabricatorArtifactStatus {
    Draft,
    Proposed,
    Approved,
    Active,
    Rejected,
    Deprecated,
}

impl BamlParity for FabricatorArtifactStatus {
    fn baml_name() -> &'static str {
        "FabricatorArtifactStatus"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Enum {
            variants: vec![
                "Draft",
                "Proposed",
                "Approved",
                "Active",
                "Rejected",
                "Deprecated",
            ],
        }
    }
}

/// fabricator.baml `FabricatorEvalClass`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum FabricatorEvalClass {
    ProductOutcome,
    ProcessCompliance,
    LearningQuality,
    AdaptationEligibility,
}

impl BamlParity for FabricatorEvalClass {
    fn baml_name() -> &'static str {
        "FabricatorEvalClass"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Enum {
            variants: vec![
                "ProductOutcome",
                "ProcessCompliance",
                "LearningQuality",
                "AdaptationEligibility",
            ],
        }
    }
}

/// fabricator.baml `EARSPattern`. Drives EARS shape validation (`validate.rs`).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum EARSPattern {
    Ubiquitous,
    EventDriven,
    StateDriven,
    OptionalFeature,
    UnwantedBehavior,
}

impl BamlParity for EARSPattern {
    fn baml_name() -> &'static str {
        "EARSPattern"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Enum {
            variants: vec![
                "Ubiquitous",
                "EventDriven",
                "StateDriven",
                "OptionalFeature",
                "UnwantedBehavior",
            ],
        }
    }
}

// ------------------------------- leaf types -------------------------------

/// fabricator.baml `FabricatorSourceRef`.
#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct FabricatorSourceRef {
    #[serde(default)]
    pub artifact_type: String,
    #[serde(default)]
    pub artifact_id: String,
    #[serde(default)]
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl BamlParity for FabricatorSourceRef {
    fn baml_name() -> &'static str {
        "FabricatorSourceRef"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["artifact_type", "artifact_id", "path", "version"],
            field_types: vec!["string", "string", "string", "string?"],
        }
    }
}

/// fabricator.baml `TraceabilityLink` — the chain edge.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceabilityLink {
    pub upstream_id: String,
    pub downstream_id: String,
    pub relation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiver_id: Option<String>,
}

impl BamlParity for TraceabilityLink {
    fn baml_name() -> &'static str {
        "TraceabilityLink"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["upstream_id", "downstream_id", "relation", "waiver_id"],
            field_types: vec!["string", "string", "string", "string?"],
        }
    }
}

// ------------------------------ chain spine -------------------------------

/// fabricator.baml `JobStory`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobStory {
    pub id: String,
    pub when_clause: String,
    pub action_clause: String,
    pub outcome_clause: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waived: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiver_reason: Option<String>,
}

impl BamlParity for JobStory {
    fn baml_name() -> &'static str {
        "JobStory"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "when_clause",
                "action_clause",
                "outcome_clause",
                "source_refs",
                "waived",
                "waiver_reason",
            ],
            field_types: vec![
                "string", "string", "string", "string", "string[]", "bool?", "string?",
            ],
        }
    }
}

/// fabricator.baml `ProductSeed`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductSeed {
    pub id: String,
    pub version: String,
    pub status: FabricatorArtifactStatus,
    pub target_user: String,
    pub product_idea: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub non_goals: Vec<String>,
    pub desired_prototype_outcome: String,
    #[serde(default)]
    pub proof_required: Vec<String>,
    #[serde(default)]
    pub linked_artifacts: Vec<FabricatorSourceRef>,
}

impl BamlParity for ProductSeed {
    fn baml_name() -> &'static str {
        "ProductSeed"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "version",
                "status",
                "target_user",
                "product_idea",
                "constraints",
                "non_goals",
                "desired_prototype_outcome",
                "proof_required",
                "linked_artifacts",
            ],
            field_types: vec![
                "string",
                "string",
                "FabricatorArtifactStatus",
                "string",
                "string",
                "string[]",
                "string[]",
                "string",
                "string[]",
                "FabricatorSourceRef[]",
            ],
        }
    }
}

/// fabricator.baml `ProductHypothesis`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductHypothesis {
    pub id: String,
    pub version: String,
    pub product_seed_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hypothesis: Option<String>,
    pub hypothesis_statement: String,
    pub target_job: String,
    pub target_user_segment: String,
    pub situation_context: String,
    #[serde(default)]
    pub pains: Vec<String>,
    #[serde(default)]
    pub desired_gains: Vec<String>,
    pub value_proposition: String,
    #[serde(default)]
    pub alternatives_today: Vec<String>,
    #[serde(default)]
    pub channels: Vec<String>,
    #[serde(default)]
    pub adoption_triggers: Vec<String>,
    #[serde(default)]
    pub adoption_barriers: Vec<String>,
    #[serde(default)]
    pub measurable_success: Vec<String>,
    #[serde(default)]
    pub riskiest_assumptions: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

impl BamlParity for ProductHypothesis {
    fn baml_name() -> &'static str {
        "ProductHypothesis"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "version",
                "product_seed_id",
                "hypothesis",
                "hypothesis_statement",
                "target_job",
                "target_user_segment",
                "situation_context",
                "pains",
                "desired_gains",
                "value_proposition",
                "alternatives_today",
                "channels",
                "adoption_triggers",
                "adoption_barriers",
                "measurable_success",
                "riskiest_assumptions",
                "source_refs",
            ],
            field_types: vec![
                "string", "string", "string", "string?", "string", "string", "string", "string",
                "string[]", "string[]", "string", "string[]", "string[]", "string[]", "string[]",
                "string[]", "string[]", "string[]",
            ],
        }
    }
}

/// fabricator.baml `EARSRequirement`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EARSRequirement {
    pub id: String,
    pub pattern: EARSPattern,
    pub system_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unwanted_condition: Option<String>,
    pub response: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub ears_text: String,
    pub testable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_testable_reason: Option<String>,
}

impl BamlParity for EARSRequirement {
    fn baml_name() -> &'static str {
        "EARSRequirement"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "pattern",
                "system_name",
                "condition",
                "trigger",
                "state",
                "feature",
                "unwanted_condition",
                "response",
                "rationale",
                "source_refs",
                "ears_text",
                "testable",
                "non_testable_reason",
            ],
            field_types: vec![
                "string",
                "EARSPattern",
                "string",
                "string?",
                "string?",
                "string?",
                "string?",
                "string?",
                "string",
                "string?",
                "string[]",
                "string",
                "bool",
                "string?",
            ],
        }
    }
}

/// fabricator.baml `YStatement`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YStatement {
    pub id: String,
    pub context: String,
    pub constraint: String,
    pub chosen_option: String,
    #[serde(default)]
    pub neglected_alternatives: Vec<String>,
    pub benefit: String,
    pub accepted_cost: String,
    #[serde(default)]
    pub linked_requirement_ids: Vec<String>,
    pub y_text: String,
}

impl BamlParity for YStatement {
    fn baml_name() -> &'static str {
        "YStatement"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "context",
                "constraint",
                "chosen_option",
                "neglected_alternatives",
                "benefit",
                "accepted_cost",
                "linked_requirement_ids",
                "y_text",
            ],
            field_types: vec![
                "string", "string", "string", "string", "string[]", "string", "string", "string[]",
                "string",
            ],
        }
    }
}

/// fabricator.baml `ProductADR`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductADR {
    pub id: String,
    pub version: String,
    pub product_seed_id: String,
    pub decision: String,
    #[serde(default)]
    pub alternatives: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub consequences: Vec<String>,
    pub rollback_path: String,
    #[serde(default)]
    pub y_statements: Vec<YStatement>,
    #[serde(default)]
    pub linked_job_ids: Vec<String>,
    #[serde(default)]
    pub linked_requirement_ids: Vec<String>,
}

impl BamlParity for ProductADR {
    fn baml_name() -> &'static str {
        "ProductADR"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "version",
                "product_seed_id",
                "decision",
                "alternatives",
                "constraints",
                "consequences",
                "rollback_path",
                "y_statements",
                "linked_job_ids",
                "linked_requirement_ids",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "string",
                "string[]",
                "string[]",
                "string[]",
                "string",
                "YStatement[]",
                "string[]",
                "string[]",
            ],
        }
    }
}

/// fabricator.baml `SDSComponent`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SDSComponent {
    pub id: String,
    pub responsibility: String,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub state_owned: Vec<String>,
    #[serde(default)]
    pub failure_modes: Vec<String>,
    #[serde(default)]
    pub linked_requirement_ids: Vec<String>,
    #[serde(default)]
    pub linked_scenario_ids: Vec<String>,
}

impl BamlParity for SDSComponent {
    fn baml_name() -> &'static str {
        "SDSComponent"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "responsibility",
                "inputs",
                "outputs",
                "state_owned",
                "failure_modes",
                "linked_requirement_ids",
                "linked_scenario_ids",
            ],
            field_types: vec![
                "string", "string", "string[]", "string[]", "string[]", "string[]", "string[]",
                "string[]",
            ],
        }
    }
}

/// fabricator.baml `GherkinScenario`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GherkinScenario {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub given_steps: Vec<String>,
    #[serde(default)]
    pub when_steps: Vec<String>,
    #[serde(default)]
    pub then_steps: Vec<String>,
    #[serde(default)]
    pub linked_requirement_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiver_reason: Option<String>,
}

impl BamlParity for GherkinScenario {
    fn baml_name() -> &'static str {
        "GherkinScenario"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "name",
                "given_steps",
                "when_steps",
                "then_steps",
                "linked_requirement_ids",
                "waiver_reason",
            ],
            field_types: vec![
                "string", "string", "string[]", "string[]", "string[]", "string[]", "string?",
            ],
        }
    }
}

/// fabricator.baml `PRD`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PRD {
    pub id: String,
    pub version: String,
    pub product_seed_id: String,
    pub user: String,
    #[serde(default)]
    pub job_story_ids: Vec<String>,
    #[serde(default)]
    pub requirements: Vec<EARSRequirement>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub non_goals: Vec<String>,
}

impl BamlParity for PRD {
    fn baml_name() -> &'static str {
        "PRD"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "version",
                "product_seed_id",
                "user",
                "job_story_ids",
                "requirements",
                "acceptance_criteria",
                "non_goals",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "string",
                "string[]",
                "EARSRequirement[]",
                "string[]",
                "string[]",
            ],
        }
    }
}

/// fabricator.baml `SDS`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SDS {
    pub id: String,
    pub version: String,
    pub prd_id: String,
    #[serde(default)]
    pub c4_level: Vec<String>,
    #[serde(default)]
    pub mermaid_diagrams: Vec<String>,
    #[serde(default)]
    pub architecture: Vec<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub interfaces: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub components: Vec<SDSComponent>,
    #[serde(default)]
    pub scenarios: Vec<GherkinScenario>,
}

impl BamlParity for SDS {
    fn baml_name() -> &'static str {
        "SDS"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "version",
                "prd_id",
                "c4_level",
                "mermaid_diagrams",
                "architecture",
                "files",
                "interfaces",
                "constraints",
                "components",
                "scenarios",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "SDSComponent[]",
                "GherkinScenario[]",
            ],
        }
    }
}

/// fabricator.baml `TDDPlan`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TDDPlan {
    pub id: String,
    pub version: String,
    pub sds_id: String,
    #[serde(default)]
    pub tests: Vec<String>,
    #[serde(default)]
    pub proof_commands: Vec<String>,
    #[serde(default)]
    pub manual_checks: Vec<String>,
    #[serde(default)]
    pub scenarios: Vec<GherkinScenario>,
}

impl BamlParity for TDDPlan {
    fn baml_name() -> &'static str {
        "TDDPlan"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "version",
                "sds_id",
                "tests",
                "proof_commands",
                "manual_checks",
                "scenarios",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "string[]",
                "string[]",
                "string[]",
                "GherkinScenario[]",
            ],
        }
    }
}

/// fabricator.baml `AgentTask`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentTask {
    pub id: String,
    pub version: String,
    pub target_path: String,
    #[serde(default)]
    pub instructions: Vec<String>,
    #[serde(default)]
    pub required_context: Vec<String>,
    #[serde(default)]
    pub excluded_context: Vec<String>,
    #[serde(default)]
    pub proof_required: Vec<String>,
    #[serde(default)]
    pub eval_spec_ids: Vec<String>,
    #[serde(default)]
    pub linked_traceability_ids: Vec<String>,
}

impl BamlParity for AgentTask {
    fn baml_name() -> &'static str {
        "AgentTask"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "version",
                "target_path",
                "instructions",
                "required_context",
                "excluded_context",
                "proof_required",
                "eval_spec_ids",
                "linked_traceability_ids",
            ],
            field_types: vec![
                "string", "string", "string", "string[]", "string[]", "string[]", "string[]",
                "string[]", "string[]",
            ],
        }
    }
}

/// fabricator.baml `FabricatorEvalCheck`. `type` is the baml field name.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FabricatorEvalCheck {
    pub id: String,
    pub eval_class: FabricatorEvalClass,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub rule: String,
    #[serde(default)]
    pub evidence_required: Vec<String>,
    #[serde(default)]
    pub linked_requirement_ids: Vec<String>,
    #[serde(default)]
    pub linked_scenario_ids: Vec<String>,
    #[serde(default)]
    pub linked_component_ids: Vec<String>,
    #[serde(default)]
    pub linked_y_statement_ids: Vec<String>,
}

impl BamlParity for FabricatorEvalCheck {
    fn baml_name() -> &'static str {
        "FabricatorEvalCheck"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "eval_class",
                "type",
                "target",
                "required",
                "rule",
                "evidence_required",
                "linked_requirement_ids",
                "linked_scenario_ids",
                "linked_component_ids",
                "linked_y_statement_ids",
            ],
            field_types: vec![
                "string",
                "FabricatorEvalClass",
                "string",
                "string",
                "bool",
                "string",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
            ],
        }
    }
}

/// fabricator.baml `FabricatorEvalSpec`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FabricatorEvalSpec {
    pub id: String,
    pub version: String,
    pub run_id: String,
    pub target_type: String,
    pub target_path: String,
    pub purpose: String,
    #[serde(default)]
    pub eval_classes: Vec<FabricatorEvalClass>,
    #[serde(default)]
    pub checks: Vec<FabricatorEvalCheck>,
    #[serde(default)]
    pub pass_condition: String,
    #[serde(default)]
    pub outputs: Vec<String>,
}

impl BamlParity for FabricatorEvalSpec {
    fn baml_name() -> &'static str {
        "FabricatorEvalSpec"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "version",
                "run_id",
                "target_type",
                "target_path",
                "purpose",
                "eval_classes",
                "checks",
                "pass_condition",
                "outputs",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "string",
                "string",
                "string",
                "FabricatorEvalClass[]",
                "FabricatorEvalCheck[]",
                "string",
                "string[]",
            ],
        }
    }
}

/// fabricator.baml `FabricatorProofRecord`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FabricatorProofRecord {
    pub id: String,
    pub run_id: String,
    pub eval_id: String,
    pub status: String,
    #[serde(default)]
    pub evidence: Vec<String>,
    pub created_at: String,
    #[serde(default)]
    pub satisfied_check_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked_eval_result_id: Option<String>,
}

impl BamlParity for FabricatorProofRecord {
    fn baml_name() -> &'static str {
        "FabricatorProofRecord"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "run_id",
                "eval_id",
                "status",
                "evidence",
                "created_at",
                "satisfied_check_ids",
                "linked_eval_result_id",
            ],
            field_types: vec![
                "string", "string", "string", "string", "string[]", "string", "string[]", "string?",
            ],
        }
    }
}

/// fabricator.baml `SemanticChainValidationReport`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SemanticChainValidationReport {
    pub id: String,
    pub passed: bool,
    #[serde(default)]
    pub missing_links: Vec<String>,
    #[serde(default)]
    pub waived_links: Vec<String>,
    #[serde(default)]
    pub blocked_claims: Vec<String>,
}

impl BamlParity for SemanticChainValidationReport {
    fn baml_name() -> &'static str {
        "SemanticChainValidationReport"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "passed",
                "missing_links",
                "waived_links",
                "blocked_claims",
            ],
            field_types: vec!["string", "bool", "string[]", "string[]", "string[]"],
        }
    }
}
