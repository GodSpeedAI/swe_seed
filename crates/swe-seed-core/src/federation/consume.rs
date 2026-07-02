//! Envelope consume validators + the `domain_model_hash` drift guard
//! (spec 0011 §3, security). Each `consume_*` validates the event type and
//! returns the payload (port of the Python adapter). The drift guard rejects
//! an envelope whose hash does not match the local resolution.

use serde_json::Value;

use super::envelope::{Envelope, NAMESPACE};
use super::flags::AuthorityVerdict;

/// A consume error: wrong event type, missing payload, or hash drift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumeError {
    WrongType {
        expected: String,
        got: String,
    },
    MissingPayload,
    WrongNamespace {
        expected: String,
        got: String,
    },
    /// Cross-repo drift: the envelope's domain_model_hash differs from local.
    HashDrift {
        expected: String,
        got: String,
    },
}

impl std::fmt::Display for ConsumeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsumeError::WrongType { expected, got } => {
                write!(f, "expected {expected} event, got '{got}'")
            }
            ConsumeError::MissingPayload => write!(f, "missing payload"),
            ConsumeError::WrongNamespace { expected, got } => {
                write!(f, "expected {expected} namespace, got '{got}'")
            }
            ConsumeError::HashDrift { expected, got } => {
                write!(f, "domain_model_hash drift: expected {expected}, got {got}")
            }
        }
    }
}

impl std::error::Error for ConsumeError {}

/// Reject an envelope whose `domain_model_hash` is absent or doesn't match the
/// local hash (cross-repo drift guard).
pub fn check_drift(envelope: &Envelope, expected_hash: &str) -> Result<(), ConsumeError> {
    match envelope.domain_model_hash() {
        Some(got) if got != expected_hash => Err(ConsumeError::HashDrift {
            expected: expected_hash.into(),
            got: got.into(),
        }),
        Some(_) => Ok(()),
        None => Err(ConsumeError::HashDrift {
            expected: expected_hash.into(),
            got: "<missing>".into(),
        }),
    }
}

fn require<'a>(envelope: &'a Envelope, event_type: &str) -> Result<&'a Value, ConsumeError> {
    let got_ns = envelope.namespace().unwrap_or("");
    if got_ns != NAMESPACE {
        return Err(ConsumeError::WrongNamespace {
            expected: NAMESPACE.into(),
            got: got_ns.to_string(),
        });
    }
    if envelope.event_type != event_type {
        return Err(ConsumeError::WrongType {
            expected: event_type.into(),
            got: envelope.event_type.clone(),
        });
    }
    if envelope.payload.is_null() {
        return Err(ConsumeError::MissingPayload);
    }
    Ok(&envelope.payload)
}

/// Consume `ContextPacketCreated` → its payload (citations).
pub fn consume_context_packet_created(envelope: &Envelope) -> Result<&Value, ConsumeError> {
    require(envelope, "ContextPacketCreated")
}

/// Consume `AuthorityChecked` → its payload. Use [`authority_verdict`] to read
/// the `result` (allow|deny|escalate).
pub fn consume_authority_checked(envelope: &Envelope) -> Result<&Value, ConsumeError> {
    require(envelope, "AuthorityChecked")
}

/// Consume `SettlementRecorded` → its payload (log-only in v0.1).
pub fn consume_settlement_recorded(envelope: &Envelope) -> Result<&Value, ConsumeError> {
    require(envelope, "SettlementRecorded")
}

/// Read an `AuthorityChecked` payload's `result` into a verdict.
pub fn authority_verdict(authority_payload: &Value) -> Option<AuthorityVerdict> {
    match authority_payload.get("result")?.as_str()? {
        "allow" => Some(AuthorityVerdict::Allow),
        "deny" => Some(AuthorityVerdict::Deny),
        "escalate" => Some(AuthorityVerdict::Escalate),
        _ => None,
    }
}
