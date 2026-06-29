//! ingest_pipeline: discover→fetch→scan→normalize→project→verify yields
//! SkillIR with provenance; a blocking scan prevents projection (spec 0007).

use std::fs;
use std::path::PathBuf;

use swe_seed_core::security::{
    exceptions::Exceptions,
    gate::{can_activate, scan_blocks_projection},
    scan_result::{ScanFinding, ScanResult, ScanStatus},
};
use swe_seed_core::skill::{discover, ingest_with_scan, pipeline_scanning};

fn real_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn fixture_root() -> PathBuf {
    // A temp root whose `.agent-harness/skills/` holds the fixture skill.
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-skill-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let skills = dir.join(".agent-harness/skills/fixture");
    fs::create_dir_all(&skills).unwrap();
    fs::copy(
        real_root().join("tests/fixtures/skill-ir.json"),
        skills.join("fixture-skill.json"),
    )
    .unwrap();
    dir
}

#[test]
fn pipeline_yields_skillir_with_provenance() {
    let root = fixture_root();
    // Inject a deterministic stub scanner → Pending, regardless of host PATH.
    let (records, blocked) = pipeline_scanning(&root, &Exceptions::new(), |_| {
        ScanResult::pending("stub", "stubbed scanner")
    })
    .unwrap();
    assert_eq!(records.len(), 1);
    let rec = &records[0];
    assert_eq!(rec.ir.id, "fixture-skill");
    assert_eq!(rec.ir.category, "test");
    // tolerant load: inputs.required coerced to the string[] field
    assert_eq!(rec.ir.inputs, vec!["a-contract".to_string()]);
    // provenance hash present
    assert!(rec.source_hash.starts_with("sha256:"));
    // Stubbed scan is Pending (no faked pass), and not blocking.
    assert_eq!(rec.scan.status, ScanStatus::Pending);
    assert!(blocked.is_empty(), "Pending is not blocking");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn blocking_scan_prevents_projection() {
    // A Critical scan with an unwaived finding blocks projection; waiving it
    // removes the block.
    let scan = ScanResult {
        skill_id: "fixture-skill".into(),
        status: ScanStatus::Critical,
        findings: vec![ScanFinding {
            id: "CVE-evil".into(),
            severity: "high".into(),
            message: "evil pattern".into(),
        }],
        detail: "critical".into(),
    };

    let exc = Exceptions::new();
    assert!(
        scan_blocks_projection(&scan, &exc),
        "critical finding must block"
    );

    let mut waived = Exceptions::new();
    waived.waive("CVE-evil");
    assert!(
        !scan_blocks_projection(&scan, &waived),
        "a waived finding must not block"
    );
}

#[test]
fn blocking_status_with_empty_findings_still_blocks() {
    // Fail-closed: a Critical status blocks even when there are no findings.
    let scan = ScanResult {
        skill_id: "x".into(),
        status: ScanStatus::Critical,
        findings: Vec::new(),
        detail: "critical".into(),
    };
    assert!(
        scan_blocks_projection(&scan, &Exceptions::new()),
        "empty findings must not bypass a blocking status"
    );
}

#[test]
fn activation_requires_source_hash_and_terminal_scan() {
    let root = fixture_root();
    // Deterministic stub scan via DI (no host PATH dependence).
    let mut rec = ingest_with_scan(
        &root.join(".agent-harness/skills/fixture/fixture-skill.json"),
        ScanResult::pending("fixture-skill", "stubbed scanner"),
    )
    .unwrap();

    // Pending scan (the default here) must NOT activate, even with a hash.
    assert!(!can_activate(&rec, &Exceptions::new()));

    // Simulate a terminal Clean scan: now activates (hash + terminal + clean).
    rec.scan.status = ScanStatus::Clean;
    assert!(can_activate(&rec, &Exceptions::new()));

    // Missing source_hash → no activation even when clean.
    rec.source_hash.clear();
    assert!(!can_activate(&rec, &Exceptions::new()));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn discover_finds_real_repo_skills() {
    // The real repo ships several skill IR files; discover must list them.
    let paths = discover(&real_root()).unwrap();
    assert!(paths.len() >= 3, "expected several skills, got {:?}", paths);
    assert!(paths
        .iter()
        .any(|p| p.file_stem().unwrap().to_str().unwrap() == "implement-with-proof"));
}
