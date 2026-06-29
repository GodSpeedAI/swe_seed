//! SweSeed outer layer (specs 0003/0009/0018). 3-layer ownership governance,
//! capability assembly, boundary validation, and idempotent regeneration.
//!
//! Every governed artifact carries [`ArtifactMetadata`] (provenance + status).

pub mod boundary;
pub mod capability;
pub mod manifest;
pub mod project_seed;
pub mod regenerate;

pub use boundary::{validate_boundaries, BoundaryFinding, BoundaryReport};
pub use capability::LayerCapability;
pub use manifest::{assemble_default, SeedPackageManifest};
pub use project_seed::ProjectSeed;
pub use regenerate::{regenerate, SeedRegenerationInput, SeedRegenerationPlan};

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};

/// The three governance layers. Order is outer → inner: a layer may reference
/// only itself or a more inner layer (`SweSeed → Harness → Fabricator`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum LayerName {
    SweSeed,
    Harness,
    Fabricator,
}

impl LayerName {
    /// Outer-to-inner rank (SweSeed=1, Harness=2, Fabricator=3). A layer at
    /// rank `r` may depend on layers at rank `>= r`; depending lower is upward.
    pub fn rank(self) -> u8 {
        match self {
            LayerName::SweSeed => 1,
            LayerName::Harness => 2,
            LayerName::Fabricator => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum SeedArtifactStatus {
    Draft,
    Candidate,
    Active,
    Deprecated,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ReviewRequirement {
    Required,
    Optional,
    NotRequired,
}

impl BamlParity for LayerName {
    fn baml_name() -> &'static str {
        "LayerName"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Enum {
            variants: vec!["SweSeed", "Harness", "Fabricator"],
        }
    }
}

impl BamlParity for SeedArtifactStatus {
    fn baml_name() -> &'static str {
        "SeedArtifactStatus"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Enum {
            variants: vec![
                "Draft",
                "Candidate",
                "Active",
                "Deprecated",
                "Approved",
                "Rejected",
            ],
        }
    }
}

impl BamlParity for ReviewRequirement {
    fn baml_name() -> &'static str {
        "ReviewRequirement"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Enum {
            variants: vec!["Required", "Optional", "NotRequired"],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedSourceRef {
    pub path: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedValidationRequirement {
    pub check: String,
    pub blocking: bool,
    pub evidence: String,
}

/// The bounded need that seeds a ProjectSeed. Mirrors `SeedNeed` in `.baml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedNeed {
    pub request: String,
    pub desired_outcome: String,
    pub constraints: Vec<String>,
    pub non_goals: Vec<String>,
    pub proof_requirements: Vec<String>,
}

/// Provenance carrier for every governed artifact (spec 0009/0018).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactMetadata {
    pub artifact_type: String,
    pub artifact_id: String,
    pub source_spec: String,
    pub generated_by: String,
    pub status: SeedArtifactStatus,
    pub version: String,
    pub requires_human_review: ReviewRequirement,
    pub linked_artifacts: Vec<String>,
}

impl BamlParity for SeedSourceRef {
    fn baml_name() -> &'static str {
        "SeedSourceRef"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["path", "summary"],
            field_types: vec!["string", "string"],
        }
    }
}

impl BamlParity for SeedValidationRequirement {
    fn baml_name() -> &'static str {
        "SeedValidationRequirement"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["check", "blocking", "evidence"],
            field_types: vec!["string", "bool", "string"],
        }
    }
}

impl BamlParity for SeedNeed {
    fn baml_name() -> &'static str {
        "SeedNeed"
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

impl BamlParity for ArtifactMetadata {
    fn baml_name() -> &'static str {
        "ArtifactMetadata"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "artifact_type",
                "artifact_id",
                "source_spec",
                "generated_by",
                "status",
                "version",
                "requires_human_review",
                "linked_artifacts",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "string",
                "SeedArtifactStatus",
                "string",
                "ReviewRequirement",
                "string[]",
            ],
        }
    }
}
