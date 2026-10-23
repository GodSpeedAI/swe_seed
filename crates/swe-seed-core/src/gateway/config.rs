//! Gateway runtime config (spec 0020 §15). The registry-of-record for gateway
//! backends. Loaded from `.swe-seed/gateway/config.json`; missing file ⇒ the
//! fully-standalone default (no backends, loopback-only listener, federation
//! off). This is NOT a competing capability store — `MCPServer` is the
//! registry artifact for an MCP backend and `GatewayBackend` is its runtime
//! projection (spec 0020 §3, §7).

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    validate_namespace, BreakerState, CatalogKind, GatewayEffectiveLimits, GatewayTransport,
    HealthState, NamespaceError,
};
use crate::federation::FederationConfig;
use crate::hooks::policy::PermissionPolicy;

/// Default gateway config root inside a project.
pub const GATEWAY_CONFIG_REL: &str = ".swe-seed/gateway/config.json";

/// `MCPServer` — the registry artifact for one MCP backend (spec 0003 id space,
/// spec 0020 §3). Gateway backends and catalog entries derive from this. Live
/// `tools/list` discovery (stage 5) augments the declared `catalog`; declared
/// entries are the deterministic, hash-pinnable baseline.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MCPServer {
    pub id: String,
    pub namespace: String,
    pub transport: GatewayTransport,
    /// stdio: the program; http: the base URL.
    pub command_or_url: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub profile_tags: Vec<String>,
    #[serde(default)]
    pub policy_tags: Vec<String>,
    /// `sha256:<hex>` source hash from provenance, when available.
    #[serde(default)]
    pub source_hash: Option<String>,
    /// `clear` | `ambiguous` | `restricted` (mirrors `ProvenanceRecord.license_status`).
    #[serde(default)]
    pub scan_status: Option<String>,
    #[serde(default)]
    pub provenance_ref: Option<String>,
    /// Declared tools/resources/prompts (the deterministic catalog baseline).
    #[serde(default)]
    pub catalog: Vec<DeclaredCatalogEntry>,
    /// Operator-pinned `definition_hash` per namespaced name; verified at call
    /// time (stage 3). Key = `<namespace>.<name>`.
    #[serde(default)]
    pub pinned_hashes: HashMap<String, String>,
}

/// A declared tool/resource/prompt on an `MCPServer` (pre-discovery catalog).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclaredCatalogEntry {
    pub kind: CatalogKind,
    /// Local name; the catalog name is `<namespace>.<name>`.
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub input_schema: Option<Value>,
    #[serde(default)]
    pub policy_tags: Vec<String>,
}

/// Listener policy (spec 0020 §15). Loopback by default; non-loopback bind
/// requires explicit TLS or tunnel mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerConfig {
    #[serde(default = "default_bind")]
    pub bind: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub tls: bool,
    #[serde(default)]
    pub tunnel: bool,
}

fn default_bind() -> String {
    "127.0.0.1".into()
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            bind: default_bind(),
            port: 0,
            tls: false,
            tunnel: false,
        }
    }
}

impl ListenerConfig {
    /// True iff the bind is loopback (no TLS/tunnel required) — spec 0020 §15.
    pub fn is_loopback(&self) -> bool {
        self.bind == "127.0.0.1" || self.bind == "localhost" || self.bind == "::1"
    }
    /// Non-loopback exposure requires TLS or explicit tunnel mode (spec 0020 §15).
    pub fn exposure_ok(&self) -> bool {
        self.is_loopback() || self.tls || self.tunnel
    }
}

/// Compiled gateway runtime snapshot: registry + policy + federation + limits.
/// Reload (stage 7) validates a new snapshot before applying it.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayConfig {
    #[serde(default)]
    pub servers: Vec<MCPServer>,
    #[serde(default)]
    pub listener: ListenerConfig,
    #[serde(default)]
    pub policy: PermissionPolicy,
    #[serde(default)]
    pub federation: FederationConfig,
    #[serde(default)]
    pub limits: GatewayEffectiveLimits,
}

impl GatewayConfig {
    /// Load from `<root>/.swe-seed/gateway/config.json`. Missing file ⇒ the
    /// standalone default (no external reads beyond the stat).
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(GATEWAY_CONFIG_REL);
        match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .with_context(|| format!("parse {}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("read {}", path.display())),
        }
    }

    /// Validate namespaces (uniqueness + format) AND listener exposure
    /// (non-loopback requires TLS or tunnel) so a bad/unsafe config fails
    /// closed at install/reload rather than mid-request (spec 0020 §7, §15).
    pub fn validate(&self) -> std::result::Result<(), super::GatewayError> {
        self.validate_namespaces().map_err(|e| super::GatewayError::InvalidReload {
            reason: e.to_string(),
        })?;
        if !self.listener.exposure_ok() {
            return Err(super::GatewayError::InvalidReload {
                reason: format!(
                    "non-loopback bind '{}' requires tls or tunnel mode (spec 0020 §15)",
                    self.listener.bind
                ),
            });
        }
        Ok(())
    }

    /// Validate every server namespace (uniqueness + format) up front so a bad
    /// config fails closed at load rather than mid-request (spec 0020 §7).
    pub fn validate_namespaces(&self) -> std::result::Result<(), NamespaceError> {
        let mut seen: HashMap<&str, ()> = HashMap::new();
        for s in &self.servers {
            validate_namespace(&s.namespace)?;
            if seen.insert(s.namespace.as_str(), ()).is_some() {
                return Err(NamespaceError::Duplicate(s.namespace.clone()));
            }
        }
        Ok(())
    }
}

/// Project an `MCPServer` into its runtime `GatewayBackend` view (spec 0020 §7).
/// Fresh backends start healthy/closed; runtime health/breaker state is owned
/// by the health module (stage 7).
pub fn project_backend(server: &MCPServer) -> super::GatewayBackend {
    super::GatewayBackend {
        id: server.id.clone(),
        namespace: server.namespace.clone(),
        transport: server.transport,
        command_or_url: server.command_or_url.clone(),
        profile_tags: server.profile_tags.clone(),
        health_state: HealthState::Healthy,
        breaker_state: BreakerState::Closed,
        provenance_ref: server.provenance_ref.clone(),
    }
}
