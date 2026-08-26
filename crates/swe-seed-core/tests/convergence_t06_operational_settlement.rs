//! Convergence plan T06 — OperationalSettlement adjudication at the SWE_SEED
//! proof-plane boundary (frozen edge E6, invariants I6/I7).
//!
//! Everything here consumes wire-shape canonical envelopes exactly as
//! SEA-Forge returns them. The cross-repo golden fixture written by the REAL
//! sea-rs emitter (see
//! sea-rs/crates/sea-forge-server/tests/convergence_t06_operational_settlement.rs)
//! is adjudicated through the real surface when present.
//!
//! Plan teeth:
//!   TOOTH 2 consuming OperationalSettlement directly as developmental
//!          settlement ⇒ impossible / no promotion occurs

use std::path::PathBuf;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use swe_seed_core::federation::{
    consume_settlement_recorded, derive_event, make_event, validate_producer, Adjudication,
    AdjudicationError, ConsumeError, Envelope, OperationalOutcome,
    OperationalSettlementAdjudicator, OperationalSettlementFacts, ProducerAuthorityError,
};

fn tmp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("swe-seed-t06-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

/// MUST equal the producing-side digest the sea-rs golden fixture records.
fn local_model_sha256() -> String {
    sha256_hex(b"canonical-model-t06-settlement-return")
}

/// The standalone-fallback pseudo-hash (ENV-I2 placeholder).
fn fallback_pseudo_hash() -> String {
    sha256_hex(b"agentic_capability_loop")
}

const WORK_REQUEST_ID: &str = "wr-gsf-001";
const AUTH_DECISION_ID: &str = "dec_01T06AUTHORITY000000";
const E5A_EVENT_ID: &str = "aaaa1111-2222-4333-8444-555555555555";
const E5B_EVENT_ID: &str = "bbbb1111-2222-4333-8444-555555555555";

/// Observed effects attesting every declared criterion.
fn attesting_effects() -> Value {
    json!([
        {"effect": "health check green", "probe": "lb/ready"},
        {"effect": "zero-downtime observed", "window_ms": 0}
    ])
}

/// A canonical OperationalSettlement envelope in the exact wire family the
/// sea-rs emitter projects (v1 shape, sea_forge stamp, content-derived
/// idempotency key, causal chain over the actual invocation).
fn settlement_envelope(
    work_request_id: &str,
    authority_decision_id: &str,
    e5a_event_id: &str,
    e5b_event_id: &str,
    settlement_status: &str,
    observed_effects: Value,
    model_hash: &str,
) -> Value {
    let payload = json!({
        "domain_model_hash": model_hash,
        "namespace": "agentic_capability_loop",
        "work_request_id": work_request_id,
        "authority_decision_id": authority_decision_id,
        "invocation_id": "inv_0123abcdef456789",
        "execution_status": "completed",
        "observed_effects": observed_effects,
        "operational_settlement_status": settlement_status,
        "evidence_refs": [
            format!("sha256:{}", sha256_hex(b"effects-digest")),
            format!("authorized_invocation:{e5a_event_id}"),
            format!("execution_observation:{e5b_event_id}"),
        ],
        "domain_model_ref": {
            "namespace": "agentic_capability_loop",
            "model_hash": model_hash,
        },
    });
    let idempotency =
        swe_seed_core::federation::idempotency_key("OperationalSettlement", &payload, None);
    json!({
        "schema_version": "v1",
        "event_id": uuid::Uuid::new_v4().to_string(),
        "source_agent": "sea-forge",
        "event_type": "OperationalSettlement",
        "occurred_at": "2026-08-25T12:00:00+00:00",
        "idempotency_key": idempotency,
        "payload": payload,
        "provenance": {
            "origin": "sea-forge",
            "chain": [
                format!("domain_model_hash:{model_hash}"),
                format!("caused_by:{e5a_event_id}"),
                format!("caused_by:{e5b_event_id}"),
            ]
        }
    })
}

fn decode(value: &Value) -> Envelope {
    serde_json::from_value(value.clone()).expect("wire envelope decodes")
}

fn valid_accepted_envelope() -> Envelope {
    decode(&settlement_envelope(
        WORK_REQUEST_ID,
        AUTH_DECISION_ID,
        E5A_EVENT_ID,
        E5B_EVENT_ID,
        "accepted",
        attesting_effects(),
        &local_model_sha256(),
    ))
}

fn adjudicator(tag: &str) -> OperationalSettlementAdjudicator {
    OperationalSettlementAdjudicator::open(&tmp_dir(tag).join("adjudications.jsonl"))
        .expect("adjudicator opens")
}

// --- E6 positive path ------------------------------------------------------------

#[test]
fn e6_golden_fixture_from_sea_forge_is_adjudicated() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/t06_operational_settlement.json");
    if !fixture_path.exists() {
        eprintln!(
            "SKIP: sea-rs producer golden fixture absent at {} (run the sea-rs T06 suite to regenerate)",
            fixture_path.display()
        );
        return;
    }
    let fixture: Value =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).unwrap()).unwrap();
    let envelope = decode(&fixture["settlement"]);
    let mut judge = adjudicator("golden-fixture");

    let chain_ids: Vec<String> = fixture["expected_chain"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let chain_refs: Vec<&str> = chain_ids.iter().map(String::as_str).collect();

    let outcome = judge
        .adjudicate(
            &envelope,
            fixture["originating_work_request_id"].as_str().unwrap(),
            &chain_refs,
            fixture["domain_model_sha256"].as_str().unwrap(),
        )
        .expect("REAL sea_forge settlement must adjudicate");
    let Adjudication::First(facts) = outcome else {
        panic!("first delivery must adjudicate as First");
    };
    assert_eq!(facts.operational_outcome, OperationalOutcome::Accepted);
    assert_eq!(
        facts.work_request_id,
        fixture["originating_work_request_id"].as_str().unwrap()
    );
    assert!(facts.evidence_refs.iter().any(|r| r.starts_with("sha256:")));
}

#[test]
fn e6_valid_operational_settlement_adjudicates_to_operational_facts_only() {
    let mut judge = adjudicator("facts-only");
    let outcome = judge
        .adjudicate(
            &valid_accepted_envelope(),
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .expect("valid settlement must adjudicate");
    assert!(outcome.first());

    // Exhaustive destructuring: these fields are ALL the type carries. No
    // proof result, no capability state, no developmental consequence can be
    // read from this surface because none is representable on it (I6/I7).
    let Adjudication::First(OperationalSettlementFacts {
        settlement_event_id,
        work_request_id,
        authority_decision_id,
        invocation_id,
        execution_status,
        operational_outcome,
        observed_effects,
        evidence_refs,
    }) = outcome
    else {
        unreachable!()
    };
    assert_eq!(work_request_id, WORK_REQUEST_ID);
    assert_eq!(authority_decision_id, AUTH_DECISION_ID);
    assert_eq!(invocation_id.as_deref(), Some("inv_0123abcdef456789"));
    assert_eq!(execution_status, "completed");
    assert_eq!(observed_effects, attesting_effects());
    assert_eq!(evidence_refs.len(), 3);
    assert!(!settlement_event_id.is_empty());
    // The two-variant outcome type admits no third, proof-shaped reading.
    match operational_outcome {
        OperationalOutcome::Accepted => {}
        OperationalOutcome::Rejected => {}
    }

    // A rejection reads back exactly as operational failure — nothing more.
    let mut judge2 = adjudicator("facts-rejected");
    let rejected = decode(&settlement_envelope(
        WORK_REQUEST_ID,
        AUTH_DECISION_ID,
        E5A_EVENT_ID,
        E5B_EVENT_ID,
        "rejected",
        json!([]),
        &local_model_sha256(),
    ));
    let Adjudication::First(facts) = judge2
        .adjudicate(
            &rejected,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .unwrap()
    else {
        panic!("first delivery must adjudicate as First");
    };
    assert_eq!(facts.operational_outcome, OperationalOutcome::Rejected);
}

// --- TOOTH 2: no developmental promotion from operational settlement -------------

#[test]
fn t06_tooth2_operational_settlement_cannot_promote_developmental_state() {
    let settlement = valid_accepted_envelope();

    // (a) The producer registry pins SettlementRecorded/CapabilityUpdated to
    // godspeed_agent EXCLUSIVELY: even a perfect copy of the received
    // settlement restamped sea_forge cannot masquerade as either.
    for developmental in ["SettlementRecorded", "CapabilityUpdated"] {
        let mut forged = make_event(
            developmental,
            settlement.payload.as_object().unwrap().clone(),
            local_model_sha256().as_str(),
        );
        forged.source_agent = "sea-forge".into();
        assert_eq!(
            validate_producer(&forged),
            Err(ProducerAuthorityError::NotAuthoritative {
                event_type: developmental.to_string(),
                authoritative: "godspeed_agent",
                got: "sea_forge".to_string(),
            }),
            "{developmental} cannot be produced by sea_forge"
        );

        // (b) Deriving a child FROM the adjudicated settlement parent does
        // not help: whatever the lineage carries, the producer registry
        // refuses any non-godspeed stamp on developmental events — so no
        // path from this surface mints a promotion.
        let derived = derive_event(
            &[&settlement],
            developmental,
            settlement.payload.as_object().unwrap().clone(),
            local_model_sha256().as_str(),
            Some(WORK_REQUEST_ID),
        )
        .expect("lineage mechanics themselves are orthogonal");
        assert!(
            validate_producer(&derived).is_err(),
            "swe_seed-stamped {developmental} derived from the settlement is still a forge"
        );
        let mut restamped = derived;
        restamped.source_agent = "sea-forge".into();
        assert!(
            validate_producer(&restamped).is_err(),
            "sea_forge-stamped {developmental} derived from the settlement is still a forge"
        );
    }
}

#[test]
fn t06_legacy_gsa_settlement_consumer_refuses_an_operational_settlement() {
    let settlement = valid_accepted_envelope();
    // The only existing consume path for SettlementRecorded requires exactly
    // that event type: an OperationalSettlement delivered there is refused,
    // so the settlement cannot ride an older developmental channel.
    assert_eq!(
        consume_settlement_recorded(&settlement),
        Err(ConsumeError::WrongType {
            expected: "SettlementRecorded".to_string(),
            got: "OperationalSettlement".to_string(),
        })
    );
}

#[test]
fn t06_proof_verdicts_cannot_smuggle_through_the_settlement_status_field() {
    for smuggled in [
        "proof_passed",
        "proof_succeeded",
        "capability_granted",
        "promoted",
        "success",
    ] {
        let envelope = decode(&settlement_envelope(
            WORK_REQUEST_ID,
            AUTH_DECISION_ID,
            E5A_EVENT_ID,
            E5B_EVENT_ID,
            smuggled,
            attesting_effects(),
            &local_model_sha256(),
        ));
        let mut judge = adjudicator("status-vocab");
        assert_eq!(
            judge.adjudicate(
                &envelope,
                WORK_REQUEST_ID,
                &[E5A_EVENT_ID, E5B_EVENT_ID],
                &local_model_sha256(),
            ),
            Err(AdjudicationError::InvalidSettlementStatus {
                got: smuggled.to_string(),
            }),
            "proof/capability vocabulary is not operational vocabulary"
        );
    }
}

// --- Duplicate delivery + conflicting re-settlement ------------------------------

#[test]
fn t06_duplicate_delivery_is_consequence_free() {
    let mut judge = adjudicator("duplicate");
    let envelope_value = serde_json::to_value(valid_accepted_envelope()).unwrap();
    let first_envelope = decode(&envelope_value);
    let first = judge
        .adjudicate(
            &first_envelope,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .unwrap();
    assert!(first.first());
    let recorded_len = judge.len();

    // Exact replay under stable identity (same event id).
    let outcome = judge
        .adjudicate(
            &decode(&envelope_value),
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .unwrap();
    assert_eq!(outcome, Adjudication::DuplicateDelivery);
    assert_eq!(judge.len(), recorded_len, "duplicates record nothing");

    // Re-wrap with a FRESH event id but identical content: the
    // content-derived idempotency key recognizes it.
    let rewrapped = decode(&settlement_envelope(
        WORK_REQUEST_ID,
        AUTH_DECISION_ID,
        E5A_EVENT_ID,
        E5B_EVENT_ID,
        "accepted",
        attesting_effects(),
        &local_model_sha256(),
    ));
    let outcome = judge
        .adjudicate(
            &rewrapped,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .unwrap();
    assert_eq!(outcome, Adjudication::DuplicateDelivery);

    // Mutated payload under the SAME event id: consequence-free redelivery —
    // duplicate recognition precedes any payload parsing, so the first
    // settlement stands and the mutation never lands.
    let mut mutated = envelope_value.clone();
    mutated["payload"]["operational_settlement_status"] = json!("rejected");
    mutated["payload"]["observed_effects"] = attesting_effects();
    let outcome = judge
        .adjudicate(
            &decode(&mutated),
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .unwrap();
    assert_eq!(outcome, Adjudication::DuplicateDelivery);
    assert_eq!(judge.len(), recorded_len);
}

#[test]
fn t06_mutated_reattestation_of_the_same_chain_is_refused() {
    let mut judge = adjudicator("conflict");
    // Chain (wr, authority decision) settles REJECTED on weak effects...
    let rejected = decode(&settlement_envelope(
        WORK_REQUEST_ID,
        AUTH_DECISION_ID,
        E5A_EVENT_ID,
        E5B_EVENT_ID,
        "rejected",
        json!([]),
        &local_model_sha256(),
    ));
    assert!(judge
        .adjudicate(
            &rejected,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .unwrap()
        .first());

    // ...a LATER envelope flips the SAME chain to success with freshly
    // attested effects. Different stable identity (so not a duplicate), same
    // invocation chain — refused outright; success was not smuggled in.
    let flipped = decode(&settlement_envelope(
        WORK_REQUEST_ID,
        AUTH_DECISION_ID,
        E5A_EVENT_ID,
        E5B_EVENT_ID,
        "accepted",
        attesting_effects(),
        &local_model_sha256(),
    ));
    assert_eq!(
        judge.adjudicate(
            &flipped,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::ConflictingResettlement {
            work_request_id: WORK_REQUEST_ID.to_string(),
            authority_decision_id: AUTH_DECISION_ID.to_string(),
        })
    );

    // The original fact stands and still replays as duplicate.
    let replay = judge
        .adjudicate(
            &rejected,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .unwrap();
    assert_eq!(replay, Adjudication::DuplicateDelivery);

    // An INDEPENDENT second invocation chain (different authority decision)
    // for the same work request remains admissible — conflicts bind per
    // chain, not per work request.
    let other = decode(&settlement_envelope(
        WORK_REQUEST_ID,
        "dec_01T06SECONDCHAIN000",
        E5A_EVENT_ID,
        E5B_EVENT_ID,
        "accepted",
        attesting_effects(),
        &local_model_sha256(),
    ));
    assert!(judge
        .adjudicate(
            &other,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .unwrap()
        .first());
}

#[test]
fn t06_duplicate_and_conflict_detection_survives_restart() {
    let dir = tmp_dir("restart");
    let ledger_path = dir.join("adjudications.jsonl");
    let mut judge = OperationalSettlementAdjudicator::open(&ledger_path).unwrap();
    let envelope = valid_accepted_envelope();
    assert!(judge
        .adjudicate(
            &envelope,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .unwrap()
        .first());

    // Drop everything in memory and reopen from disk.
    drop(judge);
    let mut reopened = OperationalSettlementAdjudicator::open(&ledger_path).unwrap();
    assert!(!reopened.is_empty(), "prior identities reload from disk");

    let replay = reopened
        .adjudicate(
            &envelope,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .unwrap();
    assert_eq!(replay, Adjudication::DuplicateDelivery);

    let flipped = decode(&settlement_envelope(
        WORK_REQUEST_ID,
        AUTH_DECISION_ID,
        E5A_EVENT_ID,
        E5B_EVENT_ID,
        "accepted",
        json!([{"effect": "health check green"}, {"effect": "zero-downtime observed"}, {"effect": "smuggled"}]),
        &local_model_sha256(),
    ));
    assert!(matches!(
        reopened.adjudicate(
            &flipped,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::ConflictingResettlement { .. })
    ));
}

// --- Binding refusals ------------------------------------------------------------

#[test]
fn t06_wrong_work_request_binding_is_refused_and_records_nothing() {
    let mut judge = adjudicator("cross-wired");
    let envelope = valid_accepted_envelope();
    assert_eq!(
        judge.adjudicate(
            &envelope,
            "wr-some-other-request",
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::CrossWiredWorkRequest {
            expected: "wr-some-other-request".to_string(),
            got: WORK_REQUEST_ID.to_string(),
        })
    );
    assert!(judge.is_empty(), "refusals record nothing durably");
}

#[test]
fn t06_causality_binding_to_the_known_invocation_chain_is_mandatory() {
    let envelope = valid_accepted_envelope();

    let mut judge = adjudicator("chain-empty");
    assert_eq!(
        judge.adjudicate(&envelope, WORK_REQUEST_ID, &[], &local_model_sha256()),
        Err(AdjudicationError::EmptyChainBinding)
    );

    // A known-chain id that is NOT among the recorded parents leaves the
    // settlement unbound to this execution.
    let stranger = "cccc9999-9999-4999-8999-999999999999";
    let mut judge = adjudicator("chain-stranger");
    let err = judge
        .adjudicate(
            &envelope,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, stranger],
            &local_model_sha256(),
        )
        .unwrap_err();
    assert!(
        matches!(err, AdjudicationError::CausalityMissing { ref expected, .. }
            if expected.contains(&stranger.to_string())),
        "unknown chain id must be named as unbound: {err}"
    );
    assert!(judge.is_empty(), "refusals record nothing durably");

    // Both known ids bind.
    let mut judge = adjudicator("chain-full");
    assert!(judge
        .adjudicate(
            &envelope,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .unwrap()
        .first());
}

#[test]
fn t06_producer_forgeries_are_refused_both_directions() {
    let mut base = serde_json::to_value(valid_accepted_envelope()).unwrap();

    // SWE_SEED forging its own executor's settlement.
    base["source_agent"] = json!("swe-seed");
    let mut judge = adjudicator("forge-swe-seed");
    assert!(matches!(
        judge.adjudicate(
            &decode(&base),
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::Boundary(
            ConsumeError::ProducerAuthority(ProducerAuthorityError::NotAuthoritative { .. })
        ))
    ));

    // The execution environment (E5B's producer) is not E6's producer either.
    let mut base = serde_json::to_value(valid_accepted_envelope()).unwrap();
    base["source_agent"] = json!("execution-environment");
    let mut judge = adjudicator("forge-executor");
    assert!(matches!(
        judge.adjudicate(
            &decode(&base),
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::Boundary(
            ConsumeError::ProducerAuthority(ProducerAuthorityError::NotAuthoritative { .. })
        ))
    ));

    // Unknown stamps fail closed.
    let mut base = serde_json::to_value(valid_accepted_envelope()).unwrap();
    base["source_agent"] = json!("mystery-component");
    let mut judge = adjudicator("forge-mystery");
    assert!(matches!(
        judge.adjudicate(
            &decode(&base),
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::Boundary(
            ConsumeError::ProducerAuthority(ProducerAuthorityError::UnknownAgent { .. })
        ))
    ));
}

// --- Field battery ---------------------------------------------------------------

#[test]
fn t06_missing_required_fields_are_refused_with_the_complete_list() {
    for omitted in [
        "work_request_id",
        "authority_decision_id",
        "execution_status",
        "observed_effects",
        "operational_settlement_status",
        "evidence_refs",
        "domain_model_ref",
    ] {
        let mut envelope = serde_json::to_value(valid_accepted_envelope()).unwrap();
        envelope["payload"]
            .as_object_mut()
            .unwrap()
            .remove(omitted)
            .unwrap_or_else(|| panic!("{omitted} existed"));
        let mut judge = adjudicator("missing-fields");
        assert_eq!(
            judge.adjudicate(
                &decode(&envelope),
                WORK_REQUEST_ID,
                &[E5A_EVENT_ID, E5B_EVENT_ID],
                &local_model_sha256(),
            ),
            Err(AdjudicationError::OpaquePayload {
                missing: vec![omitted],
            }),
            "omitting {omitted} must be refused"
        );
    }

    // Wrong vocabulary on either governed status field is refused.
    for (field, value) in [
        ("execution_status", json!("exit_zero")),
        ("execution_status", json!("ok")),
    ] {
        let mut envelope = serde_json::to_value(valid_accepted_envelope()).unwrap();
        envelope["payload"][field] = value;
        let mut judge = adjudicator("bad-status");
        assert!(matches!(
            judge.adjudicate(
                &decode(&envelope),
                WORK_REQUEST_ID,
                &[E5A_EVENT_ID, E5B_EVENT_ID],
                &local_model_sha256(),
            ),
            Err(AdjudicationError::InvalidExecutionStatus { .. })
        ));
    }

    // Shape violations on typed fields.
    for (field, value) in [
        ("observed_effects", json!("attested verbally")),
        ("evidence_refs", json!([])),
        ("evidence_refs", json!(["not-a-ref"])),
        ("evidence_refs", json!(["sha256:zz"])),
        ("case_id", json!(42)),
        ("artifact_refs", json!({"a": 1})),
        ("transcript_ref", json!([])),
        ("failure_reason", json!(42)),
    ] {
        let mut envelope = serde_json::to_value(valid_accepted_envelope()).unwrap();
        envelope["payload"][field] = value;
        let mut judge = adjudicator("bad-shape");
        assert!(
            judge
                .adjudicate(
                    &decode(&envelope),
                    WORK_REQUEST_ID,
                    &[E5A_EVENT_ID, E5B_EVENT_ID],
                    &local_model_sha256(),
                )
                .is_err(),
            "{field} shape violation must be refused"
        );
    }
}

#[test]
fn t06_namespace_schema_and_identity_tampering_is_refused() {
    // Namespace swap (payload level).
    let mut tampered = serde_json::to_value(valid_accepted_envelope()).unwrap();
    tampered["payload"]["namespace"] = json!("some_other_namespace");
    let mut judge = adjudicator("ns-payload");
    assert!(matches!(
        judge.adjudicate(
            &decode(&tampered),
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::WrongNamespace { .. })
    ));

    // Namespace swap inside domain_model_ref.
    let mut tampered = serde_json::to_value(valid_accepted_envelope()).unwrap();
    tampered["payload"]["domain_model_ref"]["namespace"] = json!("other");
    let mut judge = adjudicator("ns-ref");
    assert!(matches!(
        judge.adjudicate(
            &decode(&tampered),
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::WrongNamespace { .. })
    ));

    // Schema-version tampering.
    let mut tampered = serde_json::to_value(valid_accepted_envelope()).unwrap();
    tampered["schema_version"] = json!("v2");
    let mut judge = adjudicator("schema");
    assert!(matches!(
        judge.adjudicate(
            &decode(&tampered),
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::MalformedEnvelope(_))
    ));

    // domain_model_ref rewriting the declared identity (I10).
    let mut tampered = serde_json::to_value(valid_accepted_envelope()).unwrap();
    tampered["payload"]["domain_model_ref"]["model_hash"] = json!(sha256_hex(b"another-model"));
    let mut judge = adjudicator("ref-drift");
    assert!(matches!(
        judge.adjudicate(
            &decode(&tampered),
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::RefIdentityMismatch { .. })
    ));

    // Placeholder pseudo-identities fail closed at the T01 gate.
    for placeholder in [fallback_pseudo_hash(), "0".repeat(64)] {
        let tampered = settlement_envelope(
            WORK_REQUEST_ID,
            AUTH_DECISION_ID,
            E5A_EVENT_ID,
            E5B_EVENT_ID,
            "accepted",
            attesting_effects(),
            &placeholder,
        );
        let mut judge = adjudicator("pseudo-hash");
        assert!(matches!(
            judge.adjudicate(
                &decode(&tampered),
                WORK_REQUEST_ID,
                &[E5A_EVENT_ID, E5B_EVENT_ID],
                &local_model_sha256(),
            ),
            Err(AdjudicationError::Boundary(ConsumeError::Identity(
                swe_seed_core::federation::IdentityError::PlaceholderIdentity { .. }
            )))
        ));
    }

    // Drift against OUR local resolution.
    let drifted = settlement_envelope(
        WORK_REQUEST_ID,
        AUTH_DECISION_ID,
        E5A_EVENT_ID,
        E5B_EVENT_ID,
        "accepted",
        attesting_effects(),
        &sha256_hex(b"a-model-swe-seed-does-not-resolve"),
    );
    let mut judge = adjudicator("drift");
    assert!(matches!(
        judge.adjudicate(
            &decode(&drifted),
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::Boundary(ConsumeError::HashDrift { .. }))
    ));

    // Blank/placeholder correlation and authority identities.
    for (field, value) in [
        ("work_request_id", ""),
        ("work_request_id", "placeholder"),
        ("authority_decision_id", "<missing>"),
        ("authority_decision_id", "   "),
    ] {
        let mut tampered = serde_json::to_value(valid_accepted_envelope()).unwrap();
        tampered["payload"][field] = json!(value);
        let mut judge = adjudicator("blank-identity");
        assert!(
            matches!(
                judge.adjudicate(
                    &decode(&tampered),
                    WORK_REQUEST_ID,
                    &[E5A_EVENT_ID, E5B_EVENT_ID],
                    &local_model_sha256(),
                ),
                Err(AdjudicationError::PlaceholderField { .. })
                    | Err(AdjudicationError::OpaquePayload { .. })
            ),
            "{field}={value:?} must be refused"
        );
    }
}
