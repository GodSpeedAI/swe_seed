//! Gateway stage-10 proof (spec 0020 §14): a Fabricator proof record can cite
//! gateway audit/proof evidence without weakening semantic-chain validation.
//! Run: `cargo test gateway_fabricator_evidence`.

use swe_seed_core::fabricator::{build_chain, validate_semantic_chain};
use swe_seed_core::gateway::{cite_gateway_evidence, evidence_link};

const NEED: &str = "sample need";
const RUN_ID: &str = "run-gw-evidence";

#[test]
fn well_formed_chain_with_gateway_evidence_still_validates() {
    let mut chain = build_chain(NEED, RUN_ID);
    // Cite a gateway audit id on the proof record (spec 0020 §14).
    cite_gateway_evidence(&mut chain.proof_record, "gateway-audit-2026-07-28-0001");
    cite_gateway_evidence(&mut chain.proof_record, "gateway-proof-route-selected");
    assert_eq!(chain.proof_record.evidence_refs.len(), 2);
    // Chain validation MUST still pass (evidence_refs does not weaken the gate).
    let report = validate_semantic_chain(&chain);
    assert!(report.passed, "chain validation failed with gateway evidence cited");
}

#[test]
fn gateway_evidence_does_not_mutate_chain_topology() {
    // The gateway may add a TraceabilityLink(edge), but routing must not alter
    // existing spine links. Adding a `proves` edge from a gateway ref must keep
    // the chain valid (the gate ignores unknown relation edges).
    let mut chain = build_chain(NEED, RUN_ID);
    let n_before = chain.links.len();
    cite_gateway_evidence(&mut chain.proof_record, "gateway-audit-X");
    chain.links.push(evidence_link("gateway-audit-X", &chain.proof_record.id));
    let report = validate_semantic_chain(&chain);
    assert!(report.passed);
    assert_eq!(chain.links.len(), n_before + 1, "only ADDED an edge, never mutated existing");
}

#[test]
fn empty_evidence_refs_serialize_compact() {
    // Default record (no gateway evidence) must serialize as before — the field
    // is omitted when empty, so existing fixtures/parity stay green.
    let chain = build_chain(NEED, RUN_ID);
    let json = serde_json::to_string(&chain.proof_record).unwrap();
    assert!(
        !json.contains("evidence_refs"),
        "empty evidence_refs must be omitted: {json}"
    );
}
