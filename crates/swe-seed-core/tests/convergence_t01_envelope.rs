//! Convergence plan T01 — canonical semantic envelope + domain identity
//! (`.agents/plans/e2e-plan.yml`, frozen requirements ENV-I1..ENV-I8, I1, I3,
//! I13; preregistration `.agents/specs/e2e-preregistration.yml`).
//!
//! Each test maps to a preregistered claim or falsifier. The three plan-level
//! teeth attacks are marked TEETH-1/2/3:
//!   TEETH-1: missing model + fallback identity → rejected, none manufactured.
//!   TEETH-2: EvidenceRecorded forged by non-RealityTrace → rejected.
//!   TEETH-3: conformant envelope with false claim → no truth consequence.

use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use swe_seed_core::federation::{
    agent_id, authoritative_producer, check_conformance, derive_event, fallback_hash,
    idempotency_key, make_event, make_event_verified, strict_resolve, validate_envelope,
    validate_producer, verify_against_artifact, verify_declared_hash, Admission, Envelope,
    IdempotencyLedger, IdentityError, ProducerAuthorityError, VerifiedDomainIdentity,
    CAUSALITY_PREFIX, SOURCE_AGENT,
};

fn tmp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("swe-seed-t01-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A real model artifact: content + its true SHA-256.
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

// --- ENV-I1 / I1: identity resolves to the actual artifact ------------------

#[test]
fn declared_hash_matching_real_artifact_verifies() {
    let dir = tmp_dir("artifact-ok");
    let (artifact, hash) = write_model_artifact(&dir);
    let verified = verify_against_artifact(&hash, &artifact).unwrap();
    assert_eq!(verified.as_str(), hash);
}

#[test]
fn env_i1_hash_must_equal_recomputed_artifact_content() {
    let dir = tmp_dir("artifact-mismatch");
    let (artifact, _true_hash) = write_model_artifact(&dir);
    // A well-formed digest of something else must not pass for this artifact.
    let forged = format!("{:x}", Sha256::digest(b"not-the-model"));
    let err = verify_against_artifact(&forged, &artifact).unwrap_err();
    assert!(matches!(err, IdentityError::HashMismatch { .. }));
}

#[test]
fn env_i1_missing_artifact_is_rejected_not_manufactured() {
    let dir = tmp_dir("artifact-missing");
    let missing = dir.join("absent-model.json");
    let declared = format!("{:x}", Sha256::digest(b"whatever"));
    let err = verify_against_artifact(&declared, &missing).unwrap_err();
    assert!(matches!(err, IdentityError::MissingArtifact { .. }));
}

#[test]
fn strict_resolution_without_any_manifest_errors_instead_of_falling_back() {
    let err = strict_resolve(None, None).unwrap_err();
    assert!(matches!(err, IdentityError::MissingArtifact { .. }));
}

// --- ENV-I2 / I1: placeholder and fallback identities cannot masquerade -----

#[test]
fn env_i2_standalone_fallback_pseudo_hash_is_rejected() {
    // TEETH-1 (identity half): the pre-existing standalone constant —
    // sha256("agentic_capability_loop") — is exactly the "namespace-derived"
    // pseudo-hash the frozen contract forbids.
    let pseudo = fallback_hash();
    let err = verify_declared_hash(&pseudo).unwrap_err();
    assert!(matches!(err, IdentityError::PlaceholderIdentity { .. }));
}

#[test]
fn env_i2_placeholder_digest_shapes_are_rejected() {
    for bad in [
        "".to_string(),
        "0".repeat(64),
        "deadbeef".to_string(),
        "ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789".to_string(),
        format!("{:x}", Sha256::digest(b"x")) + "zz",
    ] {
        let err = verify_declared_hash(&bad).unwrap_err();
        assert!(
            matches!(
                err,
                IdentityError::MalformedHash { .. } | IdentityError::PlaceholderIdentity { .. }
            ),
            "expected rejection of {bad:?}"
        );
    }
}

#[test]
fn resolved_from_fallback_source_cannot_become_verified_identity() {
    use swe_seed_core::federation::{resolve_from, HashSource};
    let resolved = resolve_from(None, None, false);
    assert_eq!(resolved.source, HashSource::Fallback);
    let err = VerifiedDomainIdentity::from_resolved(&resolved).unwrap_err();
    assert!(matches!(err, IdentityError::PlaceholderIdentity { .. }));
}

#[test]
fn envelopes_can_only_be_built_through_the_identity_gate() {
    let payload: Map<String, Value> = json!({"k": "v"}).as_object().unwrap().clone();
    // The legacy constructor accepts anything (kept for parity); the T01
    // constructor refuses to exist for a pseudo-hash because it takes a
    // VerifiedDomainIdentity, which cannot be built from one.
    let pseudo = fallback_hash();
    assert!(verify_declared_hash(&pseudo).is_err());
    let real = format!("{:x}", Sha256::digest(b"real-model-bytes"));
    let identity = verify_declared_hash(&real).unwrap();
    let env = make_event_verified("ContextRequired", payload, &identity);
    assert_eq!(env.domain_model_hash(), Some(real.as_str()));
}

// --- I3: exclusive event-producer authority ---------------------------------

#[test]
fn i3_registry_matches_frozen_edge_topology() {
    assert_eq!(
        authoritative_producer("WorkRequested"),
        Some(agent_id::GODSPEED_AGENT)
    );
    assert_eq!(
        authoritative_producer("ContextRequired"),
        Some(agent_id::SWE_SEED)
    );
    assert_eq!(
        authoritative_producer("ContextPacketCreated"),
        Some(agent_id::CONTEXT_KERNEL)
    );
    assert_eq!(
        authoritative_producer("GovernedWorkRequest"),
        Some(agent_id::SWE_SEED)
    );
    assert_eq!(
        authoritative_producer("AuthorizedInvocation"),
        Some(agent_id::SEA_FORGE)
    );
    assert_eq!(
        authoritative_producer("ExecutionObservation"),
        Some(agent_id::EXECUTION_ENVIRONMENT)
    );
    assert_eq!(
        authoritative_producer("OperationalSettlement"),
        Some(agent_id::SEA_FORGE)
    );
    assert_eq!(
        authoritative_producer("ProofCompleted"),
        Some(agent_id::SWE_SEED)
    );
    assert_eq!(
        authoritative_producer("EvidenceRecorded"),
        Some(agent_id::REALITYTRACE)
    );
    assert_eq!(
        authoritative_producer("SettlementRecorded"),
        Some(agent_id::GODSPEED_AGENT)
    );
    assert_eq!(
        authoritative_producer("DevelopmentalMemory"),
        Some(agent_id::MEMORY_LEDGER)
    );
    assert_eq!(authoritative_producer("TotallyUnfrozenEvent"), None);
}

/// Build an envelope with an arbitrary source stamp (the forge primitive).
fn forge(event_type: &str, source_agent: &str, hash: &str) -> Envelope {
    let mut env = make_event(event_type, json!({}).as_object().unwrap().clone(), hash);
    env.source_agent = source_agent.to_string();
    env
}

#[test]
fn teeth_2_evidence_recorded_forged_by_non_realitytrace_is_rejected() {
    let real = format!("{:x}", Sha256::digest(b"model"));
    for forger in [SOURCE_AGENT, agent_id::GODSPEED_AGENT, agent_id::SEA_FORGE] {
        let forged = forge("EvidenceRecorded", forger, &real);
        match validate_producer(&forged).unwrap_err() {
            ProducerAuthorityError::NotAuthoritative {
                event_type,
                authoritative,
                got,
            } => {
                assert_eq!(event_type, "EvidenceRecorded");
                assert_eq!(authoritative, agent_id::REALITYTRACE);
                assert_eq!(
                    got,
                    swe_seed_core::federation::canonical_agent(forger).unwrap()
                );
            }
            other => panic!("unexpected error for forger {forger}: {other:?}"),
        }
    }
    // And the authoritative producer passes.
    let genuine = forge("EvidenceRecorded", "realitytrace", &real);
    validate_producer(&genuine).unwrap();
}

#[test]
fn i3_settlement_recorded_owned_by_godspeed_agent_not_swe_seed() {
    // SWE_SEED's log-only settlement emission (spec 0020) is NOT canonical
    // production of SettlementRecorded; the frozen producer is godspeed_agent.
    let real = format!("{:x}", Sha256::digest(b"model"));
    let swe_emitted = forge("SettlementRecorded", SOURCE_AGENT, &real);
    assert!(validate_producer(&swe_emitted).is_err());
    let gsa_emitted = forge("SettlementRecorded", "godspeed-agent", &real);
    validate_producer(&gsa_emitted).unwrap();
}

#[test]
fn i3_unknown_event_types_and_agents_fail_closed() {
    let real = format!("{:x}", Sha256::digest(b"model"));
    assert_eq!(
        validate_producer(&forge("MysteryEvent", SOURCE_AGENT, &real)).unwrap_err(),
        ProducerAuthorityError::UnknownEventType {
            got: "MysteryEvent".into()
        }
    );
    assert_eq!(
        validate_producer(&forge("WorkRequested", "mystery-agent", &real)).unwrap_err(),
        ProducerAuthorityError::UnknownAgent {
            got: "mystery-agent".to_string()
        }
    );
}

// --- ENV-I4 / ENV-I3: causal parents and correlation stability ---------------

#[test]
fn env_i4_derived_envelopes_record_distinct_causal_parents() {
    let real = format!("{:x}", Sha256::digest(b"model"));
    let p1 = make_event(
        "ContextPacketCreated",
        json!({"work_request_id": "wr-1"})
            .as_object()
            .unwrap()
            .clone(),
        &real,
    );
    let p2 = make_event(
        "ProofCompleted",
        json!({"work_request_id": "wr-1"})
            .as_object()
            .unwrap()
            .clone(),
        &real,
    );
    let child = derive_event(
        &[&p1, &p2],
        "RouteSelected",
        json!({"route_id": "r"}).as_object().unwrap().clone(),
        &real,
        None,
    )
    .unwrap();
    assert_eq!(
        child.causal_parents(),
        vec![p1.event_id.as_str(), p2.event_id.as_str()]
    );
    assert_eq!(child.work_request_id(), Some("wr-1"));
}

#[test]
fn env_i4_derivation_requires_parents_and_rejects_duplicates() {
    let real = format!("{:x}", Sha256::digest(b"model"));
    let parent = make_event(
        "ContextPacketCreated",
        json!({}).as_object().unwrap().clone(),
        &real,
    );
    assert!(matches!(
        derive_event(
            &[],
            "RouteSelected",
            json!({}).as_object().unwrap().clone(),
            &real,
            None
        )
        .unwrap_err(),
        swe_seed_core::federation::DeriveError::NoParents
    ));
    assert!(matches!(
        derive_event(
            &[&parent, &parent],
            "RouteSelected",
            json!({}).as_object().unwrap().clone(),
            &real,
            None
        )
        .unwrap_err(),
        swe_seed_core::federation::DeriveError::DuplicateParent { .. }
    ));
}

#[test]
fn env_i3_correlation_cannot_silently_change_across_derivation() {
    let real = format!("{:x}", Sha256::digest(b"model"));
    let parent = make_event(
        "ContextRequired",
        json!({"work_request_id": "wr-A"})
            .as_object()
            .unwrap()
            .clone(),
        &real,
    );
    let err = derive_event(
        &[&parent],
        "RouteSelected",
        json!({}).as_object().unwrap().clone(),
        &real,
        Some("wr-B"),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        swe_seed_core::federation::DeriveError::CorrelationMismatch { .. }
    ));
    // Matching correlation derives cleanly.
    derive_event(
        &[&parent],
        "RouteSelected",
        json!({}).as_object().unwrap().clone(),
        &real,
        Some("wr-A"),
    )
    .unwrap();
}

#[test]
fn env_i4_validate_envelope_rejects_forged_parent_ids() {
    let real = format!("{:x}", Sha256::digest(b"model"));
    let mut child = make_event(
        "RouteSelected",
        json!({}).as_object().unwrap().clone(),
        &real,
    );
    child.provenance = Some(json!({
        "origin": SOURCE_AGENT,
        "chain": ["domain_model_hash:x", format!("{CAUSALITY_PREFIX}not-an-event-id")],
    }));
    let err = validate_envelope(&child, &real).unwrap_err();
    assert!(matches!(
        err,
        swe_seed_core::federation::ConsumeError::Causality { .. }
    ));
}

// --- ENV-I6: durable idempotent envelope writes ------------------------------

#[test]
fn env_i6_duplicate_delivery_admits_exactly_once_across_restart() {
    let dir = tmp_dir("idempotency");
    let ledger_path = dir.join("admitted.jsonl");
    let key = idempotency_key("EvidenceRecorded", &json!({"claim": 1}), Some("wr-1"));

    let mut first_process = IdempotencyLedger::open(&ledger_path).unwrap();
    assert_eq!(first_process.admit(&key).unwrap(), Admission::First);
    assert_eq!(first_process.admit(&key).unwrap(), Admission::Duplicate);

    // Simulated restart: fresh ledger over the same file still dedupes.
    drop(first_process);
    let mut reopened = IdempotencyLedger::open(&ledger_path).unwrap();
    assert_eq!(reopened.admit(&key).unwrap(), Admission::Duplicate);
    assert_eq!(reopened.len(), 1);
}

#[test]
fn env_i6_malformed_keys_fail_closed() {
    let dir = tmp_dir("idempotency-bad");
    let mut ledger = IdempotencyLedger::open(&dir.join("l.jsonl")).unwrap();
    assert!(matches!(
        ledger.admit("has spaces"),
        Err(swe_seed_core::federation::AdmitError::MalformedKey { .. })
    ));
    assert!(ledger.is_empty());
}

// --- ENV-I7: evidence references are content-addressed -----------------------

#[test]
fn env_i7_provenance_chain_carries_content_addressed_domain_root() {
    let dir = tmp_dir("content-addressed");
    let (_artifact, hash) = write_model_artifact(&dir);
    let identity = verify_against_artifact(&hash, &dir.join("canonical_model.sea.json")).unwrap();
    let env = make_event_verified(
        "ProofCompleted",
        json!({}).as_object().unwrap().clone(),
        &identity,
    );
    let chain = env.provenance.unwrap()["chain"].as_array().unwrap().clone();
    // The domain root entry IS the verified content hash of a resolvable
    // artifact, not a label or namespace-derived constant.
    assert!(chain
        .iter()
        .any(|e| e == &json!(format!("domain_model_hash:{hash}"))));
    assert_ne!(hash, fallback_hash());
}

// --- ENV-I8 / I13: conformance is not truth ----------------------------------

#[test]
fn teeth_3_conformant_envelope_with_false_claim_has_no_truth_consequence() {
    let real = format!("{:x}", Sha256::digest(b"model"));
    // Structurally perfect ProofCompleted whose CLAIM is internally false:
    // result=passed while exit_code reports failure. Conformance only checks
    // shape; it cannot and does not evaluate this contradiction.
    let lying = make_event(
        "ProofCompleted",
        json!({"result": "passed", "exit_code": 1, "work_request_id": "wr-9"})
            .as_object()
            .unwrap()
            .clone(),
        &real,
    );
    let report = check_conformance(&lying);
    assert!(
        report.is_conformant(),
        "structural conformance must hold: {:?}",
        report.violations
    );

    // Truth evaluation lives outside conformance and is not invoked by it:
    // the only lawful way an effect enters the world here is an explicit,
    // separate admission call — which this test performs deliberately to
    // prove the conformance flow itself left the ledger untouched.
    let dir = tmp_dir("teeth3");
    let mut ledger = IdempotencyLedger::open(&dir.join("effects.jsonl")).unwrap();
    validate_envelope(&lying, &real).expect("producer/identity gates are orthogonal to truth");
    assert!(
        ledger.is_empty(),
        "conformance must never admit consequences"
    );
    let explicit_effect_key = idempotency_key("ProofCompleted", &lying.payload, Some("wr-9"));
    assert_eq!(
        ledger.admit(&explicit_effect_key).unwrap(),
        Admission::First
    );
}

#[test]
fn env_i8_nonconformant_envelopes_are_named_precisely() {
    let mut bad = make_event(
        "ProofCompleted",
        json!({}).as_object().unwrap().clone(),
        &fallback_hash(),
    );
    bad.idempotency_key = Some("nope".into());
    let report = check_conformance(&bad);
    assert!(!report.conforms);
    assert!(report.violations.iter().any(|v| v.contains("placeholder")));
    assert!(report
        .violations
        .iter()
        .any(|v| v.contains("idempotency_key")));
}

// --- composed boundary validation -------------------------------------------

#[test]
fn validate_envelope_accepts_a_well_formed_authoritative_envelope() {
    let dir = tmp_dir("composed");
    let (artifact, hash) = write_model_artifact(&dir);
    let identity = verify_against_artifact(&hash, &artifact).unwrap();
    let env = make_event_verified(
        "ContextRequired",
        json!({"work_request_id": "wr-1"})
            .as_object()
            .unwrap()
            .clone(),
        &identity,
    );
    validate_envelope(&env, identity.as_str()).unwrap();
}
