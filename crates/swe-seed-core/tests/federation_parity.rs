//! federation_parity: the Rust envelope port matches the Python adapter contract
//! (event types, payload keys, domain_model_hash resolution), envelopes round-
//! trip, the drift guard rejects mismatched hashes, and the authority gate
//! honors local/delegate/hybrid (spec 0011).

use std::fs;

use swe_seed_core::federation::{
    authority_gate, authority_verdict, check_drift, consume_authority_checked,
    consume_context_packet_created, consume_settlement_recorded, emit_context_required,
    emit_proof_completed, emit_proof_started, emit_route_selected, emit_work_requested,
    fallback_hash, resolve_from, AuthorityMode, AuthorityVerdict, ConsumeError, Envelope,
    FederationConfig, GateOutcome, HashSource, Risk, SettlementMode, PROOF_TYPE_LIVE,
};

/// The exact Python fallback: `hashlib.sha256(b"agentic_capability_loop").hexdigest()`.
const PYTHON_FALLBACK: &str = "dc144cbd71a483431301ca1bf32e95c014af4edba8dbcc525f505310e30a107e";

#[test]
fn fallback_hash_matches_python_adapter() {
    // Parity: the Rust fallback is byte-identical to the Python adapter's.
    assert_eq!(fallback_hash(), PYTHON_FALLBACK);
}

#[test]
fn hash_resolution_falls_back_with_warn_when_sea_absent() {
    // No SEA root, no manifest path, no walk → fallback + warn (standalone mode).
    let resolved = resolve_from(None, None, false);
    assert_eq!(resolved.source, HashSource::Fallback);
    assert!(resolved.warned);
    assert_eq!(resolved.hash, PYTHON_FALLBACK);
}

#[test]
fn hash_resolution_reads_sea_manifest_path() {
    // A manifest file with meta.sea_file_hash is read when SEA_MANIFEST_PATH points at it.
    let dir = tempdir();
    let manifest = dir.join("manifest.json");
    fs::write(
        &manifest,
        r#"{"meta":{"sea_file_hash":"deadbeef"},"other":{}}"#,
    )
    .unwrap();
    let resolved = resolve_from(None, Some(manifest.to_str().unwrap()), false);
    assert_eq!(resolved.source, HashSource::SeaManifestPath);
    assert!(!resolved.warned);
    assert_eq!(resolved.hash, "deadbeef");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn hash_resolution_reads_sea_root_layout() {
    // SEA_ROOT → <root>/<manifest-rel>; the nested layout is honored.
    let dir = tempdir();
    let rel = dir.join("docs/specs/domains/agentic_capability_loop");
    fs::create_dir_all(&rel).unwrap();
    fs::write(
        rel.join("agentic_capability_loop.manifest.json"),
        r#"{"meta":{"sea_file_hash":"cafef00d"}}"#,
    )
    .unwrap();
    let resolved = resolve_from(Some(dir.to_str().unwrap()), None, false);
    assert_eq!(resolved.source, HashSource::SeaRoot);
    assert_eq!(resolved.hash, "cafef00d");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn envelope_payload_keys_match_python_contract() {
    let h = "hash-123";
    // WorkRequested: every Python payload key present + domain_model_hash injected.
    let e = emit_work_requested(
        h,
        "wr-1",
        "actor-1",
        "implement feature",
        "src/x.rs",
        "low",
        Some("repo"),
    );
    assert_eq!(e.event_type, "WorkRequested");
    assert_eq!(e.namespace, "agentic_capability_loop");
    assert_eq!(e.payload["domain_model_hash"], h);
    for key in [
        "work_request_id",
        "actor_id",
        "operation",
        "resource",
        "risk_level",
        "repo",
    ] {
        assert!(
            e.payload.get(key).is_some(),
            "WorkRequested missing payload key: {key}"
        );
    }
    assert!(e.event_id.len() > 0 && e.payload["domain_model_hash"] == h);

    // ContextRequired payload keys.
    let e = emit_context_required(h, "wr-1", "cr-1", "corpus-1", Some("q"), 10, false, None);
    assert_eq!(e.event_type, "ContextRequired");
    for key in [
        "work_request_id",
        "context_requirement_id",
        "corpus_id",
        "query",
        "max_results",
        "is_private",
        "scope_claim",
    ] {
        assert!(
            e.payload.get(key).is_some(),
            "ContextRequired missing key: {key}"
        );
    }

    // RouteSelected payload keys.
    let e = emit_route_selected(
        h,
        "wr-1",
        "route-1",
        "implement-with-proof",
        "pc-1",
        Some("agent"),
    );
    assert_eq!(e.event_type, "RouteSelected");
    for key in [
        "work_request_id",
        "route_id",
        "route_name",
        "proof_command_id",
        "selected_by",
    ] {
        assert!(
            e.payload.get(key).is_some(),
            "RouteSelected missing key: {key}"
        );
    }

    // ProofStarted / ProofCompleted keys.
    let e = emit_proof_started(h, "pr-1", "wr-1", "route-1", Some("cargo test"), "t1");
    assert_eq!(e.event_type, "ProofStarted");
    assert!(e.payload.get("started_at").is_some());
    let e = emit_proof_completed(
        h,
        "pr-1",
        "wr-1",
        "pass",
        Some(0),
        Some("out.log"),
        PROOF_TYPE_LIVE,
        "t2",
    );
    assert_eq!(e.event_type, "ProofCompleted");
    for key in [
        "proof_result_id",
        "work_request_id",
        "result",
        "exit_code",
        "output_ref",
        "proof_type",
        "completed_at",
    ] {
        assert!(
            e.payload.get(key).is_some(),
            "ProofCompleted missing key: {key}"
        );
    }
}

#[test]
fn envelope_round_trips_through_serde() {
    // emit → serialize → deserialize → consume reproduces the payload.
    let h = fallback_hash();
    let e = emit_work_requested(&h, "wr-9", "actor", "op", "res", "medium", None);
    let json = serde_json::to_string(&e).unwrap();
    let back: swe_seed_core::federation::Envelope = serde_json::from_str(&json).unwrap();
    assert_eq!(back.event_type, e.event_type);
    assert_eq!(back.event_id, e.event_id);
    assert_eq!(back.payload, e.payload);
    assert_eq!(back.domain_model_hash(), Some(h.as_str()));
}

#[test]
fn drift_guard_rejects_mismatched_hash() {
    let e = emit_work_requested("hash-A", "wr", "a", "o", "r", "low", None);
    assert!(check_drift(&e, "hash-A").is_ok());
    assert_eq!(
        check_drift(&e, "hash-B"),
        Err(ConsumeError::HashDrift {
            expected: "hash-B".into(),
            got: "hash-A".into()
        })
    );
}

#[test]
fn drift_guard_rejects_missing_hash() {
    let mut e = emit_work_requested("hash-A", "wr", "a", "o", "r", "low", None);
    e.payload
        .as_object_mut()
        .unwrap()
        .remove("domain_model_hash");
    assert_eq!(
        check_drift(&e, "hash-A"),
        Err(ConsumeError::HashDrift {
            expected: "hash-A".into(),
            got: "<missing>".into()
        })
    );
}

#[test]
fn consume_validates_event_type_and_payload() {
    let h = fallback_hash();
    let ctx = emit_context_required(&h, "wr", "cr", "corpus", None, 1, false, None);
    // Right type → payload returned.
    assert!(consume_context_packet_created(&ctx).is_err()); // ctx is ContextRequired, not Created
    let created = swe_seed_core::federation::make_event(
        "ContextPacketCreated",
        serde_json::json!({"citations": []})
            .as_object()
            .cloned()
            .unwrap_or_default(),
        &h,
    );
    let payload = consume_context_packet_created(&created).unwrap();
    assert!(payload.get("citations").is_some());

    // AuthorityChecked → verdict parsed.
    let auth = swe_seed_core::federation::make_event(
        "AuthorityChecked",
        serde_json::json!({"result": "deny"})
            .as_object()
            .cloned()
            .unwrap_or_default(),
        &h,
    );
    let p = consume_authority_checked(&auth).unwrap();
    assert_eq!(authority_verdict(p), Some(AuthorityVerdict::Deny));

    // SettlementRecorded → payload returned.
    let set = swe_seed_core::federation::make_event(
        "SettlementRecorded",
        serde_json::json!({"outcome": "accepted"})
            .as_object()
            .cloned()
            .unwrap_or_default(),
        &h,
    );
    assert!(consume_settlement_recorded(&set).is_ok());

    // Wrong type → WrongType.
    let err = consume_authority_checked(&created).unwrap_err();
    assert!(matches!(err, ConsumeError::WrongType { .. }));
}

#[test]
fn consume_rejects_wrong_namespace() {
    let h = fallback_hash();
    let envelope = Envelope {
        event_id: "event-1".into(),
        event_type: "ContextPacketCreated".into(),
        namespace: "other_namespace".into(),
        occurred_at: "2026-06-29T00:00:00Z".into(),
        payload: serde_json::json!({
            "domain_model_hash": h,
            "citations": []
        }),
    };
    let err = consume_context_packet_created(&envelope).unwrap_err();
    assert_eq!(
        err,
        ConsumeError::WrongNamespace {
            expected: "agentic_capability_loop".into(),
            got: "other_namespace".into()
        }
    );
}

#[test]
fn authority_modes_honor_contract() {
    use swe_seed_core::federation::{AuthorityConfig, ContextMode};

    fn cfg(mode: AuthorityMode, override_deny: bool) -> FederationConfig {
        FederationConfig {
            enabled: true,
            authority: AuthorityConfig {
                mode,
                allow_external_override: override_deny,
            },
            ..FederationConfig::default()
        }
    }

    // local ignores envelopes: an incoming Deny does not block a local allow.
    let local = cfg(AuthorityMode::Local, false);
    assert_eq!(
        authority_gate(&local, Risk::Low, true, Some(AuthorityVerdict::Deny)),
        GateOutcome::Allow
    );
    assert_eq!(
        authority_gate(&local, Risk::Low, false, Some(AuthorityVerdict::Allow)),
        GateOutcome::Deny
    );

    // delegate: deny blocks, escalate escalates, allow proceeds.
    let delegate = cfg(AuthorityMode::Delegate, false);
    assert_eq!(
        authority_gate(&delegate, Risk::Low, true, Some(AuthorityVerdict::Deny)),
        GateOutcome::Deny
    );
    assert_eq!(
        authority_gate(&delegate, Risk::Low, true, Some(AuthorityVerdict::Escalate)),
        GateOutcome::Escalate
    );
    assert_eq!(
        authority_gate(&delegate, Risk::Low, true, Some(AuthorityVerdict::Allow)),
        GateOutcome::Allow
    );

    // delegate timeout: fail-closed for dangerous, local fallback otherwise.
    assert_eq!(
        authority_gate(&delegate, Risk::Dangerous, true, None),
        GateOutcome::Deny
    );
    assert_eq!(
        authority_gate(&delegate, Risk::Low, true, None),
        GateOutcome::Allow
    );

    // local deny is NOT overridden by external allow unless allow_external_override.
    assert_eq!(
        authority_gate(&delegate, Risk::Low, false, Some(AuthorityVerdict::Allow)),
        GateOutcome::Deny
    );
    let overriding = cfg(AuthorityMode::Delegate, true);
    assert_eq!(
        authority_gate(&overriding, Risk::Low, false, Some(AuthorityVerdict::Allow)),
        GateOutcome::Allow
    );

    // hybrid: a clean local allow on low risk proceeds without consulting.
    let hybrid = cfg(AuthorityMode::Hybrid, false);
    assert_eq!(
        authority_gate(&hybrid, Risk::Low, true, Some(AuthorityVerdict::Deny)),
        GateOutcome::Allow
    );
    // hybrid high-risk allow consults the envelope (deny blocks).
    assert_eq!(
        authority_gate(&hybrid, Risk::High, true, Some(AuthorityVerdict::Deny)),
        GateOutcome::Deny
    );

    // Master switch off → behaves as local regardless of configured mode.
    let mut off = cfg(AuthorityMode::Delegate, false);
    off.enabled = false;
    assert_eq!(off.authority_mode(), AuthorityMode::Local);
    assert_eq!(off.context_mode(), ContextMode::Local);
    assert_eq!(off.settlement_mode(), SettlementMode::Off);
    assert!(off.is_standalone());
    assert_eq!(
        authority_gate(&off, Risk::Low, true, Some(AuthorityVerdict::Deny)),
        GateOutcome::Allow
    );
}

fn tempdir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-fed-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}
