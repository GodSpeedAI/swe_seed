//! regression_link: a promoted RegressionCase links an `EvalCheck` that fails
//! on recurrence (spec 0016 §4, §6). A regression without a linked check is
//! rejected; the linked check re-fires when the failure mode reappears.

use std::fs;
use std::path::PathBuf;

use swe_seed_core::eval::{evaluate_check, EvalStatus};
use swe_seed_core::learning::{eval_check_for_regression, validate_regression, RegressionCase};

fn case(id: &str, failure_mode: &str, linked: &str) -> RegressionCase {
    RegressionCase {
        id: id.into(),
        source_run_id: "run-x".into(),
        failure_mode: failure_mode.into(),
        detection: "target.txt".into(),
        future_rule: "must not reintroduce the failure mode".into(),
        status: "active".into(),
        linked_eval_check: linked.into(),
    }
}

/// Write a temp target file and return its path (repo-rooted `detection` value
/// points at "target.txt"; the evaluator resolves it under `root`).
fn write_target(root: &PathBuf, body: &str) {
    fs::write(root.join("target.txt"), body).unwrap();
}

#[test]
fn regression_without_linked_check_is_rejected() {
    // Outcome 4 prerequisite: a regression must link an eval check.
    let mut c = case("reg-1", "TODO left in code", "chk-1");
    assert!(validate_regression(&c).is_ok());

    c.linked_eval_check = "  ".into();
    let err = validate_regression(&c).unwrap_err();
    assert!(err.contains("linked_eval_check"), "{err}");

    // Empty failure mode / detection / future rule also rejected.
    let mut bad = case("reg-2", "TODO left in code", "chk-2");
    bad.failure_mode = "".into();
    assert!(validate_regression(&bad).is_err());
}

#[test]
fn require_future_rule_without_content_is_rejected() {
    // A `require:` future rule with no pattern would build an EvalCheck with an
    // empty rule — reject it at validation time (aligned with eval_check_for_regression).
    let mut c = case("reg-5", "missing guard", "chk-reg-5");
    c.future_rule = "require:   ".into();
    let err = validate_regression(&c).unwrap_err();
    assert!(
        err.contains("require:") && err.contains("no rule content"),
        "{err}"
    );

    // `require:` with content is fine, and still builds a required-pattern check.
    c.future_rule = "require: fn guarded()".into();
    assert!(validate_regression(&c).is_ok());

    // A non-require future rule is not subject to the prefix check.
    c.future_rule = "must not reintroduce the failure mode".into();
    assert!(validate_regression(&c).is_ok());
}

#[test]
fn linked_check_fails_when_failure_mode_recurs() {
    // The regression's linked EvalCheck must FAIL when the captured failure
    // mode reappears in the detection target, and PASS when it's gone.
    let root = std::env::temp_dir().join(format!(
        "swe-seed-reg-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();

    let c = case("reg-3", "TODO: refactor this", "chk-reg-3");
    let check = eval_check_for_regression(&c).expect("valid case builds a check");
    assert_eq!(
        check.id, "chk-reg-3",
        "the linked eval check id is the case's linked_eval_check"
    );
    assert_eq!(check.check_type, "static_forbidden_patterns");
    assert!(check.required, "a regression check is required");

    // Recurrence: the failure mode is present → Fail (the lesson enforces itself).
    write_target(&root, "fn main() { /* TODO: refactor this */ }");
    let (status, _evidence, reason) = evaluate_check(&root, &check, false);
    assert_eq!(status, EvalStatus::Fail, "recurrence must be caught");
    assert!(reason.unwrap().contains("TODO: refactor this"));

    // Clean target: failure mode gone → Pass.
    write_target(&root, "fn main() {}");
    let (status, _evidence, _reason) = evaluate_check(&root, &check, false);
    assert_eq!(status, EvalStatus::Pass, "no recurrence → no failure");

    fs::remove_dir_all(&root).ok();
}

#[test]
fn future_rule_require_prefix_builds_a_required_pattern_check() {
    // A `require:` future rule flips the linked check to a required-pattern
    // check (the affirmative form of the same enforcement).
    let root = std::env::temp_dir().join(format!(
        "swe-seed-reg2-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();

    let mut c = case("reg-4", "missing guard", "chk-reg-4");
    c.future_rule = "require: fn guarded()".into();
    let check = eval_check_for_regression(&c).unwrap();
    assert_eq!(check.check_type, "static_required_patterns");
    assert_eq!(check.rule, "fn guarded()");

    // Missing the required guard → Fail; present → Pass.
    write_target(&root, "fn main() {}");
    let (status, _, _) = evaluate_check(&root, &check, false);
    assert_eq!(status, EvalStatus::Fail);

    write_target(&root, "fn guarded() {}");
    let (status, _, _) = evaluate_check(&root, &check, false);
    assert_eq!(status, EvalStatus::Pass);

    fs::remove_dir_all(&root).ok();
}
