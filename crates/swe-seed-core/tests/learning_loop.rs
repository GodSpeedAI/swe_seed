//! learning_loop: the full reflect → record → adaptation → proposal loop and
//! the "no proposal without a rollback" gate (spec 0016 §1, §5).

use swe_seed_core::eval::SourceRef;
use swe_seed_core::learning::{
    can_promote_candidate, default_template, reflect, validate_proposal, validate_reflection,
    LearningCandidate, LearningDisposition, LearningRecord, SkillProposal,
};
use swe_seed_core::trace::TraceRecord;

fn finished_trace() -> TraceRecord {
    // Deserialized from JSON so the fixture survives RouteResult field churn.
    let json = r#"{
        "trace_id": "20260629T000000Z-smoke",
        "created_at": "t",
        "task": "checkpoint smoke",
        "route_decision_record": "rd",
        "route": {
            "job_type": "test",
            "route_card": ".agent-harness/routes/test.json",
            "confidence": "high",
            "assumption": null,
            "required_context": [],
            "required_skills": [],
            "work_loop": [],
            "required_artifacts": [],
            "proof": [],
            "done_when": [],
            "next_action": ""
        },
        "events": [],
        "verification": [],
        "unresolved_risks": [],
        "completion_claim": {"at": "t", "claim": "the checkpoint lifecycle preserves trace continuity"}
    }"#;
    serde_json::from_str(json).unwrap()
}

#[test]
fn reflect_produces_a_proposed_learning_record() {
    // Outcome 1: finished trace → reflection → LearningRecord.
    let trace = finished_trace();
    let record = reflect(&default_template(), &trace, "traces/smoke.json").unwrap();
    assert_eq!(record.disposition, LearningDisposition::ApprovedLesson);
    assert!(!record.summary.is_empty());
    assert_eq!(record.evidence.len(), 1);
    assert_eq!(record.evidence[0].path, "traces/smoke.json");
    // An ApprovedLesson is a real lesson → evidence is mandatory.
    assert!(validate_learning_record(&record).is_ok());
}

fn validate_learning_record(r: &LearningRecord) -> Result<(), String> {
    swe_seed_core::learning::validate_learning_record(r)
}

#[test]
fn no_change_allowed_false_requires_explicit_disposition() {
    // A NoChange record with empty rationale is rejected when the template
    // forbids a silent skip; with rationale it is accepted.
    let template = default_template(); // no_change_allowed = false
    let mk = |reason: &str| LearningRecord {
        id: "learn-x".into(),
        disposition: LearningDisposition::NoChange,
        summary: "nothing reusable".into(),
        evidence: vec![SourceRef { path: "t.json".into(), summary: "trace".into() }],
        decision_reason: reason.into(),
        follow_up_artifact: None,
        validation: vec![],
    };

    assert!(validate_reflection(&template, &mk("")).is_err(), "silent skip rejected");
    assert!(validate_reflection(&template, &mk("no claim; nothing to learn")).is_ok());

    // An unfinished trace (no completion claim) still reflects explicitly.
    let mut trace = finished_trace();
    trace.completion_claim = None;
    let record = reflect(&default_template(), &trace, "t.json").unwrap();
    assert_eq!(record.disposition, LearningDisposition::NoChange);
    assert!(!record.decision_reason.is_empty(), "no silent skip from reflect");
}

#[test]
fn proposal_without_rollback_is_rejected() {
    // Outcome 2: a SkillProposal without a rollback plan is rejected.
    fn proposal(rollback: &str) -> SkillProposal {
        SkillProposal {
            id: "prop-1".into(),
            proposed_skill_id: "skill-x".into(),
            observed_problem: "routing ties are broken non-deterministically".into(),
            evidence: vec![SourceRef { path: "traces/r.json".into(), summary: "run".into() }],
            proposed_behavior: vec!["tiebreak by reverse-alpha".into()],
            evals: vec![swe_seed_core::eval::EvalCheck {
                id: "chk".into(),
                eval_class: swe_seed_core::eval::EvalClass::LearningQuality,
                check_type: "static_required_patterns".into(),
                target: "target".into(),
                required: true,
                rule: "reverse-alpha".into(),
                evidence_required: "deterministic tiebreak".into(),
            }],
            rollout_plan: "render to all hosts".into(),
            rollback_plan: rollback.into(),
            validation: vec![],
        }
    }

    assert!(validate_proposal(&proposal("revert the rendered files")).is_ok());

    let bad = proposal("   ");
    let err = validate_proposal(&bad).unwrap_err();
    assert!(err.contains("rollback_plan"), "{err}");

    // Missing evals also blocks (a proposal must say how to verify itself).
    let mut no_evals = proposal("revert");
    no_evals.evals.clear();
    assert!(validate_proposal(&no_evals).is_err());
}

#[test]
fn candidate_promotion_requires_evidence_and_linked_regression() {
    // A candidate promotes only with evidence and, when it declares a required
    // regression case, a non-empty link to one.
    use swe_seed_core::learning::PromotionOutcome;
    fn candidate(reg: Option<&str>) -> LearningCandidate {
        LearningCandidate {
            id: "c".into(),
            source_run_id: "run".into(),
            candidate_type: "SkillProposal".into(),
            claim: "route ties need a stable tiebreak".into(),
            evidence: vec![SourceRef { path: "t.json".into(), summary: "trace".into() }],
            scope: "routing".into(),
            confidence: "high".into(),
            promotion_status: "proposed".into(),
            required_regression_case: reg.map(str::to_string),
        }
    }

    assert!(matches!(can_promote_candidate(&candidate(None)), PromotionOutcome::Allow { .. }));

    let mut no_evidence = candidate(None);
    no_evidence.evidence.clear();
    assert!(matches!(can_promote_candidate(&no_evidence), PromotionOutcome::Block { .. }));

    // Required regression declared but blank → blocked.
    assert!(matches!(can_promote_candidate(&candidate(Some("   "))), PromotionOutcome::Block { .. }));
    // Required regression declared and linked → allowed.
    assert!(matches!(can_promote_candidate(&candidate(Some("reg-1"))), PromotionOutcome::Allow { .. }));

    // Empty candidate_type → blocked (no unusable Allow target).
    let mut no_type = candidate(None);
    no_type.candidate_type = "  ".into();
    match can_promote_candidate(&no_type) {
        PromotionOutcome::Block { reason } => assert!(reason.contains("candidate_type"), "{reason}"),
        other => panic!("empty candidate_type must block, got {other:?}"),
    }
}
