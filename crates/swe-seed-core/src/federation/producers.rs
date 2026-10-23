//! Exclusive event-producer authority (convergence plan task T01; frozen
//! invariant I3). The producer of each canonical event type is fixed by the
//! preregistration's edge topology (`E0`–`E10`, plus the SWE_SEED-internal
//! plane events that today have exactly one producer). Validation is
//! machine-observable and fail-closed: an unknown event type or a forged
//! source agent is rejected, never coerced.

use super::envelope::Envelope;

/// Canonical component ids, exactly the preregistration's scope vocabulary.
pub mod agent_id {
    pub const EXTERNAL_ENVIRONMENT: &str = "external_environment";
    pub const GODSPEED_AGENT: &str = "godspeed_agent";
    pub const SWE_SEED: &str = "swe_seed";
    pub const CONTEXT_KERNEL: &str = "context_kernel";
    pub const SEA_FORGE: &str = "sea_forge";
    pub const REALITYTRACE: &str = "realitytrace";
    pub const MEMORY_LEDGER: &str = "memory_ledger";
    pub const EXECUTION_ENVIRONMENT: &str = "execution_environment";
}

/// Map a wire `source_agent` string onto its canonical component id. The
/// hyphenated spellings are the pre-existing SWE_SEED stamp (`SOURCE_AGENT`)
/// and its siblings; the underscored spellings are the frozen preregistration
/// vocabulary. Unknown agents are `None` — fail closed.
pub fn canonical_agent(source_agent: &str) -> Option<&'static str> {
    match source_agent {
        "swe-seed" | "swe_seed" => Some(agent_id::SWE_SEED),
        "context-kernel" | "context_kernel" => Some(agent_id::CONTEXT_KERNEL),
        "sea-forge" | "sea_forge" => Some(agent_id::SEA_FORGE),
        "godspeed-agent" | "godspeed_agent" | "gsa" => Some(agent_id::GODSPEED_AGENT),
        "realitytrace" | "reality-trace" | "sxr" => Some(agent_id::REALITYTRACE),
        "memory-ledger" | "memory_ledger" | "agent_memory_ledger" => Some(agent_id::MEMORY_LEDGER),
        "execution-environment" | "execution_environment" => Some(agent_id::EXECUTION_ENVIRONMENT),
        "external-environment" | "external_environment" => Some(agent_id::EXTERNAL_ENVIRONMENT),
        _ => None,
    }
}

/// The authoritative producer of a canonical event type, per the frozen edge
/// topology. `None` for event types outside the canonical loop (they are
/// rejected by [`validate_producer`] rather than guessed at).
pub fn authoritative_producer(event_type: &str) -> Option<&'static str> {
    match event_type {
        // E0 external_environment -> godspeed_agent
        "DesiredDirection" => Some(agent_id::EXTERNAL_ENVIRONMENT),
        // E1 godspeed_agent -> swe_seed
        "WorkRequested" => Some(agent_id::GODSPEED_AGENT),
        // E2 swe_seed -> context_kernel
        "ContextRequired" => Some(agent_id::SWE_SEED),
        // E3 context_kernel -> swe_seed
        "ContextPacketCreated" => Some(agent_id::CONTEXT_KERNEL),
        // E4 swe_seed -> sea_forge
        "GovernedWorkRequest" => Some(agent_id::SWE_SEED),
        // E5A sea_forge -> execution_environment
        "AuthorizedInvocation" => Some(agent_id::SEA_FORGE),
        // E5B execution_environment -> sea_forge
        "ExecutionObservation" => Some(agent_id::EXECUTION_ENVIRONMENT),
        // E6 sea_forge -> swe_seed
        "OperationalSettlement" => Some(agent_id::SEA_FORGE),
        // E7 swe_seed -> realitytrace
        "ProofCompleted" => Some(agent_id::SWE_SEED),
        // E8 realitytrace -> godspeed_agent
        "EvidenceRecorded" => Some(agent_id::REALITYTRACE),
        // E9 godspeed_agent -> memory_ledger (five frozen event types)
        "SettlementRecorded"
        | "CapabilityUpdated"
        | "RepetitionPlanned"
        | "LearningProposalCreated"
        | "CoherenceBreakDetected" => Some(agent_id::GODSPEED_AGENT),
        // E10 memory_ledger -> godspeed_agent
        "DevelopmentalMemory" => Some(agent_id::MEMORY_LEDGER),
        // Authority decisions are SEA-Forge's to declare.
        "AuthorityChecked" => Some(agent_id::SEA_FORGE),
        // SWE_SEED harness-plane events (route/proof lifecycle) are produced
        // by swe_seed today; the convergence plan freezes their ownership via
        // this registry until an edge reassigns it.
        "RouteSelected" | "ProofStarted" => Some(agent_id::SWE_SEED),
        _ => None,
    }
}

/// Why producer validation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProducerAuthorityError {
    /// Event type outside the frozen topology / registry.
    UnknownEventType { got: String },
    /// A known event type emitted by any component other than its
    /// authoritative producer.
    NotAuthoritative {
        event_type: String,
        authoritative: &'static str,
        got: String,
    },
    /// `source_agent` missing or not in the canonical vocabulary.
    UnknownAgent { got: String },
}

impl std::fmt::Display for ProducerAuthorityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownEventType { got } => {
                write!(
                    f,
                    "event type outside the canonical producer registry: {got}"
                )
            }
            Self::NotAuthoritative {
                event_type,
                authoritative,
                got,
            } => write!(
                f,
                "{event_type} may only be produced by '{authoritative}', forge attempted by '{got}'"
            ),
            Self::UnknownAgent { got } => {
                write!(f, "source_agent outside the canonical vocabulary: {got}")
            }
        }
    }
}

impl std::error::Error for ProducerAuthorityError {}

/// Validate that `envelope.source_agent` is the exclusive authoritative
/// producer of `envelope.event_type`. This is the machine-observable form of
/// invariant I3 used by every T01+ consume path.
pub fn validate_producer(envelope: &Envelope) -> Result<(), ProducerAuthorityError> {
    let Some(authoritative) = authoritative_producer(&envelope.event_type) else {
        return Err(ProducerAuthorityError::UnknownEventType {
            got: envelope.event_type.clone(),
        });
    };
    let got = envelope.source_agent.as_str();
    let Some(canonical) = canonical_agent(got) else {
        return Err(ProducerAuthorityError::UnknownAgent {
            got: got.to_string(),
        });
    };
    if canonical != authoritative {
        return Err(ProducerAuthorityError::NotAuthoritative {
            event_type: envelope.event_type.clone(),
            authoritative,
            got: canonical.to_string(),
        });
    }
    Ok(())
}
