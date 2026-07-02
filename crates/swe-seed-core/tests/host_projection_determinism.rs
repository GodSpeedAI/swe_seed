mod host_test_support;

use std::collections::BTreeMap;
use std::fs;

use host_test_support::{cleanup, sync, temp_root};
use swe_seed_core::adapters::marker::merge_json;
use swe_seed_core::adapters::{all_host_ids, sync_host, HostId, SyncOptions};

fn projected_bytes(root: &std::path::Path) -> BTreeMap<String, Vec<u8>> {
    let paths = [
        ".claude/settings.json",
        ".claude/settings.local.json",
        ".codex/hooks.json",
        ".opencode/plugins/swe_seed.ts",
        ".github/hooks/swe-seed.json",
        ".agents/hooks.json",
        ".agents/ci/swe-seed-policy-check.sh",
        ".agents/ci/swe-seed-policy-check.ps1",
        ".github/workflows/swe-seed-policy.yml",
    ];
    paths
        .iter()
        .filter_map(|rel| {
            fs::read(root.join(rel))
                .ok()
                .map(|b| ((*rel).to_string(), b))
        })
        .collect()
}

#[test]
fn resync_without_manifest_change_is_byte_stable() {
    // Given: a clean repository root.
    let root = temp_root("determinism");

    // When: every host is synced twice without changing inputs.
    for host in all_host_ids() {
        sync(&root, host);
    }
    let first = projected_bytes(&root);
    for host in all_host_ids() {
        sync(&root, host);
    }
    let second = projected_bytes(&root);

    // Then: the managed bytes are identical after the second sync.
    assert_eq!(first, second);
    assert_eq!(first.len(), 9);
    cleanup(&root);
}

#[test]
fn dry_run_reports_plan_without_writing_files() {
    // Given: an empty root.
    let root = temp_root("dry-run");

    // When: a host is dry-run synced.
    let report = host_test_support::dry_run(&root, HostId::Codex);

    // Then: a deterministic plan is reported, but no file is created.
    assert_eq!(report.host_id, HostId::Codex);
    assert!(report.dry_run);
    assert_eq!(report.files.len(), 1);
    assert!(!root.join(".codex/hooks.json").exists());
    cleanup(&root);
}

#[test]
fn json_merge_removes_previously_managed_keys_that_leave_the_projection() {
    // Given: an existing host JSON with user content plus two managed keys.
    let existing = br#"{
  "swe_seed_managed": {
    "version": 1,
    "stable_id": "x",
    "managed_keys": ["hooks", "partial_support"]
  },
  "user": "kept",
  "hooks": [],
  "partial_support": ["stale"]
}"#;
    let projected = r#"{
  "swe_seed_managed": {
    "version": 1,
    "stable_id": "x"
  },
  "hooks": []
}"#;

    // When: the next deterministic projection drops one managed key.
    let merged = merge_json(Some(existing.as_slice()), projected).expect("merge json");
    let value: serde_json::Value = serde_json::from_slice(&merged).expect("parse merged");

    // Then: user content stays, current managed content stays, and stale managed content is gone.
    assert_eq!(value["user"], "kept");
    assert!(value.get("hooks").is_some());
    assert!(value.get("partial_support").is_none());
}

#[test]
fn snapshot_building_propagates_non_not_found_read_errors() {
    // Given: a path where the projected file should be, but it is a directory.
    let root = temp_root("snapshot-read-error");
    fs::create_dir_all(root.join(".codex/hooks.json")).expect("create directory at file path");

    // When: sync tries to snapshot the pre-existing path.
    let result = sync_host(&root, HostId::Codex, SyncOptions::write());

    // Then: the non-NotFound read error is returned instead of becoming an absent snapshot.
    assert!(
        result.is_err(),
        "directory read must fail snapshot creation"
    );
    cleanup(&root);
}
