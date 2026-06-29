//! adaptation_gate: a blocked `AdaptationEligibility` eval blocks promotion;
//! the decision cites the four eval results; eligibility-pass grants promotion
//! only when product + learning quality also pass (spec 0016 §3).

use swe_seed_core::eval::{EvalCheckResult, EvalClass, EvalResult, EvalStatus};
use swe_seed_core::learning::{build_adaptation_decision, promotion_gate, AdaptationDecision};

fn result(id: &str, run: &str, status: EvalStatus, class: EvalClass) -> EvalResult {
    EvalResult {
        eval_id: id.into(),
        run_id: run.into(),
        status,
        checks: vec![EvalCheckResult {
            id: format!("{id}-c"),
            eval_class: class,
            status,
            evidence: "e".into(),
            failure_reason: None,
        }],
        summary: format!("{status:?}"),
        created_at: "t".into(),
    }
}

/// One EvalResult carrying checks across all four classes (a realistic run).
fn full_run(run: &str, product: EvalStatus, process: EvalStatus, learning: EvalStatus, elig: EvalStatus) -> Vec<EvalResult> {
    let checks = vec![
        EvalCheckResult { id: "p".into(), eval_class: EvalClass::ProductOutcome, status: product, evidence: "e".into(), failure_reason: None },
        EvalCheckResult { id: "pc".into(), eval_class: EvalClass::ProcessCompliance, status: process, evidence: "e".into(), failure_reason: None },
        EvalCheckResult { id: "l".into(), eval_class: EvalClass::LearningQuality, status: learning, evidence: "e".into(), failure_reason: None },
        EvalCheckResult { id: "a".into(), eval_class: EvalClass::AdaptationEligibility, status: elig, evidence: "e".into(), failure_reason: None },
    ];
    let agg = if [product, process, learning, elig].iter().all(|s| *s == EvalStatus::Pass) {
        EvalStatus::Pass
    } else {
        EvalStatus::Fail
    };
    vec![EvalResult {
        eval_id: "full".into(),
        run_id: run.into(),
        status: agg,
        checks,
        summary: "full".into(),
        created_at: "t".into(),
    }]
}

#[test]
fn blocked_eligibility_blocks_all_promotion() {
    // Product + learning pass, but AdaptationEligibility fails → everything blocked.
    let results = full_run(
        "run-1",
        EvalStatus::Pass,
        EvalStatus::Pass,
        EvalStatus::Pass,
        EvalStatus::Fail,
    );
    let decision = build_adaptation_decision("run-1", &results);

    // Outcome 3: a blocked AdaptationEligibility eval blocks promotion.
    assert!(!decision.promotion_allowed(), "eligibility fail must block promotion: {decision:?}");
    assert!(decision.allowed_adaptations.is_empty(), "no adaptation granted on eligibility fail");
    assert!(decision.blocked_adaptations.iter().any(|a| a == "skill_promotion"));
    // The decision must cite all four eval results (spec 0016 data model).
    assert_eq!(decision.product_result, EvalStatus::Pass);
    assert_eq!(decision.adaptation_result, EvalStatus::Fail);
    assert!(decision.reason.contains("AdaptationEligibility"));

    // The promotion gate enforces it: a proposal cannot promote.
    assert!(promotion_gate(&decision).is_err());
}

#[test]
fn passing_eligibility_still_requires_product_and_learning() {
    // Eligibility passes, but learning quality fails → skill promotion blocked,
    // but the eligibility gate itself no longer short-circuits everything.
    let results = full_run(
        "run-2",
        EvalStatus::Pass,
        EvalStatus::Pass,
        EvalStatus::Fail,
        EvalStatus::Pass,
    );
    let decision = build_adaptation_decision("run-2", &results);
    assert!(!decision.promotion_allowed(), "learning fail must block promotion");
    assert!(decision.allowed_adaptations.iter().any(|a| a == "adr_update"), "adr_update only needs process");
    assert!(promotion_gate(&decision).is_err());

    // Everything passes → promotion allowed.
    let results = full_run(
        "run-3",
        EvalStatus::Pass,
        EvalStatus::Pass,
        EvalStatus::Pass,
        EvalStatus::Pass,
    );
    let decision = build_adaptation_decision("run-3", &results);
    assert!(decision.promotion_allowed(), "all-pass must allow promotion");
    assert!(decision.blocked_adaptations.is_empty());
    assert!(promotion_gate(&decision).is_ok());
}

#[test]
fn missing_eligibility_class_is_inconclusive_and_blocks() {
    // A run with no AdaptationEligibility check at all: the class is
    // Inconclusive (fail-closed), so promotion is blocked — an unknown gate
    // never grants trust.
    let results = vec![
        result("p", "run-4", EvalStatus::Pass, EvalClass::ProductOutcome),
        result("l", "run-4", EvalStatus::Pass, EvalClass::LearningQuality),
    ];
    let decision = build_adaptation_decision("run-4", &results);
    assert_eq!(decision.adaptation_result, EvalStatus::Inconclusive);
    assert!(!decision.promotion_allowed());
}

#[test]
fn decision_round_trips_through_serde() {
    // The decision is a durable record (written under .agent-harness/learning/);
    // it must round-trip JSON preserving all four cited eval statuses.
    let results = full_run(
        "run-5",
        EvalStatus::Pass,
        EvalStatus::Waived,
        EvalStatus::Fail,
        EvalStatus::Inconclusive,
    );
    let decision = build_adaptation_decision("run-5", &results);
    let json = serde_json::to_string(&decision).unwrap();
    let back: AdaptationDecision = serde_json::from_str(&json).unwrap();
    assert_eq!(back.product_result, decision.product_result);
    assert_eq!(back.process_result, decision.process_result);
    assert_eq!(back.learning_result, decision.learning_result);
    assert_eq!(back.adaptation_result, decision.adaptation_result);
    assert_eq!(back.allowed_adaptations, decision.allowed_adaptations);
    assert_eq!(back.blocked_adaptations, decision.blocked_adaptations);
}
