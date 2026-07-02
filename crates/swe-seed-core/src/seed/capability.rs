//! `LayerCapability` + the built-in capability registry (specs 0003/0018).

use serde::{Deserialize, Serialize};

use super::LayerName;
use crate::contracts::parity::{BamlParity, BamlShape};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerCapability {
    pub id: String,
    pub owner: LayerName,
    pub source_spec: String,
    pub artifact_paths: Vec<String>,
    pub purpose: String,
    pub required: bool,
}

impl BamlParity for LayerCapability {
    fn baml_name() -> &'static str {
        "LayerCapability"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "owner",
                "source_spec",
                "artifact_paths",
                "purpose",
                "required",
            ],
            field_types: vec![
                "string",
                "LayerName",
                "string",
                "string[]",
                "string",
                "bool",
            ],
        }
    }
}

/// One registry entry. `paths` may contain path-prefix hints used by boundary
/// validation to infer the layer a capability's artifacts live in.
fn cap(
    id: &str,
    owner: LayerName,
    source_spec: &str,
    artifact_paths: &[&str],
    purpose: &str,
    required: bool,
) -> LayerCapability {
    LayerCapability {
        id: id.into(),
        owner,
        source_spec: source_spec.into(),
        artifact_paths: artifact_paths.iter().map(|s| s.to_string()).collect(),
        purpose: purpose.into(),
        required,
    }
}

/// The built-in capability registry: one [`LayerCapability`] per governed area
/// across the three layers. `seed assemble` packages these into a manifest.
pub fn builtin_capabilities() -> Vec<LayerCapability> {
    use LayerName::*;
    vec![
        // --- SweSeed (outer) ---
        cap(
            "layer-governance",
            SweSeed,
            "docs/specs/0018-layer-boundary-governance.md",
            &["crates/swe-seed-core/src/seed", ".swe-seed/"],
            "3-layer ownership + boundary validation",
            true,
        ),
        cap(
            "provenance",
            SweSeed,
            "docs/specs/0009-license-and-provenance-boundaries.md",
            &[
                "crates/swe-seed-core/src/provenance",
                ".swe-seed/provenance/",
            ],
            "license + clean-room provenance records",
            true,
        ),
        cap(
            "host-adapters",
            SweSeed,
            "docs/specs/0004-host-adapter-contract.md",
            &["crates/swe-seed-core/src/adapters"],
            "project the assembled manifest into hosts",
            false,
        ),
        cap(
            "federation",
            SweSeed,
            "docs/specs/0011-sea-loop-federation.md",
            &["crates/swe-seed-core/src/federation"],
            "optional SEA envelope (default off)",
            false,
        ),
        // --- Harness (middle) ---
        cap(
            "routing",
            Harness,
            "docs/specs/0012-existing-harness-reconciliation.md",
            &["crates/swe-seed-core/src/route", ".agent-harness/routes/"],
            "RouteCard-driven task routing",
            true,
        ),
        cap(
            "traces",
            Harness,
            "docs/specs/0014-trace-and-durable-decisions.md",
            &["crates/swe-seed-core/src/trace", ".agent-harness/traces/"],
            "durable decisions + session continuity",
            true,
        ),
        cap(
            "eval-proof",
            Harness,
            "docs/specs/0013-eval-and-proof.md",
            &["crates/swe-seed-core/src/eval"],
            "eval specs, proof records, promotion gate",
            true,
        ),
        cap(
            "doctor",
            Harness,
            "docs/specs/0008-doctor-and-drift-detection.md",
            &["crates/swe-seed-core/src/doctor"],
            "validate + eval + boundary aggregation",
            true,
        ),
        cap(
            "context-plane",
            Harness,
            "docs/specs/0015-context-budget-plane.md",
            &[
                "crates/swe-seed-core/src/context",
                ".agent-harness/context/",
            ],
            "bounded context budget + pack",
            true,
        ),
        cap(
            "hook-runtime",
            Harness,
            "docs/specs/0005-normalized-hook-runtime.md",
            &["crates/swe-seed-core/src/hooks", ".agent-hooks/"],
            "normalized hook events + permissions",
            true,
        ),
        cap(
            "skill-ingestion",
            Harness,
            "docs/specs/0007-skill-ingestion-and-scan-gate.md",
            &["crates/swe-seed-core/src/skill"],
            "SkillIR ingestion + scan gate",
            false,
        ),
        cap(
            "learning-loop",
            Harness,
            "docs/specs/0016-learning-and-adaptation-loop.md",
            &["crates/swe-seed-core/src/learning"],
            "reflection → learning → adaptation",
            false,
        ),
        // --- Fabricator (inner) ---
        cap(
            "fabricator",
            Fabricator,
            "docs/specs/0017-fabricator-layer.md",
            &["crates/swe-seed-core/src/fabricator", ".fabricator/"],
            "product → prototype semantic chain",
            false,
        ),
    ]
}
