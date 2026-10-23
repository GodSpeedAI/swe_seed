//! End-to-end world loop, SWE_SEED's two hops (driven by sea-rs
//! `scripts/e2e-world-loop.sh`).
//!
//! Hop 2 turns the real E1 (GodSpeed-Agent) and E3 (Context Kernel) into the E4
//! `GovernedWorkRequest`; hop 4 turns the real E6 (SEA-Forge) into the E7
//! `ProofCompleted`. Both read their input from `E2E_DIR`, verify identity
//! against a real model artifact, and carry the one pinned `E2E_WORLD_REF`.
//! Skipped unless the driver sets `E2E_DIR`.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use swe_seed_core::federation::{
    accept_work_requested, build_governed_work_request, emit_proof_completed_verified,
    verify_against_artifact, verify_context_bundle, Adjudication, Envelope, GovernedSubmission,
    OperationalSettlementAdjudicator, ProofCompletion, ProofContract, VerifiedDomainIdentity,
    WorldRef, PROOF_TYPE_LIVE,
};

fn exchange_dir() -> Option<PathBuf> {
    std::env::var_os("E2E_DIR").map(PathBuf::from)
}

fn read(dir: &Path, name: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(dir.join(name)).unwrap_or_else(|e| panic!("{name}: {e}")),
    )
    .unwrap()
}

fn envelope(dir: &Path, name: &str) -> Envelope {
    serde_json::from_value(read(dir, name))
        .unwrap_or_else(|e| panic!("{name} is not a canonical envelope: {e}"))
}

/// Identity is verified against the real model artifact the driver wrote.
fn identity(dir: &Path) -> VerifiedDomainIdentity {
    let hash = std::env::var("E2E_DOMAIN_HASH").expect("E2E_DOMAIN_HASH");
    verify_against_artifact(&hash, &dir.join("canonical_model.sea.json"))
        .expect("identity verifies against the artifact")
}

#[test]
fn hop_2_governed_work_request_from_the_real_e1_and_e3() {
    let Some(dir) = exchange_dir() else {
        eprintln!("SKIP: E2E_DIR not set");
        return;
    };
    let world = std::env::var("E2E_WORLD_REF").expect("E2E_WORLD_REF");
    let identity = identity(&dir);
    let e1 = envelope(&dir, "e1_work_requested.json");
    let e3 = envelope(&dir, "e3_context_packet.json");

    let contract =
        accept_work_requested(&e1, identity.as_str()).expect("real GSA E1 passes the ingress gate");
    assert_eq!(contract.world_ref.as_str(), world);

    let proof = ProofContract {
        criterion: "live proof exits 0 with health checks green".into(),
        command: Some("just proof --route route-blue-green".into()),
        proof_type: Some(PROOF_TYPE_LIVE.into()),
    };
    // Dual-read: the canonical CEP context bundle is CONSUMED here (verified:
    // kind, profile, pinned world, correlation, truthful completeness, CEP
    // integrity). The legacy E3 packet remains readable during the bounded
    // migration window.
    let bundle = read(&dir, "e3_context_bundle.cep.json");
    let facts = verify_context_bundle(&bundle, &contract.work_request_id, &world, None)
        .expect("the real CK context bundle verifies at the SWE_SEED boundary");
    assert_eq!(bundle["scope"]["world_ref"], world.as_str());

    let request = build_governed_work_request(
        &e1,
        &e3,
        GovernedSubmission {
            contract: &contract,
            actor_id: "agent-operator",
            actor_role: Some("R-AA"),
            intent: "rotate deploy to blue-green and verify health gates",
            proof_contract: &proof,
            route_id: None,
            artifact_expectations: None,
            authority_context: None,
            payment_budget: None,
            context_bundle: Some(&bundle),
        },
        &identity,
    )
    .expect("E1 + E3 + the verified bundle under one world build an E4");
    assert_eq!(request.world_ref(), Some(world.as_str()));

    // The EXACT observer context is bound into the request lineage.
    let bound = &request.payload["context_bundle_ref"];
    assert_eq!(bound["envelope_id"], facts.envelope_id.as_str());
    assert_eq!(bound["content_hash"], facts.content_hash.as_str());
    assert_eq!(bound["world_ref"], world.as_str());

    std::fs::write(
        dir.join("e4_governed_work_request.json"),
        serde_json::to_string_pretty(&request).unwrap() + "\n",
    )
    .unwrap();
}

#[test]
fn hop_4_proof_completed_from_the_real_e6() {
    let Some(dir) = exchange_dir() else {
        eprintln!("SKIP: E2E_DIR not set");
        return;
    };
    let world = WorldRef::parse(&std::env::var("E2E_WORLD_REF").expect("E2E_WORLD_REF")).unwrap();
    let identity = identity(&dir);
    let work_request_id = std::env::var("E2E_WORK_REQUEST_ID").expect("E2E_WORK_REQUEST_ID");

    let e4 = envelope(&dir, "e4_governed_work_request.json");
    let e6 = envelope(&dir, "e6_operational_settlement.json");
    // The execution events SEA-Forge's settlement names as causal parents.
    let parents: Vec<String> = e6.causal_parents().iter().map(|s| s.to_string()).collect();
    let parent_refs: Vec<&str> = parents.iter().map(String::as_str).collect();

    let mut judge =
        OperationalSettlementAdjudicator::open(&dir.join("e7-adjudications.jsonl")).unwrap();
    let Adjudication::First(facts) = judge
        .adjudicate(
            &e6,
            &work_request_id,
            &parent_refs,
            identity.as_str(),
            &world,
        )
        .expect("the real SEA-Forge settlement adjudicates")
    else {
        panic!("a fresh adjudicator yields First");
    };

    let contract = &e4.payload["proof_contract"];
    let proof = ProofContract {
        criterion: contract["criterion"].as_str().unwrap().into(),
        command: contract["command"].as_str().map(Into::into),
        proof_type: contract["proof_type"].as_str().map(Into::into),
    };
    let expected = json!({"affordance": "aff-blue-green-001", "declared_result": "health check green; zero-downtime observed"});
    let bundle_ref = e4.payload["context_bundle_ref"].clone();
    assert!(
        !bundle_ref.is_null(),
        "E4 must carry the exact context bundle identity"
    );
    let proof_completed = emit_proof_completed_verified(
        &e6,
        &facts,
        &[&e4],
        ProofCompletion {
            originating_work_request_id: &work_request_id,
            proof_result_id: "pr-e2e-0001",
            proof_contract: &proof,
            proof_status: "passed",
            expected_outcome: &expected,
            artifact_refs: None,
            trace_root: None,
            output_ref: None,
            proof_evidence_refs: None,
            context_bundle_ref: Some(&bundle_ref),
        },
        &identity,
    )
    .expect("the real chain emits ProofCompleted");
    assert_eq!(proof_completed.world_ref(), Some(world.as_str()));
    // The exact bundle identity survives into the proof/evidence lineage.
    assert_eq!(
        proof_completed.payload["context_bundle_ref"]["envelope_id"],
        bundle_ref["envelope_id"]
    );

    std::fs::write(
        dir.join("e7_proof_completed.json"),
        serde_json::to_string_pretty(&proof_completed).unwrap() + "\n",
    )
    .unwrap();
}
