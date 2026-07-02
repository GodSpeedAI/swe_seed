mod host_test_support;

use host_test_support::{cleanup, read, sync, temp_root};
use swe_seed_core::adapters::HostId;

#[test]
fn antigravity_projection_is_minimal_pre_tool_config() {
    // Given: a clean root.
    let root = temp_root("antigravity");

    // When: the Antigravity adapter syncs.
    sync(&root, HostId::Antigravity);

    // Then: only conservative PreToolUse command matchers are projected.
    let hooks = read(&root, ".agents/hooks.json");
    assert!(hooks.contains("\"PreToolUse\""));
    assert!(hooks.contains("\"run_command\""));
    assert!(hooks.contains("\"shell\""));
    assert!(!hooks.contains("\"SubagentStart\""));
    cleanup(&root);
}
