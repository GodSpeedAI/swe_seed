mod host_test_support;

use host_test_support::{cleanup, read, sync, temp_root};
use swe_seed_core::adapters::HostId;

#[test]
fn ci_projection_writes_non_interactive_backstop() {
    // Given: a clean root.
    let root = temp_root("ci");

    // When: the CI adapter syncs.
    let report = sync(&root, HostId::Ci);

    // Then: shell, PowerShell, and workflow enforcement files are projected.
    assert_eq!(report.files.len(), 3);
    assert!(read(&root, ".agents/ci/swe-seed-policy-check.sh").contains("swe-seed doctor"));
    assert!(read(&root, ".agents/ci/swe-seed-policy-check.ps1").contains("swe-seed doctor"));
    assert!(read(&root, ".github/workflows/swe-seed-policy.yml").contains("swe-seed-policy"));
    cleanup(&root);
}
