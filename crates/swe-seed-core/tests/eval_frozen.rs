//! eval_frozen: deterministic eval across the 4 classes, frozen-after-handoff
//! enforcement, and live-pass-only promotion (a waived-only eval never passes).

use std::fs;
use std::path::PathBuf;

use swe_seed_core::eval::{load_eval_spec, run_eval, EvalClass, EvalStatus};

fn real_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn temp_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-eval-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

const FROZEN_SPEC: &str = r#"
id = "frozen-demo"
version = "0.1.0"
run_id = "t"
target_type = "file"
target_path = "."
purpose = "frozen demo"
eval_classes = ["ProcessCompliance"]
frozen_after_handoff = true
pass_condition = "passes"
outputs = []

[[checks]]
id = "target-exists"
eval_class = "ProcessCompliance"
check_type = "file_exists"
target = "target.txt"
required = true
rule = ""
evidence_required = "target present"
"#;

#[test]
fn frozen_spec_records_then_rejects_edits() {
    let root = temp_root();
    fs::write(root.join("target.txt"), "ok").unwrap();
    let spec_path = root.join("spec.toml");
    fs::write(&spec_path, FROZEN_SPEC).unwrap();

    // First run on a frozen spec = handoff: records the baseline and passes.
    let spec = load_eval_spec(&spec_path).unwrap();
    let result = run_eval(&root, &spec_path, &spec, false).unwrap();
    assert_eq!(result.status, EvalStatus::Pass);

    // Edit the (now handed-off) spec → hash mismatch → eval refuses to run.
    fs::write(
        &spec_path,
        FROZEN_SPEC.replacen("frozen demo", "frozen demo edited", 1),
    )
    .unwrap();
    let spec = load_eval_spec(&spec_path).unwrap();
    let err = run_eval(&root, &spec_path, &spec, false).unwrap_err();
    assert!(
        err.to_string().contains("modified after handoff"),
        "expected frozen-mismatch error, got: {err}"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn non_frozen_spec_allows_edits() {
    let root = temp_root();
    fs::write(root.join("target.txt"), "ok").unwrap();
    let spec_path = root.join("spec.toml");
    let mut spec = FROZEN_SPEC.replace(
        "frozen_after_handoff = true",
        "frozen_after_handoff = false",
    );
    fs::write(&spec_path, &spec).unwrap();
    let loaded = load_eval_spec(&spec_path).unwrap();
    assert_eq!(
        run_eval(&root, &spec_path, &loaded, false).unwrap().status,
        EvalStatus::Pass
    );
    // Edit freely — no frozen gate.
    spec = spec.replacen("frozen demo", "edited", 1);
    fs::write(&spec_path, &spec).unwrap();
    let loaded = load_eval_spec(&spec_path).unwrap();
    assert_eq!(
        run_eval(&root, &spec_path, &loaded, false).unwrap().status,
        EvalStatus::Pass
    );
    let _ = fs::remove_dir_all(&root);
}

const FOUR_CLASS_SPEC: &str = r#"
id = "four-class"
version = "0.1.0"
run_id = "t"
target_type = "file"
target_path = "."
purpose = "all four classes"
eval_classes = ["ProcessCompliance","ProductOutcome","LearningQuality","AdaptationEligibility"]
frozen_after_handoff = false
pass_condition = "passes"
outputs = []

[[checks]]
id = "c1"
eval_class = "ProcessCompliance"
check_type = "file_exists"
target = "target.txt"
required = true
rule = ""
evidence_required = "present"

[[checks]]
id = "c2"
eval_class = "ProductOutcome"
check_type = "file_exists"
target = "target.txt"
required = true
rule = ""
evidence_required = "present"

[[checks]]
id = "c3"
eval_class = "LearningQuality"
check_type = "file_exists"
target = "target.txt"
required = true
rule = ""
evidence_required = "present"

[[checks]]
id = "c4"
eval_class = "AdaptationEligibility"
check_type = "file_exists"
target = "target.txt"
required = true
rule = ""
evidence_required = "present"
"#;

#[test]
fn deterministic_run_across_four_eval_classes() {
    let root = temp_root();
    fs::write(root.join("target.txt"), "ok").unwrap();
    let spec_path = root.join("spec.toml");
    fs::write(&spec_path, FOUR_CLASS_SPEC).unwrap();
    let spec = load_eval_spec(&spec_path).unwrap();

    let r1 = run_eval(&root, &spec_path, &spec, false).unwrap();
    let r2 = run_eval(&root, &spec_path, &spec, false).unwrap();
    assert_eq!(r1.status, EvalStatus::Pass);
    // Deterministic: same inputs → same per-check statuses (timestamps differ).
    let statuses: Vec<_> = r1.checks.iter().map(|c| c.status).collect();
    assert_eq!(statuses, vec![EvalStatus::Pass; 4]);
    let statuses2: Vec<_> = r2.checks.iter().map(|c| c.status).collect();
    assert_eq!(statuses, statuses2);
    // Also compare each check's stable identity (id + class) so a reorder or
    // class/identity mix-up is caught, not just the flattened statuses.
    let identity: Vec<(&str, EvalClass)> = r1
        .checks
        .iter()
        .map(|c| (c.id.as_str(), c.eval_class))
        .collect();
    let identity2: Vec<(&str, EvalClass)> = r2
        .checks
        .iter()
        .map(|c| (c.id.as_str(), c.eval_class))
        .collect();
    assert_eq!(
        identity, identity2,
        "check identity must be stable across runs"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_required_failing_check_fails_the_eval() {
    let root = temp_root();
    // target.txt absent → the required file_exists check fails.
    let spec_path = root.join("spec.toml");
    fs::write(&spec_path, FOUR_CLASS_SPEC).unwrap();
    let spec = load_eval_spec(&spec_path).unwrap();
    let result = run_eval(&root, &spec_path, &spec, false).unwrap();
    assert_eq!(result.status, EvalStatus::Fail);
    let _ = fs::remove_dir_all(&root);
}

const OPTIONAL_ONLY_SPEC: &str = r#"
id = "optional-only"
version = "0.1.0"
run_id = "t"
target_type = "file"
target_path = "."
purpose = "waived-only must not pass"
eval_classes = ["ProcessCompliance"]
frozen_after_handoff = false
pass_condition = "live pass required"
outputs = []

[[checks]]
id = "opt-missing"
eval_class = "ProcessCompliance"
check_type = "file_exists"
target = "absent.txt"
required = false
rule = ""
evidence_required = "optional"
"#;

#[test]
fn live_pass_only_waived_eval_does_not_activate() {
    // A spec whose only check is optional and fails → Waived, and the eval must
    // NOT pass (no live pass). A waiver never activates a capability.
    let root = temp_root();
    let spec_path = root.join("spec.toml");
    fs::write(&spec_path, OPTIONAL_ONLY_SPEC).unwrap();
    let spec = load_eval_spec(&spec_path).unwrap();
    let result = run_eval(&root, &spec_path, &spec, false).unwrap();
    assert_eq!(result.checks[0].status, EvalStatus::Waived);
    assert_ne!(
        result.status,
        EvalStatus::Pass,
        "a waived-only eval must not pass"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn core_fixture_loads_and_has_four_classes() {
    // The shipped core fixture is well-formed and covers all four distinct
    // eval classes (not just four checks — a duplicated class must fail this).
    let spec_path = real_root().join("tests/fixtures/eval.toml");
    let spec = load_eval_spec(&spec_path).unwrap();
    assert_eq!(spec.id, "core-conformance");
    assert!(!spec.frozen_after_handoff);
    let classes: Vec<String> = spec
        .checks
        .iter()
        .map(|c| format!("{:?}", c.eval_class))
        .collect();
    let unique: std::collections::HashSet<&str> = classes.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        unique.len(),
        4,
        "fixture must cover 4 distinct classes, got {classes:?}"
    );
}

const COMMAND_CHECK_SPEC: &str = r#"
id = "cmd-demo"
version = "0.1.0"
run_id = "t"
target_type = "file"
target_path = "."
purpose = "command_check trusted gating"
eval_classes = ["ProcessCompliance"]
frozen_after_handoff = false
pass_condition = "passes"
outputs = []

[[checks]]
id = "cmd"
eval_class = "ProcessCompliance"
check_type = "command_check"
target = "."
required = true
rule = "true"
evidence_required = ""
"#;

#[test]
fn command_check_is_rejected_unless_trusted() {
    // Security: spec-controlled shell execution is opt-in. In the default
    // (untrusted) path the command_check is rejected (Fail), so the eval fails.
    // Only an explicit --trusted opt-in runs it.
    let root = temp_root();
    let spec_path = root.join("spec.toml");
    fs::write(&spec_path, COMMAND_CHECK_SPEC).unwrap();
    let spec = load_eval_spec(&spec_path).unwrap();

    let untrusted = run_eval(&root, &spec_path, &spec, false).unwrap();
    assert_ne!(
        untrusted.status,
        EvalStatus::Pass,
        "untrusted command_check must not pass"
    );
    assert!(untrusted.checks[0]
        .failure_reason
        .as_deref()
        .unwrap_or("")
        .contains("trusted mode"));

    let trusted = run_eval(&root, &spec_path, &spec, true).unwrap();
    assert_eq!(
        trusted.status,
        EvalStatus::Pass,
        "trusted command_check (`true`) should pass"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn manual_check_does_not_auto_pass_without_runtime_proof() {
    // A manual check has no attached runtime proof in a deterministic run, so
    // it must be Inconclusive — never auto-pass on static evidence_required.
    let spec = r#"
id = "manual-demo"
version = "0.1.0"
run_id = "t"
target_type = "file"
target_path = "."
purpose = "manual"
eval_classes = ["ProcessCompliance"]
frozen_after_handoff = false
pass_condition = "passes"
outputs = []

[[checks]]
id = "m"
eval_class = "ProcessCompliance"
check_type = "manual_check"
target = "."
required = true
rule = ""
evidence_required = "claims to have evidence"
"#;
    let root = temp_root();
    let spec_path = root.join("spec.toml");
    fs::write(&spec_path, spec).unwrap();
    let loaded = load_eval_spec(&spec_path).unwrap();
    let result = run_eval(&root, &spec_path, &loaded, false).unwrap();
    assert_eq!(result.checks[0].status, EvalStatus::Inconclusive);
    assert_ne!(result.status, EvalStatus::Pass);
    let _ = fs::remove_dir_all(&root);
}
