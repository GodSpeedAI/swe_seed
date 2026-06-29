//! chain_integrity: a broken traceability link → SemanticChainValidationReport
//! .passed = false → handoff blocked; EARS/Gherkin shape validation rejects
//! malformed requirements/scenarios (spec 0017 §2–3).

use swe_seed_core::fabricator::{
    build_chain, handoff, validate_ears_requirement, validate_gherkin_scenario,
    validate_semantic_chain, EARSPattern, EARSRequirement, GherkinScenario,
};

#[test]
fn well_formed_chain_passes_integrity() {
    let chain = build_chain("a bounded sample need", "ci-run");
    let report = validate_semantic_chain(&chain);
    assert!(
        report.passed,
        "expected pass, got missing={:?} blocked={:?}",
        report.missing_links, report.blocked_claims
    );
    assert!(report.missing_links.is_empty());
    assert!(report.blocked_claims.is_empty());
}

#[test]
fn broken_prd_to_sds_link_blocks_handoff() {
    // Outcome 2: drop the PRD→SDS link (SDS points at a non-existent PRD) →
    // passed=false and the broken edge is named in missing_links.
    let mut chain = build_chain("sample need", "ci-broken-sds");
    chain.sds.prd_id = "does-not-exist".into();
    let report = validate_semantic_chain(&chain);
    assert!(!report.passed, "broken PRD->SDS link must fail the chain");
    assert!(
        report
            .missing_links
            .iter()
            .any(|m| m.contains("PRD -> SDS")),
        "missing_links must name the PRD->SDS edge: {:?}",
        report.missing_links
    );

    // Handoff is gated: a broken chain writes no handoff manifest.
    let root = tempdir();
    let result = handoff(&chain, &root);
    assert!(result.is_err(), "handoff must be blocked on broken chain");
    assert!(!root
        .join(".fabricator/runs/ci-broken-sds/handoff/MANIFEST.json")
        .exists());
}

#[test]
fn broken_seed_to_hypothesis_link_is_caught() {
    let mut chain = build_chain("sample need", "ci-broken-hyp");
    chain.hypothesis.product_seed_id = "wrong".into();
    let report = validate_semantic_chain(&chain);
    assert!(!report.passed);
    assert!(report
        .missing_links
        .iter()
        .any(|m| m.contains("ProductSeed -> ProductHypothesis")));
}

#[test]
fn unlinked_sds_component_is_caught() {
    // A component that links no requirement (or an unknown one) breaks the
    // requirement-traceability edge PRD -> SDSComponent.
    let mut chain = build_chain("sample need", "ci-broken-comp");
    chain.sds.components[0].linked_requirement_ids = vec!["REQ-DOES-NOT-EXIST".into()];
    let report = validate_semantic_chain(&chain);
    assert!(!report.passed);
    assert!(report
        .missing_links
        .iter()
        .any(|m| m.contains("SDSComponent")));
}

#[test]
fn eval_check_linking_unknown_requirement_blocks_proof() {
    // An eval check that cites a non-existent requirement is an unresolvable
    // proof claim → blocked_claims (not just a missing link).
    let mut chain = build_chain("sample need", "ci-blocked-claim");
    chain.eval_spec.checks[0]
        .linked_requirement_ids
        .push("GHOST-REQ".into());
    let report = validate_semantic_chain(&chain);
    assert!(!report.passed);
    assert!(report
        .blocked_claims
        .iter()
        .any(|c| c.contains("GHOST-REQ")));
}

#[test]
fn waived_link_is_acknowledged_not_failed() {
    // A documented edge with a waiver_id lands in waived_links and does NOT
    // flip passed to false by itself.
    use swe_seed_core::fabricator::TraceabilityLink;
    let mut chain = build_chain("sample need", "ci-waiver");
    chain.links.push(TraceabilityLink {
        upstream_id: chain.product_seed.id.clone(),
        downstream_id: chain.job_story.id.clone(),
        relation: "derives".into(),
        waiver_id: Some("W-001".into()),
    });
    let report = validate_semantic_chain(&chain);
    assert!(report.passed, "a waiver alone must not fail the chain");
    assert!(report.waived_links.iter().any(|w| w.contains("W-001")));
}

#[test]
fn ears_event_driven_requires_trigger() {
    // Outcome 3: EARS shape validation. EventDriven without a trigger → reject.
    let mut req = ears_req("req-ev", EARSPattern::EventDriven);
    req.trigger = None;
    assert!(validate_ears_requirement(&req).is_err());

    // With a trigger → ok.
    req.trigger = Some("the player presses start".into());
    assert!(validate_ears_requirement(&req).is_ok());

    // Each non-Ubiquitous pattern needs its named clause.
    for (pattern, field) in [
        (EARSPattern::StateDriven, "state"),
        (EARSPattern::OptionalFeature, "feature"),
        (EARSPattern::UnwantedBehavior, "unwanted_condition"),
    ] {
        let mut r = ears_req("req-x", pattern);
        set_clause(&mut r, field, None);
        assert!(
            validate_ears_requirement(&r).is_err(),
            "{pattern:?} without {field} must fail"
        );
        set_clause(&mut r, field, Some("present".into()));
        assert!(
            validate_ears_requirement(&r).is_ok(),
            "{pattern:?} with {field} must pass"
        );
    }
}

#[test]
fn non_testable_ears_requires_reason() {
    let mut req = ears_req("req-nt", EARSPattern::Ubiquitous);
    req.testable = false;
    req.non_testable_reason = None;
    assert!(
        validate_ears_requirement(&req).is_err(),
        "non-testable needs a reason"
    );
    req.non_testable_reason = Some("requires human judgement".into());
    assert!(validate_ears_requirement(&req).is_ok());
}

#[test]
fn gherkin_scenario_requires_given_when_then() {
    let mk = |given: &[&str], when: &[&str], then: &[&str]| GherkinScenario {
        id: "scn".into(),
        name: "scenario".into(),
        given_steps: given.iter().map(|s| s.to_string()).collect(),
        when_steps: when.iter().map(|s| s.to_string()).collect(),
        then_steps: then.iter().map(|s| s.to_string()).collect(),
        linked_requirement_ids: Vec::new(),
        waiver_reason: None,
    };

    // Complete skeleton → ok.
    assert!(validate_gherkin_scenario(&mk(&["a"], &["b"], &["c"])).is_ok());
    // Missing any of Given/When/Then → reject.
    assert!(validate_gherkin_scenario(&mk(&[], &["b"], &["c"])).is_err());
    assert!(validate_gherkin_scenario(&mk(&["a"], &[], &["c"])).is_err());
    assert!(validate_gherkin_scenario(&mk(&["a"], &["b"], &[])).is_err());
    // All-blank steps count as missing.
    assert!(validate_gherkin_scenario(&mk(&["  "], &["b"], &["c"])).is_err());
}

// ----------------------------- helpers -----------------------------

fn ears_req(id: &str, pattern: EARSPattern) -> EARSRequirement {
    EARSRequirement {
        id: id.into(),
        pattern,
        system_name: "system".into(),
        condition: None,
        trigger: None,
        state: None,
        feature: None,
        unwanted_condition: None,
        response: "respond".into(),
        rationale: None,
        source_refs: Vec::new(),
        ears_text: "The system shall respond.".into(),
        testable: true,
        non_testable_reason: None,
    }
}

fn set_clause(req: &mut EARSRequirement, field: &str, val: Option<String>) {
    match field {
        "condition" => req.condition = val,
        "trigger" => req.trigger = val,
        "state" => req.state = val,
        "feature" => req.feature = val,
        "unwanted_condition" => req.unwanted_condition = val,
        _ => {}
    }
}

fn tempdir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-fab-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
