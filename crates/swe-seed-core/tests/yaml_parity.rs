//! yaml_parity: serde_yaml parses every existing config into typed structs.
//! A real config file that fails to parse → this test fails (spec 0019).

use std::path::PathBuf;

use swe_seed_core::config::{
    load_yaml, BudgetPolicyConfig, FabricatorConfig, HooksConfig, StrategyConfig,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

#[test]
fn yaml_parity_budget() {
    let cfg: BudgetPolicyConfig =
        load_yaml(&root().join(".agent-harness/context/budget-policy.yaml")).unwrap();
    assert_eq!(cfg.version, 1);
    assert!(cfg.defaults.summarize_before_context);
    assert_eq!(cfg.raw_output_policy.max_default_lines, 200);
    assert!(!cfg.raw_output_policy.prefer.is_empty());
    assert_eq!(
        cfg.tool_output_containment.containment_action,
        "summarize_before_context"
    );
    assert!(
        cfg.tool_output_containment
            .durable_storage
            .generated_records_gitignored
    );
    assert!(!cfg.session_continuity.memory.is_empty());
}

#[test]
fn yaml_parity_hooks() {
    let cfg: HooksConfig = load_yaml(&root().join(".agent-hooks/config.yaml")).unwrap();
    assert_eq!(cfg.version, 1);
    assert_eq!(cfg.paths.index_db, ".agent-hooks/index/hooks.rusql");
    assert!(cfg.logging.rotate_by_date);
    assert!(!cfg.redaction.key_substrings.is_empty());
    assert!(!cfg.redaction.value_patterns.is_empty());
    assert_eq!(cfg.exports.otel_format, "opentelemetry-compatible-json");
}

#[test]
fn yaml_parity_fabricator() {
    let cfg: FabricatorConfig = load_yaml(&root().join(".fabricator/config.yaml")).unwrap();
    assert_eq!(cfg.version, "0.1");
    assert!(!cfg.required_generated_artifacts.is_empty());
    assert_eq!(cfg.defaults.proof_mode, "static_html_scan");
}

#[test]
fn yaml_parity_strategy() {
    let cfg: StrategyConfig = load_yaml(&root().join(".strategy/config.yaml")).unwrap();
    assert_eq!(cfg.version, "0.1");
    assert_eq!(cfg.layer_mode, "standalone_optional");
    assert!(!cfg.paths.is_empty());
    assert!(!cfg.required_example_artifacts.is_empty());
}
