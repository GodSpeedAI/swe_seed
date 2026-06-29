mod host_test_support;

use std::fs;

use host_test_support::{cleanup, drift, sync, temp_root};
use swe_seed_core::adapters::{DriftStatus, HostId};

#[test]
fn drift_detection_catches_managed_file_edits() {
    // Given: a synced Codex hook projection.
    let root = temp_root("drift");
    sync(&root, HostId::Codex);
    assert_eq!(drift(&root, HostId::Codex).status, DriftStatus::Clean);

    // When: the managed file is manually edited.
    let path = root.join(".codex/hooks.json");
    let mut content = fs::read_to_string(&path).expect("read hooks");
    content.push_str("\nmanual edit\n");
    fs::write(&path, content).expect("edit hooks");

    // Then: doctor-grade drift detection reports drift.
    let report = drift(&root, HostId::Codex);
    assert_eq!(report.status, DriftStatus::Drifted);
    assert_eq!(report.drifted_files, vec![".codex/hooks.json"]);
    cleanup(&root);
}

#[test]
fn drift_detection_catches_snapshot_only_files() {
    // Given: a synced Codex projection with an older snapshot-managed file still on disk.
    let root = temp_root("drift-stale");
    sync(&root, HostId::Codex);
    let stale_rel = ".codex/removed-hooks.json";
    fs::write(root.join(stale_rel), "{}\n").expect("write stale file");
    let snapshot_path = root.join(".swe-seed/host-snapshots/codex.json");
    let mut snapshot: serde_json::Value =
        serde_json::from_slice(&fs::read(&snapshot_path).expect("read snapshot"))
            .expect("parse snapshot");
    snapshot["entries"]
        .as_array_mut()
        .expect("snapshot entries")
        .push(serde_json::json!({
            "rel_path": stale_rel,
            "existed": false,
            "bytes": [],
        }));
    fs::write(
        &snapshot_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&snapshot).expect("format snapshot")
        ),
    )
    .expect("write snapshot");

    // When: drift is checked against the current projection plan.
    let report = drift(&root, HostId::Codex);

    // Then: the stale snapshot-only managed file is reported as drift.
    assert_eq!(report.status, DriftStatus::Drifted);
    assert!(report.drifted_files.contains(&stale_rel.to_string()));
    cleanup(&root);
}
