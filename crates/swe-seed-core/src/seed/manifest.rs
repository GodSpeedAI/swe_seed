//! `SeedPackageManifest` — the assembled registry (specs 0003/0018).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::capability::{builtin_capabilities, LayerCapability};
use super::project_seed::ProjectSeed;
use super::{ArtifactMetadata, SeedArtifactStatus, SeedValidationRequirement};
use crate::contracts::parity::{BamlParity, BamlShape};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedPackageManifest {
    pub metadata: ArtifactMetadata,
    pub project_seed_path: String,
    pub capabilities: Vec<LayerCapability>,
    pub generated_paths: Vec<String>,
    pub validation: Vec<SeedValidationRequirement>,
}

impl BamlParity for SeedPackageManifest {
    fn baml_name() -> &'static str {
        "SeedPackageManifest"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "metadata",
                "project_seed_path",
                "capabilities",
                "generated_paths",
                "validation",
            ],
            field_types: vec![
                "ArtifactMetadata",
                "string",
                "LayerCapability[]",
                "string[]",
                "SeedValidationRequirement[]",
            ],
        }
    }
}

/// Default output location for assembled manifests.
pub const DEFAULT_MANIFEST_PATH: &str = ".swe-seed/seed-package-manifest.json";

/// Assemble a [`SeedPackageManifest`] from a project seed and its capabilities.
/// `seed_path` is recorded as `project_seed_path` (the resolvable source of the
/// seed), so assembled manifests preserve real provenance for later reads and
/// regeneration rather than a fixed placeholder.
pub fn assemble(
    seed_path: &str,
    seed: &ProjectSeed,
    capabilities: &[LayerCapability],
) -> SeedPackageManifest {
    let generated_paths = capabilities
        .iter()
        .flat_map(|c| c.artifact_paths.iter().cloned())
        .collect();
    SeedPackageManifest {
        metadata: ArtifactMetadata {
            artifact_type: "SeedPackageManifest".into(),
            artifact_id: "swe-seed-package".into(),
            source_spec: ".agents/specs/0018-layer-boundary-governance.md".into(),
            generated_by: "swe-seed v0.1".into(),
            status: SeedArtifactStatus::Active,
            version: "0.1.0".into(),
            requires_human_review: super::ReviewRequirement::Optional,
            linked_artifacts: vec![seed.metadata.artifact_id.clone()],
        },
        project_seed_path: seed_path.to_string(),
        capabilities: capabilities.to_vec(),
        generated_paths,
        validation: seed.validation.clone(),
    }
}

/// The default manifest: self-describing seed + built-in registry. The self
/// seed derives from its governing spec, so that path is recorded.
pub fn assemble_default() -> SeedPackageManifest {
    let seed = super::project_seed::self_project_seed();
    assemble(&seed.metadata.source_spec, &seed, &builtin_capabilities())
}

/// Write a manifest as pretty JSON. Creates parent dirs.
pub fn write_manifest(manifest: &SeedPackageManifest, path: &Path) -> Result<PathBuf> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(manifest)?;
    std::fs::write(path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(path.to_path_buf())
}

/// Read a manifest from JSON.
pub fn read_manifest(path: &Path) -> Result<SeedPackageManifest> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}
