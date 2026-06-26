//! Idempotent regeneration (spec 0018 req 5). Unchanged inputs → empty diff;
//! approved decisions preserved; deterministic across runs.

use std::path::PathBuf;

use swe_seed_core::seed::{regenerate, SeedRegenerationInput};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn input() -> SeedRegenerationInput {
    SeedRegenerationInput {
        swe_seed_spec_path: ".agents/specs/0018-layer-boundary-governance.md".into(),
        approved_project_seeds: vec![".agents/specs/0002-swe-seed-centralization-layer.md".into()],
        approved_seed_package_manifests: vec![],
        approved_layer_capability_maps: vec![".agents/specs/0003-capability-registry.md".into()],
        approved_lower_layer_artifact_refs: vec![
            ".agents/specs/0012-existing-harness-reconciliation.md".into(),
        ],
    }
}

#[test]
fn regenerate_is_idempotent_on_unchanged_inputs() {
    let root = root();
    let plan1 = regenerate(&root, &input());
    let plan2 = regenerate(&root, &input());

    // Unchanged (present) inputs => no proposed diffs.
    assert!(
        plan1.proposed_diffs.is_empty(),
        "expected empty diff, got {:?}",
        plan1.proposed_diffs
    );
    // Approved decisions are preserved.
    assert!(!plan1.preserved_decisions.is_empty());
    // Deterministic: identical plans across runs.
    assert_eq!(plan1, plan2, "regenerate must be deterministic");
}

#[test]
fn regenerate_flags_missing_approved_artifact() {
    // A non-existent approved artifact becomes a proposed diff (to recreate).
    let root = root();
    let input = SeedRegenerationInput {
        swe_seed_spec_path: "x".into(),
        approved_project_seeds: vec!["does/not/exist.md".into()],
        approved_seed_package_manifests: vec![],
        approved_layer_capability_maps: vec![],
        approved_lower_layer_artifact_refs: vec![],
    };
    let plan = regenerate(&root, &input);
    assert!(!plan.proposed_diffs.is_empty());
    assert!(plan.preserved_decisions.is_empty());
}

#[test]
fn regenerate_includes_governing_spec_in_plan() {
    // The swe_seed_spec_path is part of plan identity (resolved, recorded),
    // so two inputs differing only in spec path no longer collapse to one plan.
    let root = root();
    let plan = regenerate(&root, &input());
    assert!(plan
        .source_artifacts
        .iter()
        .any(|s| s.path.ends_with("0018-layer-boundary-governance.md")
            && s.summary == "governing spec"));
    assert!(plan
        .preserved_decisions
        .iter()
        .any(|p| p.ends_with("0018-layer-boundary-governance.md")));
}
