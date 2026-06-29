use std::fs;
use std::process::Command;

fn temp_root(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-cli-{label}-{}-{}",
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
fn doctor_summary_reports_combined_host_failure() {
    // Given: a temp root with a synced Codex host projection.
    let bin = env!("CARGO_BIN_EXE_swe-seed");
    let root = temp_root("doctor-summary");
    let sync = Command::new(bin)
        .env("SWE_SEED_ROOT", &root)
        .args(["sync", "--host", "codex"])
        .output()
        .expect("sync codex");
    assert!(sync.status.success(), "sync stderr: {:?}", sync.stderr);
    fs::write(root.join(".codex/hooks.json"), "{}\nmanual edit\n").expect("edit managed file");

    // When: doctor checks the drifted host.
    let doctor = Command::new(bin)
        .env("SWE_SEED_ROOT", &root)
        .args(["doctor", "--host", "codex"])
        .output()
        .expect("doctor codex");

    // Then: the printed overall line matches the non-zero exit status.
    assert!(!doctor.status.success());
    let stdout = String::from_utf8_lossy(&doctor.stdout);
    assert!(stdout.contains("Drifted\thost:codex"), "stdout: {stdout}");
    assert!(stdout.contains("Fail\toverall"), "stdout: {stdout}");
    let _ = fs::remove_dir_all(root);
}
