//! Integration (live binary): SWE_SEED acquires a cited ContextPacket from the
//! real Context Kernel over MCP stdio through the canonical E2/E3 boundary
//! (convergence T02). Skips when SWE_SEED_CONTEXT_KERNEL_BIN is unset.

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use sha2::{Digest, Sha256};
use swe_seed_core::federation::{
    verify_declared_hash, ContextClientError, ContextKernelClient, ContextRequest,
    VerifiedDomainIdentity,
};

fn ck_bin() -> Option<PathBuf> {
    let p = std::env::var("SWE_SEED_CONTEXT_KERNEL_BIN").ok()?;
    let pb = PathBuf::from(p);
    if pb.exists() {
        Some(pb)
    } else {
        None
    }
}

fn model_hash() -> &'static str {
    static H: OnceLock<String> = OnceLock::new();
    H.get_or_init(|| format!("{:x}", Sha256::digest(b"t02-live-model")))
}

fn identity() -> VerifiedDomainIdentity {
    verify_declared_hash(model_hash()).unwrap()
}

#[test]
fn context_kernel_returns_cited_packet_satisfying_policy() {
    let bin = match ck_bin() {
        Some(b) => b,
        None => {
            eprintln!(
                "SKIP: SWE_SEED_CONTEXT_KERNEL_BIN not set or missing \
                 (set it to the Context Kernel's ck-bin binary)"
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

    // Happy path: required context resolves with citations and passes every
    // boundary gate (producer authority, identity, drift, causality).
    let packet = client
        .acquire_context(ContextRequest {
            work_request_id: "wr-int",
            context_requirement_id: "cr-int",
            corpus_id: "policy",
            query: Some("context packet"),
            domain_identity: &identity(),
            authority_decision_id: Some("sea-decision-test"),
            authority_reference: Some("AuthorityChecked#evt_test"),
            required: true,
        })
        .expect("required context acquisition must succeed with citations");

    assert!(packet.citation_count() >= 1, "POL-ACL-003: >=1 citation");
    assert_eq!(packet.work_request_id(), Some("wr-int"));
    assert_eq!(packet.domain_model_hash(), Some(model_hash()));
    assert_eq!(
        packet.authority_reference(),
        Some("AuthorityChecked#evt_test")
    );

    // Governed no-context: absent corpus + required must be an EXPLICIT
    // non-success — never a silent zero-citation settlement.
    let err = client
        .acquire_context(ContextRequest {
            work_request_id: "wr-int-2",
            context_requirement_id: "cr-int-2",
            corpus_id: "absent-corpus",
            query: None,
            domain_identity: &identity(),
            authority_decision_id: None,
            authority_reference: None,
            required: true,
        })
        .unwrap_err();
    assert!(
        matches!(err, ContextClientError::GovernedNoContext { .. }),
        "zero-citation required context must be the governed outcome: {err}"
    );

    fs::remove_dir_all(&corpus_dir).ok();
}
