//! eval_declared_proof: a spec's command_check exported as a proof for an independent verifier
//! (readiness P05). Pinned by digest, parsed by SWE_SEED, read-only, and the same verdict as `run_eval`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};
use swe_seed_core::eval::{
    declared_proof, load_eval_spec, run_eval, DeclaredProofError, EvalStatus,
};

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-proof-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

const SPEC: &str = r#"
id = "proof-demo"
version = "1.2.0"
run_id = "t"
target_type = "file"
target_path = "."
purpose = "proof export"
eval_classes = ["ProcessCompliance"]
pass_condition = "passes"
outputs = []

[[checks]]
id = "tests-pass"
eval_class = "ProcessCompliance"
check_type = "command_check"
rule = "  test \"$(cat answer.txt)\" = 42  "

[[checks]]
id = "readme-exists"
eval_class = "ProcessCompliance"
check_type = "file_exists"
target = "README.md"

[[checks]]
id = "blank"
eval_class = "ProcessCompliance"
check_type = "command_check"
rule = "   "
"#;

fn write_spec(root: &Path) -> (PathBuf, String) {
    let path = root.join("eval.toml");
    fs::write(&path, SPEC).unwrap();
    (path, hex(SPEC.as_bytes()))
}

#[test]
fn exports_the_command_check_as_swe_seed_runs_it() {
    let root = temp_root("ok");
    let (path, digest) = write_spec(&root);
    let proof = declared_proof(&path, &digest, "tests-pass").unwrap();
    assert_eq!(proof.schema, "swe_seed.eval_proof.v1");
    assert_eq!(proof.spec_id, "proof-demo");
    assert_eq!(proof.spec_version, "1.2.0");
    assert_eq!(proof.spec_sha256, format!("sha256:{digest}"));
    assert_eq!(proof.check_id, "tests-pass");
    assert!(proof.required);
    // command_check trims the rule and runs it with sh -c in the root.
    assert_eq!(
        proof.argv,
        vec!["sh", "-c", "test \"$(cat answer.txt)\" = 42"]
    );
    assert_eq!(proof.cwd, ".");
    // Read-only: no frozen sidecar, nothing else written.
    let entries: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(entries, vec![std::ffi::OsString::from("eval.toml")]);
}

#[test]
fn the_pin_is_checked_against_the_bytes_and_accepts_prefixed_upper_case() {
    let root = temp_root("pin");
    let (path, digest) = write_spec(&root);
    let upper = format!("sha256:{}", digest.to_uppercase());
    assert!(declared_proof(&path, &upper, "tests-pass").is_ok());

    let mut wrong = digest.clone().into_bytes();
    wrong[0] = if wrong[0] == b'0' { b'1' } else { b'0' };
    let wrong = String::from_utf8(wrong).unwrap();
    match declared_proof(&path, &wrong, "tests-pass") {
        Err(e @ DeclaredProofError::DigestMismatch { .. }) => {
            assert_eq!(e.code(), "spec_digest_mismatch");
            assert!(
                e.to_string().contains(&digest),
                "names the actual digest: {e}"
            );
        }
        other => panic!("expected a digest mismatch, got {other:?}"),
    }

    // A one-byte change to the spec is refused under the old pin.
    fs::write(&path, SPEC.replace("= 42", "= 43")).unwrap();
    assert_eq!(
        declared_proof(&path, &digest, "tests-pass")
            .unwrap_err()
            .code(),
        "spec_digest_mismatch"
    );

    for bad in [
        "",
        "abc",
        &digest[..63],
        &format!("{}g", &digest[..63]),
        &format!("md5:{digest}"),
    ] {
        assert_eq!(
            declared_proof(&path, bad, "tests-pass").unwrap_err().code(),
            "invalid_pin",
            "{bad:?}"
        );
    }
}

#[test]
fn refuses_checks_that_are_not_a_runnable_command() {
    let root = temp_root("checks");
    let (path, digest) = write_spec(&root);
    assert_eq!(
        declared_proof(&path, &digest, "readme-exists")
            .unwrap_err()
            .code(),
        "not_command_check"
    );
    assert_eq!(
        declared_proof(&path, &digest, "nope").unwrap_err().code(),
        "unknown_check"
    );
    assert_eq!(
        declared_proof(&path, &digest, "blank").unwrap_err().code(),
        "empty_rule"
    );
    assert_eq!(
        declared_proof(&root.join("missing.toml"), &digest, "tests-pass")
            .unwrap_err()
            .code(),
        "spec_unreadable"
    );
}

#[test]
fn parses_with_swe_seeds_own_rules() {
    let root = temp_root("json");
    let json = serde_json::json!({
        "id": "j", "version": "1", "run_id": "t", "target_type": "file", "target_path": ".",
        "purpose": "p", "eval_classes": ["ProcessCompliance"], "pass_condition": "passes", "outputs": [],
        "checks": [{ "id": "c", "eval_class": "ProcessCompliance", "check_type": "command_check", "rule": "true" }]
    })
    .to_string();
    let path = root.join("eval.json");
    fs::write(&path, &json).unwrap();
    assert_eq!(
        declared_proof(&path, &hex(json.as_bytes()), "c")
            .unwrap()
            .argv[2],
        "true"
    );

    // The loader's structure rule (non-empty checks) applies.
    let empty = SPEC.split("[[checks]]").next().unwrap().to_string() + "checks = []\n";
    let path = root.join("empty.toml");
    fs::write(&path, &empty).unwrap();
    assert_eq!(
        declared_proof(&path, &hex(empty.as_bytes()), "tests-pass")
            .unwrap_err()
            .code(),
        "spec_invalid"
    );
}

/// The exported argv reaches the same verdict as SWE_SEED's own trusted run of that check.
#[test]
fn exported_proof_agrees_with_run_eval() {
    for (answer, expected) in [("42", EvalStatus::Pass), ("41", EvalStatus::Fail)] {
        let root = temp_root("agree");
        let spec_text = SPEC
            .split("[[checks]]")
            .take(2)
            .collect::<Vec<_>>()
            .join("[[checks]]");
        let path = root.join("eval.toml");
        fs::write(&path, &spec_text).unwrap();
        fs::write(root.join("answer.txt"), answer).unwrap();
        let proof = declared_proof(&path, &hex(spec_text.as_bytes()), "tests-pass").unwrap();

        let ran = Command::new(&proof.argv[0])
            .args(&proof.argv[1..])
            .current_dir(root.join(&proof.cwd))
            .env_clear()
            .status()
            .unwrap();
        let spec = load_eval_spec(&path).unwrap();
        let result = run_eval(&root, &path, &spec, true).unwrap();
        let theirs = result
            .checks
            .iter()
            .find(|c| c.id == "tests-pass")
            .unwrap()
            .status;
        assert_eq!(theirs, expected, "run_eval with answer {answer}");
        assert_eq!(
            ran.success(),
            expected == EvalStatus::Pass,
            "exported argv with answer {answer}"
        );
    }
}
