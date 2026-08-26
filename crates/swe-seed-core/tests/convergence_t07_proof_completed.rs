//! Convergence plan T07 — canonical ProofCompleted emission at the frozen E7
//! boundary (swe_seed -> realitytrace).
//!
//! Everything here exercises the REAL surfaces: settlements are adjudicated
//! through the REAL T06 `OperationalSettlementAdjudicator` before any proof
//! completion exists (there is no hand-built `OperationalSettlementFacts`
//! anywhere in this file), and emission goes through
//! [`emit_proof_completed_verified`] only. When the sxr checkout is present,
//! the REAL emitter regenerates the cross-repo golden fixture consumed by
//! `sxr-core/tests/convergence_t07_proof_ingestion.rs` (the T03/T04/T06
//! pattern with the direction reversed).
//!
//! Plan teeth exercised here (emission basis):
//!   TOOTH 1 basis half: no emission exists unless it binds to a REAL,
//!          adjudicated OperationalSettlement for the SAME work_request_id;
//!   TOOTH 2 basis half: the settlement reference is CONTENT-ADDRESSED, so
//!          altered upstream bytes cannot masquerade as the cited fact.

use std::path::PathBuf;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use swe_seed_core::federation::{
    check_conformance, emit_proof_completed_verified, idempotency_key, make_event,
    operational_settlement_ref, validate_producer, Adjudication, AdjudicationError, EmissionError,
    Envelope, HashSource, IdentityError, OperationalOutcome, ProducerAuthorityError,
    ProofCompletion, ProofContract, ResolvedHash, VerifiedDomainIdentity,
};
use swe_seed_core::federation::{
    OperationalSettlementAdjudicator, OperationalSettlementFacts, PROOF_STATUSES,
};

fn tmp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("swe-seed-t07-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

/// MUST equal the consuming-side digest the sxr golden fixture records.
fn local_model_sha256() -> String {
    sha256_hex(b"canonical-model-t07-proof-completion")
}

/// The standalone-fallback pseudo-hash (the ENV-I2 placeholder digest).
fn fallback_pseudo_hash() -> String {
    sha256_hex(b"agentic_capability_loop")
}

const WORK_REQUEST_ID: &str = "wr-gsf-001";
const AUTH_DECISION_ID: &str = "dec_01T07AUTHORITY000000";
const E5A_EVENT_ID: &str = "aaaa7777-2222-4333-8444-555555555555";
const E5B_EVENT_ID: &str = "bbbb7777-2222-4333-8444-555555555555";

fn attesting_effects() -> Value {
    json!([
        {"effect": "health check green", "probe": "lb/ready"},
        {"effect": "zero-downtime observed", "window_ms": 0}
    ])
}

/// A canonical OperationalSettlement envelope in the exact v1 wire family the
/// sea-rs emitter projects (the same shape the T06 suite consumes).
fn settlement_envelope(
    work_request_id: &str,
    settlement_status: &str,
    observed_effects: Value,
    model_hash: &str,
) -> Value {
    let payload = json!({
        "domain_model_hash": model_hash,
        "namespace": "agentic_capability_loop",
        "work_request_id": work_request_id,
        "authority_decision_id": AUTH_DECISION_ID,
        "invocation_id": "inv_0123abcdef456789",
        "execution_status": "completed",
        "observed_effects": observed_effects,
        "operational_settlement_status": settlement_status,
        "evidence_refs": [
            format!("sha256:{}", sha256_hex(b"effects-digest-t07")),
            format!("authorized_invocation:{E5A_EVENT_ID}"),
            format!("execution_observation:{E5B_EVENT_ID}"),
        ],
        "domain_model_ref": {
            "namespace": "agentic_capability_loop",
            "model_hash": model_hash,
        },
    });
    let idempotency = idempotency_key("OperationalSettlement", &payload, None);
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
                format!("caused_by:{E5A_EVENT_ID}"),
                format!("caused_by:{E5B_EVENT_ID}"),
            ]
        }
    })
}

fn decode(value: &Value) -> Envelope {
    serde_json::from_value(value.clone()).expect("wire envelope decodes")
}

/// Adjudicate a settlement envelope through the REAL T06 surface and return
/// (envelope, facts). This is the ONLY source of facts in this file.
fn real_adjudicated_settlement(
    tag: &str,
    work_request_id: &str,
    settlement_status: &str,
    observed_effects: Value,
) -> (Envelope, OperationalSettlementFacts) {
    let mut judge =
        OperationalSettlementAdjudicator::open(&tmp_dir(tag).join("adjudications.jsonl"))
            .expect("adjudicator opens");
    let wire = settlement_envelope(
        work_request_id,
        settlement_status,
        observed_effects,
        &local_model_sha256(),
    );
    let envelope = decode(&wire);
    let outcome = judge
        .adjudicate(
            &envelope,
            work_request_id,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        )
        .expect("REAL sea_forge settlement must adjudicate");
    match outcome {
        Adjudication::First(facts) => (envelope, facts),
        Adjudication::DuplicateDelivery => panic!("fresh adjudicator must yield First"),
    }
}

fn verified_identity() -> VerifiedDomainIdentity {
    VerifiedDomainIdentity::from_resolved(&ResolvedHash {
        hash: local_model_sha256(),
        source: HashSource::SeaRoot,
        warned: false,
    })
    .expect("canonical identity verifies")
}

#[derive(Clone)]
struct CompletionInputs {
    proof_result_id: String,
    criterion: String,
    command: Option<String>,
    proof_type: Option<String>,
    proof_status: String,
    expected_outcome: Value,
}

impl Default for CompletionInputs {
    fn default() -> Self {
        Self {
            proof_result_id: "pr-t07-0001".to_string(),
            criterion: "convergence_t07 suites exit zero in both repos".to_string(),
            command: Some("just e2e-gate T07".to_string()),
            proof_type: Some("live".to_string()),
            proof_status: "passed".to_string(),
            expected_outcome: json!({
                "affordance": "convergence-e2e-gate",
                "declared_result": "all frozen contract tests exit zero"
            }),
        }
    }
}

fn contract(inputs: &CompletionInputs) -> ProofContract {
    ProofContract {
        criterion: inputs.criterion.clone(),
        command: inputs.command.clone(),
        proof_type: inputs.proof_type.clone(),
    }
}

fn complete<'a>(
    settlement: &'a Envelope,
    facts: &'a OperationalSettlementFacts,
    additional_parents: &[&Envelope],
    inputs: &'a CompletionInputs,
    contract: &'a ProofContract,
) -> Result<Envelope, EmissionError> {
    emit_proof_completed_verified(
        settlement,
        facts,
        additional_parents,
        ProofCompletion {
            originating_work_request_id: WORK_REQUEST_ID,
            proof_result_id: &inputs.proof_result_id,
            proof_contract: contract,
            proof_status: &inputs.proof_status,
            expected_outcome: &inputs.expected_outcome,
            artifact_refs: Some(&json!([format!(
                "sha256:{}",
                sha256_hex(b"t07-artifact-bytes")
            )])),
            trace_root: Some("trace://t07/golden-chain-root"),
            output_ref: Some("output://t07/proof-run.json"),
            proof_evidence_refs: Some(&json!([format!(
                "sha256:{}",
                sha256_hex(b"t07-proof-output")
            )])),
        },
        &verified_identity(),
    )
}

/// An E4 GovernedWorkRequest stand-in built through the REAL canonical
/// builder (`make_event`), which stamps the authoritative swe_seed producer.
fn governed_request_envelope(work_request_id: &str, model_hash: &str) -> Envelope {
    let payload: serde_json::Map<String, Value> =
        serde_json::from_value(json!({ "work_request_id": work_request_id })).unwrap();
    make_event("GovernedWorkRequest", payload, model_hash)
}

/// Emit through the full real chain: adjudicated settlement + E4 parent.
fn emit_from_real_chain(tag: &str, inputs: &CompletionInputs) -> Envelope {
    let (settlement, facts) =
        real_adjudicated_settlement(tag, WORK_REQUEST_ID, "accepted", attesting_effects());
    let e4_parent = governed_request_envelope(WORK_REQUEST_ID, &local_model_sha256());
    let c = contract(inputs);
    complete(&settlement, &facts, &[&e4_parent], inputs, &c).expect("REAL chain must emit")
}

const _: () = assert!(PROOF_STATUSES.len() == 2);

// --- E7 positive path ---------------------------------------------------------

#[test]
fn e7_golden_fixture_from_real_emitter_round_trips_and_carries_the_frozen_payload() {
    // One real chain drives both the field battery and the golden fixture.
    let (settlement, facts) =
        real_adjudicated_settlement("golden", WORK_REQUEST_ID, "accepted", attesting_effects());
    let e4_parent = governed_request_envelope(WORK_REQUEST_ID, &local_model_sha256());
    let inputs = CompletionInputs::default();
    let c = contract(&inputs);
    let envelope =
        complete(&settlement, &facts, &[&e4_parent], &inputs, &c).expect("REAL chain must emit");

    // All 7 frozen required fields are present AND individually validated.
    for field in [
        "work_request_id",
        "proof_result_id",
        "proof_contract",
        "proof_status",
        "expected_outcome",
        "operational_settlement_ref",
        "domain_model_ref",
    ] {
        let present = match envelope.payload.get(field) {
            None | Some(Value::Null) => false,
            Some(Value::String(s)) => !s.trim().is_empty(),
            Some(_) => true,
        };
        assert!(present, "{field} must be present on the wire");
    }
    assert_eq!(envelope.payload["work_request_id"], WORK_REQUEST_ID);
    assert_eq!(envelope.payload["proof_status"], "passed");
    assert_eq!(
        envelope.payload["domain_model_ref"]["model_hash"],
        local_model_sha256()
    );

    // Exclusive-producer stamp (I3) + full v1-family conformance.
    validate_producer(&envelope).expect("swe_seed is the exclusive ProofCompleted producer");
    let report = check_conformance(&envelope);
    assert!(
        report.is_conformant(),
        "canonical emitter output must conform: {:?}",
        report.violations
    );

    // Causality names the REAL adjudicated settlement first (ENV-I4), and the
    // content-addressed reference resolves against its exact canonical bytes.
    let parents = envelope.causal_parents();
    assert_eq!(parents.len(), 2, "settlement + governed request");
    assert_eq!(
        parents[0], facts.settlement_event_id,
        "causal parent MUST be the real adjudicated settlement"
    );
    assert_eq!(
        envelope.payload["operational_settlement_ref"],
        operational_settlement_ref(&settlement),
        "reference binds to the exact settlement bytes"
    );

    // Cross-repo golden fixture: regenerate into the sxr checkout when
    // present so the REAL ingestion gate consumes REAL emitter output.
    let sxr_root = std::env::var("SXR_ROOT")
        .unwrap_or_else(|_| format!("{}/projects/sxr", std::env::var("HOME").unwrap_or_default()));
    let fixture_path =
        PathBuf::from(&sxr_root).join("sxr-core/tests/fixtures/t07_proof_completed.json");
    if !PathBuf::from(&sxr_root).join("sxr-core").exists() {
        eprintln!(
            "SKIP: sxr checkout not found at {} (fixture not regenerated)",
            fixture_path.display()
        );
        return;
    }

    let body = json!({
        "domain_model_sha256": local_model_sha256(),
        "work_request_id": WORK_REQUEST_ID,
        "expected_settlement_event_id": settlement.event_id,
        "referenced_settlement": serde_json::to_value(&settlement).unwrap(),
        "proof_completed": serde_json::to_value(&envelope).unwrap(),
    });
    if let Some(parent) = fixture_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(
        &fixture_path,
        format!("{}\n", serde_json::to_string_pretty(&body).unwrap()),
    )
    .expect("write golden fixture");
    println!("regenerated {}", fixture_path.display());

    // Round-trip: what sxr receives decodes as the same JSON document.
    let decoded: Value =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).unwrap()).unwrap();
    assert_eq!(decoded["proof_completed"]["event_type"], "ProofCompleted");
    assert_eq!(
        decoded["proof_completed"]["payload"]["operational_settlement_ref"],
        envelope.payload["operational_settlement_ref"]
    );
    assert_eq!(
        decoded["referenced_settlement"]["event_id"],
        settlement.event_id
    );
}

#[test]
fn e7_expected_outcome_is_carried_verbatim() {
    let envelope = emit_from_real_chain("verbatim", &Default::default());
    assert_eq!(
        envelope.payload["expected_outcome"],
        CompletionInputs::default().expected_outcome,
        "the declared expectation travels unchanged (no normalization)"
    );
}

#[test]
fn e7_failed_proof_status_is_representable_bogus_vocabulary_is_not() {
    let failed = CompletionInputs {
        proof_status: "failed".to_string(),
        ..CompletionInputs::default()
    };
    let envelope = emit_from_real_chain("failed-proof", &failed);
    assert_eq!(envelope.payload["proof_status"], "failed");

    let (settlement, facts) = real_adjudicated_settlement(
        "bad-status",
        WORK_REQUEST_ID,
        "accepted",
        attesting_effects(),
    );
    let inputs = CompletionInputs {
        proof_status: "success".to_string(),
        ..Default::default()
    };
    let c = contract(&inputs);
    let err = complete(&settlement, &facts, &[], &inputs, &c).unwrap_err();
    assert!(matches!(err, EmissionError::InvalidProofStatus { got } if got == "success"));
}

// --- TOOTH 1 basis: only a REAL adjudicated settlement backs an emission ------

#[test]
fn e7_facts_paired_with_a_foreign_settlement_envelope_are_refused() {
    let (real_settlement, _) = real_adjudicated_settlement(
        "pair-real",
        WORK_REQUEST_ID,
        "accepted",
        attesting_effects(),
    );
    let (_, stranger_facts) =
        real_adjudicated_settlement("pair-stranger", WORK_REQUEST_ID, "rejected", json!([]));
    assert_ne!(real_settlement.event_id, stranger_facts.settlement_event_id);

    let inputs = CompletionInputs::default();
    let c = contract(&inputs);
    let err = complete(&real_settlement, &stranger_facts, &[], &inputs, &c).unwrap_err();
    assert!(
        matches!(err, EmissionError::SettlementNotAdjudicated(_)),
        "facts must name THIS envelope: {err}"
    );
}

#[test]
fn e7_a_restamped_or_drifted_settlement_envelope_is_refused_before_projection() {
    let (settlement, facts) =
        real_adjudicated_settlement("restamp", WORK_REQUEST_ID, "accepted", attesting_effects());

    // SWE_SEED forging its own executor's settlement envelope.
    let mut forged_wire = serde_json::to_value(&settlement).unwrap();
    forged_wire["source_agent"] = json!("swe-seed");
    let forged = decode(&forged_wire);
    let inputs = CompletionInputs::default();
    let c = contract(&inputs);
    let err = complete(&forged, &facts, &[], &inputs, &c).unwrap_err();
    assert!(matches!(err, EmissionError::SettlementNotAdjudicated(_)));

    // A drifted model identity cannot be adjudicated at all (T06 gate), so no
    // facts exist for it — and the emitter independently re-validates.
    let mut judge =
        OperationalSettlementAdjudicator::open(&tmp_dir("drift").join("l.jsonl")).unwrap();
    let drifted = decode(&settlement_envelope(
        WORK_REQUEST_ID,
        "accepted",
        attesting_effects(),
        &sha256_hex(b"a-model-swe-seed-does-not-resolve"),
    ));
    assert!(matches!(
        judge.adjudicate(
            &drifted,
            WORK_REQUEST_ID,
            &[E5A_EVENT_ID, E5B_EVENT_ID],
            &local_model_sha256(),
        ),
        Err(AdjudicationError::Boundary(_))
    ));
    let err = complete(&drifted, &facts, &[], &inputs, &c).unwrap_err();
    assert!(matches!(err, EmissionError::SettlementNotAdjudicated(_)));
}

#[test]
fn e7_foreign_cycle_settlement_never_backs_this_cycles_proof() {
    // A genuinely adjudicated settlement — but for a DIFFERENT cycle.
    let (_other, other_facts) = real_adjudicated_settlement(
        "foreign-cycle",
        "wr-some-other-cycle",
        "accepted",
        attesting_effects(),
    );
    let (this_cycle, _) = real_adjudicated_settlement(
        "foreign-cycle-envelope",
        WORK_REQUEST_ID,
        "accepted",
        attesting_effects(),
    );

    let inputs = CompletionInputs::default();
    let c = contract(&inputs);

    // Facts from another cycle presented against this cycle's envelope.
    let err = complete(&this_cycle, &other_facts, &[], &inputs, &c).unwrap_err();
    assert!(matches!(err, EmissionError::SettlementNotAdjudicated(_)));
}

#[test]
fn e7_cross_wired_correlation_between_facts_and_request_is_refused() {
    let (settlement, facts) = real_adjudicated_settlement(
        "cross-wired",
        WORK_REQUEST_ID,
        "accepted",
        attesting_effects(),
    );
    assert_eq!(facts.work_request_id, WORK_REQUEST_ID);

    // The emitter binds to THIS work request; a caller asking it to attest a
    // different correlation than the settlement carries is refused.
    let inputs = CompletionInputs::default();
    let c = contract(&inputs);
    // Request-side cross-wire: originating id differs from the settlement's.
    let result = emit_proof_completed_verified(
        &settlement,
        &facts,
        &[],
        ProofCompletion {
            originating_work_request_id: "wr-not-mine",
            proof_result_id: &inputs.proof_result_id,
            proof_contract: &c,
            proof_status: &inputs.proof_status,
            expected_outcome: &inputs.expected_outcome,
            artifact_refs: None,
            trace_root: None,
            output_ref: None,
            proof_evidence_refs: None,
        },
        &verified_identity(),
    );
    assert!(matches!(
        result,
        Err(EmissionError::CrossWiredWorkRequest { .. })
    ));
}

// --- Required-field / vocabulary / optional-shape refusals ----------------------

#[test]
fn e7_incomplete_or_placeholder_proof_inputs_are_refused() {
    let (settlement, facts) = real_adjudicated_settlement(
        "incomplete",
        WORK_REQUEST_ID,
        "accepted",
        attesting_effects(),
    );

    let base = CompletionInputs::default();
    let cases: Vec<(&str, CompletionInputs)> = vec![
        (
            "blank proof_result_id",
            CompletionInputs {
                proof_result_id: String::new(),
                ..base.clone()
            },
        ),
        (
            "placeholder proof_result_id",
            CompletionInputs {
                proof_result_id: "<missing>".to_string(),
                ..base.clone()
            },
        ),
        (
            "empty criterion",
            CompletionInputs {
                criterion: String::new(),
                ..base.clone()
            },
        ),
        (
            "placeholder criterion",
            CompletionInputs {
                criterion: "tbd".to_string(),
                ..base.clone()
            },
        ),
    ];
    for (label, inputs) in cases {
        let c = contract(&inputs);
        let err = complete(&settlement, &facts, &[], &inputs, &c).expect_err(label);
        assert!(
            matches!(err, EmissionError::IncompleteProof { .. }),
            "{label}: {err}"
        );
    }

    // Null expected_outcome carries no declaration — refused.
    let inputs = CompletionInputs {
        expected_outcome: Value::Null,
        ..base.clone()
    };
    let c = contract(&inputs);
    assert!(matches!(
        complete(&settlement, &facts, &[], &inputs, &c),
        Err(EmissionError::IncompleteProof { .. })
    ));
}

#[test]
fn e7_optional_fields_are_typed_or_refused() {
    let (settlement, facts) =
        real_adjudicated_settlement("optional", WORK_REQUEST_ID, "accepted", attesting_effects());
    let base = CompletionInputs::default();

    let emit_with = |artifact_refs: Option<&Value>,
                     trace_root: Option<&str>,
                     output_ref: Option<&str>,
                     evidence: Option<&Value>| {
        let inputs = base.clone();
        let c = contract(&inputs);
        emit_proof_completed_verified(
            &settlement,
            &facts,
            &[],
            ProofCompletion {
                originating_work_request_id: WORK_REQUEST_ID,
                proof_result_id: &inputs.proof_result_id,
                proof_contract: &c,
                proof_status: &inputs.proof_status,
                expected_outcome: &inputs.expected_outcome,
                artifact_refs,
                trace_root,
                output_ref,
                proof_evidence_refs: evidence,
            },
            &verified_identity(),
        )
    };

    // Well-formed optionals project verbatim.
    let ok = emit_with(
        Some(&json!([
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ])),
        Some("trace://x"),
        Some("output://y"),
        Some(&json!([
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        ])),
    )
    .expect("well-formed optionals accepted");
    assert_eq!(
        ok.payload["artifact_refs"][0],
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );

    for (label, artifact_refs, trace_root, output_ref, evidence) in [
        (
            "artifact_refs not array",
            Some(&json!("sha256:x")),
            None,
            None,
            None,
        ),
        (
            "artifact_refs non-string item",
            Some(&json!([42])),
            None,
            None,
            None,
        ),
        (
            "artifact_refs malformed ref",
            Some(&json!(["not-a-ref"])),
            None,
            None,
            None,
        ),
        (
            "artifact_refs bad digest",
            Some(&json!(["sha256:zz"])),
            None,
            None,
            None,
        ),
        (
            "trace_root whitespace",
            None,
            Some("trace // x"),
            None,
            None,
        ),
        ("output_ref blank", None, None, Some("   "), None),
        (
            "evidence_refs not array",
            None,
            None,
            None,
            Some(&json!({"a": 1})),
        ),
        (
            "evidence_refs malformed ref",
            None,
            None,
            None,
            Some(&json!(["no scheme here"])),
        ),
    ] {
        let err = emit_with(artifact_refs, trace_root, output_ref, evidence).expect_err(label);
        assert!(
            matches!(
                err,
                EmissionError::InvalidOptionalField { .. } | EmissionError::InvalidRef { .. }
            ),
            "{label}: {err}"
        );
    }
}

// --- Causality -------------------------------------------------------------------

#[test]
fn e7_causality_records_the_whole_available_chain_and_refuses_cross_wired_parents() {
    let (settlement, facts) = real_adjudicated_settlement(
        "causality",
        WORK_REQUEST_ID,
        "accepted",
        attesting_effects(),
    );
    let e4 = governed_request_envelope(WORK_REQUEST_ID, &local_model_sha256());
    let inputs = CompletionInputs::default();
    let c = contract(&inputs);

    let envelope = complete(&settlement, &facts, &[&e4], &inputs, &c).expect("emits");
    let parents = envelope.causal_parents();
    assert_eq!(
        parents,
        vec![settlement.event_id.as_str(), e4.event_id.as_str()]
    );
    assert_eq!(envelope.payload["work_request_id"], WORK_REQUEST_ID);

    // An upstream envelope correlating to another cycle is refused even as a
    // mere ADDITIONAL parent.
    let stranger = governed_request_envelope("wr-not-this-cycle", &local_model_sha256());
    assert!(matches!(
        complete(&settlement, &facts, &[&stranger], &inputs, &c),
        Err(EmissionError::CrossWiredWorkRequest { .. })
    ));

    // A malformed upstream envelope (wrong producer for its declared type) is
    // refused at the boundary before derivation.
    let mut impostor_wire = serde_json::to_value(&e4).unwrap();
    impostor_wire["source_agent"] = json!("sea-forge");
    let impostor = decode(&impostor_wire);
    assert!(matches!(
        complete(&settlement, &facts, &[&impostor], &inputs, &c),
        Err(EmissionError::Boundary(_))
    ));
}

// --- Producer authority both directions -------------------------------------------

#[test]
fn e7_producer_authority_is_exclusive_in_both_directions_on_the_wire() {
    let envelope = emit_from_real_chain("authority", &Default::default());

    // realitytrace / sea_forge stamps on ProofCompleted are forgeries.
    for impostor in [
        "realitytrace",
        "sea-forge",
        "godspeed-agent",
        "mystery-component",
    ] {
        let mut wire = serde_json::to_value(&envelope).unwrap();
        wire["source_agent"] = json!(impostor);
        let err = validate_producer(&decode(&wire)).unwrap_err();
        assert!(
            matches!(
                err,
                ProducerAuthorityError::NotAuthoritative { .. }
                    | ProducerAuthorityError::UnknownAgent { .. }
            ),
            "{impostor}: {err}"
        );
    }

    // Mirror direction: swe_seed forging RealityTrace's EvidenceRecorded (E8)
    // is refused by the same registry.
    let mut forged_e8 = serde_json::to_value(&envelope).unwrap();
    forged_e8["event_type"] = json!("EvidenceRecorded");
    assert_eq!(
        validate_producer(&decode(&forged_e8)),
        Err(ProducerAuthorityError::NotAuthoritative {
            event_type: "EvidenceRecorded".to_string(),
            authoritative: "realitytrace",
            got: "swe_seed".to_string(),
        })
    );
}

// --- Identity gates -----------------------------------------------------------------

#[test]
fn e7_identity_gates_refuse_placeholder_and_fallback_identities() {
    // The task-pinned fallback constant and the all-zero digest.
    for pseudo in [fallback_pseudo_hash(), "0".repeat(64)] {
        assert_eq!(
            VerifiedDomainIdentity::from_resolved(&ResolvedHash {
                hash: pseudo.clone(),
                source: HashSource::SeaRoot,
                warned: false,
            }),
            Err(IdentityError::PlaceholderIdentity {
                got: pseudo.clone()
            }),
        );
        assert_eq!(
            swe_seed_core::federation::verify_declared_hash(&pseudo),
            Err(IdentityError::PlaceholderIdentity {
                got: pseudo.clone()
            }),
        );
    }
    // A fallback-sourced resolution is refused outright — no pseudo-identity
    // can enter emission through the environment chain.
    assert!(VerifiedDomainIdentity::from_resolved(&ResolvedHash {
        hash: local_model_sha256(),
        source: HashSource::Fallback,
        warned: true,
    })
    .is_err());
}

// --- TOOTH 2 basis: the settlement reference is content-addressed ------------------

#[test]
fn e7_operational_settlement_reference_is_content_addressed_and_immutable() {
    let (settlement, _) = real_adjudicated_settlement(
        "ref-content",
        WORK_REQUEST_ID,
        "accepted",
        attesting_effects(),
    );
    let reference = operational_settlement_ref(&settlement);

    // ENV-I7 preferred form: sha256:<64 lowercase hex>.
    let digest = reference
        .strip_prefix("sha256:")
        .expect("content-addressed form");
    assert_eq!(digest.len(), 64);
    assert!(digest
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));

    // Deterministic over identical bytes…
    let again = decode(&serde_json::to_value(&settlement).unwrap());
    assert_eq!(operational_settlement_ref(&again), reference);

    // …and sensitive to ANY alteration of the cited settlement — altered
    // upstream bytes produce a different reference, never a silent match.
    let mut tampered = serde_json::to_value(&settlement).unwrap();
    tampered["payload"]["observed_effects"][0]["effect"] = json!("rewritten after the fact");
    assert_ne!(
        operational_settlement_ref(&decode(&tampered)),
        reference,
        "altered observation history cannot masquerade as the cited settlement"
    );
    let mut flipped = serde_json::to_value(&settlement).unwrap();
    flipped["payload"]["operational_settlement_status"] = json!("rejected");
    assert_ne!(operational_settlement_ref(&decode(&flipped)), reference);
}

#[test]
fn e7_rejected_operational_outcomes_still_bind_honestly() {
    // A rejected settlement may back a FAILED proof report: binding to real
    // operational evidence is the requirement; endorsement is not implied.
    let (settlement, facts) =
        real_adjudicated_settlement("rejected-bind", WORK_REQUEST_ID, "rejected", json!([]));
    assert_eq!(facts.operational_outcome, OperationalOutcome::Rejected);
    let inputs = CompletionInputs {
        proof_status: "failed".to_string(),
        ..Default::default()
    };
    let c = contract(&inputs);
    let envelope = complete(&settlement, &facts, &[], &inputs, &c).expect("emits");
    assert_eq!(envelope.payload["proof_status"], "failed");
    assert_eq!(
        envelope.payload["operational_settlement_ref"],
        operational_settlement_ref(&settlement)
    );
}
