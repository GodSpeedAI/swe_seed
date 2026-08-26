//! Convergence plan T02 — cited context slice (E2, E3, I4).
//!
//! Frozen target: `.agents/specs/e2e-preregistration.yml` (sea-rs). These are
//! the unit-level teeth: every rejection path of the canonical E2/E3 boundary
//! is exercised against the pure response adjudicator without spawning the CK
//! binary. Live-binary integration lives in `context_kernel_client.rs`.

use std::sync::OnceLock;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use swe_seed_core::federation::{
    adjudicate_context_response, agent_id, authoritative_producer, fallback_hash,
    make_event_verified, validate_producer, verify_declared_hash, ContextClientError,
    ExpectedContext, VerifiedDomainIdentity,
};

const WR: &str = "wr-t02";
const CR: &str = "cr-t02";
const REQUEST_EVENT: &str = "11111111-2222-3333-4444-555555555555";

fn digest(seed: &[u8]) -> String {
    format!("{:x}", Sha256::digest(seed))
}

fn model_hash() -> &'static str {
    static H: OnceLock<String> = OnceLock::new();
    H.get_or_init(|| digest(b"t02-model"))
}

fn identity() -> VerifiedDomainIdentity {
    verify_declared_hash(model_hash()).unwrap()
}

fn citation(n: u8) -> Value {
    json!({
        "citation_id": format!("cit-{n}"),
        "source": format!("corpus://doc{n}"),
        "content": "cited content",
        "fetched_at": "2026-08-25T00:00:00Z",
    })
}

/// A well-formed E3 response as CK produces it for our request.
fn good_response() -> Value {
    let payload = json!({
        "domain_model_hash": model_hash(),
        "namespace": "agentic_capability_loop",
        "work_request_id": WR,
        "context_requirement_id": CR,
        "context_packet_id": "ctx-T02",
        "corpus_id": "docs",
        "citations": [citation(1), citation(2)],
    });
    json!({
        "context_envelope": {
            "schema_version": "v1",
            "event_id": "22222222-2222-3333-4444-555555555555",
            "source_agent": "context-kernel",
            "event_type": "ContextPacketCreated",
            "occurred_at": "2026-08-25T00:00:00Z",
            "idempotency_key": digest(b"t02-idem"),
            "payload": payload,
            "provenance": {
                "origin": "context-kernel",
                "chain": [
                    format!("domain_model_hash:{}", model_hash()),
                    format!("caused_by:{REQUEST_EVENT}"),
                ],
            },
        },
        "outcome": "cited",
    })
}

fn expected(required: bool) -> ExpectedContext<'static> {
    ExpectedContext {
        work_request_id: WR,
        context_requirement_id: CR,
        domain_model_hash: model_hash(),
        request_event_id: REQUEST_EVENT,
        required,
    }
}

// --- E3 happy path -----------------------------------------------------------

#[test]
fn e3_cited_packet_passes_all_boundary_gates() {
    let packet = adjudicate_context_response(&good_response(), &expected(true)).unwrap();
    assert_eq!(packet.citation_count(), 2);
    assert_eq!(packet.work_request_id(), Some(WR));
    assert_eq!(packet.context_requirement_id(), Some(CR));
}

// --- frozen falsifier: cross-wired packets -----------------------------------

#[test]
fn t02_cross_wired_work_request_is_rejected() {
    let mut resp = good_response();
    resp["context_envelope"]["payload"]["work_request_id"] = json!("wr-OTHER");
    match adjudicate_context_response(&resp, &expected(true)) {
        Err(ContextClientError::CrossWired { field, .. }) => {
            assert_eq!(field, "work_request_id");
        }
        other => panic!("cross-wired wr must be rejected: {other:?}"),
    }
}

#[test]
fn t02_cross_wired_context_requirement_is_rejected() {
    let mut resp = good_response();
    resp["context_envelope"]["payload"]["context_requirement_id"] = json!("cr-OTHER");
    match adjudicate_context_response(&resp, &expected(true)) {
        Err(ContextClientError::CrossWired { field, .. }) => {
            assert_eq!(field, "context_requirement_id");
        }
        other => panic!("cross-wired cr must be rejected: {other:?}"),
    }
}

// --- frozen falsifier: lost causal-parent identity ----------------------------

#[test]
fn t02_packet_not_citing_our_e2_request_is_rejected() {
    let mut resp = good_response();
    resp["context_envelope"]["provenance"]["chain"] =
        json!([format!("domain_model_hash:{}", model_hash())]);
    match adjudicate_context_response(&resp, &expected(true)) {
        Err(ContextClientError::CausalityBroken {
            expected_parent, ..
        }) => {
            assert_eq!(expected_parent, REQUEST_EVENT);
        }
        other => panic!("missing causal parent must be rejected: {other:?}"),
    }
}

// --- producer authority at the boundary (I3 adoption) ------------------------

#[test]
fn t02_non_context_kernel_producer_is_rejected_at_boundary() {
    for forger in [
        "swe-seed",
        "godspeed-agent",
        "realitytrace",
        "mystery-agent",
    ] {
        let mut resp = good_response();
        resp["context_envelope"]["source_agent"] = json!(forger);
        let err = adjudicate_context_response(&resp, &expected(true)).unwrap_err();
        assert!(
            matches!(err, ContextClientError::BoundaryRejected(_)),
            "forger {forger} must fail the boundary gate: {err}"
        );
    }
}

// --- zero-citation outcomes are explicit, never silent success ---------------

#[test]
fn t02_required_zero_citation_is_explicit_governed_no_context() {
    let mut resp = good_response();
    resp["context_envelope"]["payload"]["citations"] = json!([]);
    resp["outcome"] = json!("no_context_governed");
    resp["reason"] = json!("corpus_not_found");
    match adjudicate_context_response(&resp, &expected(true)) {
        Err(ContextClientError::GovernedNoContext { reason }) => {
            assert_eq!(reason.as_deref(), Some("corpus_not_found"));
        }
        other => panic!("required no-context must be explicit error: {other:?}"),
    }
}

#[test]
fn t02_zero_citations_claiming_success_is_protocol_violation() {
    // The exact Delta-0 forbidden shape: empty citations under a success
    // outcome must never adjudicate as acquisition.
    let mut resp = good_response();
    resp["context_envelope"]["payload"]["citations"] = json!([]);
    let err = adjudicate_context_response(&resp, &expected(true)).unwrap_err();
    assert!(
        matches!(err, ContextClientError::InvalidResponse(_)),
        "{err}"
    );
}

#[test]
fn t02_optional_no_context_is_also_non_success() {
    let mut resp = good_response();
    resp["context_envelope"]["payload"]["citations"] = json!([]);
    resp["outcome"] = json!("empty_optional");
    resp["reason"] = json!("no_matching_content");
    let err = adjudicate_context_response(&resp, &expected(false)).unwrap_err();
    assert!(
        matches!(err, ContextClientError::GovernedNoContext { .. }),
        "{err}"
    );
}

// --- domain identity drift / placeholders ------------------------------------

#[test]
fn t02_identity_drift_from_our_declaration_is_rejected() {
    let mut resp = good_response();
    resp["context_envelope"]["payload"]["domain_model_hash"] = json!(digest(b"a-different-model"));
    let err = adjudicate_context_response(&resp, &expected(true)).unwrap_err();
    assert!(
        matches!(err, ContextClientError::BoundaryRejected(_)),
        "{err}"
    );
}

#[test]
fn t02_placeholder_identity_in_packet_is_rejected() {
    let mut resp = good_response();
    resp["context_envelope"]["payload"]["domain_model_hash"] = json!(fallback_hash());
    let err = adjudicate_context_response(&resp, &expected(true)).unwrap_err();
    assert!(
        matches!(err, ContextClientError::BoundaryRejected(_)),
        "{err}"
    );
}

// --- I4: authority references are pass-through only --------------------------

#[test]
fn i4_ck_cannot_emit_authority_checked_even_with_correct_stamp() {
    // AuthorityChecked's exclusive producer is sea_forge. Even if some surface
    // stamped source_agent="context-kernel" onto an AuthorityChecked event,
    // the registry rejects it — CK cannot create execution authority.
    let mut forged = make_event_verified("AuthorityChecked", Default::default(), &identity());
    forged.source_agent = "context-kernel".into();
    assert_eq!(
        authoritative_producer("AuthorityChecked"),
        Some(agent_id::SEA_FORGE)
    );
    assert!(validate_producer(&forged).is_err());

    // And an authority reference rides through the packet as an opaque string:
    let mut resp = good_response();
    resp["context_envelope"]["payload"]["authority_reference"] =
        json!("AuthorityChecked#evt_sea_1");
    let packet = adjudicate_context_response(&resp, &expected(true)).unwrap();
    assert_eq!(
        packet.authority_reference(),
        Some("AuthorityChecked#evt_sea_1")
    );
}
