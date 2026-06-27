//! `trace finish` without `--claim` must error (clap rejects the missing
//! required flag before any execution). Spec 0014 / Phase 2 falsification.

use std::process::Command;

#[test]
fn trace_finish_without_claim_errors() {
    let bin = env!("CARGO_BIN_EXE_swe-seed");
    let out = Command::new(bin)
        .args(["trace", "finish", "some-trace-id"])
        .output()
        .expect("run swe-seed");
    assert!(
        !out.status.success(),
        "trace finish without --claim must exit non-zero"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("--claim") || err.contains("required") || err.contains("Usage"),
        "expected a missing-claim usage error, got stderr: {err}"
    );
}
