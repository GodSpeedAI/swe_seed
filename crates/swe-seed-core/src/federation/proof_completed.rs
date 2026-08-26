//! Canonical ProofCompleted emission (convergence plan task T07; frozen edge
//! E7). SWE_SEED is the exclusive authoritative producer of ProofCompleted;
//! RealityTrace (sxr) is its consumer.
//!
//! The frozen contract: proof success REFERENCES operational evidence rather
//! than rewriting SEA-Forge's observation history. This surface therefore
//! emits ONLY against a REAL adjudicated settlement — the caller supplies the
//! actual [`OperationalSettlement`] envelope plus the [`OperationalSettlementFacts`]
//! returned by [`OperationalSettlementAdjudicator::adjudicate`] for it, and
//! every projected field is re-checked here (T04 debt D1: validate EVERY
//! projected field). `operational_settlement_ref` is CONTENT-ADDRESSED
//! (ENV-I7 preferred form): the SHA-256 of the settlement envelope's canonical
//! JSON, so the reference resolves to immutable bytes instead of a mutable
//! path. A forged, stale, foreign-cycle, or fabricated settlement cannot pass:
//! the facts must name this envelope's event id, agree with its correlation,
//! and survive the composed T01 boundary gate under OUR model identity.
//!
//! Causality (ENV-I4) is mandatory: the settlement event id is always a
//! `caused_by:` parent, plus any additional upstream envelopes supplied (e.g.
//! the E4 GovernedWorkRequest) — each independently boundary-validated and
//! correlation-bound by [`derive_event`]. `expected_outcome` is carried
//! VERBATIM (immutable projection): the declared expectation travels to
//! RealityTrace byte-exactly so the later expected-versus-observed comparison
//! (T08) binds to what was actually declared, never to a downstream rewrite.
//!
//! The spec-0011-era [`super::emit::emit_proof_completed`] remains untouched;
//! this is the canonical E7 surface built alongside it.

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use super::consume::{validate_envelope, ConsumeError};
use super::envelope::{derive_event, Envelope, NAMESPACE};
use super::governed_submission::ProofContract;
use super::identity::VerifiedDomainIdentity;
use super::operational_settlement::OperationalSettlementFacts;

/// The event type owned exclusively by `swe_seed` at edge E7.
pub const EVENT_TYPE: &str = "ProofCompleted";

/// The frozen required payload fields of a ProofCompleted.
pub const REQUIRED_FIELDS: [&str; 7] = [
    "work_request_id",
    "proof_result_id",
    "proof_contract",
    "proof_status",
    "expected_outcome",
    "operational_settlement_ref",
    "domain_model_ref",
];

/// Governed proof-status vocabulary. `passed`/`failed` report the PROOF
/// stage's own verdict; neither implies operational settlement nor
/// developmental consequence (preregistration settlement_semantics.proof).
pub const PROOF_STATUSES: [&str; 2] = ["passed", "failed"];

/// Explicit placeholder identities that must never bind a proof completion.
const PLACEHOLDER_IDS: [&str; 5] = ["placeholder", "unknown", "none", "<missing>", "tbd"];

/// Why a ProofCompleted could not be emitted. Every refusal happens BEFORE
/// any canonical envelope exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmissionError {
    /// The supplied facts/envelope pair does not describe a REAL adjudicated
    /// settlement: unknown event id pairing, boundary failure, or a settlement
    /// envelope that fails the canonical gate under our identity.
    SettlementNotAdjudicated(String),
    /// The settlement (or an additional parent) correlates to a different
    /// work request than the one the proof completes.
    CrossWiredWorkRequest { expected: String, got: String },
    /// A required semantic input is absent, blank, or a placeholder.
    IncompleteProof { field: &'static str },
    /// `proof_status` outside the governed vocabulary.
    InvalidProofStatus { got: String },
    /// An optional-but-typed field arrived with the wrong shape.
    InvalidOptionalField { field: &'static str },
    /// An evidence-style reference is empty, whitespace-bearing, malformed,
    /// or (for content-addressed refs) not a SHA-256 digest (ENV-I7).
    InvalidRef { got: String },
    /// An upstream envelope failed the canonical boundary gate.
    Boundary(ConsumeError),
    /// Derivation failed (causality/correlation mechanics).
    Derive(super::DeriveError),
}

impl std::fmt::Display for EmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SettlementNotAdjudicated(reason) => write!(
                f,
                "operational_settlement_ref does not resolve to the real adjudicated settlement: {reason}"
            ),
            Self::CrossWiredWorkRequest { expected, got } => write!(
                f,
                "cross-wired proof completion: work_request_id expected {expected:?}, settlement carries {got:?}"
            ),
            Self::IncompleteProof { field } => write!(
                f,
                "proof completion missing semantic field '{field}'"
            ),
            Self::InvalidProofStatus { got } => write!(
                f,
                "proof_status outside the governed vocabulary: {got}"
            ),
            Self::InvalidOptionalField { field } => {
                write!(f, "optional field {field} has the wrong shape")
            }
            Self::InvalidRef { got } => write!(f, "malformed reference: {got:?}"),
            Self::Boundary(e) => write!(f, "upstream envelope rejected at boundary: {e}"),
            Self::Derive(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for EmissionError {}

/// Compact JSON with recursively sorted keys — the same canonicalization the
/// sxr ingestion side applies, so content references computed here resolve
/// there byte-identically.
fn canonical_json(value: &Value) -> String {
    fn sort(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                let mut out = Map::new();
                for key in keys {
                    out.insert(key.clone(), sort(&map[key]));
                }
                Value::Object(out)
            }
            Value::Array(items) => Value::Array(items.iter().map(sort).collect()),
            other => other.clone(),
        }
    }
    serde_json::to_string(&sort(value)).unwrap_or_default()
}

fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

/// The ENV-I7 content-addressed operational-settlement reference: the SHA-256
/// of the settlement envelope's canonical JSON. Immutable by construction —
/// the bytes it names ARE the adjudicated settlement, not a path to whatever
/// currently sits somewhere.
pub fn operational_settlement_ref(settlement: &Envelope) -> String {
    let wire = serde_json::to_value(settlement).unwrap_or(Value::Null);
    format!("sha256:{}", sha256_hex(canonical_json(&wire).as_bytes()))
}

fn check_identity_string(field: &'static str, value: &str) -> Result<(), EmissionError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || PLACEHOLDER_IDS.contains(&trimmed.to_ascii_lowercase().as_str()) {
        return Err(EmissionError::IncompleteProof { field });
    }
    Ok(())
}

/// Validate one evidence-style reference: non-empty, whitespace-free, scheme
/// shaped (`scheme:value`), and — when content-addressed — a real SHA-256
/// digest (ENV-I7).
fn validate_ref(r: &str) -> Result<(), EmissionError> {
    let trimmed = r.trim();
    if trimmed.is_empty() || trimmed.contains(char::is_whitespace) || !trimmed.contains(':') {
        return Err(EmissionError::InvalidRef { got: r.to_string() });
    }
    if let Some(digest) = trimmed.strip_prefix("sha256:") {
        let hex_ok = digest.len() == 64
            && digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if !hex_ok {
            return Err(EmissionError::InvalidRef { got: r.to_string() });
        }
    }
    Ok(())
}

fn validate_string_array(field: &'static str, value: &Value) -> Result<(), EmissionError> {
    let items = value
        .as_array()
        .ok_or(EmissionError::InvalidOptionalField { field })?;
    for item in items {
        match item.as_str() {
            Some(s) => validate_ref(s)?,
            None => return Err(EmissionError::InvalidOptionalField { field }),
        }
    }
    Ok(())
}

/// Everything frozen E7 requires, as typed inputs so an incomplete proof
/// completion is unrepresentable at the call site.
pub struct ProofCompletion<'a> {
    /// The work request this proof completes — the frozen cycle correlation.
    pub originating_work_request_id: &'a str,
    pub proof_result_id: &'a str,
    /// The SWE_SEED-owned proof obligation declared at submission time (E4).
    pub proof_contract: &'a ProofContract,
    /// `passed` | `failed` ([`PROOF_STATUSES`]).
    pub proof_status: &'a str,
    /// The declared expectation, carried VERBATIM (immutable projection).
    pub expected_outcome: &'a Value,
    pub artifact_refs: Option<&'a Value>,
    pub trace_root: Option<&'a str>,
    pub output_ref: Option<&'a str>,
    pub proof_evidence_refs: Option<&'a Value>,
}

/// Emit the canonical `ProofCompleted` envelope from a REAL adjudicated
/// settlement.
///
/// Binding chain — every link mandatory, checked before anything exists:
/// 1. the settlement envelope passes the composed T01 boundary gate under OUR
///    model identity (exclusive `sea_forge` producer, non-placeholder
///    identity, no drift);
/// 2. the supplied facts name THIS settlement's event id and agree with its
///    payload correlation — facts paired with the wrong envelope are refused;
/// 3. correlation binding: facts ↔ settlement ↔ requested work request all
///    equal (ENV-I3);
/// 4. every frozen required field validated, optional fields typed or
///    refused (T04 debt D1/D2: nothing projected unvalidated);
/// 5. derivation records the settlement (always) plus validated additional
///    parents as `caused_by:` causality and pins the correlation (ENV-I4/I3).
pub fn emit_proof_completed_verified(
    settlement: &Envelope,
    facts: &OperationalSettlementFacts,
    additional_parents: &[&Envelope],
    completion: ProofCompletion<'_>,
    identity: &VerifiedDomainIdentity,
) -> Result<Envelope, EmissionError> {
    // 1. The settlement must itself be canonical under OUR identity.
    validate_envelope(settlement, identity.as_str())
        .map_err(|e| EmissionError::SettlementNotAdjudicated(e.to_string()))?;
    if settlement.event_type != super::operational_settlement::EVENT_TYPE {
        return Err(EmissionError::SettlementNotAdjudicated(format!(
            "expected {} envelope, got '{}'",
            super::operational_settlement::EVENT_TYPE,
            settlement.event_type
        )));
    }

    // 2. Facts must describe THIS adjudicated settlement — a fabricated or
    //    mismatched pairing is exactly the forged-reference falsifier.
    if facts.settlement_event_id != settlement.event_id {
        return Err(EmissionError::SettlementNotAdjudicated(format!(
            "facts name settlement {}, supplied envelope is {}",
            facts.settlement_event_id, settlement.event_id
        )));
    }

    // 3. Correlation binding across facts / settlement / request (ENV-I3).
    let work_request_id = completion.originating_work_request_id.trim();
    check_identity_string("work_request_id", work_request_id)?;
    let settlement_wr = settlement.work_request_id().unwrap_or_default();
    if facts.work_request_id != settlement_wr {
        return Err(EmissionError::CrossWiredWorkRequest {
            expected: facts.work_request_id.clone(),
            got: settlement_wr.to_string(),
        });
    }
    if settlement_wr != work_request_id {
        return Err(EmissionError::CrossWiredWorkRequest {
            expected: work_request_id.to_string(),
            got: settlement_wr.to_string(),
        });
    }

    // 4. Required-field battery.
    let proof_result_id = completion.proof_result_id.trim();
    check_identity_string("proof_result_id", proof_result_id)?;
    match completion.proof_status.trim() {
        s if PROOF_STATUSES.contains(&s) => {}
        other => {
            return Err(EmissionError::InvalidProofStatus {
                got: other.to_string(),
            })
        }
    }
    let criterion = completion.proof_contract.criterion.trim();
    if criterion.is_empty() || PLACEHOLDER_IDS.contains(&criterion.to_ascii_lowercase().as_str()) {
        return Err(EmissionError::IncompleteProof {
            field: "proof_contract.criterion",
        });
    }
    if let Some(cmd) = &completion.proof_contract.command {
        if cmd.trim().is_empty() {
            return Err(EmissionError::IncompleteProof {
                field: "proof_contract.command",
            });
        }
    }
    if let Some(pt) = &completion.proof_contract.proof_type {
        if pt.trim().is_empty() {
            return Err(EmissionError::IncompleteProof {
                field: "proof_contract.proof_type",
            });
        }
    }
    if matches!(completion.expected_outcome, Value::Null) {
        return Err(EmissionError::IncompleteProof {
            field: "expected_outcome",
        });
    }

    // Optional-but-typed fields: refused, not coerced.
    if let Some(v) = completion.artifact_refs {
        validate_string_array("artifact_refs", v)?;
    }
    for (field, value) in [
        ("trace_root", completion.trace_root),
        ("output_ref", completion.output_ref),
    ] {
        if let Some(v) = value {
            if v.trim().is_empty() || v.contains(char::is_whitespace) {
                return Err(EmissionError::InvalidOptionalField { field });
            }
        }
    }
    if let Some(v) = completion.proof_evidence_refs {
        validate_string_array("proof_evidence_refs", v)?;
    }

    // 5. Frozen E7 payload. The settlement reference is computed from the
    //    adjudicated envelope's own canonical bytes; `expected_outcome` is
    //    inserted verbatim.
    let mut payload = Map::new();
    payload.insert(
        "work_request_id".into(),
        Value::String(work_request_id.to_string()),
    );
    payload.insert(
        "proof_result_id".into(),
        Value::String(proof_result_id.to_string()),
    );
    payload.insert(
        "proof_contract".into(),
        json!({
            "criterion": completion.proof_contract.criterion,
            "command": completion.proof_contract.command,
            "proof_type": completion.proof_contract.proof_type,
        }),
    );
    payload.insert(
        "proof_status".into(),
        Value::String(completion.proof_status.trim().to_string()),
    );
    payload.insert(
        "expected_outcome".into(),
        completion.expected_outcome.clone(),
    );
    payload.insert(
        "operational_settlement_ref".into(),
        Value::String(operational_settlement_ref(settlement)),
    );
    payload.insert(
        "domain_model_ref".into(),
        json!({"namespace": NAMESPACE, "model_hash": identity.as_str()}),
    );
    if let Some(v) = completion.artifact_refs {
        payload.insert("artifact_refs".into(), v.clone());
    }
    if let Some(v) = completion.trace_root.map(str::to_string) {
        payload.insert("trace_root".into(), Value::String(v));
    }
    if let Some(v) = completion.output_ref.map(str::to_string) {
        payload.insert("output_ref".into(), Value::String(v));
    }
    if let Some(v) = completion.proof_evidence_refs {
        payload.insert("proof_evidence_refs".into(), v.clone());
    }

    // Additional upstream parents are boundary-validated before derivation.
    for parent in additional_parents {
        validate_envelope(parent, identity.as_str()).map_err(EmissionError::Boundary)?;
        if let Some(w) = parent.work_request_id() {
            if w != work_request_id {
                return Err(EmissionError::CrossWiredWorkRequest {
                    expected: work_request_id.to_string(),
                    got: w.to_string(),
                });
            }
        }
    }

    let mut parents: Vec<&Envelope> = Vec::with_capacity(1 + additional_parents.len());
    parents.push(settlement);
    parents.extend_from_slice(additional_parents);

    // 6. Derivation records causality and pins the frozen correlation via the
    //    substrate's own mismatch refusal (ENV-I4/I3).
    derive_event(
        &parents,
        EVENT_TYPE,
        payload,
        identity.as_str(),
        Some(work_request_id),
    )
    .map_err(EmissionError::Derive)
}
