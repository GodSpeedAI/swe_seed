//! scan_gate: SkillSpector never fakes a pass — with no binary it returns
//! Pending; blocking statuses don't activate; the gate maps statuses correctly
//! (spec 0007).

use std::fs;
use std::path::PathBuf;

use swe_seed_core::security::{
    exceptions_store_path, gate::{can_activate, scan_blocks_projection},
    scan_result::{ScanFinding, ScanResult, ScanStatus},
    skillspector::{parse_skillspector_output, run_skillspector_with, skillspector_available},
    Exceptions,
};

fn real_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

#[test]
fn skillspector_is_pending_when_not_installed() {
    // Deterministic: force the "not installed" path via the availability flag.
    // The external SkillSpector gate must never fabricate a Clean pass.
    let path = real_root().join(".agent-harness/skills/20-implementation/implement-with-proof.json");
    let scan = run_skillspector_with(&path, false);
    assert_eq!(scan.status, ScanStatus::Pending, "no faked pass: {scan:?}");
    assert!(scan.detail.contains("not installed"), "detail should explain: {}", scan.detail);
    // A Pending scan never blocks projection.
    assert!(!scan_blocks_projection(&scan, &Exceptions::new()));
    // (The host may actually have the binary; that path is exercised below by
    // parse_skillspector_output, not by relying on PATH.)
    let _ = skillspector_available();
}

#[test]
fn trusted_skillpector_output_maps_to_status() {
    // Zero-exit output honoring the documented contract maps to a real status,
    // so a genuine Clean can activate (never faked: only recognized output).
    let mut ok = std::process::Command::new("true").output().unwrap();
    ok.stdout = br#"{"status":"clean","findings":[]}"#.to_vec();
    assert_eq!(parse_skillspector_output("s", &ok).status, ScanStatus::Clean);

    ok.stdout = br#"{"status":"warning","findings":[{"id":"x","severity":"low","message":"m"}]}"#.to_vec();
    let warned = parse_skillspector_output("s", &ok);
    assert_eq!(warned.status, ScanStatus::Warning);
    assert_eq!(warned.findings.len(), 1);

    ok.stdout = br#"{"status":"critical","findings":[]}"#.to_vec();
    let critical = parse_skillspector_output("s", &ok);
    assert_eq!(critical.status, ScanStatus::Critical);
    assert!(scan_blocks_projection(&critical, &Exceptions::new()));

    // Unrecognized status value → Pending (no faked pass).
    ok.stdout = br#"{"status":"bogus"}"#.to_vec();
    assert_eq!(parse_skillspector_output("s", &ok).status, ScanStatus::Pending);

    // Non-JSON / empty stdout → Pending.
    ok.stdout = b"not json".to_vec();
    assert_eq!(parse_skillspector_output("s", &ok).status, ScanStatus::Pending);
}

#[test]
fn nonzero_skillpector_exit_is_blocking_error() {
    let mut bad = std::process::Command::new("false").output().unwrap();
    bad.stderr = b"scanner exploded".to_vec();
    let scan = parse_skillspector_output("s", &bad);
    assert_eq!(scan.status, ScanStatus::Error);
    assert!(scan.status.is_blocking());
}

#[test]
fn exceptions_persist_and_lift_blocks() {
    // Approve persists a waived finding id; reloading it unblocks a critical scan.
    let store = std::env::temp_dir().join(format!(
        "swe-seed-exc-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = exceptions_store_path(&store); // exercise the path helper import

    let mut ex = Exceptions::new();
    ex.waive("CVE-evil");
    ex.save(&store).unwrap();
    let reloaded = Exceptions::load(&store);
    assert!(reloaded.waives("CVE-evil"));

    let critical = ScanResult {
        skill_id: "x".into(),
        status: ScanStatus::Critical,
        findings: vec![ScanFinding {
            id: "CVE-evil".into(),
            severity: "high".into(),
            message: "m".into(),
        }],
        detail: "".into(),
    };
    assert!(scan_blocks_projection(&critical, &Exceptions::new()));
    assert!(!scan_blocks_projection(&critical, &reloaded), "waived finding must not block");

    let _ = std::fs::remove_file(&store);
}

#[test]
fn blocking_statuses_are_isolated_from_terminal_passes() {
    // Status semantics: Pending/Critical never activate; only a terminal
    // non-blocking status (Clean/Warning) can.
    fn mk(status: ScanStatus) -> ScanResult {
        ScanResult {
            skill_id: "x".into(),
            status,
            findings: Vec::new(),
            detail: "x".into(),
        }
    }
    assert!(mk(ScanStatus::Critical).status.is_blocking());
    assert!(mk(ScanStatus::Error).status.is_blocking());
    assert!(!mk(ScanStatus::Pending).status.is_terminal());
    assert!(mk(ScanStatus::Clean).status.is_terminal());
    assert!(mk(ScanStatus::Warning).status.is_terminal() && !mk(ScanStatus::Warning).status.is_blocking());
}

#[test]
fn activation_gate_is_honest_about_state() {
    use swe_seed_core::skill::SkillRecord;
    // Build a record with a source_hash and vary the scan status.
    let mut rec = SkillRecord {
        ir: serde_json::from_str::<swe_seed_core::skill::SkillIR>(
            r#"{"id":"x","version":1,"category":"c","jtbd":"j","description":"d","triggers":["t"],"procedure":["p"],"evidence_required":["e"],"forbidden_behaviors":["f"],"outputs":["o"],"success_criteria":["s"],"failure_modes":["fm"],"render_targets":["checklist"],"status":"active"}"#,
        )
        .unwrap(),
        source_hash: "sha256:abc".into(),
        scan: ScanResult {
            skill_id: "x".into(),
            status: ScanStatus::Pending,
            findings: Vec::new(),
            detail: "".into(),
        },
        path: PathBuf::from("x.json"),
    };

    // Pending + hash → no activation (not verified).
    assert!(!can_activate(&rec, &Exceptions::new()));

    // Critical + hash → no activation.
    rec.scan.status = ScanStatus::Critical;
    rec.scan.findings.push(ScanFinding { id: "f".into(), severity: "high".into(), message: "m".into() });
    assert!(!can_activate(&rec, &Exceptions::new()));

    // Clean + hash → activates. Waiving the (now-absent) finding is irrelevant.
    rec.scan.status = ScanStatus::Clean;
    rec.scan.findings.clear();
    assert!(can_activate(&rec, &Exceptions::new()));

    // _ = silence unused fs in case env removes
    let _ = fs::metadata(&rec.path).is_ok();
}
