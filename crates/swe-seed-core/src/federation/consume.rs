//! Envelope consume validators + the `domain_model_hash` drift guard
//! (spec 0011 §3, security). Each `consume_*` validates the event type and
//! returns the payload (port of the Python adapter). The drift guard rejects
//! an envelope whose hash does not match the local resolution.

use serde_json::Value;

use super::envelope::{Envelope, NAMESPACE, SCHEMA_VERSION};
use super::flags::AuthorityVerdict;
use super::identity::{self, IdentityError};
use super::producers::{validate_producer, ProducerAuthorityError};

/// A consume error: wrong event type, missing payload, hash drift, or any
/// T01 gate failure (producer authority / identity / causality).
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
    /// Exclusive-producer authority violation (invariant I3).
    ProducerAuthority(ProducerAuthorityError),
    /// Canonical identity gate failure (ENV-I1/I2/I1).
    Identity(IdentityError),
    /// Malformed causal-parent record (ENV-I4).
    Causality {
        reason: String,
    },
    /// Destination-only input at a work-contract boundary: a required
    /// affordance/outcome/criteria field is absent or empty (frozen E1
    /// falsifier — an ungrounded objective cannot settle as work).
    DestinationOnly {
        field: String,
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
            ConsumeError::ProducerAuthority(e) => write!(f, "{e}"),
            ConsumeError::Identity(e) => write!(f, "{e}"),
            ConsumeError::Causality { reason } => write!(f, "causality violation: {reason}"),
            ConsumeError::DestinationOnly { field } => {
                write!(
                    f,
                    "destination-only input rejected: missing/empty '{field}'"
                )
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

// --- T01 gates --------------------------------------------------------------

fn well_formed_event_id(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok() || ulid::Ulid::from_string(id).is_ok()
}

/// Full canonical-boundary validation for one envelope:
///
/// 1. exclusive producer authority ([`producers::validate_producer`], I3);
/// 2. canonical identity — declared `domain_model_hash` must be a real
///    digest and not a placeholder (ENV-I1/I2), then drift against the local
///    resolution;
/// 3. every recorded causal parent id is a well-formed event id (ENV-I4).
///
/// This is the single entry point later edge tasks compose with their
/// per-edge payload checks.
pub fn validate_envelope(envelope: &Envelope, expected_hash: &str) -> Result<(), ConsumeError> {
    validate_producer(envelope).map_err(ConsumeError::ProducerAuthority)?;
    let declared = match envelope.domain_model_hash() {
        Some(d) => d,
        None => {
            return Err(ConsumeError::Identity(IdentityError::MalformedHash {
                got: "<missing>".into(),
            }))
        }
    };
    identity::verify_declared_hash(declared).map_err(ConsumeError::Identity)?;
    check_drift(envelope, expected_hash)?;
    for parent in envelope.causal_parents() {
        if !well_formed_event_id(parent) {
            return Err(ConsumeError::Causality {
                reason: format!("causal parent id is not an event id: {parent}"),
            });
        }
    }
    Ok(())
}

/// Pure structural conformance of an envelope against the v1 shape
/// (`sea.agent.event.v1.json`) plus the T01 well-formedness rules.
///
/// Deliberately returns data, never performs effects, never evaluates the
/// truth of the enclosed claim, and takes no expected-hash input: schema/CEP
/// conformance establishes conformance only (preregistration ENV-I8, I13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceReport {
    pub conforms: bool,
    pub violations: Vec<String>,
}

impl ConformanceReport {
    pub fn is_conformant(&self) -> bool {
        self.conforms
    }
}

pub fn check_conformance(envelope: &Envelope) -> ConformanceReport {
    let mut violations: Vec<String> = Vec::new();
    let mut push = |v: String| violations.push(v);
    if envelope.schema_version != SCHEMA_VERSION {
        push(format!(
            "schema_version must be {SCHEMA_VERSION}, got {}",
            envelope.schema_version
        ));
    }
    if envelope.event_id.is_empty() || !well_formed_event_id(&envelope.event_id) {
        push(format!(
            "event_id is not a well-formed id: {}",
            envelope.event_id
        ));
    }
    if super::producers::canonical_agent(&envelope.source_agent).is_none() {
        push(format!(
            "source_agent outside canonical vocabulary: {}",
            envelope.source_agent
        ));
    }
    if super::producers::authoritative_producer(&envelope.event_type).is_none() {
        push(format!(
            "event_type outside canonical registry: {}",
            envelope.event_type
        ));
    }
    if envelope.namespace() != Some(NAMESPACE) {
        push(format!(
            "namespace mismatch: want {NAMESPACE}, got {:?}",
            envelope.namespace().unwrap_or("<missing>")
        ));
    }
    match envelope.domain_model_hash() {
        None => push("payload missing domain_model_hash".to_string()),
        Some(h) => {
            if identity::verify_declared_hash(h).is_err() {
                push(format!(
                    "domain_model_hash fails canonical shape/placeholder rules: {h}"
                ));
            }
        }
    }
    match &envelope.idempotency_key {
        Some(k) => {
            if k.len() != 64 || !k.bytes().all(|b| b.is_ascii_hexdigit()) {
                push(format!("idempotency_key is not sha256-hex: {k}"));
            }
        }
        None => push("idempotency_key missing".to_string()),
    }
    if envelope.payload.is_null() {
        push("payload missing".to_string());
    }
    for parent in envelope.causal_parents() {
        if !well_formed_event_id(parent) {
            push(format!("causal parent id malformed: {parent}"));
        }
    }
    ConformanceReport {
        conforms: violations.is_empty(),
        violations,
    }
}
