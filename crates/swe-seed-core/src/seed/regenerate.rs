//! Idempotent regeneration (spec 0018 req 5). Produces a deterministic
//! `SeedRegenerationPlan` that preserves approved decisions and is a no-op
//! (empty `proposed_diffs`) when approved artifacts are unchanged/present.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::SeedSourceRef;
use crate::contracts::parity::{BamlParity, BamlShape};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedRegenerationInput {
    pub swe_seed_spec_path: String,
    pub approved_project_seeds: Vec<String>,
    pub approved_seed_package_manifests: Vec<String>,
    pub approved_layer_capability_maps: Vec<String>,
    pub approved_lower_layer_artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedRegenerationPlan {
    pub source_artifacts: Vec<SeedSourceRef>,
    pub proposed_diffs: Vec<String>,
    pub migration_notes: Vec<String>,
    pub preserved_decisions: Vec<String>,
    pub validation: Vec<super::SeedValidationRequirement>,
}

impl BamlParity for SeedRegenerationInput {
    fn baml_name() -> &'static str {
        "SeedRegenerationInput"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "swe_seed_spec_path",
                "approved_project_seeds",
                "approved_seed_package_manifests",
                "approved_layer_capability_maps",
                "approved_lower_layer_artifact_refs",
            ],
            field_types: vec!["string", "string[]", "string[]", "string[]", "string[]"],
        }
    }
}

impl BamlParity for SeedRegenerationPlan {
    fn baml_name() -> &'static str {
        "SeedRegenerationPlan"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "source_artifacts",
                "proposed_diffs",
                "migration_notes",
                "preserved_decisions",
                "validation",
            ],
            field_types: vec![
                "SeedSourceRef[]",
                "string[]",
                "string[]",
                "string[]",
                "SeedValidationRequirement[]",
            ],
        }
    }
}

/// Produce a deterministic regeneration plan. Resolves each approved artifact
/// path (and the governing `swe_seed_spec_path`) against `root`; a missing
/// artifact becomes a proposed diff (to recreate it), a present artifact is
/// preserved. Output is sorted, so identical inputs yield byte-identical plans
/// (idempotent) and the plan fully reflects `SeedRegenerationInput`.
pub fn regenerate(root: &Path, input: &SeedRegenerationInput) -> SeedRegenerationPlan {
    let mut source_artifacts = Vec::new();
    let mut proposed_diffs = Vec::new();
    let mut preserved_decisions = Vec::new();

    // Resolve the governing spec so it is part of plan identity.
    resolve_one(
        root,
        &input.swe_seed_spec_path,
        "governing spec",
        &mut source_artifacts,
        &mut proposed_diffs,
        &mut preserved_decisions,
    );

    let mut approved: Vec<&String> = Vec::new();
    approved.extend(input.approved_project_seeds.iter());
    approved.extend(input.approved_seed_package_manifests.iter());
    approved.extend(input.approved_layer_capability_maps.iter());
    approved.extend(input.approved_lower_layer_artifact_refs.iter());
    approved.sort();

    for path in approved {
        resolve_one(
            root,
            path,
            "approved artifact",
            &mut source_artifacts,
            &mut proposed_diffs,
            &mut preserved_decisions,
        );
    }

    preserved_decisions.sort();
    proposed_diffs.sort();
    source_artifacts.sort_by(|a, b| a.path.cmp(&b.path));

    SeedRegenerationPlan {
        source_artifacts,
        proposed_diffs,
        migration_notes: Vec::new(),
        preserved_decisions,
        validation: Vec::new(),
    }
}

fn resolve_one(
    root: &Path,
    path: &str,
    kind: &str,
    source_artifacts: &mut Vec<SeedSourceRef>,
    proposed_diffs: &mut Vec<String>,
    preserved_decisions: &mut Vec<String>,
) {
    if path.is_empty() {
        return;
    }
    source_artifacts.push(SeedSourceRef {
        path: path.to_string(),
        summary: kind.to_string(),
    });
    if root.join(path).exists() {
        preserved_decisions.push(path.to_string());
    } else {
        proposed_diffs.push(format!("recreate {kind}: {path}"));
    }
}
