//! `ProjectSeed` — the bounded declaration of a centralized repo (spec 0018).

use serde::{Deserialize, Serialize};

use super::{ArtifactMetadata, SeedValidationRequirement};
use crate::contracts::parity::{BamlParity, BamlShape};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectSeed {
    pub metadata: ArtifactMetadata,
    pub repository_purpose: String,
    pub toolchain: Vec<String>,
    pub command_contract: Vec<String>,
    pub scaffold_outcome: String,
    pub constraints: Vec<String>,
    pub non_goals: Vec<String>,
    pub proof_requirements: Vec<String>,
    pub validation: Vec<SeedValidationRequirement>,
}

impl BamlParity for ProjectSeed {
    fn baml_name() -> &'static str {
        "ProjectSeed"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "metadata",
                "repository_purpose",
                "toolchain",
                "command_contract",
                "scaffold_outcome",
                "constraints",
                "non_goals",
                "proof_requirements",
                "validation",
            ],
            field_types: vec![
                "ArtifactMetadata",
                "string",
                "string[]",
                "string[]",
                "string",
                "string[]",
                "string[]",
                "string[]",
                "SeedValidationRequirement[]",
            ],
        }
    }
}

/// The self-describing `ProjectSeed` for the SWE_SEED repository.
/// Used by `seed assemble` when no `--project-seed` override is given.
pub fn self_project_seed() -> ProjectSeed {
    ProjectSeed {
        metadata: ArtifactMetadata {
            artifact_type: "ProjectSeed".into(),
            artifact_id: "swe-seed-self".into(),
            source_spec: ".agents/specs/0018-layer-boundary-governance.md".into(),
            generated_by: "swe-seed v0.1".into(),
            status: super::SeedArtifactStatus::Active,
            version: "0.1.0".into(),
            requires_human_review: super::ReviewRequirement::Optional,
            linked_artifacts: Vec::new(),
        },
        repository_purpose: "Sovereign multi-layer agent harness: SweSeed → Harness → Fabricator.".into(),
        toolchain: vec!["cargo".into(), "just".into(), "python".into()],
        command_contract: vec![
            "cargo build".into(),
            "cargo test".into(),
            "cargo run -p swe-seed -- --help".into(),
        ],
        scaffold_outcome: "Cargo workspace with swe-seed bin + swe-seed-core lib.".into(),
        constraints: vec![
            "3-layer ownership: downward-only references (SweSeed → Harness → Fabricator).".into(),
            "No LLM runtime; .baml is contracts-as-data.".into(),
            "Inner stack passes with federation disabled (standalone invariant).".into(),
        ],
        non_goals: vec![
            "No vendoring of reference repos.".into(),
            "No baml-py / baml_client dependency.".into(),
        ],
        proof_requirements: vec![
            "cargo build".into(),
            "cargo test baml_parity".into(),
            "cargo test yaml_parity".into(),
            "cargo run -p swe-seed -- seed validate-boundaries".into(),
        ],
        validation: vec![SeedValidationRequirement {
            check: "workspace builds".into(),
            blocking: true,
            evidence: "cargo build".into(),
        }],
    }
}
