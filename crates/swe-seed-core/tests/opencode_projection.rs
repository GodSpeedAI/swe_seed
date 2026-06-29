mod host_test_support;

use host_test_support::{cleanup, read, sync, temp_root};
use swe_seed_core::adapters::HostId;

#[test]
fn opencode_projection_writes_plugin() {
    // Given: a clean root.
    let root = temp_root("opencode");

    // When: the OpenCode adapter syncs.
    sync(&root, HostId::OpenCode);

    // Then: a TypeScript plugin maps native events to the shared hook port.
    let plugin = read(&root, ".opencode/plugins/swe_seed.ts");
    assert!(plugin.contains("BEGIN SWE_SEED MANAGED"));
    assert!(plugin.contains("tool.execute.before"));
    assert!(plugin.contains("permission.asked"));
    assert!(plugin.contains("swe-seed agent-hooks capture"));
    cleanup(&root);
}
