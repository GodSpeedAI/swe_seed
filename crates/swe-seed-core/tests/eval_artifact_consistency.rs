//! artifact_consistency compares the artifact's bytes with the digest the proof
//! recorded. Before this, it was an existence check: a proven artifact replaced
//! after the proof still passed (godspeed-readiness S03, 2026-10-21).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use swe_seed_core::eval::{evaluate_check, EvalCheck, EvalClass, EvalStatus};

struct TempRoot(PathBuf);
impl TempRoot {
    fn new() -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "swe-seed-artifact-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&p).unwrap();
        TempRoot(p)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn root_with(content: &str) -> TempRoot {
    let dir = TempRoot::new();
    fs::write(dir.path().join("artifact.txt"), content).unwrap();
    dir
}

fn digest_of(content: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(content.as_bytes()))
}

fn check(rule: &str) -> EvalCheck {
    EvalCheck {
        id: "artifact".into(),
        eval_class: EvalClass::ProductOutcome,
        check_type: "artifact_consistency".into(),
        target: "artifact.txt".into(),
        required: true,
        rule: rule.into(),
        evidence_required: String::new(),
    }
}

const V1: &str = "release artifact v1\n";

#[test]
fn intact_artifact_matching_the_recorded_digest_passes() {
    let root = root_with(V1);
    let (status, evidence, reason) = evaluate_check(root.path(), &check(&digest_of(V1)), false);
    assert_eq!(status, EvalStatus::Pass, "{reason:?}");
    assert!(evidence.contains("sha256:"), "evidence names the observed digest: {evidence}");
}

#[test]
fn artifact_changed_after_the_proof_fails() {
    let root = root_with("tampered\n");
    let (status, _, reason) = evaluate_check(root.path(), &check(&digest_of(V1)), false);
    assert_eq!(status, EvalStatus::Fail);
    assert!(reason.unwrap().contains("changed since it was proven"));
}

#[test]
fn recorded_digest_hex_case_does_not_matter() {
    let root = root_with(V1);
    let hex = digest_of(V1).trim_start_matches("sha256:").to_ascii_uppercase();
    let (status, _, reason) = evaluate_check(root.path(), &check(&format!("sha256:{hex}")), false);
    assert_eq!(status, EvalStatus::Pass, "{reason:?}");
}

#[test]
fn no_recorded_digest_fails_closed() {
    let root = root_with(V1);
    for rule in ["", "anything", "sha256:", "sha256:xyz", "md5:0123"] {
        let (status, _, reason) = evaluate_check(root.path(), &check(rule), false);
        assert_eq!(status, EvalStatus::Fail, "rule {rule:?} must not pass");
        assert!(reason.unwrap().contains("needs rule"), "rule {rule:?}");
    }
}

#[test]
fn missing_artifact_fails() {
    let dir = TempRoot::new();
    let (status, _, _) = evaluate_check(dir.path(), &check(&digest_of(V1)), false);
    assert_eq!(status, EvalStatus::Fail);
}

#[test]
fn file_exists_is_unchanged() {
    let root = root_with("anything\n");
    let mut c = check("");
    c.check_type = "file_exists".into();
    assert_eq!(evaluate_check(root.path(), &c, false).0, EvalStatus::Pass);
}
