//! Fail-closed provenance verification (spec 0009). verify fails on missing
//! source_hash/license_tag, code_copied=true, or a non-clear license status.

use swe_seed_core::provenance::{
    verify_dir, verify_manifest_records, verify_record, ProvenanceProblem, ProvenanceRecord,
};
use swe_seed_core::seed;

fn clean_record() -> ProvenanceRecord {
    ProvenanceRecord {
        capability_id: "example".into(),
        source_id: "example-src".into(),
        source_kind: "git".into(),
        source_location: "https://example.invalid/repo".into(),
        source_ref: "v1.0.0".into(),
        source_hash: "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            .into(),
        license_tag: "MIT".into(),
        license_status: "clear".into(),
        inspected_at: "2026-06-26".into(),
        scan_ref: ".swe-seed/scans/example/abc.json".into(),
        code_copied: false,
        notes: "".into(),
    }
}

#[test]
fn complete_record_passes() {
    assert!(verify_record(&clean_record()).is_ok());
}

#[test]
fn missing_source_hash_fails() {
    let mut r = clean_record();
    r.source_hash = "".into();
    assert_eq!(verify_record(&r), Err(ProvenanceProblem::MissingSourceHash));
}

#[test]
fn missing_license_tag_fails() {
    let mut r = clean_record();
    r.license_tag = "  ".into();
    assert_eq!(verify_record(&r), Err(ProvenanceProblem::MissingLicenseTag));
}

#[test]
fn code_copied_fails() {
    let mut r = clean_record();
    r.code_copied = true;
    assert_eq!(verify_record(&r), Err(ProvenanceProblem::CodeCopied));
}

#[test]
fn non_clear_license_fails() {
    let mut r = clean_record();
    r.license_status = "ambiguous".into();
    match verify_record(&r) {
        Err(ProvenanceProblem::LicenseNotClear { status }) => assert_eq!(status, "ambiguous"),
        other => panic!("expected LicenseNotClear, got {other:?}"),
    }
}

#[test]
fn malformed_source_hash_fails() {
    // Valid prefix, non-hex payload.
    let mut r = clean_record();
    r.source_hash = "sha256:xyz-not-hex".into();
    assert_eq!(
        verify_record(&r),
        Err(ProvenanceProblem::MalformedSourceHash)
    );
    // Missing prefix entirely.
    let mut r2 = clean_record();
    r2.source_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into();
    assert_eq!(
        verify_record(&r2),
        Err(ProvenanceProblem::MalformedSourceHash)
    );
}

#[test]
fn verify_dir_collects_problems_and_records() {
    let dir = std::env::temp_dir().join(format!("swe-seed-prov-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // good.json: complete, clear.
    let good = clean_record();
    std::fs::write(dir.join("good.json"), serde_json::to_string(&good).unwrap()).unwrap();

    // bad.json: code_copied=true.
    let mut bad = clean_record();
    bad.capability_id = "bad-cap".into();
    bad.code_copied = true;
    std::fs::write(dir.join("bad.json"), serde_json::to_string(&bad).unwrap()).unwrap();

    let (records, problems) = verify_dir(&dir).unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].0, "bad-cap");
    assert_eq!(problems[0].1, ProvenanceProblem::CodeCopied);

    std::fs::remove_dir_all(&dir).unwrap();

    // A missing directory is not an error (vacuously ok). Use a unique child
    // under temp_dir() left uncreated so the test is portable, not pinned to
    // an environment-specific absolute path.
    let nope = std::env::temp_dir().join(format!(
        "swe-seed-prov-missing-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let (records, problems) = verify_dir(&nope).unwrap();
    assert!(records.is_empty());
    assert!(problems.is_empty());
}

#[test]
fn strict_manifest_verification_fails_when_active_capability_records_are_missing() {
    // Given: an assembled manifest with active capabilities and an empty provenance directory.
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-prov-strict-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let manifest = seed::assemble_default();

    // When: provenance is verified against the manifest.
    let (_records, problems) = verify_manifest_records(&dir, &manifest).unwrap();

    // Then: missing capability records are reported instead of passing vacuously.
    assert!(
        problems
            .iter()
            .any(|(id, problem)| id == "provenance" && *problem == ProvenanceProblem::MissingRecord),
        "expected a missing provenance capability record, got {problems:?}"
    );
    assert!(
        problems.len() >= manifest.capabilities.len(),
        "every active capability should require a provenance record"
    );

    std::fs::remove_dir_all(&dir).unwrap();
}
