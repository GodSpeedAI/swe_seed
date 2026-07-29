//! MCPGate runtime models and typed errors (spec 0020).
//!
//! Stage 1 skeleton: this module defines the gateway-runtime domain model that
//! *derives* from the existing registry (`MCPServer`, `LayerCapability`,
//! `ArtifactMetadata`) and the typed failure model from spec 0020 §18. It does
//! not forward traffic, build catalogs, or enforce policy yet — those land in
//! stages 2+. The registry remains the single source of truth; these structs
//! are runtime projections, not a competing capability store (spec 0020 §2).
//!
//! Failure model: every request must complete with a response OR a typed
//! `GatewayError` (spec 0020 §5). Each variant carries a stable `reason_code`
//! so local audit records (§5) and semantic-envelope dispositions (§9) cite a
//! machine-readable reason. The codes are snake_case and frozen for the life
//! of the variant.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod audit;
pub mod catalog;
pub mod config;
pub mod fabricator;
pub mod federation_proj;
pub mod governance;
pub mod health;
pub mod import;
pub mod jsonrpc;
pub mod policy;
pub mod redaction;
pub mod request;
pub mod routing;
pub mod serve;

pub use audit::{AuditRecord, AuditWriter, AUDIT_DIR_REL, AUDIT_FILE};
pub use catalog::{
    canonical_json, definition_hash, list_catalog, project_catalog, search_catalog,
    validate_local_name, validate_namespace, CompactCatalogItem, DroppedEntry, NamespaceError,
    DEFAULT_DISCOVERY_CAP,
};
pub use config::{
    project_backend, DeclaredCatalogEntry, GatewayConfig, ListenerConfig, MCPServer,
    GATEWAY_CONFIG_REL,
};
pub use policy::{evaluate_invoke, is_risky, scan_is_clear, InvokeDecision};
pub use redaction::{builtin_redaction, load_redaction};
pub use request::{handle_request, RequestContext};
pub use governance::{generate_session_id, Governance, GOVERNANCE_JOURNAL_REL};
pub use health::{
    BackendMetrics, BreakerConfig, HealthBook, Metrics, MetricsSnapshot, SnapshotManager,
};
pub use import::import_openapi;
pub use jsonrpc::{is_allowed_method, json_depth, parse_jsonrpc, validate_request, JsonRpcRequest, JSON_DEPTH_CAP, REQUEST_SIZE_CAP};
pub use federation_proj::{consume_authority, project_route_selected, project_settlement_recorded, reject_on_drift, Disposition};
pub use fabricator::{cite_gateway_evidence, evidence_link};
pub use routing::{
    ALLOWED_METHODS, BackendTransport, HttpTransport, MockTransport, Router, StdioTransport,
};
pub use serve::GatewayServer;

// ── transports / states ──────────────────────────────────────────────────────

/// Backend transport (spec 0020 §7 GatewayBackend.transport).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GatewayTransport {
    #[default]
    Stdio,
    Http,
    Sse,
    Websocket,
}

/// Per-backend health (spec 0020 §7, §17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    Healthy,
    Degraded,
    Down,
}

/// Per-backend circuit breaker (spec 0020 §7, §18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Catalog entry kind (spec 0020 §7 GatewayCatalogEntry.kind).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CatalogKind {
    Tool,
    Resource,
    Prompt,
}

/// Session lifecycle state (spec 0020 §7 GatewaySession.state). Note
/// `LimitExceeded` is a governance state, not a backend failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Active,
    LimitExceeded,
    Terminated,
}

// ── domain models (spec 0020 §7) ─────────────────────────────────────────────

/// Live runtime view of a registered `MCPServer` (spec 0020 §7). Derived from
/// the registry; not a second source of truth. The namespace MUST be unique
/// and a namespaced call MUST route only to its owning backend (enforced in
/// later stages; the invariant is documented here).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayBackend {
    pub id: String,
    pub namespace: String,
    pub transport: GatewayTransport,
    pub command_or_url: String,
    #[serde(default)]
    pub profile_tags: Vec<String>,
    pub health_state: HealthState,
    pub breaker_state: BreakerState,
    /// `ArtifactMetadata` id or provenance record id (spec 0020 §7).
    #[serde(default)]
    pub provenance_ref: Option<String>,
}

/// Runtime projection of a tool, resource, or prompt (spec 0020 §7).
/// `definition_hash` is SHA-256 over the canonical `wire_definition` JSON;
/// `pinned_hash`, when present, is verified at call time (stage 3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayCatalogEntry {
    pub kind: CatalogKind,
    pub namespaced_name: String,
    pub backend_id: String,
    /// Canonical JSON definition sent to clients.
    pub wire_definition: Value,
    /// SHA-256 hex over canonical `wire_definition`.
    pub definition_hash: String,
    #[serde(default)]
    pub source_hash: Option<String>,
    #[serde(default)]
    pub policy_tags: Vec<String>,
    /// Operator-pinned expected `definition_hash`; verified at call time.
    #[serde(default)]
    pub pinned_hash: Option<String>,
    #[serde(default)]
    pub input_schema: Option<Value>,
}

/// Per-client runtime sandbox (spec 0020 §7). `session_id` is server-generated
/// (>=128 bits of entropy in stage 6); clients MUST NOT supply or override it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewaySession {
    pub session_id: String,
    pub client_id: String,
    #[serde(default)]
    pub route_card_id: Option<String>,
    #[serde(default)]
    pub allowed_backends: Vec<String>,
    pub state: SessionState,
    /// ISO-8601 UTC start time.
    pub started_at: String,
    #[serde(default)]
    pub call_count: u64,
    #[serde(default)]
    pub max_calls: u64,
    #[serde(default)]
    pub max_duration_secs: u64,
}

/// Effective limits after PermissionPolicy and config narrowing (spec 0020 §7).
/// Stage 6 applies these inside the atomic governance critical section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GatewayEffectiveLimits {
    #[serde(default)]
    pub session_max_calls: u64,
    #[serde(default)]
    pub session_max_duration_secs: u64,
    #[serde(default)]
    pub client_max_requests: u64,
    #[serde(default)]
    pub tool_max_requests: u64,
}

/// Authoritative in-memory governance counters (spec 0020 §7). Check-and-
/// increment happens in one critical section in stage 6; the durable counter
/// delta is recorded before forwarding so a crash cannot undercount. Here we
/// only model the state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GatewayGovernanceState {
    #[serde(default)]
    pub session_calls: HashMap<String, u64>,
    #[serde(default)]
    pub client_counters: HashMap<String, u64>,
    #[serde(default)]
    pub tool_counters: HashMap<String, u64>,
    /// ISO-8601 UTC window start.
    #[serde(default)]
    pub utc_window_start: Option<String>,
    #[serde(default)]
    pub limits: GatewayEffectiveLimits,
}

// ── typed failure model (spec 0020 §18) ──────────────────────────────────────

/// Typed gateway error (spec 0020 §18). Every blocked path returns one of these
/// and writes a local audit record citing `reason_code()` before completing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayError {
    /// Backend timeout, connection refused, process exit, or breaker open.
    BackendUnavailable { backend: String },
    /// Malformed backend response (distinct from unreachable).
    BackendError { backend: String, reason: String },
    /// Call-time pinned-hash mismatch — blocks forwarding (spec 0020 §7, §19).
    CapabilityHashMismatch {
        namespaced_name: String,
        expected: String,
        got: String,
    },
    /// Failed authentication.
    AuthFailed { reason: String },
    /// Denied scope (PermissionPolicy deny overrides allow; spec 0020 §10).
    ScopeDenied { scope: String },
    /// Dangerous tool group requires explicit approval (spec 0020 §10).
    ApprovalRequired { namespaced_name: String },
    /// Unscanned dangerous capability fail-closed (spec 0020 §10).
    ScanBlocked { namespaced_name: String, reason: String },
    /// Risky capability lacks live proof / provenance (spec 0020 §10).
    ProvenanceMissing { namespaced_name: String },
    /// Budget counter exceeded (spec 0020 §7, §18).
    BudgetExceeded { limit: u64 },
    /// Session call/duration limit reached (governance, not backend failure).
    SessionLimitExceeded { session_id: String },
    /// Malformed JSON-RPC or size/depth limit violation (spec 0020 §8).
    InvalidRequest { reason: String },
    /// Reload snapshot invalid — last-known-good preserved (spec 0020 §15).
    InvalidReload { reason: String },
    /// Audit record could not be written — request MUST fail closed (§18).
    AuditWriteFailed { reason: String },
    /// Online envelope dispatch required but no disposition recorded (§9, §18).
    FederationUnavailable { reason: String },
    /// A namespaced call would reroute to a non-owning backend — forbidden
    /// (spec 0020 §7: namespaced calls never fail over).
    NamespacedRerouteBlocked { namespaced_name: String },
}

impl GatewayError {
    /// Stable, machine-readable reason code for audit and envelope disposition
    /// (spec 0020 §5 `reason_code`). Frozen for the life of each variant.
    pub fn reason_code(&self) -> &'static str {
        match self {
            GatewayError::BackendUnavailable { .. } => "backend_unavailable",
            GatewayError::BackendError { .. } => "backend_error",
            GatewayError::CapabilityHashMismatch { .. } => "capability_hash_mismatch",
            GatewayError::AuthFailed { .. } => "auth_failed",
            GatewayError::ScopeDenied { .. } => "scope_denied",
            GatewayError::ApprovalRequired { .. } => "approval_required",
            GatewayError::ScanBlocked { .. } => "scan_blocked",
            GatewayError::ProvenanceMissing { .. } => "provenance_missing",
            GatewayError::BudgetExceeded { .. } => "budget_exceeded",
            GatewayError::SessionLimitExceeded { .. } => "session_limit_exceeded",
            GatewayError::InvalidRequest { .. } => "invalid_request",
            GatewayError::InvalidReload { .. } => "invalid_reload",
            GatewayError::AuditWriteFailed { .. } => "audit_write_failed",
            GatewayError::FederationUnavailable { .. } => "federation_unavailable",
            GatewayError::NamespacedRerouteBlocked { .. } => "namespaced_reroute_blocked",
        }
    }
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GatewayError::BackendUnavailable { backend } => {
                write!(f, "backend '{backend}' unavailable (timeout/refused/breaker open)")
            }
            GatewayError::BackendError { backend, reason } => {
                write!(f, "backend '{backend}' error: {reason}")
            }
            GatewayError::CapabilityHashMismatch {
                namespaced_name,
                expected,
                got,
            } => write!(
                f,
                "capability hash mismatch for '{namespaced_name}': expected {expected}, got {got}"
            ),
            GatewayError::AuthFailed { reason } => write!(f, "authentication failed: {reason}"),
            GatewayError::ScopeDenied { scope } => write!(f, "scope denied: {scope}"),
            GatewayError::ApprovalRequired { namespaced_name } => {
                write!(f, "approval required for '{namespaced_name}'")
            }
            GatewayError::ScanBlocked {
                namespaced_name,
                reason,
            } => write!(f, "scan blocked for '{namespaced_name}': {reason}"),
            GatewayError::ProvenanceMissing { namespaced_name } => {
                write!(f, "provenance missing for '{namespaced_name}'")
            }
            GatewayError::BudgetExceeded { limit } => {
                write!(f, "budget exceeded (limit {limit})")
            }
            GatewayError::SessionLimitExceeded { session_id } => {
                write!(f, "session limit exceeded: {session_id}")
            }
            GatewayError::InvalidRequest { reason } => write!(f, "invalid request: {reason}"),
            GatewayError::InvalidReload { reason } => write!(f, "invalid reload: {reason}"),
            GatewayError::AuditWriteFailed { reason } => {
                write!(f, "audit write failed: {reason}")
            }
            GatewayError::FederationUnavailable { reason } => {
                write!(f, "federation unavailable: {reason}")
            }
            GatewayError::NamespacedRerouteBlocked { namespaced_name } => {
                write!(f, "namespaced reroute blocked for '{namespaced_name}'")
            }
        }
    }
}

impl std::error::Error for GatewayError {}

/// Convenience alias so gateway call sites read as `Result<T, GatewayError>`.
pub type GatewayResult<T> = Result<T, GatewayError>;
