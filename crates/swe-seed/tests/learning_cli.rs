use std::fs;
use std::process::Command;

fn temp_root(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-learning-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("create temp root");
    dir
}

#[test]
fn learn_promote_blocks_skill_proposal_when_adaptation_decision_blocks_promotion() {
    // Given: a SkillProposal learning record, a valid proposal candidate, and a
    // matching AdaptationDecision that blocks skill_promotion.
    let bin = env!("CARGO_BIN_EXE_swe-seed");
    let root = temp_root("blocked-promotion");
    let records = root.join("records");
    fs::create_dir_all(&records).expect("create records");
    let record_path = records.join("learn-blocked.json");
    fs::write(
        &record_path,
        r#"{
  "id": "learn-blocked",
  "disposition": "SkillProposal",
  "summary": "block promotion without proof",
  "evidence": [{"path": "trace.json", "summary": "finished trace"}],
  "decision_reason": "candidate requires adaptation eligibility"
}
"#,
    )
    .expect("write learning record");

    let proposal_path = root.join("proposal.json");
    fs::write(
        &proposal_path,
        r#"{
  "id": "proposal-blocked",
  "proposed_skill_id": "blocked-skill",
  "observed_problem": "promotion was not gated",
  "evidence": [{"path": "trace.json", "summary": "finished trace"}],
  "proposed_behavior": ["gate promotion on AdaptationDecision"],
  "evals": [{
    "id": "eval-blocked",
    "eval_class": "AdaptationEligibility",
    "check_type": "manual",
    "target": "",
    "required": true,
    "rule": "",
    "evidence_required": "decision blocks promotion"
  }],
  "rollout_plan": "write through review queue",
  "rollback_plan": "delete the proposal artifact"
}
"#,
    )
    .expect("write proposal");

    let decisions = root.join(".agent-harness/learning/decisions");
    fs::create_dir_all(&decisions).expect("create decisions");
    fs::write(
        decisions.join("adapt-learn-blocked.json"),
        r#"{
  "run_id": "learn-blocked",
  "decision_id": "adapt-learn-blocked",
  "product_result": "Pass",
  "process_result": "Pass",
  "learning_result": "Pass",
  "adaptation_result": "Fail",
  "allowed_adaptations": [],
  "blocked_adaptations": ["skill_promotion"],
  "reason": "adaptation eligibility failed",
  "required_next_actions": ["rerun eligibility eval"],
  "created_artifacts": []
}
"#,
    )
    .expect("write decision");

    // When: the CLI attempts to promote the proposal.
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

    // Then: promotion fails and no proposal artifact is written.
    assert!(!out.status.success(), "stdout: {:?}", out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("promotion blocked") || stderr.contains("AdaptationDecision"),
        "stderr: {stderr}"
    );
    assert!(
        !root
            .join(".agent-harness/learning/proposals/proposal-blocked.json")
            .exists(),
        "blocked promotion must not write the proposal artifact"
    );

    let _ = fs::remove_dir_all(root);
}
