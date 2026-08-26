//! Convergence plan T04 — governed work submission (frozen edge E4).
//!
//! Frozen target: `.agents/specs/e2e-preregistration.yml` (edge E4,
//! event_ownership.GovernedWorkRequest = swe_seed, I1/I3, ENV-I2/I3/I4).
//! The emission surface (`build_governed_work_request`) refuses every
//! preregistered falsifier before any canonical envelope exists. The
//! cross-repo golden fixture consumed by SEA-Forge's ingress gate is written
//! here from this REAL emitter output (see
//! sea-rs/crates/sea-forge-server/tests/convergence_t04_governed_ingress.rs).

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use swe_seed_core::federation::{
    accept_work_requested, build_governed_work_request, context_packet_ref, fallback_hash,
    make_event, verify_against_artifact, verify_declared_hash, ConsumeError, Envelope,
    GovernedSubmission, HashSource, ProofContract, ResolvedHash, SubmissionError,
    VerifiedDomainIdentity, WorkRequestContract, PROOF_TYPE_LIVE,
};

fn tmp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("swe-seed-t04-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A real canonical model artifact: bytes + their true SHA-256.
fn write_model_artifact(dir: &Path) -> (PathBuf, String) {
    let path = dir.join("canonical_model.sea.json");
    std::fs::write(
        &path,
        b"{\"model\":\"godspeed-canonical\",\"version\":\"0.1.0\"}",
    )
    .unwrap();
    let mut h = Sha256::new();
    h.update(std::fs::read(&path).unwrap());
    (path, format!("{:x}", h.finalize()))
}

/// A boundary-clean E1 envelope: properly stamped by its exclusive producer.
fn work_requested_envelope(work_request_id: &str, hash: &str) -> Envelope {
    let mut env = make_event(
        "WorkRequested",
        json!({
            "work_request_id": work_request_id,
            "affordance_id": "aff-blue-green-001",
            "desired_outcome": "deploy service via blue-green rotation",
            "settlement_criteria": ["health check green", "zero-downtime observed"],
        })
        .as_object()
        .unwrap()
        .clone(),
        hash,
    );
    env.source_agent = "godspeed-agent".into();
    env
}

/// A boundary-clean E3 envelope: properly stamped by context_kernel.
fn context_packet_envelope(work_request_id: &str, hash: &str) -> Envelope {
    let mut env = make_event(
        "ContextPacketCreated",
        json!({
            "work_request_id": work_request_id,
            "context_requirement_id": "cr-t04",
            "context_packet_id": "ctx_01J9T04PACKET0000000000",
            "citations": [
                {"source": "runbook://deploy/blue-green", "sha256": format!("{:x}", Sha256::digest(b"cited"))}
            ],
        })
        .as_object()
        .unwrap()
        .clone(),
        hash,
    );
    env.source_agent = "context-kernel".into();
    env
}

struct Chain {
    identity: VerifiedDomainIdentity,
    work_requested: Envelope,
    packet: Envelope,
}

/// The full upstream chain under one verified model identity, reduced through
/// the real T03 ingress exactly as production would.
fn upstream(tag: &str, work_request_id: &str) -> Chain {
    let dir = tmp_dir(tag);
    let (artifact, hash) = write_model_artifact(&dir);
    let identity = verify_against_artifact(&hash, &artifact).unwrap();
    let work_requested = work_requested_envelope(work_request_id, &hash);
    let packet = context_packet_envelope(work_request_id, &hash);
    Chain {
        identity,
        work_requested,
        packet,
    }
}

fn proof() -> ProofContract {
    ProofContract {
        criterion: "live proof exits 0 with health checks green".into(),
        command: Some("just proof --route route-blue-green".into()),
        proof_type: Some(PROOF_TYPE_LIVE.into()),
    }
}

fn submission<'a>(
    contract: &'a WorkRequestContract,
    proof_contract: &'a ProofContract,
) -> GovernedSubmission<'a> {
    GovernedSubmission {
        contract,
        actor_id: "agent-operator",
        actor_role: Some("R-AA"),
        intent: "rotate deploy to blue-green and verify health gates",
        proof_contract,
        route_id: None,
        artifact_expectations: None,
        authority_context: None,
        payment_budget: None,
    }
}

// --- E4 happy path ------------------------------------------------------------

#[test]
fn e4_complete_governed_submission_carries_the_frozen_contract() {
    let chain = upstream("happy", "wr-t04");
    let contract = accept_work_requested(&chain.work_requested, chain.identity.as_str()).unwrap();
    let proof = proof();
    let request = build_governed_work_request(
        &chain.work_requested,
        &chain.packet,
        submission(&contract, &proof),
        &chain.identity,
    )
    .unwrap();

    // Producer authority + identity + derivation mechanics.
    assert_eq!(request.event_type, "GovernedWorkRequest");
    assert_eq!(request.source_agent, "swe-seed");
    assert_eq!(request.domain_model_hash(), Some(chain.identity.as_str()));
    assert_eq!(
        request.causal_parents(),
        vec![
            chain.work_requested.event_id.as_str(),
            chain.packet.event_id.as_str()
        ]
    );

    // All eight frozen required payload fields.
    for field in [
        "work_request_id",
        "affordance_id",
        "actor",
        "intent",
        "context_packet_ref",
        "domain_model_ref",
        "proof_contract",
        "settlement_criteria",
    ] {
        assert!(
            request.payload.get(field).is_some_and(|v| !v.is_null()),
            "required field {field} missing"
        );
    }
    assert_eq!(request.work_request_id(), Some("wr-t04"));
    assert_eq!(request.payload["affordance_id"], "aff-blue-green-001");
    assert_eq!(request.payload["actor"]["actor_id"], "agent-operator");
    assert_eq!(
        request.payload["context_packet_ref"],
        json!("ctx_01J9T04PACKET0000000000")
    );
    assert_eq!(
        request.payload["domain_model_ref"],
        json!({"namespace": "agentic_capability_loop", "model_hash": chain.identity.as_str()})
    );
    // Distinct facts under distinct keys.
    assert!(request.payload["proof_contract"].is_object());
    assert!(request.payload["settlement_criteria"].is_array());
}

// --- frozen falsifier: opaque command-only requests ---------------------------

#[test]
fn t04_opaque_command_only_submission_cannot_be_built() {
    let chain = upstream("opaque", "wr-t04");
    let contract = accept_work_requested(&chain.work_requested, chain.identity.as_str()).unwrap();
    let empty_proof = ProofContract {
        criterion: " ".into(),
        command: None,
        proof_type: None,
    };

    // No semantic intent: executable detail alone cannot be submitted.
    let proof = proof();
    let mut sub = submission(&contract, &proof);
    sub.intent = "   ";
    let err =
        build_governed_work_request(&chain.work_requested, &chain.packet, sub, &chain.identity)
            .unwrap_err();
    assert!(
        matches!(err, SubmissionError::IncompleteIntent { field } if field == "intent"),
        "{err}"
    );

    // No actor.
    let mut sub = submission(&contract, &proof);
    sub.actor_id = "";
    let err =
        build_governed_work_request(&chain.work_requested, &chain.packet, sub, &chain.identity)
            .unwrap_err();
    assert!(
        matches!(err, SubmissionError::IncompleteIntent { field } if field == "actor.actor_id"),
        "{err}"
    );

    // No proof obligation.
    let sub = submission(&contract, &empty_proof);
    let err =
        build_governed_work_request(&chain.work_requested, &chain.packet, sub, &chain.identity)
            .unwrap_err();
    assert!(
        matches!(err, SubmissionError::IncompleteIntent { field } if field == "proof_contract.criterion"),
        "{err}"
    );
}

// --- frozen falsifier: cross-wired context packets ------------------------------

#[test]
fn t04_cross_wired_context_packet_is_rejected_before_emission() {
    let chain = upstream("crosswire", "wr-t04");
    let contract = accept_work_requested(&chain.work_requested, chain.identity.as_str()).unwrap();
    let foreign = context_packet_envelope("wr-SOMEONE-ELSE", chain.identity.as_str());
    let err = build_governed_work_request(
        &chain.work_requested,
        &foreign,
        submission(&contract, &proof()),
        &chain.identity,
    )
    .unwrap_err();
    assert!(
        matches!(err, SubmissionError::CrossWiredContextPacket { ref expected, ref got }
            if expected == "wr-t04" && got == "wr-SOMEONE-ELSE"),
        "{err}"
    );
}

#[test]
fn t04_contract_envelope_mismatch_is_rejected_before_emission() {
    let chain = upstream("mismatch", "wr-t04");
    let mut forged = accept_work_requested(&chain.work_requested, chain.identity.as_str()).unwrap();
    forged.work_request_id = "wr-OTHER".into();
    let err = build_governed_work_request(
        &chain.work_requested,
        &chain.packet,
        submission(&forged, &proof()),
        &chain.identity,
    )
    .unwrap_err();
    assert!(
        matches!(err, SubmissionError::ContractEnvelopeMismatch { .. }),
        "{err}"
    );
}

// --- frozen falsifier: different domain model -----------------------------------

#[test]
fn t04_different_domain_model_is_rejected_before_emission() {
    let chain = upstream("drift-packet", "wr-t04");
    let contract = accept_work_requested(&chain.work_requested, chain.identity.as_str()).unwrap();

    // A valid-but-different model digest on the context packet.
    let other = format!("{:x}", Sha256::digest(b"a-different-model"));
    let drifted_packet = context_packet_envelope("wr-t04", &other);
    let err = build_governed_work_request(
        &chain.work_requested,
        &drifted_packet,
        submission(&contract, &proof()),
        &chain.identity,
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            SubmissionError::Boundary(ConsumeError::HashDrift { .. })
        ),
        "{err}"
    );

    // The same drift on the WorkRequested parent.
    let drifted_e1 = work_requested_envelope("wr-t04", &other);
    let err = build_governed_work_request(
        &drifted_e1,
        &chain.packet,
        submission(&contract, &proof()),
        &chain.identity,
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            SubmissionError::Boundary(ConsumeError::HashDrift { .. })
        ),
        "{err}"
    );
}

// --- producer stamps / placeholder identities -----------------------------------

#[test]
fn t04_wrong_producer_stamp_on_upstream_envelopes_is_refused() {
    let chain = upstream("stamps", "wr-t04");
    let contract = accept_work_requested(&chain.work_requested, chain.identity.as_str()).unwrap();

    // E1 forged with SWE_SEED's own stamp (self-production attempt).
    let self_stamped_e1 = make_event(
        "WorkRequested",
        json!({}).as_object().unwrap().clone(),
        chain.identity.as_str(),
    );
    let err = build_governed_work_request(
        &self_stamped_e1,
        &chain.packet,
        submission(&contract, &proof()),
        &chain.identity,
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            SubmissionError::Boundary(ConsumeError::ProducerAuthority(_))
        ),
        "{err}"
    );

    // E3 stamped by anyone but context_kernel.
    let mut forged_packet = context_packet_envelope("wr-t04", chain.identity.as_str());
    forged_packet.source_agent = "godspeed-agent".into();
    let err = build_governed_work_request(
        &chain.work_requested,
        &forged_packet,
        submission(&contract, &proof()),
        &chain.identity,
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            SubmissionError::Boundary(ConsumeError::ProducerAuthority(_))
        ),
        "{err}"
    );
}

#[test]
fn t04_fallback_pseudo_identity_cannot_reach_the_boundary() {
    // Type-level refusal: the emission surface takes a VerifiedDomainIdentity,
    // which cannot be constructed from the standalone fallback constant or any
    // placeholder digest — there is no code path that stamps them.
    assert!(verify_declared_hash(&fallback_hash()).is_err());
    assert!(verify_declared_hash(&"0".repeat(64)).is_err());
    let resolved = ResolvedHash {
        hash: fallback_hash(),
        source: HashSource::Fallback,
        warned: true,
    };
    assert!(VerifiedDomainIdentity::from_resolved(&resolved).is_err());
}

// --- distinct obligations --------------------------------------------------------

#[test]
fn t04_proof_contract_and_settlement_criteria_remain_distinct_facts() {
    let chain = upstream("distinct", "wr-t04");
    let contract = accept_work_requested(&chain.work_requested, chain.identity.as_str()).unwrap();
    let proof = proof();
    let request = build_governed_work_request(
        &chain.work_requested,
        &chain.packet,
        submission(&contract, &proof),
        &chain.identity,
    )
    .unwrap();
    let proof_value = &request.payload["proof_contract"];
    let criteria = &request.payload["settlement_criteria"];
    assert_ne!(proof_value, criteria);
    assert_eq!(proof_value["proof_type"], json!(PROOF_TYPE_LIVE));
    assert_eq!(
        criteria,
        &json!(["health check green", "zero-downtime observed"])
    );
}

#[test]
fn t04_optional_governance_fields_pass_through_when_present() {
    let chain = upstream("optional", "wr-t04");
    let contract = accept_work_requested(&chain.work_requested, chain.identity.as_str()).unwrap();
    let bare = build_governed_work_request(
        &chain.work_requested,
        &chain.packet,
        submission(&contract, &proof()),
        &chain.identity,
    )
    .unwrap();
    assert!(bare.payload.get("route_id").is_none());

    let expectations = json!({"artifacts": ["manifest.json"]});
    let budget = json!({"max_cost_usd": 5});
    let authority = json!({"authority_reference": "auth-ref-1"});
    let proof = proof();
    let mut rich = submission(&contract, &proof);
    rich.route_id = Some("route-blue-green");
    rich.artifact_expectations = Some(&expectations);
    rich.payment_budget = Some(&budget);
    rich.authority_context = Some(&authority);
    let request =
        build_governed_work_request(&chain.work_requested, &chain.packet, rich, &chain.identity)
            .unwrap();
    assert_eq!(request.payload["route_id"], json!("route-blue-green"));
    assert_eq!(request.payload["artifact_expectations"], expectations);
    assert_eq!(request.payload["payment_budget"], budget);
    assert_eq!(request.payload["authority_context"], authority);
}

// --- cross-repo golden fixture: this repo's REAL emitter output ------------------

#[test]
fn t04_writes_golden_fixture_for_sea_forge_ingress() {
    let root = std::env::var("SEA_RS_ROOT").unwrap_or_else(|_| {
        format!(
            "{}/projects/sea-rs",
            std::env::var("HOME").unwrap_or_default()
        )
    });
    let fixture = Path::new(&root)
        .join("crates/sea-forge-server/tests/fixtures/t04_governed_work_request.json");
    if !Path::new(&root).join("crates/sea-forge-server").exists() {
        eprintln!(
            "SKIP: sea-rs checkout not found at {} (fixture not regenerated)",
            fixture.display()
        );
        return;
    }

    let chain = upstream("golden-fixture", "wr-gsf-001");
    let contract = accept_work_requested(&chain.work_requested, chain.identity.as_str()).unwrap();
    let proof = proof();
    let request = build_governed_work_request(
        &chain.work_requested,
        &chain.packet,
        submission(&contract, &proof),
        &chain.identity,
    )
    .unwrap();
    assert_eq!(
        context_packet_ref(&chain.packet).as_deref(),
        Some("ctx_01J9T04PACKET0000000000")
    );

    // The fixture carries SEA-Forge's locally-resolvable model digest
    // out-of-band so the consumer gate drift-checks against an independently
    // declared identity rather than trusting the request's own claim.
    let body = json!({
        "domain_model_sha256": chain.identity.as_str(),
        "request": request,
        "context_packet": chain.packet,
    });
    if let Some(parent) = fixture.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(
        &fixture,
        format!("{}\n", serde_json::to_string_pretty(&body).unwrap()),
    )
    .expect("write golden fixture");
    println!("regenerated {}", fixture.display());

    // Round-trip: what SEA-Forge receives decodes as canonical envelopes.
    let decoded: Value = serde_json::from_str(&std::fs::read_to_string(&fixture).unwrap()).unwrap();
    let req: Envelope = serde_json::from_value(decoded["request"].clone()).unwrap();
    assert_eq!(req.event_type, "GovernedWorkRequest");
}
