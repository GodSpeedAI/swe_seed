mod host_test_support;

use host_test_support::{cleanup, read, sync, temp_root};
use swe_seed_core::adapters::HostId;

#[test]
fn codex_projection_writes_command_hooks() {
    // Given: a clean root.
    let root = temp_root("codex");

    // When: the Codex adapter syncs.
    sync(&root, HostId::Codex);

    // Then: Codex command hooks are projected without claiming prompt hooks as native.
    let hooks = read(&root, ".codex/hooks.json");
    assert!(hooks.contains("\"swe_seed_managed\""));
    assert!(hooks.contains("\"PreToolUse\""));
    assert!(hooks.contains("\"command\""));
    assert!(hooks.contains("partial"));
    cleanup(&root);
}
