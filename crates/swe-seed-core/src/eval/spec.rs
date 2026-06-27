//! `EvalSpec` + loading + frozen-after-handoff enforcement (spec 0013).

use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::contracts::parity::{BamlParity, BamlShape};
use crate::provenance::content_hash;
use super::check::{EvalCheck, SourceRef};

#[derive(Debug, Clone, Deserialize)]
pub struct EvalSpec {
    pub id: String,
    pub version: String,
    pub run_id: String,
    pub target_type: String,
    pub target_path: String,
    pub purpose: String,
    pub eval_classes: Vec<crate::eval::EvalClass>,
    pub checks: Vec<EvalCheck>,
    pub pass_condition: String,
    pub outputs: Vec<String>,
    #[serde(default)]
    pub frozen_after_handoff: bool,
    #[serde(default)]
    pub sources: Vec<SourceRef>,
}

impl BamlParity for EvalSpec {
    fn baml_name() -> &'static str {
        "EvalSpec"
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
                "frozen_after_handoff",
                "sources",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "string",
                "string",
                "string",
                "EvalClass[]",
                "EvalCheck[]",
                "string",
                "string[]",
                "bool",
                "SourceRef[]",
            ],
        }
    }
}

/// Load an EvalSpec from a TOML or JSON file (by extension).
pub fn load_eval_spec(path: &Path) -> Result<EvalSpec> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let spec: EvalSpec = match ext {
        "json" => serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?,
        _ => {
            let text = std::str::from_utf8(&bytes)
                .with_context(|| format!("non-utf8 spec {}", path.display()))?;
            toml::from_str::<EvalSpec>(text).with_context(|| format!("parse {}", path.display()))?
        }
    };
    validate_structure(&spec, path)?;
    Ok(spec)
}

fn validate_structure(spec: &EvalSpec, path: &Path) -> Result<()> {
    if spec.checks.is_empty() {
        bail!("{}: EvalSpec checks must be a non-empty list", path.display());
    }
    Ok(())
}

/// Enforce frozen-after-handoff: a frozen spec's file bytes are pinned at first
/// run (handoff) under `.swe-seed/eval-handoff/<id>.sha256`; any later edit
/// changes the hash and fails the check. Non-frozen specs are a no-op.
pub fn check_frozen(root: &Path, spec_path: &Path, spec: &EvalSpec) -> Result<()> {
    if !spec.frozen_after_handoff {
        return Ok(());
    }
    let bytes = std::fs::read(spec_path).with_context(|| format!("read {}", spec_path.display()))?;
    let current = content_hash(&bytes);
    let dir = root.join(".swe-seed").join("eval-handoff");
    // Collision-resistant key: a hash of the raw spec id. (A lossy char-map
    // would collapse distinct ids like "a/b" and "a-b" onto one sidecar file.)
    let key = content_hash(spec.id.as_bytes())
        .strip_prefix("sha256:")
        .unwrap_or("unknown")
        .to_string();
    let sidecar = dir.join(format!("{key}.sha256"));
    if sidecar.is_file() {
        let recorded = std::fs::read_to_string(&sidecar)?;
        if recorded.trim() != current {
            bail!(
                "frozen EvalSpec '{}' modified after handoff (hash mismatch)",
                spec.id
            );
        }
    } else {
        std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        std::fs::write(&sidecar, &current).with_context(|| format!("write {}", sidecar.display()))?;
    }
    Ok(())
}
