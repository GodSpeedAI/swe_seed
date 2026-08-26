//! Canonical GovernedWorkRequest emission (convergence plan task T04; frozen
//! edge E4). SWE_SEED is the exclusive authoritative producer of
//! GovernedWorkRequest; SEA-Forge is its consumer.
//!
//! The frozen contract: the request carries semantic work intent plus
//! governance context — never merely an opaque command — and binds work
//! identity (work_request_id), affordance, actor, cited context, canonical
//! domain identity, proof contract, and settlement criteria into one
//! derivation whose causal parents are the upstream WorkRequested (E1) and
//! ContextPacketCreated (E3) envelopes. The proof contract remains a distinct
//! fact from the operational settlement criteria (preregistration I6 starts
//! here): they are separate typed values emitted under separate payload keys.

use serde_json::{json, Map, Value};

use super::consume::validate_envelope;
use super::envelope::{derive_event, Envelope, NAMESPACE};
use super::identity::VerifiedDomainIdentity;
use super::work_ingress::WorkRequestContract;

/// Why a governed submission could not be emitted. Every variant is a frozen
/// falsifier refused BEFORE any canonical envelope exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionError {
    /// The referenced context packet correlates to a different work request
    /// (frozen falsifier: cross-wired context never reaches SEA-Forge).
    CrossWiredContextPacket { expected: String, got: String },
    /// The accepted work contract and the supplied WorkRequested envelope
    /// disagree on the cycle correlation.
    ContractEnvelopeMismatch { expected: String, got: String },
    /// A required semantic field of the submission is absent or empty — an
    /// opaque command-only request cannot be built through this surface.
    IncompleteIntent { field: &'static str },
    /// Upstream envelope failed the canonical boundary gate (producer
    /// authority, identity shape, drift, or causality records).
    Boundary(super::ConsumeError),
    /// Derivation failed (causality/correlation mechanics).
    Derive(super::DeriveError),
}

impl std::fmt::Display for SubmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CrossWiredContextPacket { expected, got } => write!(
                f,
                "cross-wired context packet: work_request_id expected {expected:?}, packet carries {got:?}"
            ),
            Self::ContractEnvelopeMismatch { expected, got } => write!(
                f,
                "work contract {expected:?} does not match WorkRequested envelope {got:?}"
            ),
            Self::IncompleteIntent { field } => write!(
                f,
                "governed submission missing semantic intent field '{field}' — opaque commands cannot be submitted"
            ),
            Self::Boundary(e) => write!(f, "upstream envelope rejected at boundary: {e}"),
            Self::Derive(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for SubmissionError {}

/// The SWE_SEED-owned proof obligation. Deliberately its own type — it can
/// never be confused with, collapsed into, or substituted by the route's
/// settlement criteria (frozen E4: distinct facts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofContract {
    /// The declared proof criterion the SWE_SEED proof stage must satisfy.
    pub criterion: String,
    /// Optional reference to the proof command/route that will carry it out.
    pub command: Option<String>,
    /// `live` or `simulation` (simulation never activates downstream).
    pub proof_type: Option<String>,
}

/// One governed submission: everything frozen E4 requires, as typed inputs so
/// an incomplete request is unrepresentable at the call site.
pub struct GovernedSubmission<'a> {
    /// The spendable work contract accepted at the T03 ingress.
    pub contract: &'a WorkRequestContract,
    pub actor_id: &'a str,
    pub actor_role: Option<&'a str>,
    /// The semantic work intent — what SEA-Forge governs, not an argv blob.
    pub intent: &'a str,
    pub proof_contract: &'a ProofContract,
    pub route_id: Option<&'a str>,
    pub artifact_expectations: Option<&'a Value>,
    pub authority_context: Option<&'a Value>,
    pub payment_budget: Option<&'a Value>,
}

fn require_str(value: Option<&str>, field: &'static str) -> Result<String, SubmissionError> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or(SubmissionError::IncompleteIntent { field })
}

/// The id a governed request references its context packet by: the packet's
/// `context_packet_id` when present (the CK egress contract), else the packet
/// event id itself.
pub fn context_packet_ref(context_packet: &Envelope) -> Option<String> {
    context_packet
        .payload
        .get("context_packet_id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| Some(context_packet.event_id.clone()))
}

/// Build the canonical `GovernedWorkRequest` envelope from the accepted E1
/// work contract and its adjudicated E3 context packet.
///
/// Both parent envelopes must pass the composed boundary gate against the
/// SAME verified domain identity the request will declare (producer authority:
/// godspeed_agent owns E1, context_kernel owns E3, swe_seed stamps E4), the
/// packet must correlate to the same work_request_id, and every required
/// semantic field must be present. Construction goes through
/// [`derive_event`], so causality (ENV-I4) and correlation stability
/// (ENV-I3) are recorded by the substrate rather than by hand.
pub fn build_governed_work_request(
    work_requested: &Envelope,
    context_packet: &Envelope,
    sub: GovernedSubmission<'_>,
    identity: &VerifiedDomainIdentity,
) -> Result<Envelope, SubmissionError> {
    // 1. Both causal parents must already be boundary-clean under OUR model
    //    identity: exclusive producers, non-placeholder identity, no drift.
    validate_envelope(work_requested, identity.as_str()).map_err(SubmissionError::Boundary)?;
    validate_envelope(context_packet, identity.as_str()).map_err(SubmissionError::Boundary)?;

    // 2. Correlation binding: contract, E1 envelope, and E3 packet must all
    //    name the same work_request_id before anything is derived.
    let work_request_id = sub.contract.work_request_id.as_str();
    if let Some(got) = work_requested.work_request_id() {
        if got != work_request_id {
            return Err(SubmissionError::ContractEnvelopeMismatch {
                expected: work_request_id.to_string(),
                got: got.to_string(),
            });
        }
    }
    let packet_wr = context_packet.work_request_id().unwrap_or("");
    if packet_wr != work_request_id {
        return Err(SubmissionError::CrossWiredContextPacket {
            expected: work_request_id.to_string(),
            got: packet_wr.to_string(),
        });
    }

    // 3. Semantic completeness: an opaque command-only request cannot pass.
    let intent = require_str(Some(sub.intent), "intent")?;
    let actor_id = require_str(Some(sub.actor_id), "actor.actor_id")?;
    require_str(
        Some(sub.proof_contract.criterion.as_str()),
        "proof_contract.criterion",
    )?;
    let packet_ref = require_str(
        Some(
            context_packet_ref(context_packet)
                .as_deref()
                .unwrap_or_default(),
        ),
        "context_packet_ref",
    )?;

    // 4. Frozen E4 payload: all eight required fields, optional governance
    //    fields passed through untouched when supplied.
    let mut payload = Map::new();
    payload.insert(
        "work_request_id".into(),
        Value::String(work_request_id.to_string()),
    );
    payload.insert(
        "affordance_id".into(),
        Value::String(sub.contract.affordance_id.clone()),
    );
    let mut actor = Map::new();
    actor.insert("actor_id".into(), Value::String(actor_id));
    if let Some(role) = sub.actor_role.map(str::to_string) {
        actor.insert("role".into(), Value::String(role));
    }
    payload.insert("actor".into(), Value::Object(actor));
    payload.insert("intent".into(), Value::String(intent));
    payload.insert("context_packet_ref".into(), Value::String(packet_ref));
    // Same canonical DomainModelRef the upstream edges used — realized in the
    // v1 wire family by make_event's `domain_model_hash` injection below, and
    // named explicitly here per the frozen payload contract.
    payload.insert(
        "domain_model_ref".into(),
        json!({"namespace": NAMESPACE, "model_hash": identity.as_str()}),
    );
    payload.insert(
        "proof_contract".into(),
        json!({
            "criterion": sub.proof_contract.criterion,
            "command": sub.proof_contract.command,
            "proof_type": sub.proof_contract.proof_type,
        }),
    );
    payload.insert(
        "settlement_criteria".into(),
        Value::Array(
            sub.contract
                .settlement_criteria
                .iter()
                .cloned()
                .map(Value::String)
                .collect(),
        ),
    );
    if let Some(route_id) = sub.route_id {
        payload.insert("route_id".into(), Value::String(route_id.to_string()));
    }
    if let Some(v) = sub.artifact_expectations {
        payload.insert("artifact_expectations".into(), v.clone());
    }
    if let Some(v) = sub.authority_context {
        payload.insert("authority_context".into(), v.clone());
    }
    if let Some(v) = sub.payment_budget {
        payload.insert("payment_budget".into(), v.clone());
    }

    // 5. Derivation records both parents (`caused_by:`) and pins the frozen
    //    correlation via the substrate's own mismatch refusal (ENV-I4/I3).
    derive_event(
        &[work_requested, context_packet],
        "GovernedWorkRequest",
        payload,
        identity.as_str(),
        Some(work_request_id),
    )
    .map_err(SubmissionError::Derive)
}
