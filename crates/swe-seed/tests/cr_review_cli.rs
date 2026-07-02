use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn temp_root(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-cr-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("create temp root");
    dir
}

fn write_hooks_config(root: &std::path::Path) {
    let dir = root.join(".agent-hooks");
    fs::create_dir_all(&dir).expect("create hooks config dir");
    fs::write(
        dir.join("config.yaml"),
        r#"version: 1
paths:
  logs: .agent-hooks/logs
  index_db: .agent-hooks/index/hooks.rusql
logging:
  compact_min_size_bytes: 131072
redaction:
  key_substrings: []
  value_patterns: []
"#,
    )
    .expect("write hooks config");
}

fn write_valid_skill(path: &std::path::Path, id: &str) {
    fs::write(
        path,
        format!(
            r#"{{
  "id": "{id}",
  "version": 1,
  "category": "test",
  "jtbd": "exercise scan lookup",
  "description": "test skill",
  "triggers": ["test"],
  "inputs": {{"required": []}},
  "procedure": ["run scan"],
  "evidence_required": ["scan output"],
  "forbidden_behaviors": ["stop early"],
  "outputs": ["scan"],
  "success_criteria": ["matched id"],
  "failure_modes": ["malformed peer"],
  "render_targets": ["checklist"],
  "status": "active"
}}"#
        ),
    )
    .expect("write valid skill");
}

#[test]
fn agent_hooks_capture_rejects_invalid_json_stdin() {
    // Given: hook capture receives malformed stdin.
    let bin = env!("CARGO_BIN_EXE_swe-seed");
    let root = temp_root("bad-hook-json");
    write_hooks_config(&root);

    // When: the capture command runs with unreadable JSON payload content.
    let mut child = Command::new(bin)
        .env("SWE_SEED_ROOT", &root)
        .args(["agent-hooks", "capture", "tool.post"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn capture");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"{not-json")
        .expect("write stdin");
    let out = child.wait_with_output().expect("capture output");

    // Then: the command fails instead of logging an empty attributes object.
    assert!(!out.status.success(), "stdout: {:?}", out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("parse hook payload"), "stderr: {stderr}");
    assert!(
        !root.join(".agent-hooks/logs/events.jsonl").exists(),
        "invalid payload must not create a hook event"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn provenance_show_rejects_path_traversal_ids() {
    // Given: a JSON file outside the provenance record directory but still under .swe-seed.
    let bin = env!("CARGO_BIN_EXE_swe-seed");
    let root = temp_root("show-traversal");
    fs::create_dir_all(root.join(".swe-seed/provenance")).expect("create provenance dir");
    fs::write(root.join(".swe-seed/secret.json"), r#"{"leaked":true}"#).expect("write secret");

    // When: provenance show is called with a parent-directory id.
    let out = Command::new(bin)
        .env("SWE_SEED_ROOT", &root)
        .args(["provenance", "show", "../secret"])
        .output()
        .expect("run provenance show");

    // Then: the unsafe id is rejected before file read.
    assert!(!out.status.success(), "stdout: {:?}", out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("invalid capability id"), "stderr: {stderr}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn provenance_verify_ignores_stale_persisted_manifest() {
    // Given: an empty persisted manifest that would make strict verification vacuously pass.
    let bin = env!("CARGO_BIN_EXE_swe-seed");
    let root = temp_root("stale-provenance-manifest");
    fs::create_dir_all(root.join(".swe-seed")).expect("create swe seed dir");
    fs::write(
        root.join(".swe-seed/seed-package-manifest.json"),
        r#"{
  "metadata": {
    "artifact_type": "SeedPackageManifest",
    "artifact_id": "stale",
    "source_spec": "stale",
    "generated_by": "test",
    "status": "Active",
    "version": "0",
    "requires_human_review": "Optional",
    "linked_artifacts": []
  },
  "project_seed_path": "stale",
  "capabilities": [],
  "generated_paths": [],
  "validation": []
}
"#,
    )
    .expect("write stale manifest");

    // When: provenance verification runs.
    let out = Command::new(bin)
        .env("SWE_SEED_ROOT", &root)
        .args(["provenance", "verify"])
        .output()
        .expect("run provenance verify");

    // Then: current default capabilities are checked, so missing records fail.
    assert!(!out.status.success(), "stdout: {:?}", out.stdout);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("FAIL provenance"), "stdout: {stdout}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn seed_validate_boundaries_ignores_stale_persisted_manifest() {
    // Given: a stale persisted manifest with no required capabilities.
    let bin = env!("CARGO_BIN_EXE_swe-seed");
    let root = temp_root("stale-boundary-manifest");
    fs::create_dir_all(root.join(".swe-seed")).expect("create swe seed dir");
    fs::write(
        root.join(".swe-seed/seed-package-manifest.json"),
        r#"{
  "metadata": {
    "artifact_type": "SeedPackageManifest",
    "artifact_id": "stale",
    "source_spec": "stale",
    "generated_by": "test",
    "status": "Active",
    "version": "0",
    "requires_human_review": "Optional",
    "linked_artifacts": []
  },
  "project_seed_path": "stale",
  "capabilities": [],
  "generated_paths": [],
  "validation": []
}
"#,
    )
    .expect("write stale manifest");

    // When: boundary validation runs.
    let out = Command::new(bin)
        .env("SWE_SEED_ROOT", &root)
        .args(["seed", "validate-boundaries"])
        .output()
        .expect("run validate-boundaries");

    // Then: validation uses the fresh default manifest and passes.
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(r#""passed": true"#), "stdout: {stdout}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn skill_scan_skips_malformed_unrelated_skill_files() {
    // Given: a malformed skill file sorts before a valid requested skill.
    let bin = env!("CARGO_BIN_EXE_swe-seed");
    let root = temp_root("skill-scan-skip");
    let skills = root.join(".agent-harness/skills/test");
    fs::create_dir_all(&skills).expect("create skills");
    fs::write(skills.join("00-bad.json"), b"{bad").expect("write bad skill");
    write_valid_skill(&skills.join("99-good.json"), "wanted-skill");

    // When: scan targets the valid skill id.
    let out = Command::new(bin)
        .env("SWE_SEED_ROOT", &root)
        .args(["skill", "scan", "wanted-skill"])
        .output()
        .expect("run skill scan");

    // Then: the unrelated malformed skill does not stop lookup.
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Pending") || stdout.contains("Clean"),
        "stdout: {stdout}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn federation_status_resolves_domain_hash_from_swe_seed_root() {
    // Given: SWE_SEED_ROOT contains a SEA manifest with a distinctive hash.
    let bin = env!("CARGO_BIN_EXE_swe-seed");
    let root = temp_root("federation-root");
    let sea = root.join("SEA");
    let manifest_dir = sea.join("docs/specs/domains/agentic_capability_loop");
    fs::create_dir_all(&manifest_dir).expect("create manifest dir");
    fs::create_dir_all(sea.join("tools")).expect("create marker dir");
    fs::write(sea.join("tools/sea_parse.py"), "").expect("write marker");
    fs::write(
        manifest_dir.join("agentic_capability_loop.manifest.json"),
        r#"{"meta":{"sea_file_hash":"root-hash-123"}}"#,
    )
    .expect("write sea manifest");

    // When: federation status runs from the repository cwd with SWE_SEED_ROOT elsewhere.
    let out = Command::new(bin)
        .env("SWE_SEED_ROOT", &root)
        .args(["federation", "status"])
        .output()
        .expect("run federation status");

    // Then: the reported hash comes from SWE_SEED_ROOT, not the process cwd.
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root-hash-123"), "stdout: {stdout}");
    let _ = fs::remove_dir_all(root);
}
