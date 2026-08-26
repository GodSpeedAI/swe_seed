//! SEA federation boundary (spec 0011). The ONLY module that touches the SEA
//! contract. With `enabled=false` (default) SWE_Seed is fully standalone — no
//! external call is ever made; every gate resolves locally. Ports the
//! first-party Python `agentic_capability_loop/adapters.py` 1:1 (event types,
//! payload keys, hash resolution order).

pub mod consume;
pub mod context_client;
pub mod emit;
pub mod envelope;
pub mod flags;
pub mod governed_submission;
pub mod idempotency;
pub mod identity;
pub mod operational_settlement;
pub mod producers;
pub mod proof_completed;
pub mod signing;
pub mod work_ingress;

/// The signature algorithm label carried on signed envelopes.
pub const SIGNING_ALGORITHM: &str = "ed25519";

pub use consume::{
    authority_verdict, check_conformance, check_drift, consume_authority_checked,
    consume_context_packet_created, consume_settlement_recorded, validate_envelope,
    ConformanceReport, ConsumeError,
};
pub use context_client::{
    adjudicate_context_response, ContextClientError, ContextKernelClient, ContextPacket,
    ContextRequest, ExpectedContext,
};
pub use emit::{
    dispatch, emit_context_required, emit_proof_completed, emit_proof_started, emit_route_selected,
    emit_settlement_recorded, emit_work_requested, Dispatch, PROOF_TYPE_LIVE,
    PROOF_TYPE_SIMULATION,
};
pub use envelope::{
    derive_event, fallback_hash, idempotency_key, make_event, make_event_verified,
    resolve_domain_model_hash, resolve_from, resolve_from_root, DeriveError, Envelope, HashSource,
    ResolvedHash, CAUSALITY_PREFIX, NAMESPACE, SCHEMA_VERSION, SOURCE_AGENT,
};
pub use flags::{
    authority_gate, standalone, AuthorityConfig, AuthorityMode, AuthorityVerdict, ContextConfig,
    ContextMode, FederationConfig, GateOutcome, Risk, SettlementConfig, SettlementMode,
};
pub use governed_submission::{
    build_governed_work_request, context_packet_ref, GovernedSubmission, ProofContract,
    SubmissionError,
};
pub use idempotency::{Admission, AdmitError, IdempotencyLedger};
pub use identity::{
    strict_resolve, verify_against_artifact, verify_declared_hash, IdentityError,
    VerifiedDomainIdentity,
};
pub use operational_settlement::{
    OperationalSettlementAdjudicator, OperationalOutcome, OperationalSettlementFacts,
    Adjudication, AdjudicationError,
};
pub use proof_completed::{
    emit_proof_completed_verified, operational_settlement_ref, EmissionError, ProofCompletion,
    PROOF_STATUSES,
};
pub use producers::{
    agent_id, authoritative_producer, canonical_agent, validate_producer, ProducerAuthorityError,
};
pub use signing::{
    canonical_signing_string, generate_signing_key, load_signing_key, private_key_path,
    public_key_b64, public_key_from_b64, public_key_path, sign_envelope, signing_key_from_bytes,
    verify_envelope, write_keypair, LoadError, SignaturePayload, VerifyError,
};
pub use work_ingress::{accept_work_requested, WorkRequestContract};
