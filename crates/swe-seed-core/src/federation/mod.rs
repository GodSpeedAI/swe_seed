//! SEA federation boundary (spec 0011). The ONLY module that touches the SEA
//! contract. With `enabled=false` (default) SWE_Seed is fully standalone — no
//! external call is ever made; every gate resolves locally. Ports the
//! first-party Python `agentic_capability_loop/adapters.py` 1:1 (event types,
//! payload keys, hash resolution order).

pub mod consume;
pub mod emit;
pub mod envelope;
pub mod flags;

pub use consume::{
    authority_verdict, check_drift, consume_authority_checked, consume_context_packet_created,
    consume_settlement_recorded, ConsumeError,
};
pub use emit::{
    dispatch, emit_context_required, emit_proof_completed, emit_proof_started, emit_route_selected,
    emit_work_requested, Dispatch, PROOF_TYPE_LIVE, PROOF_TYPE_SIMULATION,
};
pub use envelope::{
    fallback_hash, make_event, resolve_domain_model_hash, resolve_from, Envelope, HashSource,
    ResolvedHash, NAMESPACE,
};
pub use flags::{
    authority_gate, standalone, AuthorityConfig, AuthorityMode, AuthorityVerdict, ContextConfig,
    ContextMode, FederationConfig, GateOutcome, Risk, SettlementConfig, SettlementMode,
};
