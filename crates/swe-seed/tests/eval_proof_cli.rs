//! `swe-seed eval proof`: JSON on stdout, exit 0 on success and exit 2 with a JSON error otherwise.

use std::fs;
use std::process::Command;

use swe_seed_core::provenance::content_hash;

const SPEC: &str = r#"
id = "cli-proof"
version = "1"
run_id = "t"
target_type = "file"
target_path = "."
purpose = "p"
eval_classes = ["ProcessCompliance"]
pass_condition = "passes"
outputs = []

[[checks]]
id = "c"
eval_class = "ProcessCompliance"
check_type = "command_check"
rule = "true"
"#;

#[test]
fn eval_proof_prints_the_proof_or_a_typed_error() {
    let root = std::env::temp_dir().join(format!("swe-seed-proof-cli-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("eval.toml"), SPEC).unwrap();
    let digest = content_hash(SPEC.as_bytes())
        .trim_start_matches("sha256:")
        .to_string();
    let run = |pin: &str| {
        Command::new(env!("CARGO_BIN_EXE_swe-seed"))
            .current_dir(&root)
            .args([
                "eval",
                "proof",
                "--spec",
                "eval.toml",
                "--check",
                "c",
                "--sha256",
                pin,
            ])
            .output()
            .unwrap()
    };

    let ok = run(&digest);
    assert_eq!(
        ok.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&ok.stderr)
    );
    let proof: serde_json::Value = serde_json::from_slice(&ok.stdout).unwrap();
    assert_eq!(proof["argv"], serde_json::json!(["sh", "-c", "true"]));
    assert_eq!(proof["spec_sha256"], format!("sha256:{digest}"));

    let bad = run(&"0".repeat(64));
    assert_eq!(bad.status.code(), Some(2));
    let error: serde_json::Value = serde_json::from_slice(&bad.stdout).unwrap();
    assert_eq!(error["error"], "spec_digest_mismatch");
    assert!(
        !root.join(".swe-seed").exists(),
        "eval proof writes nothing"
    );
    fs::remove_dir_all(&root).ok();
}
