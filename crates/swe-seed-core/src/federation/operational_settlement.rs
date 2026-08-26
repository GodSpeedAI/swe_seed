//! OperationalSettlement adjudication (convergence plan task T06; frozen edge
//! E6, invariants I6/I7). SEA-Forge is the exclusive authoritative producer of
//! OperationalSettlement; SWE_SEED is its consumer — the proof plane.
//!
//! This surface accepts a canonical OperationalSettlement envelope ONLY when
//! it is bound to the ORIGINATING work request: producer authority
//! (`sea_forge` exclusively, I3), canonical domain identity against OUR local
//! resolution (ENV-I1/I2), the loop namespace, every frozen required payload
//! field parsed and validated, mandatory causality naming the known
//! invocation-chain event ids (ENV-I4), and correlation equality with the
//! originating `work_request_id` (ENV-I3).
//!
//! Duplicate delivery is consequence-free under stable envelope identity and
//! survives restart (ENV-I6, via the durable [`IdempotencyLedger`]); a NEW
//! envelope claiming the SAME invocation chain (work_request_id +
//! authority_decision_id) with different content is a conflicting
//! re-settlement and is refused — the first settlement stands, so observed
//! effects cannot be mutated into success after the fact.
//!
//! What this surface returns is OPERATIONAL FACTS ONLY:
//! [`OperationalSettlementFacts`] carries nothing capability- or
//! developmental-shaped, and no proof verdict. The proof contract remains a
//! distinct obligation downstream (I6), and there is no path from this
//! surface to `SettlementRecorded`, `CapabilityUpdated`, or any
//! capability promotion: the producer registry refuses `sea_forge` stamps on
//! those developmental events outright (I7) — machine-checked by tests.

use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::consume::{validate_envelope, ConsumeError};
use super::envelope::{Envelope, NAMESPACE};
use super::idempotency::{AdmitError, IdempotencyLedger};

/// The event type owned exclusively by `sea_forge` at edge E6.
pub const EVENT_TYPE: &str = "OperationalSettlement";

/// The frozen required payload fields of an OperationalSettlement.
pub const REQUIRED_FIELDS: [&str; 7] = [
    "work_request_id",
    "authority_decision_id",
    "execution_status",
    "observed_effects",
    "operational_settlement_status",
    "evidence_refs",
    "domain_model_ref",
];

/// The governed execution-status vocabulary carried over from edge E5B.
const EXECUTION_STATUSES: [&str; 5] = [
    "completed",
    "spawn_failed",
    "timed_out",
    "sandbox_violation",
    "suspected_sandbox_violation",
];

/// Explicit placeholder identities that must never bind a settlement.
const PLACEHOLDER_IDS: [&str; 5] = ["placeholder", "unknown", "none", "<missing>", "tbd"];

/// Why an OperationalSettlement could not be adjudicated. Every refusal
/// happens BEFORE any fact exists; a refused envelope settles nothing.
#[derive(Debug, Clone, PartialEq)]
pub enum AdjudicationError {
    /// Not the canonical v1 wire shape this surface adjudicates.
    MalformedEnvelope(String),
    /// Right shape, wrong event type (including an OperationalSettlement
    /// delivered to another consume path).
    WrongType { expected: String, got: String },
    /// The payload namespace is missing or is not the canonical namespace.
    WrongNamespace { expected: String, got: String },
    /// T01 boundary failure: exclusive-producer authority (I3), identity
    /// shape/placeholder (ENV-I1/I2), drift against local resolution, or a
    /// malformed causal-parent id (ENV-I4).
    Boundary(ConsumeError),
    /// Required payload fields absent, empty, or of the wrong shape; the
    /// complete missing list is reported together.
    OpaquePayload { missing: Vec<&'static str> },
    /// A required identity/correlation field is blank or a literal
    /// placeholder.
    PlaceholderField { field: String },
    /// `execution_status` outside the governed vocabulary.
    InvalidExecutionStatus { got: String },
    /// `operational_settlement_status` outside the governed vocabulary —
    /// which also refuses any proof-shaped verdict smuggled into the field.
    InvalidSettlementStatus { got: String },
    /// An optional-but-typed field arrived with the wrong shape.
    InvalidOptionalField { field: &'static str },
    /// An evidence reference is empty, whitespace-bearing, or (for
    /// content-addressed refs) not a SHA-256 digest (ENV-I7).
    InvalidEvidenceRef { got: String },
    /// `domain_model_ref.model_hash` disagrees with the declared identity —
    /// upstream facts may be projected but never rewritten (I10).
    RefIdentityMismatch { declared: String, ref_hash: String },
    /// The envelope does not cite the known invocation chain as its causal
    /// parents (ENV-I4): a settlement unbound to its actual execution is
    /// unrepresentable here.
    CausalityMissing {
        expected: Vec<String>,
        recorded: Vec<String>,
    },
    /// Adjudication was invoked without any known chain ids to bind to — a
    /// settlement bound to nothing proves nothing.
    EmptyChainBinding,
    /// The settlement correlates to a different work request than the one
    /// this surface is adjudicating for.
    CrossWiredWorkRequest { expected: String, got: String },
    /// A NEW envelope claims the ALREADY-adjudicated invocation chain
    /// (same work_request_id + authority_decision_id) with different content.
    /// The first settlement stands; mutated re-attestations are refused.
    ConflictingResettlement {
        work_request_id: String,
        authority_decision_id: String,
    },
    /// Durable idempotency-ledger failure (reported as its display form so
    /// refusals stay comparable).
    Ledger(String),
}

impl std::fmt::Display for AdjudicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedEnvelope(m) => write!(f, "malformed canonical envelope: {m}"),
            Self::WrongType { expected, got } => {
                write!(f, "expected {expected} event, got '{got}'")
            }
            Self::WrongNamespace { expected, got } => {
                write!(f, "expected {expected} namespace, got '{got}'")
            }
            Self::Boundary(e) => write!(f, "{e}"),
            Self::OpaquePayload { missing } => write!(
                f,
                "opaque OperationalSettlement payload rejected — missing/invalid fields: {}",
                missing.join(", ")
            ),
            Self::PlaceholderField { field } => {
                write!(f, "{field} is a placeholder or blank identity")
            }
            Self::InvalidExecutionStatus { got } => {
                write!(f, "execution_status outside the governed vocabulary: {got}")
            }
            Self::InvalidSettlementStatus { got } => write!(
                f,
                "operational_settlement_status outside the governed vocabulary: {got}"
            ),
            Self::InvalidOptionalField { field } => {
                write!(f, "optional field {field} has the wrong shape")
            }
            Self::InvalidEvidenceRef { got } => {
                write!(f, "malformed evidence reference: {got:?}")
            }
            Self::RefIdentityMismatch {
                declared,
                ref_hash,
            } => write!(
                f,
                "domain_model_ref names {ref_hash} but the envelope declares {declared}"
            ),
            Self::CausalityMissing {
                expected,
                recorded,
            } => write!(
                f,
                "settlement does not cite the known invocation chain: expected {expected:?}, recorded {recorded:?}"
            ),
            Self::EmptyChainBinding => write!(
                f,
                "adjudication requires at least one known invocation-chain id to bind to"
            ),
            Self::CrossWiredWorkRequest { expected, got } => write!(
                f,
                "cross-wired settlement: adjudicating for work_request_id {expected:?}, envelope carries {got:?}"
            ),
            Self::ConflictingResettlement {
                work_request_id,
                authority_decision_id,
            } => write!(
                f,
                "conflicting re-settlement of invocation chain ({work_request_id}, {authority_decision_id}): first settlement stands"
            ),
            Self::Ledger(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for AdjudicationError {}

impl From<AdmitError> for AdjudicationError {
    fn from(e: AdmitError) -> Self {
        Self::Ledger(e.to_string())
    }
}

/// The operational outcome recorded on the wire (`accepted` | `rejected`).
/// Deliberately its own two-variant type: it is NOT a proof verdict, NOT a
/// developmental settlement, and carries no capability semantics (I6/I7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationalOutcome {
    /// The governed execution satisfied every declared operational criterion.
    Accepted,
    /// The governed execution failed its declared contract (including any
    /// exit-zero run whose criteria were not attested).
    Rejected,
}

/// The operational facts an accepted adjudication yields. Exhaustively
/// operational: no proof result, no proof contract, no capability state, and
/// no developmental consequence lives on this type — consuming it cannot
/// promote anything (I6/I7).
#[derive(Debug, Clone, PartialEq)]
pub struct OperationalSettlementFacts {
    /// The settlement envelope's own stable identity.
    pub settlement_event_id: String,
    pub work_request_id: String,
    /// The authority decision that authorized the settled invocation —
    /// reported from the wire record of the authorized chain.
    pub authority_decision_id: String,
    pub invocation_id: Option<String>,
    pub execution_status: String,
    pub operational_outcome: OperationalOutcome,
    pub observed_effects: Value,
    pub evidence_refs: Vec<String>,
}

/// What happened when an OperationalSettlement reached this surface.
#[derive(Debug, Clone, PartialEq)]
pub enum Adjudication {
    /// Bound to the originating work request and admitted exactly once.
    First(OperationalSettlementFacts),
    /// A redelivery under stable envelope identity (event id or
    /// content-derived idempotency key already durably recorded).
    /// Consequence-free: the first adjudication stands, nothing is rewritten.
    DuplicateDelivery,
}

impl Adjudication {
    /// True only for a first-time adjudication of this settlement.
    pub fn first(&self) -> bool {
        matches!(self, Self::First(_))
    }
}

fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

/// Stable conflict-detection key for one invocation chain. Hex-encoded so it
/// can live in the same durable ledger as envelope identities.
fn chain_key(work_request_id: &str, authority_decision_id: &str) -> String {
    sha256_hex(format!("chain|{work_request_id}|{authority_decision_id}").as_bytes())
}

/// Stable dedupe identity for one envelope event id. The durable ledger
/// records hex-only lines, so event ids are folded through SHA-256 — still a
/// faithful 1:1 identity for duplicate recognition.
fn event_key(event_id: &str) -> String {
    sha256_hex(format!("event|{event_id}").as_bytes())
}

fn meaningful(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn check_identity_string(field: &'static str, value: &str) -> Result<(), AdjudicationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || PLACEHOLDER_IDS.contains(&trimmed.to_ascii_lowercase().as_str()) {
        return Err(AdjudicationError::PlaceholderField {
            field: field.to_string(),
        });
    }
    Ok(())
}

fn validate_evidence_ref(r: &str) -> Result<(), AdjudicationError> {
    let trimmed = r.trim();
    if trimmed.is_empty() || trimmed.contains(char::is_whitespace) || !trimmed.contains(':') {
        return Err(AdjudicationError::InvalidEvidenceRef { got: r.to_string() });
    }
    if let Some(digest) = trimmed.strip_prefix("sha256:") {
        let hex_ok = digest.len() == 64
            && digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if !hex_ok {
            return Err(AdjudicationError::InvalidEvidenceRef { got: r.to_string() });
        }
    }
    Ok(())
}

/// Durable adjudicator over canonical OperationalSettlement envelopes.
///
/// Envelope identities (event ids, content-derived idempotency keys) and
/// invocation-chain keys share one append-only JSONL ledger, so both
/// duplicate recognition and conflicting-resettlement refusal survive
/// restart.
#[derive(Debug)]
pub struct OperationalSettlementAdjudicator {
    ledger: IdempotencyLedger,
}

impl OperationalSettlementAdjudicator {
    /// Open (or create) the durable ledger backing this adjudicator.
    pub fn open(path: &Path) -> Result<Self, AdjudicationError> {
        Ok(Self {
            ledger: IdempotencyLedger::open(path)?,
        })
    }

    /// Number of durably recorded identities (envelope + chain keys).
    pub fn len(&self) -> usize {
        self.ledger.len()
    }

    /// Whether the underlying ledger records no identities.
    pub fn is_empty(&self) -> bool {
        self.ledger.is_empty()
    }

    /// True when this exact identity was already durably recorded.
    pub fn contains_identity(&self, key: &str) -> bool {
        self.ledger.contains(key)
    }

    /// Adjudicate one canonical OperationalSettlement against the ORIGINATING
    /// work request.
    ///
    /// Binding chain — every link mandatory, checked before anything is
    /// recorded:
    /// 1. canonical v1 shape, loop namespace, and the exact event type;
    /// 2. the composed T01 boundary gate: exclusive producer `sea_forge`
    ///    (I3), non-placeholder identity, drift against OUR local resolution,
    ///    well-formed causal-parent ids (ENV-I4);
    /// 3. duplicate/conflict recognition BEFORE any consequence: a redelivery
    ///    under stable identity is [`Adjudication::DuplicateDelivery`]; a new
    ///    envelope claiming an already-adjudicated chain is a
    ///    [`AdjudicationError::ConflictingResettlement`];
    /// 4. every frozen required field parsed and validated, typed optionals
    ///    refused rather than coerced;
    /// 5. causality binding: EVERY supplied known-chain id must appear among
    ///    the recorded `caused_by:` parents;
    /// 6. correlation binding: payload `work_request_id` equals the
    ///    originating work request (ENV-I3);
    /// 7. durable admission of all three identities before [`Adjudication::First`]
    ///    is returned (crash-conservative: worst case a crash after write is
    ///    a duplicate later, never a double settlement).
    pub fn adjudicate(
        &mut self,
        envelope: &Envelope,
        originating_work_request_id: &str,
        expected_chain: &[&str],
        local_model_sha256: &str,
    ) -> Result<Adjudication, AdjudicationError> {
        // 1. Shape, namespace, event type.
        if envelope.schema_version != super::envelope::SCHEMA_VERSION {
            return Err(AdjudicationError::MalformedEnvelope(format!(
                "schema_version must be {:?}",
                super::envelope::SCHEMA_VERSION
            )));
        }
        if envelope.event_type != EVENT_TYPE {
            return Err(AdjudicationError::WrongType {
                expected: EVENT_TYPE.to_string(),
                got: envelope.event_type.clone(),
            });
        }
        let got_ns = envelope.namespace().unwrap_or("");
        if got_ns != NAMESPACE {
            return Err(AdjudicationError::WrongNamespace {
                expected: NAMESPACE.into(),
                got: got_ns.to_string(),
            });
        }

        // 2. Composed T01 boundary gate (producer/identity/drift/parents).
        validate_envelope(envelope, local_model_sha256).map_err(AdjudicationError::Boundary)?;

        // 3. Duplicate / conflicting-resettlement recognition FIRST.
        let event_id = envelope.event_id.clone();
        let idem = envelope.idempotency_key.clone().unwrap_or_default();
        if self.ledger.contains(&event_key(&event_id))
            || (!idem.is_empty() && self.ledger.contains(&idem))
        {
            return Ok(Adjudication::DuplicateDelivery);
        }

        // 4. Payload battery: presence of ALL frozen required fields first.
        let mut missing: Vec<&'static str> = Vec::new();
        for field in REQUIRED_FIELDS {
            let present = match envelope.payload.get(field) {
                None | Some(Value::Null) => false,
                Some(Value::String(s)) => !s.trim().is_empty(),
                Some(_) => true,
            };
            if !present {
                missing.push(field);
            }
        }
        if !missing.is_empty() {
            return Err(AdjudicationError::OpaquePayload { missing });
        }

        let work_request_id = meaningful(
            envelope
                .payload
                .get("work_request_id")
                .and_then(Value::as_str),
        )
        .ok_or(AdjudicationError::OpaquePayload {
            missing: vec!["work_request_id"],
        })?;
        check_identity_string("work_request_id", &work_request_id)?;
        let authority_decision_id = meaningful(
            envelope
                .payload
                .get("authority_decision_id")
                .and_then(Value::as_str),
        )
        .ok_or(AdjudicationError::OpaquePayload {
            missing: vec!["authority_decision_id"],
        })?;
        check_identity_string("authority_decision_id", &authority_decision_id)?;

        let execution_status = match envelope
            .payload
            .get("execution_status")
            .and_then(Value::as_str)
        {
            Some(s) if EXECUTION_STATUSES.contains(&s) => s.to_string(),
            Some(other) => {
                return Err(AdjudicationError::InvalidExecutionStatus {
                    got: other.to_string(),
                })
            }
            None => unreachable!("presence battery"),
        };
        let operational_outcome = match envelope
            .payload
            .get("operational_settlement_status")
            .and_then(Value::as_str)
        {
            Some("accepted") => OperationalOutcome::Accepted,
            Some("rejected") => OperationalOutcome::Rejected,
            Some(other) => {
                return Err(AdjudicationError::InvalidSettlementStatus {
                    got: other.to_string(),
                })
            }
            None => unreachable!("presence battery"),
        };
        let observed_effects = match envelope.payload.get("observed_effects") {
            Some(e @ (Value::Array(_) | Value::Object(_))) => e.clone(),
            _ => {
                return Err(AdjudicationError::OpaquePayload {
                    missing: vec!["observed_effects"],
                })
            }
        };
        let evidence_refs: Vec<String> = match envelope.payload.get("evidence_refs") {
            Some(Value::Array(items)) if !items.is_empty() => {
                let mut refs = Vec::with_capacity(items.len());
                for item in items {
                    match item.as_str() {
                        Some(s) => {
                            validate_evidence_ref(s)?;
                            refs.push(s.to_string());
                        }
                        None => {
                            return Err(AdjudicationError::InvalidEvidenceRef {
                                got: "<non-string ref>".into(),
                            })
                        }
                    }
                }
                refs
            }
            _ => {
                return Err(AdjudicationError::OpaquePayload {
                    missing: vec!["evidence_refs"],
                })
            }
        };

        let declared = envelope.domain_model_hash().unwrap_or_default();
        let model_ref = envelope
            .payload
            .get("domain_model_ref")
            .and_then(Value::as_object)
            .ok_or(AdjudicationError::OpaquePayload {
                missing: vec!["domain_model_ref"],
            })?;
        if model_ref.get("namespace").and_then(Value::as_str) != Some(NAMESPACE) {
            return Err(AdjudicationError::WrongNamespace {
                expected: NAMESPACE.into(),
                got: model_ref
                    .get("namespace")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            });
        }
        let ref_hash = model_ref
            .get("model_hash")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if ref_hash != declared {
            return Err(AdjudicationError::RefIdentityMismatch {
                declared: declared.to_string(),
                ref_hash: ref_hash.to_string(),
            });
        }

        // Typed optional fields: refused, not coerced.
        for field in ["case_id", "run_id", "transcript_ref"] {
            if let Some(v) = envelope.payload.get(field) {
                if v.as_str().is_none() {
                    return Err(AdjudicationError::InvalidOptionalField { field });
                }
            }
        }
        if let Some(v) = envelope.payload.get("artifact_refs") {
            if v.as_array().is_none() {
                return Err(AdjudicationError::InvalidOptionalField {
                    field: "artifact_refs",
                });
            }
        }
        if let Some(v) = envelope.payload.get("failure_reason") {
            if !matches!(v, Value::String(_) | Value::Object(_)) {
                return Err(AdjudicationError::InvalidOptionalField {
                    field: "failure_reason",
                });
            }
        }
        let invocation_id = meaningful(
            envelope
                .payload
                .get("invocation_id")
                .and_then(Value::as_str),
        );

        // Conflict recognition needs the chain key; it fires only when the
        // ENVELOPE identity is fresh but the CHAIN was already settled.
        let key = chain_key(&work_request_id, &authority_decision_id);
        if self.ledger.contains(&key) {
            return Err(AdjudicationError::ConflictingResettlement {
                work_request_id,
                authority_decision_id,
            });
        }

        // 5. Causality binding to the KNOWN invocation chain (ENV-I4).
        if expected_chain.is_empty() {
            return Err(AdjudicationError::EmptyChainBinding);
        }
        let recorded = envelope.causal_parents();
        let unbound: Vec<String> = expected_chain
            .iter()
            .filter(|e| !recorded.contains(e))
            .map(|e| e.to_string())
            .collect();
        if !unbound.is_empty() {
            return Err(AdjudicationError::CausalityMissing {
                expected: unbound,
                recorded: recorded.iter().map(|s| s.to_string()).collect(),
            });
        }

        // 6. Correlation binding to the ORIGINATING work request (ENV-I3).
        if work_request_id != originating_work_request_id.trim() {
            return Err(AdjudicationError::CrossWiredWorkRequest {
                expected: originating_work_request_id.trim().to_string(),
                got: work_request_id,
            });
        }

        // 7. Durable admission — all three identities, then report facts.
        for identity in [event_key(&event_id), idem.clone(), key] {
            if identity.is_empty() {
                continue;
            }
            self.ledger.admit(&identity)?;
        }

        Ok(Adjudication::First(OperationalSettlementFacts {
            settlement_event_id: event_id,
            work_request_id,
            authority_decision_id,
            invocation_id,
            execution_status,
            operational_outcome,
            observed_effects,
            evidence_refs,
        }))
    }
}
