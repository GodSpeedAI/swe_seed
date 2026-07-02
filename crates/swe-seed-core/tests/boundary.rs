//! Boundary validation (spec 0018). Falsifiable: a misplaced or upward-
//! referencing artifact makes `BoundaryReport.passed = false`.

use swe_seed_core::seed::{assemble_default, validate_boundaries, LayerCapability, LayerName};

#[test]
fn clean_default_registry_passes() {
    let manifest = assemble_default();
    let report = validate_boundaries(&manifest);
    assert!(
        report.passed,
        "clean registry should pass, findings: {:?}",
        report.findings
    );
}

#[test]
fn upward_reference_fails() {
    // Fabricator (innermost) owning a Harness-layer artifact = inner → outer =
    // upward reference. Dependencies must point inward only.
    let mut manifest = assemble_default();
    manifest.capabilities.push(LayerCapability {
        id: "bad-upward".into(),
        owner: LayerName::Fabricator,
        source_spec: "docs/specs/0017-fabricator-layer.md".into(),
        artifact_paths: vec![".agent-harness/traces/".into()],
        purpose: "misplaced".into(),
        required: false,
    });
    let report = validate_boundaries(&manifest);
    assert!(
        !report.passed,
        "upward reference must fail the boundary report"
    );
    let found = report
        .findings
        .iter()
        .any(|f| f.artifact_path.contains(".agent-harness/traces"));
    assert!(
        found,
        "expected an upward-reference finding, got {:?}",
        report.findings
    );
}

#[test]
fn harness_referencing_fabricator_is_allowed() {
    // Harness depending on a Fabricator artifact is inward (allowed).
    let mut manifest = assemble_default();
    manifest.capabilities.push(LayerCapability {
        id: "ok-inward".into(),
        owner: LayerName::Harness,
        source_spec: "docs/specs/0013-eval-and-proof.md".into(),
        artifact_paths: vec![".fabricator/runs/".into()],
        purpose: "eval reuses fabricator handoff".into(),
        required: false,
    });
    let report = validate_boundaries(&manifest);
    assert!(
        report.passed,
        "inward reference should pass, findings: {:?}",
        report.findings
    );
}

#[test]
fn missing_required_capability_fails() {
    let mut manifest = assemble_default();
    manifest.capabilities.retain(|c| c.id != "routing");
    let report = validate_boundaries(&manifest);
    assert!(!report.passed);
    let found = report
        .findings
        .iter()
        .any(|f| f.issue.contains("missing required capability 'routing'"));
    assert!(
        found,
        "expected a missing-required finding, got {:?}",
        report.findings
    );
}

#[test]
fn duplicate_capability_id_fails() {
    let mut manifest = assemble_default();
    let dup = manifest
        .capabilities
        .first()
        .expect("registry non-empty")
        .clone();
    manifest.capabilities.push(dup);
    let report = validate_boundaries(&manifest);
    assert!(!report.passed);
    assert!(report
        .findings
        .iter()
        .any(|f| f.issue.contains("duplicate capability id")));
}

#[test]
fn unclassified_artifact_path_fails() {
    // A path matching no layer prefix surfaces as a finding instead of being
    // silently treated as a valid SweSeed artifact.
    let mut manifest = assemble_default();
    manifest.capabilities.push(LayerCapability {
        id: "bad-path".into(),
        owner: LayerName::SweSeed,
        source_spec: "docs/specs/0018-layer-boundary-governance.md".into(),
        artifact_paths: vec!["totally/unknown/zone/".into()],
        purpose: "unrecognized".into(),
        required: false,
    });
    let report = validate_boundaries(&manifest);
    assert!(!report.passed);
    assert!(report
        .findings
        .iter()
        .any(|f| f.issue.contains("unclassified artifact path")));
}

#[test]
fn assemble_records_project_seed_path() {
    // assemble_default derives project_seed_path from the seed's source spec,
    // not a placeholder.
    let manifest = assemble_default();
    assert!(
        !manifest.project_seed_path.is_empty() && manifest.project_seed_path != "<self>",
        "project_seed_path should be the seed's real source path, got {:?}",
        manifest.project_seed_path
    );
}
