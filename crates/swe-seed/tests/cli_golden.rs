//! cli_golden: Phase 10 golden parity for the Rust CLIs BEYOND `route`
//! (which has its own `route_golden.rs`). Captures `doctor --json`,
//! `context-plan`, and the `seed assemble` manifest, normalizes volatile bits,
//! and diffs against committed fixtures. Run via `just parity` to gate on the
//! RELEASE binary (`SWE_SEED_BIN=target/release/swe-seed`); defaults to the
//! debug test binary.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn bin() -> PathBuf {
    // Release gate: `just parity` sets SWE_SEED_BIN=target/release/swe-seed.
    std::env::var("SWE_SEED_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_swe-seed").into())
}

fn run(args: &[&str]) -> String {
    let out = Command::new(bin())
        .args(args)
        .current_dir(root())
        .output()
        .unwrap_or_else(|e| panic!("failed to run swe-seed {:?}: {e}", args));
    assert!(
        out.status.success(),
        "swe-seed {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// Normalize volatile bits: absolute repo root → `<ROOT>`, ISO8601 timestamps →
/// `<TS>`, timestamp-prefixed run/trace ids → `<RUN_ID>`. The captured CLIs are
/// deterministic today; this is defensive against future drift.
fn normalize(s: &str) -> String {
    let mut out = s.replace(&root().display().to_string(), "<ROOT>");
    let ts = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+\d{2}:\d{2}").unwrap();
    out = ts.replace_all(&out, "<TS>").into_owned();
    let run_id = Regex::new(r"\d{8}T\d{6}Z-[a-z0-9-]+").unwrap();
    out = run_id.replace_all(&out, "<RUN_ID>").into_owned();
    if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&out) {
        if let Some(obj) = value
            .get_mut("context_budget")
            .and_then(serde_json::Value::as_object_mut)
        {
            obj.insert(
                "stale_context_warnings".into(),
                serde_json::json!(["<STALE_CONTEXT_WARNINGS>"]),
            );
        }
        if let Ok(json) = serde_json::to_string_pretty(&value) {
            return json;
        }
    }
    out
}

fn fixture(name: &str) -> PathBuf {
    root().join("tests/fixtures/golden").join(name)
}

fn assert_golden(name: &str, actual: &str) {
    let path = fixture(name);
    let expected = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "golden fixture missing: {}: {e}. \
             Run `cargo test -p swe-seed --test cli_golden regen -- --ignored` to create it.",
            path.display()
        )
    });
    assert_eq!(
        normalize(actual).trim_end(),
        expected.trim_end(),
        "golden drift for {name} — regenerate the fixture if the CLI output changed intentionally"
    );
}

#[test]
fn doctor_json_matches_golden() {
    assert_golden("doctor.json", &run(&["doctor", "--json"]));
}

#[test]
fn context_plan_matches_golden() {
    assert_golden(
        "context-plan.json",
        &run(&["context-plan", "checkpoint smoke"]),
    );
}

#[test]
fn seed_assemble_manifest_matches_golden() {
    run(&["seed", "assemble"]);
    let manifest = fs::read_to_string(root().join(".swe-seed/seed-package-manifest.json")).unwrap();
    assert_golden("seed-manifest.json", &manifest);
}

/// Regenerate all golden fixtures. Ignored by default; run on purpose:
/// `cargo test -p swe-seed --test cli_golden regen -- --ignored`.
#[test]
#[ignore = "fixture regenerator — run with --ignored after an intentional CLI output change"]
fn regen() {
    let dir = fixture("");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        fixture("doctor.json"),
        &normalize(&run(&["doctor", "--json"])),
    )
    .unwrap();
    fs::write(
        fixture("context-plan.json"),
        &normalize(&run(&["context-plan", "checkpoint smoke"])),
    )
    .unwrap();
    run(&["seed", "assemble"]);
    let manifest = fs::read_to_string(root().join(".swe-seed/seed-package-manifest.json")).unwrap();
    fs::write(fixture("seed-manifest.json"), &normalize(&manifest)).unwrap();
    println!("wrote golden fixtures to {}", dir.display());
}

#[test]
fn release_binary_smoke() {
    // A behavior parity smoke that runs against whichever binary SWE_SEED_BIN
    // points at (release under `just parity`, debug otherwise): the harness
    // validate command must pass and a route must resolve.
    let validate = Command::new(bin())
        .arg("harness")
        .current_dir(root())
        .output()
        .unwrap();
    assert!(
        validate.status.success(),
        "swe-seed harness failed ({}): {}",
        bin().display(),
        String::from_utf8_lossy(&validate.stderr)
    );
    let route = Command::new(bin())
        .args(["route", "checkpoint smoke"])
        .current_dir(root())
        .output()
        .unwrap();
    assert!(route.status.success(), "swe-seed route failed");
    assert!(String::from_utf8_lossy(&route.stdout).contains("\"job_type\""));
}

// Silence unused import when only a subset of helpers is used in a given build.
#[allow(dead_code)]
fn _path_ref(_p: &Path) {}
