//! proof_evidence: ProofRecord rejects claims without evidence, and a Waived
//! check requires a reason (spec 0013).

use swe_seed_core::eval::{
    validate_proof, waived_requires_reason, EvalCheckResult, EvalClass, EvalStatus, ProofRecord,
    SourceRef,
};

fn proof(id: &str, claims: &[&str], evidence: &[&str]) -> ProofRecord {
    ProofRecord {
        id: id.into(),
        run_id: "r".into(),
        claims: claims.iter().map(|s| s.to_string()).collect(),
        evidence: evidence
            .iter()
            .map(|p| SourceRef {
                path: p.to_string(),
                summary: "evidence".into(),
            })
            .collect(),
        skipped_checks: Vec::new(),
        unresolved_risks: Vec::new(),
    }
}

#[test]
fn claim_without_evidence_is_rejected() {
    // claim present, evidence empty → rejected.
    let rec = proof("p1", &["the behavior works"], &[]);
    assert!(validate_proof(&rec).is_err());

    // claim backed by evidence → ok.
    let rec = proof("p2", &["the behavior works"], &["tests/x.rs"]);
    assert!(validate_proof(&rec).is_ok());

    // no claims → vacuously ok regardless of evidence.
    let rec = proof("p3", &[], &[]);
    assert!(validate_proof(&rec).is_ok());
}

#[test]
fn every_claim_needs_its_own_evidence() {
    // Two claims but only one evidence ref → rejected (one SourceRef can't
    // back multiple claims; positional per-claim linkage).
    let rec = proof("p4", &["claim a", "claim b"], &["tests/a.rs"]);
    assert!(
        validate_proof(&rec).is_err(),
        "two claims with one evidence ref must be rejected"
    );

    // Two claims, two evidence refs → ok.
    let rec = proof("p5", &["claim a", "claim b"], &["tests/a.rs", "tests/b.rs"]);
    assert!(validate_proof(&rec).is_ok());

    // Extra evidence beyond claims is fine.
    let rec = proof("p6", &["claim a"], &["tests/a.rs", "tests/b.rs"]);
    assert!(validate_proof(&rec).is_ok());
}

#[test]
fn evidence_with_empty_path_is_rejected() {
    use swe_seed_core::eval::ProofRecord;
    let rec = ProofRecord {
        id: "p7".into(),
        run_id: "r".into(),
        claims: vec!["a claim".into()],
        evidence: vec![SourceRef {
            path: "  ".into(),
            summary: "empty path".into(),
        }],
        skipped_checks: Vec::new(),
        unresolved_risks: Vec::new(),
    };
    assert!(validate_proof(&rec).is_err());
}

fn check_result(status: EvalStatus, reason: Option<&str>) -> EvalCheckResult {
    EvalCheckResult {
        id: "c".into(),
        eval_class: EvalClass::ProcessCompliance,
        status,
        evidence: "e".into(),
        failure_reason: reason.map(str::to_string),
    }
}

#[test]
fn waived_check_requires_a_reason() {
    // Waived with no reason → rejected.
    let r = check_result(EvalStatus::Waived, None);
    assert!(waived_requires_reason(&r).is_err());

    // Waived with an empty reason → rejected.
    let r = check_result(EvalStatus::Waived, Some("   "));
    assert!(waived_requires_reason(&r).is_err());

    // Waived with a reason → ok.
    let r = check_result(EvalStatus::Waived, Some("optional; deferred to next phase"));
    assert!(waived_requires_reason(&r).is_ok());

    // Non-waived statuses do not require a reason.
    assert!(waived_requires_reason(&check_result(EvalStatus::Pass, None)).is_ok());
    assert!(waived_requires_reason(&check_result(EvalStatus::Fail, None)).is_ok());
}
