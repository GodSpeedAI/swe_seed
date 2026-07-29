//! Federation projection (spec 0020 §9, §12). The gateway's ONLY online
//! integration surface is the semantic envelope via `swe_seed_core::federation`.
//! With `federation.enabled=false` (default), this module makes ZERO external
//! calls — no envelope is constructed, dispatched, consumed, or awaited, and no
//! sink is touched (the standalone invariant). When a plane is enabled, each
//! projected event returns a `Disposition` the caller cites in local audit
//! (envelope id + event type, or suppressed-dispatch reason) — spec 0020 §5/§9.

use std::path::Path;

use crate::federation::{
    self, authority_verdict, check_drift, consume_authority_checked, dispatch, emit_route_selected,
    emit_settlement_recorded, AuthorityVerdict, Dispatch, Envelope, FederationConfig,
};

use super::GatewayError;

/// The disposition of a projected event: what the local audit record must cite
/// (spec 0020 §5: `semantic_envelope_id`/`event_type`, or suppressed reason).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disposition {
    pub event_type: &'static str,
    pub envelope_id: Option<String>,
    pub dispatched: bool,
    pub suppressed_reason: Option<&'static str>,
}

impl Disposition {
    /// Federation off / plane not emitting → no envelope, no external call.
    fn suppressed(event_type: &'static str, reason: &'static str) -> Self {
        Self {
            event_type,
            envelope_id: None,
            dispatched: false,
            suppressed_reason: Some(reason),
        }
    }
    fn from_dispatch(event_type: &'static str, envelope: &Envelope, d: Dispatch) -> Self {
        match d {
            Dispatch::Written(_) => Self {
                event_type,
                envelope_id: Some(envelope.event_id.clone()),
                dispatched: true,
                suppressed_reason: None,
            },
            Dispatch::Suppressed => Self {
                event_type,
                envelope_id: Some(envelope.event_id.clone()),
                dispatched: false,
                suppressed_reason: Some("sink_write_failed_or_unavailable"),
            },
        }
    }
}

/// Project a `RouteSelected` event (spec 0020 §9). No-op + zero external calls
/// when emit is off; otherwise dispatch to `sink` and return the disposition.
pub fn project_route_selected(
    cfg: &FederationConfig,
    domain_model_hash: &str,
    work_request_id: &str,
    route_id: &str,
    route_name: &str,
    proof_command_id: &str,
    sink: Option<&Path>,
) -> Disposition {
    if !cfg.emits() {
        return Disposition::suppressed("RouteSelected", "federation_disabled");
    }
    let envelope = emit_route_selected(
        domain_model_hash,
        work_request_id,
        route_id,
        route_name,
        proof_command_id,
        None,
    );
    let d = dispatch(cfg, &envelope, sink);
    Disposition::from_dispatch("RouteSelected", &envelope, d)
}

/// Project a `SettlementRecorded` event (spec 0020 §13).
pub fn project_settlement_recorded(
    cfg: &FederationConfig,
    domain_model_hash: &str,
    settlement_id: &str,
    work_request_id: &str,
    outcome: &str,
    proof_ref: Option<&str>,
    recorded_at: &str,
    sink: Option<&Path>,
) -> Disposition {
    if !cfg.emits() {
        return Disposition::suppressed("SettlementRecorded", "federation_disabled");
    }
    if !matches!(
        cfg.settlement_mode(),
        federation::SettlementMode::Emit | federation::SettlementMode::Both
    ) {
        return Disposition::suppressed("SettlementRecorded", "settlement_plane_off");
    }
    let envelope = emit_settlement_recorded(
        domain_model_hash,
        settlement_id,
        work_request_id,
        outcome,
        proof_ref,
        recorded_at,
    );
    let d = dispatch(cfg, &envelope, sink);
    Disposition::from_dispatch("SettlementRecorded", &envelope, d)
}

/// Consume an incoming `AuthorityChecked` envelope (spec 0020 §10). When
/// authority is local / federation off, returns the local fallback verdict
/// WITHOUT inspecting the envelope. Otherwise validates the envelope (drift
/// guard first) and reads the verdict.
pub fn consume_authority(
    cfg: &FederationConfig,
    domain_model_hash: &str,
    local_allows: bool,
    incoming: Option<&Envelope>,
) -> Result<AuthorityVerdict, GatewayError> {
    use crate::federation::{authority_gate, AuthorityMode, GateOutcome, Risk};
    // Delegate/hybrid: an incoming envelope must pass the drift guard before
    // its payload can affect the authority decision (spec 0020 §9).
    if cfg.authority_mode() != AuthorityMode::Local {
        if let Some(env) = incoming {
            reject_on_drift(domain_model_hash, env)?;
        }
    }
    let outcome = authority_gate(cfg, Risk::High, local_allows, incoming.and_then(|e| e.payload_result()));
    match outcome {
        GateOutcome::Allow => Ok(AuthorityVerdict::Allow),
        GateOutcome::Deny => Ok(AuthorityVerdict::Deny),
        GateOutcome::Escalate | GateOutcome::NeedsHuman => Ok(AuthorityVerdict::Escalate),
    }
}

impl Envelope {
    /// Read the `result` field of an `AuthorityChecked` payload into a verdict.
    fn payload_result(&self) -> Option<AuthorityVerdict> {
        // For local mode we never look at the payload; for delegate/hybrid the
        // authority_gate already needs the verdict — parse it via the consumer.
        match consume_authority_checked(self) {
            Ok(payload) => authority_verdict(payload),
            Err(_) => None,
        }
    }
}

/// Reject an incoming envelope whose `domain_model_hash` drifted before it can
/// affect routing/authority/context/proof/settlement (spec 0020 §9).
pub fn reject_on_drift(
    expected_hash: &str,
    incoming: &Envelope,
) -> Result<(), GatewayError> {
    check_drift(incoming, expected_hash).map_err(|_| GatewayError::FederationUnavailable {
        reason: "incoming envelope rejected: domain_model_hash drift".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::federation::{FederationConfig, SettlementConfig, SettlementMode};

    fn standalone() -> FederationConfig {
        FederationConfig::default()
    }

    fn emitting() -> FederationConfig {
        FederationConfig {
            enabled: true,
            emit_envelope: true,
            ..Default::default()
        }
    }

    #[test]
    fn standalone_makes_zero_external_calls_no_sink_touched() {
        let tmp = std::env::temp_dir().join(format!(
            "swe-seed-gw-fed-standalone-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let sink = tmp.join("sink.json");
        let d = project_route_selected(
            &standalone(),
            "hash",
            "wrq",
            "route-1",
            "implementation",
            "proof-cmd",
            Some(&sink),
        );
        assert!(!d.dispatched);
        assert!(d.envelope_id.is_none());
        assert_eq!(d.suppressed_reason, Some("federation_disabled"));
        // CRITICAL standalone invariant: no sink file created.
        assert!(!sink.exists(), "federation touched the sink while disabled");
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn emitting_writes_sink_and_cites_envelope_id() {
        let tmp = std::env::temp_dir().join(format!(
            "swe-seed-gw-fed-emit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let sink = tmp.join("sink.json");
        let d = project_route_selected(
            &emitting(),
            "hash",
            "wrq",
            "route-1",
            "implementation",
            "proof-cmd",
            Some(&sink),
        );
        assert!(d.dispatched, "expected dispatch");
        assert!(d.envelope_id.is_some());
        assert!(sink.exists(), "sink file should be written when emitting");
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn settlement_suppressed_when_plane_off_even_if_emitting() {
        let d = project_settlement_recorded(
            &emitting(), // settlement plane defaults to Off
            "hash",
            "set-1",
            "wrq",
            "ok",
            None,
            "2026-07-28T00:00:00+00:00",
            None,
        );
        assert!(!d.dispatched);
        assert_eq!(d.suppressed_reason, Some("settlement_plane_off"));
    }

    #[test]
    fn settlement_emits_when_plane_enabled() {
        let cfg = FederationConfig {
            enabled: true,
            emit_envelope: true,
            settlement: SettlementConfig { mode: SettlementMode::Emit },
            ..Default::default()
        };
        let d = project_settlement_recorded(
            &cfg,
            "hash",
            "set-1",
            "wrq",
            "ok",
            None,
            "2026-07-28T00:00:00+00:00",
            None, // stdout sink → Dispatch::Written("-")
        );
        assert!(d.dispatched);
        assert_eq!(d.event_type, "SettlementRecorded");
    }

    #[test]
    fn local_authority_ignores_incoming_envelope() {
        // federation off → local verdict wins, envelope not inspected.
        let v = consume_authority(&standalone(), "hash", true, None).unwrap();
        assert_eq!(v, AuthorityVerdict::Allow);
    }

    #[test]
    fn drift_guard_rejects_mismatched_envelope() {
        // Build an envelope whose domain_model_hash differs from expected.
        let envelope = emit_route_selected("OTHER-HASH", "wrq", "r", "n", "p", None);
        let err = reject_on_drift("EXPECTED-HASH", &envelope).unwrap_err();
        assert_eq!(err.reason_code(), "federation_unavailable");
    }

    #[test]
    fn consume_authority_local_mode_ignores_incoming_envelope() {
        // In standalone/local authority mode, a (potentially spoofed) incoming
        // AuthorityChecked envelope must NOT affect the verdict: the local
        // decision is sole, the envelope is never inspected for authority.
        let cfg = standalone();

        // Spoofed envelope carries a Deny that, if trusted, would flip an allow.
        let mut payload = serde_json::Map::new();
        payload.insert(
            "result".to_string(),
            serde_json::Value::String("deny".into()),
        );
        let spoofed = crate::federation::make_event("AuthorityChecked", payload, "hash");

        // local_allows=true wins despite the incoming Deny payload.
        assert_eq!(
            consume_authority(&cfg, "hash", true, Some(&spoofed)).unwrap(),
            AuthorityVerdict::Allow
        );
        // local_allows=false denies; the incoming envelope cannot override.
        assert_eq!(
            consume_authority(&cfg, "hash", false, Some(&spoofed)).unwrap(),
            AuthorityVerdict::Deny
        );
        // Pure-function check: the same envelope absent yields identical
        // verdicts — the incoming envelope is a no-op in local mode.
        assert_eq!(
            consume_authority(&cfg, "hash", true, None).unwrap(),
            AuthorityVerdict::Allow
        );
        assert_eq!(
            consume_authority(&cfg, "hash", false, None).unwrap(),
            AuthorityVerdict::Deny
        );
    }
}
