//! Boundary validation (spec 0018). Enforces 3-layer ownership and
//! downward-only references. `passed=false` blocks release (run by doctor).

use serde::{Deserialize, Serialize};

use super::capability::builtin_capabilities;
use super::{LayerName, SeedPackageManifest};
use crate::contracts::parity::{BamlParity, BamlShape};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundaryFinding {
    pub layer: LayerName,
    pub artifact_path: String,
    pub issue: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundaryReport {
    pub metadata: super::ArtifactMetadata,
    pub findings: Vec<BoundaryFinding>,
    pub passed: bool,
    pub validation: Vec<super::SeedValidationRequirement>,
}

impl BamlParity for BoundaryFinding {
    fn baml_name() -> &'static str {
        "BoundaryFinding"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["layer", "artifact_path", "issue", "severity"],
            field_types: vec!["LayerName", "string", "string", "string"],
        }
    }
}

impl BamlParity for BoundaryReport {
    fn baml_name() -> &'static str {
        "BoundaryReport"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec!["metadata", "findings", "passed", "validation"],
            field_types: vec![
                "ArtifactMetadata",
                "BoundaryFinding[]",
                "bool",
                "SeedValidationRequirement[]",
            ],
        }
    }
}

/// Infer the layer an artifact path belongs to from its prefix. Returns
/// `None` for unrecognized paths so they surface as findings instead of being
/// silently treated as a valid outer-layer artifact.
pub fn infer_layer(path: &str) -> Option<LayerName> {
    use LayerName::*;
    let p = path.trim_start_matches("./");
    if p.starts_with(".fabricator") || p.contains("src/fabricator") {
        Some(Fabricator)
    } else if p.starts_with(".agent-harness")
        || p.starts_with(".agent-hooks")
        || p.contains("src/route")
        || p.contains("src/trace")
        || p.contains("src/eval")
        || p.contains("src/context")
        || p.contains("src/hooks")
        || p.contains("src/skill")
        || p.contains("src/learning")
        || p.contains("src/doctor")
    {
        Some(Harness)
    } else if p.starts_with(".swe-seed")
        || p.starts_with(".agents")
        || p.starts_with("crates/swe-seed")
        || p.contains("src/seed")
        || p.contains("src/provenance")
        || p.contains("src/adapters")
        || p.contains("src/federation")
    {
        Some(SweSeed)
    } else {
        None
    }
}

/// Validate a manifest's layer boundaries.
///
/// Findings:
/// - **unclassified path**: an artifact path does not match any layer prefix.
/// - **upward reference**: a capability owns artifacts that live in a more
///   outer layer than its `owner` (dependencies must point inward only).
/// - **duplicate id**: two capabilities share an id.
/// - **missing required**: a registry capability marked `required` is absent.
pub fn validate_boundaries(manifest: &SeedPackageManifest) -> BoundaryReport {
    let mut findings = Vec::new();

    // Ownership / upward-reference / classification checks.
    let mut seen = std::collections::HashSet::new();
    for cap in &manifest.capabilities {
        if !seen.insert(cap.id.clone()) {
            findings.push(BoundaryFinding {
                layer: cap.owner,
                artifact_path: format!("<capability:{}>", cap.id),
                issue: format!("duplicate capability id '{}'", cap.id),
                severity: "high".into(),
            });
        }
        for ap in &cap.artifact_paths {
            match infer_layer(ap) {
                None => findings.push(BoundaryFinding {
                    layer: cap.owner,
                    artifact_path: ap.clone(),
                    issue: format!(
                        "unclassified artifact path '{ap}': not recognized as any layer"
                    ),
                    severity: "medium".into(),
                }),
                Some(inferred) if cap.owner.rank() > inferred.rank() => {
                    findings.push(BoundaryFinding {
                        layer: inferred,
                        artifact_path: ap.clone(),
                        issue: format!(
                            "capability '{}' owned by {:?} references a {:?}-layer (outer) artifact; \
                             dependencies must point inward only",
                            cap.id, cap.owner, inferred
                        ),
                        severity: "high".into(),
                    });
                }
                _ => {}
            }
        }
    }

    // Required-capability presence vs the registry.
    let present: std::collections::HashSet<&str> = manifest
        .capabilities
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    for reg in builtin_capabilities() {
        if reg.required && !present.contains(reg.id.as_str()) {
            findings.push(BoundaryFinding {
                layer: reg.owner,
                artifact_path: format!("<capability:{}>", reg.id),
                issue: format!("missing required capability '{}'", reg.id),
                severity: "high".into(),
            });
        }
    }

    let passed = findings.is_empty();
    BoundaryReport {
        metadata: super::ArtifactMetadata {
            artifact_type: "BoundaryReport".into(),
            artifact_id: "swe-seed-boundary".into(),
            source_spec: "docs/specs/0018-layer-boundary-governance.md".into(),
            generated_by: "swe-seed v0.1".into(),
            status: if passed {
                super::SeedArtifactStatus::Active
            } else {
                super::SeedArtifactStatus::Rejected
            },
            version: "0.1.0".into(),
            requires_human_review: super::ReviewRequirement::Optional,
            linked_artifacts: vec![manifest.metadata.artifact_id.clone()],
        },
        findings,
        passed,
        validation: Vec::new(),
    }
}
