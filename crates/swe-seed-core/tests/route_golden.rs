//! route_golden: the Rust router selects the same card and emits the same
//! route-decision shape as the captured Python record for "checkpoint smoke".
//!
//! Timestamps (trace_id, created_at) are inherently non-reproducible, so parity
//! is asserted on the deterministic routing object + task + decision_basis.

use std::path::PathBuf;

use serde_json::Value;

use swe_seed_core::route::{build_route_result, REQUIRED_JOB_TYPES};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

/// The captured Python route-decision record for "checkpoint smoke".
fn golden_record() -> Value {
    let path =
        root().join(".agent-harness/traces/route-decisions/20260617T002316Z-checkpoint-smoke.json");
    let bytes = std::fs::read(&path).expect("read golden");
    serde_json::from_slice(&bytes).expect("parse golden")
}

#[test]
fn route_matches_golden_checkpoint_smoke() {
    let result = build_route_result(&root(), "checkpoint smoke").expect("route");
    let golden = golden_record();
    let golden_route = golden.get("route").expect("golden.route");

    // Byte-compatible routing decision (all deterministic fields).
    let got = serde_json::to_value(&result).unwrap();
    assert_eq!(
        got, *golden_route,
        "Rust route result differs from the captured Python golden record"
    );

    // The decision_basis label is part of the frozen format.
    assert_eq!(
        golden.get("decision_basis").and_then(Value::as_str),
        Some("deterministic token overlap against route-card triggers, examples, and job type"),
    );
}

#[test]
fn checkpoint_smoke_routes_to_test_low_confidence() {
    // The canonical no-match case: ties resolve by reverse-alpha id → "test",
    // low confidence, fallback assumption.
    let result = build_route_result(&root(), "checkpoint smoke").expect("route");
    assert_eq!(result.job_type, "test");
    assert_eq!(result.confidence, "low");
    assert_eq!(
        result.assumption.as_deref(),
        Some("No strong semantic match; selected safest default route.")
    );
    assert!(result.route_card.ends_with("routes/test.json"));
}

#[test]
fn all_required_job_types_resolve() {
    // Outcome 4: every one of the 11 required job types has a resolvable card.
    let cards = swe_seed_core::route::load_routes(&root()).expect("load routes");
    let have: std::collections::HashSet<&str> = cards.iter().map(|c| c.job_type.as_str()).collect();
    for jt in REQUIRED_JOB_TYPES {
        assert!(
            have.contains(*jt),
            "no route card resolves required job type '{jt}'"
        );
    }
    assert_eq!(REQUIRED_JOB_TYPES.len(), 11);
}
