//! Normalized hook runtime (spec 0005). JSONL event log + SQLite index +
//! secret redaction + OTEL/JUnit export + compaction, plus the 5 required
//! v0.1 lifecycle events and the HookPolicy/PermissionPolicy gate.

pub mod decision;
pub mod events;
pub mod export;
pub mod index;
pub mod policy;
pub mod port;
pub mod redact;
pub mod runtime;
pub mod runtime_map;

pub use events::{CanonicalHookEvent, ALL_CANONICAL_HOOK_EVENTS};
pub use policy::{
    gate_action, ActionGate, HookPolicy, PermissionPolicy, REQUIRED_LIFECYCLE_EVENTS,
};
pub use redact::{redact_value, RedactionConfig};
pub use runtime::{append_event, compact_logs, iter_events};
