use std::fs;
use std::process::Command;

fn temp_root(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-cr-learning-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("create temp root");
    dir
}

fn write_skill_record(path: &std::path::Path) {
    fs::write(
        path,
        r#"{
  "id": "learn-owned",
  "disposition": "SkillProposal",
  "summary": "owned promotion",
  "evidence": [{"path": "trace-owned.json", "summary": "owned trace"}],
  "decision_reason": "candidate ownership check"
}
"#,
    )
    .expect("write learning record");
}

fn write_regression_record(path: &std::path::Path) {
    fs::write(
        path,
        r#"{
  "id": "learn-owned",
  "disposition": "RegressionCase",
  "summary": "owned regression",
  "evidence": [{"path": "trace-owned.json", "summary": "owned trace"}],
  "decision_reason": "candidate ownership check"
}
"#,
    )
    .expect("write learning record");
}

fn write_allowing_decision(root: &std::path::Path) {
    let dir = root.join(".agent-harness/learning/decisions");
    fs::create_dir_all(&dir).expect("create decisions");
    fs::write(
        dir.join("adapt-learn-owned.json"),
        r#"{
  "run_id": "learn-owned",
  "decision_id": "adapt-learn-owned",
  "product_result": "Pass",
  "process_result": "Pass",
  "learning_result": "Pass",
  "adaptation_result": "Pass",
  "allowed_adaptations": ["skill_promotion", "regression_registration"],
  "blocked_adaptations": [],
  "reason": "eligible",
  "required_next_actions": [],
  "created_artifacts": []
}
"#,
    )
    .expect("write decision");
}

#[test]
fn learn_promote_rejects_skill_proposal_without_record_evidence() {
    // Given: a valid shape proposal whose evidence does not belong to the loaded record.
    let bin = env!("CARGO_BIN_EXE_swe-seed");
    let root = temp_root("proposal-ownership");
    let record_path = root.join("learn-owned.json");
    write_skill_record(&record_path);
    write_allowing_decision(&root);
    let proposal_path = root.join("proposal.json");
    fs::write(
        &proposal_path,
        r#"{
  "id": "proposal-mismatch",
  "proposed_skill_id": "mismatch",
  "observed_problem": "shape-only validation",
  "evidence": [{"path": "other-trace.json", "summary": "other trace"}],
  "proposed_behavior": ["must belong to record"],
  "evals": [{
    "id": "eval-owned",
    "eval_class": "AdaptationEligibility",
    "check_type": "manual",
    "target": "",
    "required": true,
    "rule": "",
    "evidence_required": "ownership check"
  }],
  "rollout_plan": "write candidate",
  "rollback_plan": "remove candidate"
}
"#,
    )
    .expect("write proposal");

    // When: promotion runs.
    let out = Command::new(bin)
        .env("SWE_SEED_ROOT", &root)
        .args([
            "learn",
            "promote",
            record_path.to_str().expect("record path"),
            "--proposal",
            proposal_path.to_str().expect("proposal path"),
        ])
        .output()
        .expect("run learn promote");

    // Then: the artifact is not persisted.
    assert!(!out.status.success(), "stdout: {:?}", out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not backed by LearningRecord"),
        "stderr: {stderr}"
    );
    assert!(!root
        .join(".agent-harness/learning/proposals/proposal-mismatch.json")
        .exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn learn_promote_rejects_regression_for_different_source_record() {
    // Given: a regression case points at a different source record than the loaded record.
    let bin = env!("CARGO_BIN_EXE_swe-seed");
    let root = temp_root("regression-ownership");
    let record_path = root.join("learn-owned.json");
    write_regression_record(&record_path);
    write_allowing_decision(&root);
    let regression_path = root.join("regression.json");
    fs::write(
        &regression_path,
        r#"{
  "id": "regression-mismatch",
  "source_run_id": "other-record",
  "failure_mode": "bad recurrence",
  "detection": "target.txt",
  "future_rule": "bad recurrence",
  "status": "candidate",
  "linked_eval_check": "regression-check"
}
"#,
    )
    .expect("write regression");

    // When: promotion runs.
    let out = Command::new(bin)
        .env("SWE_SEED_ROOT", &root)
        .args([
            "learn",
            "promote",
            record_path.to_str().expect("record path"),
            "--regression",
            regression_path.to_str().expect("regression path"),
        ])
        .output()
        .expect("run learn promote");

    // Then: the regression is rejected before persistence.
    assert!(!out.status.success(), "stdout: {:?}", out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("source_run_id"), "stderr: {stderr}");
    assert!(!root
        .join(".agent-harness/learning/regressions/regression-mismatch.json")
        .exists());
    let _ = fs::remove_dir_all(root);
}
