//! WP-5 (F-04) integration: SWE_SEED obtains a cited ContextPacket from the
//! Context Kernel over MCP, satisfying POL-ACL-001/003.
//!
//! This test drives the REAL Context Kernel binary (`context-kernal`) over
//! stdio via `ContextKernelClient`. It is SKIPPED unless
//! `SWE_SEED_CONTEXT_KERNEL_BIN` points at the built binary — the test
//! harness / CI sets this when both repos are checked out. When skipped, the
//! unit-level client construction is still exercised.

use std::fs;
use std::path::PathBuf;

use swe_seed_core::federation::ContextKernelClient;

/// Resolve the CK binary from env (SWE_SEED_CONTEXT_KERNEL_BIN), else skip.
fn ck_bin() -> Option<PathBuf> {
    let p = std::env::var("SWE_SEED_CONTEXT_KERNEL_BIN").ok()?;
    let pb = PathBuf::from(p);
    if pb.exists() {
        Some(pb)
    } else {
        None
    }
}

/// POL-ACL-001/003 check: a high-risk work request must carry a context packet
/// with ≥1 citation. Mirrors the policy gate's documented requirement.
fn packet_satisfies_policy(packet: &swe_seed_core::federation::ContextPacket) -> bool {
    packet.citation_count() >= 1 && packet.authorized()
}

#[test]
fn context_kernel_returns_cited_packet_satisfying_policy() {
    let bin = match ck_bin() {
        Some(b) => b,
        None => {
            eprintln!(
                "SKIP: SWE_SEED_CONTEXT_KERNEL_BIN not set or missing \
                 (set it to the Context Kernel's context-kernal binary)"
            );
            return;
        }
    };

    let corpus_dir = std::env::temp_dir().join(format!(
        "swe-seed-ctx-corpus-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let docs = corpus_dir.join("policy");
    fs::create_dir_all(&docs).unwrap();
    fs::write(
        docs.join("acl.md"),
        "# POL-ACL-001\nA cited context packet is required before authority allows high-risk work.",
    )
    .unwrap();

    let mut client =
        ContextKernelClient::spawn(bin.to_str().unwrap(), Some(corpus_dir.to_str().unwrap()))
            .expect("spawn CK binary");

    let packet = client
        .context_required("wr-int", "cr-int", "policy", Some("context packet"), false, None)
        .expect("context_required call");

    assert!(
        packet.citation_count() >= 1,
        "POL-ACL-003: packet must carry >=1 citation, got {}",
        packet.citation_count()
    );
    assert!(packet.authorized(), "POL-ACL-002: public corpus must be authorized");
    assert!(
        packet_satisfies_policy(&packet),
        "packet must satisfy POL-ACL-001/003"
    );

    fs::remove_dir_all(&corpus_dir).ok();
}
