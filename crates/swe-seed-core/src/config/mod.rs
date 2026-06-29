//! Typed config files parsed via `serde_yaml` (spec 0019 / Phase 0).
//! Pure-data structs; later phases attach behavior. Kept lenient (`#[serde(default)]`)
//! so incidental fields do not break parsing of the real config files.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

// ----------------------------- budget-policy.yaml -----------------------------
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BudgetPolicyConfig {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub defaults: BudgetDefaults,
    #[serde(default)]
    pub raw_output_policy: RawOutputPolicy,
    #[serde(default)]
    pub tool_output_containment: ToolOutputContainment,
    #[serde(default)]
    pub session_continuity: SessionContinuity,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct BudgetDefaults {
    #[serde(default)]
    pub route_required_context_first: bool,
    #[serde(default)]
    pub summarize_before_context: bool,
    #[serde(default)]
    pub script_bulk_analysis: bool,
    #[serde(default)]
    pub trace_durable_decisions: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct RawOutputPolicy {
    #[serde(default)]
    pub max_default_lines: u32,
    #[serde(default)]
    pub prefer: Vec<String>,
    #[serde(default)]
    pub avoid: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct ToolOutputContainment {
    #[serde(default)]
    pub high_volume_tools: Vec<String>,
    #[serde(default)]
    pub containment_action: String,
    #[serde(default)]
    pub durable_storage: DurableStorage,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct DurableStorage {
    #[serde(default)]
    pub traces: String,
    #[serde(default)]
    pub generated_records_gitignored: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct SessionContinuity {
    #[serde(default)]
    pub route_decisions: String,
    #[serde(default)]
    pub trace_records: String,
    #[serde(default)]
    pub memory: String,
    #[serde(default)]
    pub restart_rule: String,
}

// ------------------------------ agent-hooks config ------------------------------
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HooksConfig {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub paths: HooksPaths,
    #[serde(default)]
    pub logging: HooksLogging,
    #[serde(default)]
    pub redaction: RedactionConfig,
    #[serde(default)]
    pub security: HooksSecurity,
    #[serde(default)]
    pub hooks: HooksSettings,
    #[serde(default)]
    pub exports: HooksExports,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct HooksPaths {
    #[serde(default)]
    pub root: String,
    #[serde(default)]
    pub logs: String,
    #[serde(default)]
    pub payloads: String,
    #[serde(default)]
    pub artifacts: String,
    #[serde(default)]
    pub index_db: String,
    #[serde(default)]
    pub vector_index_root: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct HooksLogging {
    #[serde(default)]
    pub schema_version: String,
    #[serde(default)]
    pub rotate_by_date: bool,
    #[serde(default)]
    pub compact_min_size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct RedactionConfig {
    #[serde(default)]
    pub key_substrings: Vec<String>,
    #[serde(default)]
    pub value_patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct HooksSecurity {
    #[serde(default)]
    pub stdin_max_bytes: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct HooksSettings {
    #[serde(default)]
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct HooksExports {
    #[serde(default)]
    pub otel_format: String,
    #[serde(default)]
    pub junit_suite_name: String,
}

// ------------------------------ fabricator config ------------------------------
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FabricatorConfig {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub default_product_type: String,
    #[serde(default)]
    pub template_root: String,
    #[serde(default)]
    pub schema_root: String,
    #[serde(default)]
    pub run_root: String,
    #[serde(default)]
    pub generated_dir_name: String,
    #[serde(default)]
    pub prototype_dir_name: String,
    #[serde(default)]
    pub proof_dir_name: String,
    #[serde(default)]
    pub handoff_dir_name: String,
    #[serde(default)]
    pub required_generated_artifacts: Vec<String>,
    #[serde(default)]
    pub defaults: FabricatorDefaults,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct FabricatorDefaults {
    #[serde(default)]
    pub proof_mode: String,
    #[serde(default)]
    pub no_skill_record: String,
    #[serde(default)]
    pub handoff_manifest: String,
}

// ------------------------------- strategy config -------------------------------
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrategyConfig {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub layer_mode: String,
    #[serde(default)]
    pub python_runtime: StrategyPython,
    #[serde(default)]
    pub paths: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub required_example_artifacts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct StrategyPython {
    #[serde(default)]
    pub preference: String,
    #[serde(default)]
    pub fallback_virtualenv: String,
}

/// Generic loader: parse any config type from a YAML file.
pub fn load_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let cfg: T =
        serde_yaml::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?;
    Ok(cfg)
}
