//! v1 contract test (WP-1, F-01): every envelope SWE_Seed emits serializes to
//! JSON that satisfies `sea.agent.event.v1.json`. The ledger enforces v1
//! fail-closed (`additionalProperties:false`); before this fix SWE_Seed's
//! `federation::Envelope` was the family-A shape (top-level `namespace`, no
//! `schema_version`/`source_agent`/`idempotency_key`) which the ledger would
//! DLQ.
//!
//! We mirror jsonschema's enforcement of the schema's `required` list and
//! `additionalProperties:false` by reading the vendored schema directly, so the
//! allowed-property set is the schema's, not a hand-maintained copy. The full
//! jsonschema pass lives at the ledger; this is the producer-side teeth-check.

use std::collections::HashSet;

use serde_json::Value;
use swe_seed_core::federation::{
    fallback_hash, idempotency_key, make_event, Envelope,
    SOURCE_AGENT,
};

fn schema_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/fixtures/sea.agent.event.v1.json")
}

fn schema() -> Value {
    serde_json::from_str(&std::fs::read_to_string(schema_path()).unwrap()).unwrap()
}

/// Mirror jsonschema's `required` + `additionalProperties:false` enforcement,
/// reading the allowed property set from the schema itself.
fn assert_v1(envelope: &Envelope) {
    let s = schema();
    let allowed: HashSet<String> = s["properties"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    let required: Vec<String> = s["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let json = serde_json::to_value(envelope).unwrap();
    let obj = json.as_object().unwrap();
    for field in &required {
        assert!(
            obj.contains_key(field),
            "v1 required field missing: {field}"
        );
    }
    let extra: Vec<&String> = obj.keys().filter(|k| !allowed.contains(*k)).collect();
    assert!(
        extra.is_empty(),
        "v1 additionalProperties:false violated by top-level keys: {extra:?}"
    );
    assert_eq!(json["schema_version"], "v1");
    assert_eq!(json["source_agent"], SOURCE_AGENT);
    assert!(json["payload"].is_object(), "payload must be an object");
    // namespace rides inside payload, never top-level (the family-A->v1 port).
    assert_eq!(json["payload"]["namespace"], "agentic_capability_loop");
    assert!(!obj.contains_key("namespace"), "top-level namespace rejected by v1");
    // F-06: every SWE_Seed envelope carries a populated provenance block.
    let prov = &json["provenance"];
    assert!(prov.is_object(), "provenance required");
    assert!(prov["origin"].as_str().unwrap_or("").len() > 0, "provenance.origin required");
    assert!(prov["chain"].is_array(), "provenance.chain must be an array");
}

#[test]
fn emitted_envelopes_are_v1_conformant() {
    let h = fallback_hash();
    let cases: Vec<Envelope> = vec![
        swe_seed_core::federation::emit_work_requested(
            &h, "wr-1", "actor", "implement", "src/x.rs", "low", None,
        ),
        swe_seed_core::federation::emit_context_required(
            &h, "wr-1", "cr-1", "corpus", Some("q"), 5, false, None,
        ),
        swe_seed_core::federation::emit_route_selected(
            &h, "wr-1", "route-1", "implement-with-proof", "pc-1", Some("agent"),
        ),
        swe_seed_core::federation::emit_proof_started(
            &h, "pr-1", "wr-1", "route-1", Some("cargo test"), "t1",
        ),
        swe_seed_core::federation::emit_proof_completed(
            &h, "pr-1", "wr-1", "pass", Some(0), Some("out.log"),
            swe_seed_core::federation::PROOF_TYPE_LIVE, "t2",
        ),
    ];
    assert!(cases.len() >= 5);
    for e in &cases {
        assert_v1(e);
    }
}

#[test]
fn envelope_round_trips_through_serde_as_v1() {
    let h = fallback_hash();
    let e = swe_seed_core::federation::emit_work_requested(&h, "wr-9", "a", "o", "r", "low", None);
    let json = serde_json::to_string(&e).unwrap();
    let back: Envelope = serde_json::from_str(&json).unwrap();
    assert_eq!(back.event_id, e.event_id);
    assert_eq!(back.source_agent, SOURCE_AGENT);
    assert_eq!(back.schema_version, "v1");
    // The serialized form has no top-level namespace.
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(!v.as_object().unwrap().contains_key("namespace"));
    assert_v1(&back);
}

#[test]
fn teeth_top_level_namespace_is_rejected() {
    // Simulate the old family-A shape: inject a top-level `namespace`. The v1
    // additionalProperties:false check must reject it — this is the contract's
    // teeth.
    let h = fallback_hash();
    let e = swe_seed_core::federation::emit_work_requested(&h, "wr", "a", "o", "r", "low", None);
    let mut json = serde_json::to_value(&e).unwrap();
    let obj = json.as_object_mut().unwrap();
    obj.insert("namespace".to_string(), Value::String("agentic_capability_loop".into()));
    let allowed: HashSet<String> = schema()["properties"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    let extra: Vec<&String> = obj.keys().filter(|k| !allowed.contains(*k)).collect();
    assert_eq!(extra, vec![&"namespace".to_string()]);
}

#[test]
fn replay_shares_idempotency_key_not_event_id() {
    // F-10: a re-wrapped replay (fresh event_id + occurred_at, identical
    // payload) keeps the idempotency_key — the old family-A envelope had no
    // such key at all.
    let h = fallback_hash();
    let e = swe_seed_core::federation::emit_work_requested(&h, "wr", "a", "o", "r", "low", None);
    let original_key = e.idempotency_key.clone().unwrap();
    assert_ne!(original_key, e.event_id, "idempotency_key must not equal event_id");
    // Re-derive from the same payload: stable.
    let rederived = idempotency_key(&e.event_type, &e.payload, None);
    assert_eq!(original_key, rederived);
}

#[test]
fn make_event_injects_namespace_and_hash_into_payload() {
    let e = make_event(
        "ContextPacketCreated",
        serde_json::json!({"citations": []})
            .as_object()
            .cloned()
            .unwrap_or_default(),
        "hash-xyz",
    );
    assert_eq!(e.payload["domain_model_hash"], "hash-xyz");
    assert_eq!(e.payload["namespace"], "agentic_capability_loop");
    assert_eq!(e.namespace(), Some("agentic_capability_loop"));
    assert_v1(&e);
}

#[test]
fn vendored_v1_schema_matches_contracts_pin() {
    // WP-2 drift check: the vendored v1 schema must be byte-identical to the
    // canonical copy in hassos-addon-agent-memory-ledger (pin recorded in that
    // repo's contracts/CONTRACTS_VERSION). Editing this copy by one byte fails.
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(schema_path()).unwrap();
    let digest = {
        let mut h = Sha256::new();
        h.update(&bytes);
        format!("{:x}", h.finalize())
    };
    assert_eq!(
        digest,
        "d2a245183f7a52672dec11d7a8e9236d3db78dac674ab066d944fb758443e195",
        "vendored v1 schema drifted from the ledger's CONTRACTS_VERSION pin"
    );
}
