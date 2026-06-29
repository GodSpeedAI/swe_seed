mod host_test_support;

use host_test_support::{cleanup, read, sync, temp_root};
use swe_seed_core::adapters::HostId;

#[test]
fn github_copilot_projection_writes_hook_json() {
    // Given: a clean root.
    let root = temp_root("github-copilot");

    // When: the GitHub Copilot adapter syncs.
    sync(&root, HostId::GithubCopilot);

    // Then: the hook JSON uses PascalCase canonical events and records cloud partials.
    let hooks = read(&root, ".github/hooks/swe-seed.json");
    assert!(hooks.contains("\"UserPromptSubmit\""));
    assert!(hooks.contains("\"PermissionRequest\""));
    assert!(hooks.contains("cloud"));
    cleanup(&root);
}
