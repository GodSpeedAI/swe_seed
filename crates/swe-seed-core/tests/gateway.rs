//! Gateway stage-1 proof (spec 0020 §7, §18): model serialization round-trips
//! and every `GatewayError` variant maps to a unique, stable, snake_case
//! `reason_code` usable by local audit (§5) and semantic-envelope disposition
//! (§9). Run: `cargo test -q -p swe-seed-core gateway`.

use swe_seed_core::gateway::{
    BreakerState, CatalogKind, GatewayBackend, GatewayCatalogEntry, GatewayEffectiveLimits,
    GatewayError, GatewayGovernanceState, GatewaySession, GatewayTransport, HealthState,
    SessionState,
};

fn sample_backend() -> GatewayBackend {
    GatewayBackend {
        id: "fs".into(),
        namespace: "fs".into(),
        transport: GatewayTransport::Stdio,
        command_or_url: "npx -y @modelcontextprotocol/server-filesystem".into(),
        profile_tags: vec!["readonly".into()],
        health_state: HealthState::Healthy,
        breaker_state: BreakerState::Closed,
        provenance_ref: Some("provenance/fs.json".into()),
    }
}

fn sample_catalog_entry() -> GatewayCatalogEntry {
    GatewayCatalogEntry {
        kind: CatalogKind::Tool,
        namespaced_name: "fs.read_file".into(),
        backend_id: "fs".into(),
        wire_definition: serde_json::json!({"name":"read_file","description":"read a file"}),
        definition_hash: "sha256:abc".into(),
        source_hash: Some("sha256:src".into()),
        policy_tags: vec!["readonly".into()],
        pinned_hash: Some("sha256:abc".into()),
        input_schema: Some(serde_json::json!({"type":"object"})),
    }
}

fn sample_session() -> GatewaySession {
    GatewaySession {
        session_id: "0123456789abcdef0123456789abcdef".into(),
        client_id: "cli".into(),
        route_card_id: Some("implementation".into()),
        allowed_backends: vec!["fs".into()],
        state: SessionState::Active,
        started_at: "2026-07-28T00:00:00Z".into(),
        call_count: 0,
        max_calls: 100,
        max_duration_secs: 600,
    }
}

fn sample_governance() -> GatewayGovernanceState {
    let mut s = GatewayGovernanceState::default();
    s.session_calls.insert("sess".into(), 1);
    s.client_counters.insert("cli".into(), 1);
    s.tool_counters.insert("fs.read_file".into(), 1);
    s.utc_window_start = Some("2026-07-28T00:00:00Z".into());
    s.limits = GatewayEffectiveLimits {
        session_max_calls: 100,
        session_max_duration_secs: 600,
        client_max_requests: 1000,
        tool_max_requests: 100,
    };
    s
}

#[test]
fn gateway_backend_serializes_roundtrip() {
    let backend = sample_backend();
    let json = serde_json::to_string(&backend).unwrap();
    let back: GatewayBackend = serde_json::from_str(&json).unwrap();
    assert_eq!(backend, back);
    // serde rename_all = lowercase on the transport enum
    assert!(json.contains("\"transport\":\"stdio\""));
    assert!(json.contains("\"health_state\":\"healthy\""));
    assert!(json.contains("\"breaker_state\":\"closed\""));
}

#[test]
fn gateway_catalog_entry_serializes_roundtrip() {
    let entry = sample_catalog_entry();
    let json = serde_json::to_string(&entry).unwrap();
    let back: GatewayCatalogEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(entry, back);
    assert!(json.contains("\"kind\":\"tool\""));
    assert!(json.contains("\"namespaced_name\":\"fs.read_file\""));
}

#[test]
fn gateway_session_serializes_roundtrip() {
    let session = sample_session();
    let json = serde_json::to_string(&session).unwrap();
    let back: GatewaySession = serde_json::from_str(&json).unwrap();
    assert_eq!(session, back);
    assert!(json.contains("\"state\":\"active\""));
}

#[test]
fn gateway_governance_serializes_roundtrip() {
    let gov = sample_governance();
    let json = serde_json::to_string(&gov).unwrap();
    let back: GatewayGovernanceState = serde_json::from_str(&json).unwrap();
    assert_eq!(gov, back);
    assert_eq!(back.session_calls.get("sess"), Some(&1));
}

/// The reason_code contract is the audit/envelope surface; it must be total,
/// unique, non-empty, and stable snake_case. This is the single load-bearing
/// invariant for typed error mapping (spec 0020 §5, §18).
#[test]
fn gateway_error_reason_codes_are_stable_unique_snake_case() {
    let samples = gateway_error_one_of_each();
    let mut codes: Vec<&str> = samples.iter().map(|e| e.reason_code()).collect();
    for c in &codes {
        assert!(!c.is_empty(), "empty reason_code");
        assert!(
            c.bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_'),
            "reason_code not snake_case: {c}"
        );
    }
    codes.sort_unstable();
    let before = codes.len();
    codes.dedup();
    assert_eq!(codes.len(), before, "duplicate reason_code after dedup");
    // Spot-check the spec-named codes.
    assert_eq!(
        GatewayError::BackendUnavailable {
            backend: "b".into()
        }
        .reason_code(),
        "backend_unavailable"
    );
    assert_eq!(
        GatewayError::CapabilityHashMismatch {
            namespaced_name: "n".into(),
            expected: "e".into(),
            got: "g".into()
        }
        .reason_code(),
        "capability_hash_mismatch"
    );
    assert_eq!(
        GatewayError::InvalidReload { reason: "x".into() }.reason_code(),
        "invalid_reload"
    );
}

#[test]
fn gateway_error_is_std_error_and_lifts_into_anyhow() {
    // `GatewayError: std::error::Error` is required so `?` lifts it through
    // `anyhow::Result` at gateway call sites.
    fn require_error(_e: &dyn std::error::Error) {}
    let e = GatewayError::BudgetExceeded { limit: 7 };
    require_error(&e);
    let lifted: anyhow::Error = e.into();
    assert!(lifted.to_string().contains("budget exceeded"));
}

fn gateway_error_one_of_each() -> Vec<GatewayError> {
    vec![
        GatewayError::BackendUnavailable {
            backend: "b".into(),
        },
        GatewayError::BackendError {
            backend: "b".into(),
            reason: "r".into(),
        },
        GatewayError::CapabilityHashMismatch {
            namespaced_name: "n".into(),
            expected: "e".into(),
            got: "g".into(),
        },
        GatewayError::AuthFailed { reason: "r".into() },
        GatewayError::ScopeDenied { scope: "s".into() },
        GatewayError::ApprovalRequired {
            namespaced_name: "n".into(),
        },
        GatewayError::ScanBlocked {
            namespaced_name: "n".into(),
            reason: "r".into(),
        },
        GatewayError::ProvenanceMissing {
            namespaced_name: "n".into(),
        },
        GatewayError::BudgetExceeded { limit: 1 },
        GatewayError::SessionLimitExceeded {
            session_id: "s".into(),
        },
        GatewayError::InvalidRequest { reason: "r".into() },
        GatewayError::InvalidReload { reason: "r".into() },
        GatewayError::AuditWriteFailed { reason: "r".into() },
        GatewayError::FederationUnavailable { reason: "r".into() },
        GatewayError::NamespacedRerouteBlocked {
            namespaced_name: "n".into(),
        },
    ]
}
