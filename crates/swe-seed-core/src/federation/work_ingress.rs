//! Canonical WorkRequested ingress (convergence plan task T03; frozen edge E1).
//!
//! GodSpeed-Agent is the exclusive authoritative producer of WorkRequested.
//! A destination-only input — no affordance identity, no desired outcome, or
//! no settlement criteria — can never settle as a canonical work contract, and
//! a domain-identity cross-wire is rejected before SWE_SEED accepts anything.

use serde_json::Value;

use super::consume::{check_drift, ConsumeError};
use super::envelope::Envelope;
use super::identity;
use super::producers::validate_producer;
use super::world::WorldRef;

/// The parsed spendable work contract carried by a canonical E1 envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRequestContract {
    pub work_request_id: String,
    pub affordance_id: String,
    pub desired_outcome: String,
    pub settlement_criteria: Vec<String>,
    /// The semantic world the request is pinned to. Every downstream artifact
    /// of this cycle must name this same world.
    pub world_ref: WorldRef,
}

/// Accept one canonical `WorkRequested` envelope at the SWE_SEED boundary:
///
/// 1. producer authority — only `godspeed_agent` may emit WorkRequested (I3);
/// 2. canonical identity — declared hash must be a real digest, not a
///    placeholder (ENV-I2), and must match the local resolution (drift);
/// 3. semantic world — `world_ref` is required and well-formed;
/// 4. contract completeness — work_request_id correlation, affordance_id,
///    desired_outcome, and non-empty settlement_criteria are all required
///    (frozen falsifier: destination-only inputs are rejected).
pub fn accept_work_requested(
    envelope: &Envelope,
    expected_hash: &str,
) -> Result<WorkRequestContract, ConsumeError> {
    validate_producer(envelope).map_err(ConsumeError::ProducerAuthority)?;

    let declared = envelope.domain_model_hash().ok_or_else(|| {
        ConsumeError::Identity(identity::IdentityError::MalformedHash {
            got: "<missing>".into(),
        })
    })?;
    identity::verify_declared_hash(declared).map_err(ConsumeError::Identity)?;
    check_drift(envelope, expected_hash)?;

    // Semantic world: required and well-formed (syntax only; SEA-Forge
    // verifies the digest). An alias or label is not a world identity.
    let world_ref = envelope
        .verified_world_ref()
        .map_err(ConsumeError::World)?;

    let require_str = |field: &str| -> Result<&str, ConsumeError> {
        envelope
            .payload
            .get(field)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or(ConsumeError::DestinationOnly {
                field: field.to_string(),
            })
    };

    let work_request_id = require_str("work_request_id")?.to_string();
    let affordance_id = require_str("affordance_id")?.to_string();
    let desired_outcome = require_str("desired_outcome")?.to_string();
    let settlement_criteria: Vec<String> = envelope
        .payload
        .get("settlement_criteria")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .filter(|s| !s.trim().is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    if settlement_criteria.is_empty() {
        return Err(ConsumeError::DestinationOnly {
            field: "settlement_criteria".to_string(),
        });
    }

    // Correlation stability: the payload's work_request_id IS the cycle
    // identity this contract will carry downstream; it must be present (it
    // doubles as the idempotency correlation upstream).
    Ok(WorkRequestContract {
        work_request_id,
        affordance_id,
        desired_outcome,
        settlement_criteria,
        world_ref,
    })
}
