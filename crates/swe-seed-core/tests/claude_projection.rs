mod host_test_support;

use host_test_support::{cleanup, read, sync, temp_root};
use swe_seed_core::adapters::HostId;

#[test]
fn claude_projection_writes_settings_hooks() {
    // Given: a clean root.
    let root = temp_root("claude");

    // When: the Claude adapter syncs.
    let report = sync(&root, HostId::Claude);

    // Then: both Claude settings files contain deterministic managed hook config.
    assert_eq!(report.files.len(), 2);
    let settings = read(&root, ".claude/settings.json");
    let local = read(&root, ".claude/settings.local.json");
    assert!(settings.contains("\"swe_seed_managed\""));
    assert!(settings.contains("\"PreToolUse\""));
    assert!(settings.contains("\"SubagentStart\""));
    assert!(local.contains("\"local_overrides\""));
    cleanup(&root);
}
