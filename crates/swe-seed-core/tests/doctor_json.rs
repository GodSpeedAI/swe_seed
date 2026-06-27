//! doctor_json: doctor aggregates boundary + eval + frozen + drift; the JSON
//! schema is stable; overall is Fail (non-zero) iff any check fails.

use std::fs;
use std::path::PathBuf;

use swe_seed_core::doctor::{run_doctor, DoctorStatus};
use swe_seed_core::eval::EvalStatus;

fn real_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

/// A temp repo with the files the core eval spec checks against.
fn temp_repo() -> PathBuf {
    let real = real_root();
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-doctor-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(dir.join("tests/fixtures")).unwrap();
    fs::copy(real.join("AGENTS.md"), dir.join("AGENTS.md")).unwrap();
    fs::copy(real.join("README.md"), dir.join("README.md")).unwrap();
    fs::copy(
        real.join("tests/fixtures/eval.toml"),
        dir.join("tests/fixtures/eval.toml"),
    )
    .unwrap();
    dir
}

#[test]
fn doctor_passes_on_clean_repo() {
    let root = temp_repo();
    let report = run_doctor(&root);
    assert_eq!(
        report.overall,
        DoctorStatus::Pass,
        "doctor should pass on a clean repo: {report:?}"
    );
    // Aggregates boundary + eval + frozen + drift.
    let names: Vec<&str> = report.checks.iter().map(|c| c.name.as_str()).collect();
    assert!(names.iter().any(|n| n.starts_with("boundary")));
    assert!(names.iter().any(|n| n.starts_with("eval:")));
    assert!(names.iter().any(|n| *n == "frozen-integrity"));
    assert!(names.iter().any(|n| *n == "manifest-drift"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn doctor_json_schema_is_stable() {
    let root = temp_repo();
    let report = run_doctor(&root);

    // Serialize → parse as a generic value to lock the public schema shape.
    let json = serde_json::to_string(&report).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let obj = v.as_object().expect("report is object");
    assert!(obj.contains_key("checks"));
    assert!(obj.contains_key("overall"));
    let overall = obj["overall"].as_str().expect("overall is string");
    assert!(["Pass", "Fail", "Warn"].contains(&overall));
    for c in obj["checks"].as_array().unwrap() {
        let co = c.as_object().unwrap();
        assert!(co.contains_key("name"));
        assert!(["Pass", "Fail", "Warn"].contains(&co["status"].as_str().unwrap()));
        assert!(co.contains_key("detail"));
    }
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn doctor_fails_closed_when_core_eval_fails() {
    let root = temp_repo();
    // Break the core eval: remove README.md so the required readme-exists check fails.
    fs::remove_file(root.join("README.md")).unwrap();
    let report = run_doctor(&root);
    assert_eq!(
        report.overall,
        DoctorStatus::Fail,
        "a failing required eval check must fail doctor"
    );
    // The eval check itself must be Fail (not pass via waiver).
    let eval_check = report
        .checks
        .iter()
        .find(|c| c.name.starts_with("eval:"))
        .unwrap();
    assert_eq!(eval_check.status, DoctorStatus::Fail);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn eval_status_serializes_canonical_pascalcase() {
    // Lock the canonical status vocabulary (baml EvalStatus) in JSON.
    assert_eq!(
        serde_json::to_string(&EvalStatus::Pass).unwrap(),
        "\"Pass\""
    );
    assert_eq!(
        serde_json::to_string(&EvalStatus::Waived).unwrap(),
        "\"Waived\""
    );
}

#[test]
fn doctor_warns_preserved_when_core_spec_missing() {
    // No core eval spec → eval + frozen-integrity are both Warn; overall must
    // be Warn (not collapsed to Pass), and the frozen-integrity check is still
    // emitted so the report stays consistent (boundary + eval + frozen + drift).
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-doctor-nospec-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let report = run_doctor(&dir);
    assert_eq!(report.overall, DoctorStatus::Warn, "warn-only run must not collapse to Pass");
    let names: Vec<&str> = report.checks.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"eval"), "missing: {names:?}");
    assert!(names.contains(&"frozen-integrity"), "frozen check must be emitted even without a core spec: {names:?}");
    assert!(names.contains(&"manifest-drift"));
    assert!(names.iter().any(|n| n.starts_with("boundary")));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn drift_baseline_updates_on_mismatch() {
    // A stale baseline is detected as Warn AND rewritten to the current hash,
    // so the next run compares against the latest state (no stale comparison).
    use swe_seed_core::doctor::drift::manifest_drift_check;
    let root = temp_repo();

    // First run records the baseline.
    let _ = manifest_drift_check(&root);
    let sidecar = root.join(".swe-seed").join("manifest.sha256");
    assert!(sidecar.is_file(), "baseline should be recorded");

    // Corrupt the baseline with a stale hash.
    fs::write(&sidecar, "sha256:stale").unwrap();
    let warned = manifest_drift_check(&root);
    assert_eq!(warned.status, DoctorStatus::Warn, "stale baseline must warn");

    // The baseline must now hold the current hash (updated), so the next run
    // reports unchanged — proving the baseline was refreshed, not left stale.
    let next = manifest_drift_check(&root);
    assert_eq!(next.status, DoctorStatus::Pass, "baseline should have been updated");
    assert!(next.detail.contains("unchanged"));

    let _ = fs::remove_dir_all(&root);
}
