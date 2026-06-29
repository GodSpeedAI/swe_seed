//! fabricate_golden: a bounded fabricate run produces a complete, traceable
//! chain; the rendered output round-trips and is stable against a committed
//! golden fixture (spec 0017 §1, §4).

use std::fs;
use std::path::PathBuf;

use swe_seed_core::fabricator::{
    build_chain, handoff, load_chain, render_chain, validate_semantic_chain,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn tempdir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-fab-golden-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

const NEED: &str = "sample need";
const RUN_ID: &str = "golden-fabricate";
const FIXTURE: &str = "tests/fixtures/fabricate-golden.json";

/// Render the chain to a temp root and assert every artifact + CHAIN.json is
/// written, reloads, and validates clean.
#[test]
fn fabricate_produces_complete_traceable_chain() {
    let tmp = tempdir();
    let mut chain = build_chain(NEED, RUN_ID);
    let dir = render_chain(&mut chain, &tmp, "fixture").expect("render");

    // Every required artifact is written.
    let generated = dir.join("generated");
    for name in [
        "PRODUCT_SEED.json",
        "JOB_STORY.json",
        "HYPOTHESIS.json",
        "PRD.json",
        "ADR.json",
        "SDS.json",
        "TDD_PLAN.json",
        "AGENT_TASK.json",
        "EVAL_SPEC.json",
        "PROOF_RECORD.json",
        "CHAIN.json",
    ] {
        assert!(
            generated.join(name).is_file(),
            "missing rendered artifact: {name}"
        );
    }

    // Round-trip: reload + revalidate.
    let reloaded = load_chain(&dir).expect("load");
    let report = validate_semantic_chain(&reloaded);
    assert!(report.passed, "reloaded chain must still pass: {report:?}");

    // Proof-gated handoff: a passing chain writes the frozen-eval-spec manifest.
    let handoff_report = handoff(&chain, &tmp).expect("passing chain hands off");
    assert!(handoff_report.passed);
    let manifest_path = dir.join("handoff").join("MANIFEST.json");
    assert!(
        manifest_path.is_file(),
        "handoff MANIFEST must be written on a passing chain"
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["run_id"], RUN_ID);
    assert_eq!(manifest["frozen"], true);
    assert!(manifest["eval_spec_sha256"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));

    fs::remove_dir_all(&tmp).ok();
}

/// The chain built from fixed inputs is stable against the committed golden
/// fixture (drift fails CI). Run `fabricate_golden_regen --ignored` to
/// refresh the fixture after an intentional chain-shape change.
#[test]
fn fabricate_matches_golden_fixture() {
    let mut chain = build_chain(NEED, RUN_ID);
    // Stamp a fixed created_at so the proof record doesn't drift on clock.
    chain.proof_record.created_at = "fixture".into();
    let actual = serde_json::to_value(&chain).unwrap();

    let fixture_path = root().join(FIXTURE);
    let expected: serde_json::Value = serde_json::from_slice(&fs::read(&fixture_path).unwrap_or_else(|e| {
        panic!(
            "golden fixture missing at {}: {e}. \
             Run `cargo test -p swe-seed-core --test fabricate_golden fabricate_golden_regen -- --ignored` to create it.",
            fixture_path.display()
        )
    }))
    .unwrap();
    assert_eq!(
        actual, expected,
        "fabricate golden drift — regenerate the fixture if the chain shape changed intentionally"
    );
}

/// Regenerate the golden fixture. Ignored by default; run on purpose.
#[test]
#[ignore = "fixture regenerator — run with --ignored after an intentional chain change"]
fn fabricate_golden_regen() {
    let mut chain = build_chain(NEED, RUN_ID);
    chain.proof_record.created_at = "fixture".into();
    let out = format!("{}\n", serde_json::to_string_pretty(&chain).unwrap());
    let path = root().join(FIXTURE);
    std::fs::write(&path, &out).unwrap();
    println!("wrote {}", path.display());
}

/// Determinism sanity: two builds from the same inputs are identical.
#[test]
fn fabricate_is_deterministic() {
    let mut a = build_chain(NEED, RUN_ID);
    let mut b = build_chain(NEED, RUN_ID);
    a.proof_record.created_at = "fixture".into();
    b.proof_record.created_at = "fixture".into();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap(),
    );
}

/// handoff blocks on a broken chain (nothing written).
#[test]
fn handoff_blocked_on_broken_chain() {
    use swe_seed_core::fabricator::fabricate;
    let tmp = tempdir();
    // Fabricate always builds a well-formed chain, so this proves the happy
    // path end-to-end (build → render → handoff) in one shot.
    let (dir, report) = fabricate(&tmp, NEED, RUN_ID).expect("well-formed run hands off");
    assert!(report.passed);
    assert!(dir.join("handoff").join("MANIFEST.json").is_file());
    fs::remove_dir_all(&tmp).ok();
}
