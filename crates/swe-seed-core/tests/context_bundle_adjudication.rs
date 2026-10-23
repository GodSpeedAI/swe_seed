//! Dual-read adjudication of the canonical CEP `godspeed.context_bundle` at
//! the SWE_SEED boundary (convergence T02, bundle half).
//!
//! The bundle is verified before anything is handed to the caller: envelope
//! kind, profile declaration, pinned world, work-request/requirement
//! correlation, truthful complete/partial/none accounting, CEP integrity, and
//! cross-consistency with the legacy packet carried in the same response.
//! Legacy-only responses keep adjudicating during the bounded migration
//! window (dual-read), but single-write means CK's canonical output is the
//! bundle.

use serde_json::{json, Value};

use swe_seed_core::federation::{
    adjudicate_context_response, bundle_content_hash, verify_context_bundle, ContextClientError,
    ExpectedContext, RetrievalCompleteness, VerifiedDomainIdentity,
};

const WR: &str = "wr-bundle";
const CR: &str = "cr-bundle";
const REQUEST_EVENT: &str = "11111111-2222-3333-4444-555555555555";
const WORLD: &str =
    "world:bundle@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn identity() -> VerifiedDomainIdentity {
    swe_seed_core::federation::verify_declared_hash(HASH).unwrap()
}

/// A self-consistent CEP `godspeed.context_bundle` for `wr`. `level` is the
/// retrieval_completeness (`complete` | `partial` | `none`).
fn bundle(level: &str) -> Value {
    let (refs, omissions, questions, status) = match level {
        "complete" => (
            json!([{
                "ref_id": "ref-chk-1",
                "ref_type": "chunk",
                "target_uri_or_id": "docs/a.md#h",
                "availability_status": "available",
                "integrity_status": "unverified",
            }]),
            json!([]),
            None,
            "complete_for_context",
        ),
        "partial" => (
            json!([{
                "ref_id": "ref-chk-1",
                "ref_type": "chunk",
                "target_uri_or_id": "docs/a.md#h",
                "availability_status": "available",
                "integrity_status": "unverified",
            }]),
            json!([{"omission_id": "om-1", "omission_type": "max_results_reached", "reason": "max_results_reached"}]),
            None,
            "partial",
        ),
        "none" => (
            json!(null),
            json!([{"omission_id": "om-1", "omission_type": "corpus_not_found", "reason": "corpus_not_found"}]),
            Some(json!([{"question_id": CR, "question_form": "What context applies?"}])),
            "minimal",
        ),
        other => panic!("unknown level {other}"),
    };
    let mut env = json!({
        "envelope_id": "env-ctx-BUNDLE",
        "cep_version": "1.0.0",
        "envelope_version": "1.0",
        "envelope_kind": "context_bundle",
        "created_at": "2026-10-07T12:00:00Z",
        "created_by": "context-kernel",
        "scope": {"world_ref": WORLD},
        "boundary_record": {
            "scope": "context for wr-bundle",
            "included_sections": [],
            "excluded_sections": [],
            "known_omissions": [],
            "unknowns": [],
            "redactions": [],
            "compression_notes": [],
            "out_of_scope_entities": [],
            "limitations": [],
        },
        "completeness_status": status,
        "omission_status": if level == "complete" { "none_known" } else { "marked" },
        "provenance_refs": ["prov-ctx-BUNDLE"],
        "validation_status": "valid",
        "lineage_refs": [],
        "provenance": [{
            "provenance_id": "prov-ctx-BUNDLE",
            "source_system_refs": ["corpus:docs"],
            "producer_refs": ["context-kernel"],
            "production_method": "retrieval",
            "created_at": "2026-10-07T12:00:00Z",
        }],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.context_bundle", "profile_version": "1.0.0"},
            "godspeed.context_bundle": {
                "work_request_ref": WR,
                "retrieval_completeness": level,
                "context_packet_id": "ctx-BUNDLE",
                "context_requirement_id": CR,
                "domain_model_hash": HASH,
            },
        },
    });
    if let Some(r) = refs.as_array() {
        env["references"] = Value::Array(r.clone());
    }
    if let Some(o) = omissions.as_array() {
        if !o.is_empty() {
            env["omissions"] = Value::Array(o.clone());
        }
    }
    if let Some(q) = questions {
        env["questions"] = q;
    }
    env["integrity"] = json!({
        "content_hash": bundle_content_hash(&env),
        "verification_method": "sha256-canonical-json-v1",
        "verification_status": "verified",
        "tamper_status": "intact",
    });
    env
}

/// The legacy companion envelope of the same retrieval (same ids/outcome).
fn packet(level_payload: &str, citations: usize) -> Value {
    let cites: Vec<Value> = (0..citations)
        .map(|i| json!({"citation_id": format!("c{i}"), "source": "docs/a.md", "content": "x", "fetched_at": "2026-10-07T12:00:00Z", "truncated": false}))
        .collect();
    json!({
        "schema_version": "v1",
        "event_id": "22222222-2222-3333-4444-555555555555",
        "source_agent": "context-kernel",
        "event_type": "ContextPacketCreated",
        "occurred_at": "2026-10-07T12:00:00Z",
        "idempotency_key": "1".repeat(64),
        "payload": {
            "domain_model_hash": HASH,
            "world_ref": WORLD,
            "retrieval_completeness": level_payload,
            "omissions": if level_payload == "complete" { json!([]) } else { json!([level_payload]) },
            "namespace": "agentic_capability_loop",
            "work_request_id": WR,
            "context_requirement_id": CR,
            "context_packet_id": "ctx-BUNDLE",
            "corpus_id": "docs",
            "citations": cites,
        },
        "provenance": {
            "origin": "context-kernel",
            "chain": [format!("domain_model_hash:{HASH}"), format!("caused_by:{REQUEST_EVENT}")],
        },
    })
}

fn response_with(level: &str) -> Value {
    let cites = if level == "none" { 0 } else { 1 };
    json!({
        "context_bundle": bundle(level),
        "context_envelope": packet(level, cites),
        "outcome": if level == "none" { "no_context_governed" } else { "cited" },
        "reason": if level == "none" { json!(level) } else { Value::Null },
    })
}

fn expected() -> ExpectedContext<'static> {
    ExpectedContext {
        work_request_id: WR,
        context_requirement_id: CR,
        domain_model_hash: HASH,
        request_event_id: REQUEST_EVENT,
        required: true,
        world_ref: WORLD,
        require_complete: false,
    }
}

#[test]
fn a_valid_bundle_adjudicates_and_binds_its_identity() {
    let packet = adjudicate_context_response(&response_with("complete"), &expected()).unwrap();
    assert_eq!(packet.bundle_envelope_id(), Some("env-ctx-BUNDLE"));
    assert!(packet.bundle_content_hash().unwrap().starts_with("sha256:"));
    assert_eq!(
        packet.bundle().unwrap()["extensions"]["godspeed.context_bundle"]["work_request_ref"],
        WR
    );
    // ... and the identity recomputes from the envelope bytes.
    assert_eq!(
        packet.bundle_content_hash(),
        Some(bundle_content_hash(packet.bundle().unwrap()).as_str())
    );
}

#[test]
fn legacy_only_responses_keep_adjudicating_during_the_migration_window() {
    let mut resp = response_with("complete");
    resp.as_object_mut().unwrap().remove("context_bundle");
    let packet = adjudicate_context_response(&resp, &expected()).unwrap();
    assert!(packet.bundle().is_none());
    assert_eq!(packet.retrieval_completeness(), RetrievalCompleteness::Complete);
}

#[test]
fn a_tampered_bundle_is_refused_even_when_the_packet_is_valid() {
    let mut resp = response_with("complete");
    resp["context_bundle"]["references"][0]["target_uri_or_id"] = json!("docs/forged.md");
    let err = adjudicate_context_response(&resp, &expected()).unwrap_err();
    assert!(
        matches!(err, ContextClientError::BundleRejected(_)),
        "tampered content must be refused: {err:?}"
    );
}

#[test]
fn a_bundle_in_another_world_is_refused() {
    let mut resp = response_with("complete");
    let other = "world:evil@sha256:".to_string() + &"d".repeat(64);
    resp["context_bundle"]["scope"]["world_ref"] = json!(other);
    resp["context_bundle"]["integrity"]["content_hash"] =
        json!(bundle_content_hash(&resp["context_bundle"]));
    let err = adjudicate_context_response(&resp, &expected()).unwrap_err();
    assert!(
        matches!(err, ContextClientError::BundleRejected(_)),
        "a bundle in another world must be refused: {err:?}"
    );
}

#[test]
fn a_falsely_complete_bundle_is_refused() {
    let mut resp = response_with("partial");
    // promote the extension + status without fixing the facts
    resp["context_bundle"]["extensions"]["godspeed.context_bundle"]["retrieval_completeness"] =
        json!("complete");
    resp["context_bundle"]["completeness_status"] = json!("complete_for_context");
    resp["context_bundle"]["integrity"]["content_hash"] =
        json!(bundle_content_hash(&resp["context_bundle"]));
    let err = adjudicate_context_response(&resp, &expected()).unwrap_err();
    assert!(
        matches!(err, ContextClientError::BundleRejected(_)),
        "falsely-marked-complete must be refused: {err:?}"
    );
}

#[test]
fn a_cross_wired_bundle_is_refused() {
    let mut resp = response_with("complete");
    resp["context_bundle"]["extensions"]["godspeed.context_bundle"]["work_request_ref"] =
        json!("wr-someone-else");
    resp["context_bundle"]["integrity"]["content_hash"] =
        json!(bundle_content_hash(&resp["context_bundle"]));
    let err = adjudicate_context_response(&resp, &expected()).unwrap_err();
    assert!(
        matches!(err, ContextClientError::BundleRejected(_)),
        "{err:?}"
    );
}

#[test]
fn a_bundle_that_disagrees_with_its_packet_is_refused() {
    let mut resp = response_with("partial");
    resp["context_envelope"]["payload"]["retrieval_completeness"] = json!("complete");
    resp["context_envelope"]["payload"]["omissions"] = json!([]);
    let err = adjudicate_context_response(&resp, &expected()).unwrap_err();
    assert!(
        matches!(err, ContextClientError::BundleRejected(_)),
        "{err:?}"
    );
}

#[test]
fn a_wrong_profile_declaration_is_refused() {
    let mut resp = response_with("complete");
    resp["context_bundle"]["extensions"]["cep.profile"]["profile_id"] =
        json!("godspeed.authority_request");
    let err = adjudicate_context_response(&resp, &expected()).unwrap_err();
    assert!(
        matches!(err, ContextClientError::BundleRejected(_)),
        "{err:?}"
    );
}

#[test]
fn verify_context_bundle_covers_all_three_states() {
    for level in ["complete", "partial", "none"] {
        let facts = verify_context_bundle(&bundle(level), WR, WORLD, Some(CR))
            .unwrap_or_else(|e| panic!("{level}: {e}"));
        assert_eq!(facts.retrieval_completeness, level);
        assert_eq!(facts.world_ref, WORLD);
    }
}
