//! Integration (live binary): SWE_SEED acquires a cited ContextPacket from the
//! real Context Kernel over MCP stdio through the canonical E2/E3 boundary
//! (convergence T02). Skips when SWE_SEED_CONTEXT_KERNEL_BIN is unset.

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use sha2::{Digest, Sha256};
use swe_seed_core::federation::{
    verify_declared_hash, ContextClientError, ContextKernelClient, ContextRequest,
    RetrievalCompleteness, VerifiedDomainIdentity, WorldRef,
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

fn world() -> WorldRef {
    WorldRef::parse(&format!("world:t02-live@sha256:{}", "b".repeat(64))).unwrap()
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
            world_ref: &world(),
            require_complete: true,
            authority_decision_id: Some("sea-decision-test"),
            authority_reference: Some("AuthorityChecked#evt_test"),
            required: true,
        })
        .expect("required context acquisition must succeed with citations");

    assert!(packet.citation_count() >= 1, "POL-ACL-003: >=1 citation");
    assert_eq!(packet.work_request_id(), Some("wr-int"));
    assert_eq!(packet.domain_model_hash(), Some(model_hash()));
    // CEP-0008: the real Context Kernel echoes the pinned world and states
    // that one matching document, returned in full, is a complete retrieval.
    assert_eq!(packet.world_ref(), Some(world().as_str()));
    assert_eq!(packet.retrieval_completeness(), RetrievalCompleteness::Complete);
    assert!(packet.omissions().is_empty());
    assert_eq!(
        packet.authority_reference(),
        Some("AuthorityChecked#evt_test")
    );
    // Dual-read consumption: CK now writes the canonical CEP bundle; SWE_SEED
    // verifies it at the boundary and binds ITS identity (never the legacy
    // packet hash) into the downstream lineage.
    assert!(packet.bundle().is_some(), "canonical bundle must be consumed");
    let bundle = packet.bundle().unwrap();
    assert_eq!(bundle["envelope_kind"], "context_bundle");
    assert_eq!(
        bundle["extensions"]["cep.profile"]["profile_id"],
        "godspeed.context_bundle"
    );
    assert_eq!(bundle["scope"]["world_ref"], world().as_str());
    assert_eq!(packet.bundle_envelope_id(), bundle["envelope_id"].as_str());
    let hash = packet.bundle_content_hash().expect("bundle integrity hash");
    assert!(hash.starts_with("sha256:"));
    assert_eq!(Some(hash), bundle["integrity"]["content_hash"].as_str());
    // The integrity recompute matches SWE_SEED's independent hash.
    assert_eq!(
        hash,
        swe_seed_core::federation::bundle_content_hash(bundle).as_str()
    );
    let _ = identity;

    // Governed no-context: absent corpus + required must be an EXPLICIT
    // non-success — never a silent zero-citation settlement.
    let err = client
        .acquire_context(ContextRequest {
            work_request_id: "wr-int-2",
            context_requirement_id: "cr-int-2",
            corpus_id: "absent-corpus",
            query: None,
            domain_identity: &identity(),
            world_ref: &world(),
            require_complete: false,
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

#[test]
fn ck_unavailable_is_explicit_and_never_fabricated() {
    // No env configured: from_env reports explicit unavailability (None),
    // which callers surface as the governed CK-unavailable outcome rather
    // than fabricating context.
    if std::env::var("SWE_SEED_CONTEXT_KERNEL_BIN").is_err()
        && std::env::var("SWE_SEED_CONTEXT_KERNEL_URL").is_err()
    {
        assert!(ContextKernelClient::from_env().is_none());
    }

    // A persistent-service client pointed at a dead endpoint fails EXPLICITLY
    // (transport error) instead of inventing a response.
    let mut client = ContextKernelClient::connect("http://127.0.0.1:1");
    let err = client
        .call_tool("get_status", serde_json::json!({}))
        .unwrap_err();
    assert!(
        matches!(err, ContextClientError::Transport(_)),
        "a dead CK service must be an explicit transport failure: {err:?}"
    );
}
