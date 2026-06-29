mod host_test_support;

use std::fs;

use host_test_support::{cleanup, read, rollback, sync, temp_root};
use swe_seed_core::adapters::HostId;

#[test]
fn rollback_restores_preexisting_file_bytes() {
    // Given: a host file that existed before SWE_SEED sync.
    let root = temp_root("rollback-existing");
    let codex = root.join(".codex");
    fs::create_dir_all(&codex).expect("create codex dir");
    fs::write(codex.join("hooks.json"), "{\"user\":\"kept\"}\n").expect("write user file");

    // When: the file is synced and then rolled back.
    sync(&root, HostId::Codex);
    assert!(read(&root, ".codex/hooks.json").contains("swe_seed_managed"));
    rollback(&root, HostId::Codex);

    // Then: the original bytes are restored exactly.
    assert_eq!(read(&root, ".codex/hooks.json"), "{\"user\":\"kept\"}\n");
    cleanup(&root);
}

#[test]
fn rollback_removes_files_created_by_swe_seed() {
    // Given: no Antigravity hook file exists.
    let root = temp_root("rollback-created");

    // When: sync creates the file and rollback runs.
    sync(&root, HostId::Antigravity);
    assert!(root.join(".agents/hooks.json").exists());
    rollback(&root, HostId::Antigravity);

    // Then: the created file is removed.
    assert!(!root.join(".agents/hooks.json").exists());
    cleanup(&root);
}
