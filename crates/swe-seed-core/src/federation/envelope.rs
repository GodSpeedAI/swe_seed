//! Canonical SEA envelope + `domain_model_hash` resolution (spec 0011).
//! Pure port of `agentic_capability_loop/adapters.py` `_load_hash` / `_event`.
//! In standalone mode the envelope is never constructed; this module only
//! provides the boundary artifact for `emit`/`consume`.
//!
//! The envelope is v1-conformant (`sea.agent.event.v1.json`): `namespace`
//! rides inside `payload` because the schema's `additionalProperties:false`
//! rejects a top-level `namespace`. The signing canonical string still takes
//! `namespace` as an explicit header line — see [`Envelope::namespace`].

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::util::utc_now;

/// SEA federation namespace (matches the Python adapter).
pub const NAMESPACE: &str = "agentic_capability_loop";

/// Manifest path relative to a SEA root (matches the Python adapter).
const MANIFEST_REL: &str =
    "docs/specs/domains/agentic_capability_loop/agentic_capability_loop.manifest.json";

/// The manifest-relative path, for identity gates that must name what they
/// could not resolve.
pub(crate) fn manifest_rel() -> &'static str {
    MANIFEST_REL
}

/// The fallback hash input — `sha256("agentic_capability_loop")` (Python parity).
const FALLBACK_INPUT: &[u8] = b"agentic_capability_loop";

/// The v1 wire schema version label.
pub const SCHEMA_VERSION: &str = "v1";

/// The `source_agent` SWE_Seed stamps on its envelopes.
pub const SOURCE_AGENT: &str = "swe-seed";

/// Canonical v1 event envelope. Field names + set match
/// `sea.agent.event.v1.json` exactly; `namespace` is NOT a top-level field
/// (it lives in `payload`) because the schema enforces
/// `additionalProperties:false`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub schema_version: String,
    pub event_id: String,
    pub source_agent: String,
    pub event_type: String,
    pub occurred_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    pub payload: Value,
    /// CEP-0008 SS15 provenance record (F-06). `{origin, chain}`; populated by
    /// `make_event` so every SWE_Seed envelope answers "who produced this".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Value>,
}

/// Where the resolved hash came from (testability + the standalone warning).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashSource {
    SeaRoot,
    SeaManifestPath,
    MarkerWalk,
    Fallback,
}

#[derive(Debug, Clone)]
pub struct ResolvedHash {
    pub hash: String,
    pub source: HashSource,
    /// True when the SEA manifest was absent and the fallback hash was used
    /// (the Python adapter emits a warning in this case).
    pub warned: bool,
}

/// The standalone fallback hash: `sha256("agentic_capability_loop")` hex.
/// Matches `hashlib.sha256(b"agentic_capability_loop").hexdigest()`.
pub fn fallback_hash() -> String {
    let mut h = Sha256::new();
    h.update(FALLBACK_INPUT);
    format!("{:x}", h.finalize())
}

/// Resolve `domain_model_hash` from the process environment, exactly as the
/// Python adapter does: `SEA_ROOT` → `SEA_MANIFEST_PATH` → marker walk →
/// fallback (+ warn).
pub fn resolve_domain_model_hash() -> ResolvedHash {
    let sea_root = std::env::var("SEA_ROOT").ok();
    let sea_manifest = std::env::var("SEA_MANIFEST_PATH").ok();
    resolve_from(sea_root.as_deref(), sea_manifest.as_deref(), true)
}

pub fn resolve_from_root(root: &Path) -> ResolvedHash {
    let sea_root = std::env::var("SEA_ROOT").ok();
    let sea_manifest = std::env::var("SEA_MANIFEST_PATH").ok();
    let from_env = resolve_from(sea_root.as_deref(), sea_manifest.as_deref(), false);
    if from_env.source != HashSource::Fallback {
        return from_env;
    }
    for candidate in [root.join("SEA").join(MANIFEST_REL), root.join(MANIFEST_REL)] {
        if let Some(hash) = read_manifest_hash(&candidate) {
            return ResolvedHash {
                hash,
                source: HashSource::MarkerWalk,
                warned: false,
            };
        }
    }
    from_env
}

/// Pure resolution entry point (testable, no process-env read). Same order as
/// the Python adapter; pass `walk=false` to skip the cwd marker walk.
pub fn resolve_from(
    sea_root: Option<&str>,
    sea_manifest: Option<&str>,
    walk: bool,
) -> ResolvedHash {
    // 1. SEA_ROOT → <root>/<MANIFEST_REL>.
    if let Some(root) = sea_root.filter(|s| !s.is_empty()) {
        let cand = PathBuf::from(root).join(MANIFEST_REL);
        if let Some(h) = read_manifest_hash(&cand) {
            return ResolvedHash {
                hash: h,
                source: HashSource::SeaRoot,
                warned: false,
            };
        }
    }
    // 2. SEA_MANIFEST_PATH → that file.
    if let Some(p) = sea_manifest.filter(|s| !s.is_empty()) {
        let cand = PathBuf::from(p);
        if cand.is_file() {
            if let Some(h) = read_manifest_hash(&cand) {
                return ResolvedHash {
                    hash: h,
                    source: HashSource::SeaManifestPath,
                    warned: false,
                };
            }
        }
    }
    // 3. Walk parent dirs for a SEA repo (best-effort, matches Python markers).
    if walk {
        if let Some(h) = marker_walk_hash() {
            return ResolvedHash {
                hash: h,
                source: HashSource::MarkerWalk,
                warned: false,
            };
        }
    }
    // 4. Fallback (+ warn).
    ResolvedHash {
        hash: fallback_hash(),
        source: HashSource::Fallback,
        warned: true,
    }
}

pub(crate) fn read_manifest_hash(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let v: Value = serde_json::from_slice(&bytes).ok()?;
    v.get("meta")?
        .get("sea_file_hash")?
        .as_str()
        .map(|s| s.to_string())
}

/// Walk parent directories looking for a SEA repo marker; return its manifest
/// hash if found. Mirrors the Python marker walk (kept best-effort).
fn marker_walk_hash() -> Option<String> {
    let markers = ["tools/sea_parse.py", "docs/specs"];
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let sea_dir = dir.join("SEA");
        if sea_dir.is_dir() && markers.iter().any(|m| sea_dir.join(m).exists()) {
            let cand = sea_dir.join(MANIFEST_REL);
            if let Some(h) = read_manifest_hash(&cand) {
                return Some(h);
            }
        }
        if markers.iter().any(|m| dir.join(m).exists()) && dir.join("libs").is_dir() {
            let cand = dir.join(MANIFEST_REL);
            if let Some(h) = read_manifest_hash(&cand) {
                return Some(h);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Build a canonical v1 envelope, injecting `domain_model_hash` and
/// `namespace` as the first payload keys (matches Python
/// `{"domain_model_hash": ..., "namespace": ..., **payload}`). The
/// `idempotency_key` is content-derived (F-10), matching SEA/GSA's
/// `sha256("{correlation_id}|{event_type}|{canonical_payload}")`.
pub fn make_event(event_type: &str, payload: Map<String, Value>, hash: &str) -> Envelope {
    let mut full = Map::new();
    full.insert("domain_model_hash".into(), Value::String(hash.into()));
    full.insert("namespace".into(), Value::String(NAMESPACE.into()));
    for (k, v) in payload {
        full.insert(k, v);
    }
    let payload_value = Value::Object(full);
    let event_id = Uuid::new_v4().to_string();
    let idempotency = idempotency_key(event_type, &payload_value, None);
    let provenance = serde_json::json!({
        "origin": SOURCE_AGENT,
        "chain": [format!("domain_model_hash:{hash}")],
    });
    Envelope {
        schema_version: SCHEMA_VERSION.into(),
        event_id,
        source_agent: SOURCE_AGENT.into(),
        event_type: event_type.into(),
        occurred_at: utc_now(),
        idempotency_key: Some(idempotency),
        payload: payload_value,
        provenance: Some(provenance),
    }
}

/// Content-derived dedup key (F-10). Mirrors SEA/GSA: sha256 over
/// `"{correlation_id}|{event_type}|{canonical_payload_json}"`. Stable across
/// re-wraps that reassign event_id/occurred_at but leave the payload intact.
pub fn idempotency_key(event_type: &str, payload: &Value, correlation_id: Option<&str>) -> String {
    let payload_json = super::signing::canonical_payload_json(payload);
    let content = format!(
        "{}|{}|{}",
        correlation_id.unwrap_or(""),
        event_type,
        payload_json
    );
    let mut h = Sha256::new();
    h.update(content.as_bytes());
    format!("{:x}", h.finalize())
}

// --- T01: canonical-identity construction + causal derivation ---------------

/// Provenance-chain entry prefix marking a causal parent envelope id
/// (preregistration ENV-I4: derived envelopes MUST identify their parents).
pub const CAUSALITY_PREFIX: &str = "caused_by:";

/// Why an envelope could not be derived from declared parents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeriveError {
    /// Causality requires at least one parent.
    NoParents,
    /// The same parent was declared twice.
    DuplicateParent { event_id: String },
    /// Frozen invariant ENV-I3: `work_request_id` is stable across one cycle.
    /// A derived envelope may not declare a different correlation than its
    /// parents carry.
    CorrelationMismatch {
        parent_work_request_id: Option<String>,
        given: Option<String>,
    },
}

impl std::fmt::Display for DeriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoParents => write!(f, "derived envelopes require at least one causal parent"),
            Self::DuplicateParent { event_id } => {
                write!(f, "causal parent declared twice: {event_id}")
            }
            Self::CorrelationMismatch {
                parent_work_request_id,
                given,
            } => write!(
                f,
                "correlation mismatch: parent carries {parent_work_request_id:?}, derivation declared {given:?}"
            ),
        }
    }
}

impl std::error::Error for DeriveError {}

/// Build a canonical v1 envelope whose `domain_model_hash` has passed the
/// T01 identity gate. Unlike [`make_event`], this cannot stamp a fallback or
/// placeholder digest: only a [`VerifiedDomainIdentity`] unlocks construction.
pub fn make_event_verified(
    event_type: &str,
    payload: Map<String, Value>,
    identity: &super::identity::VerifiedDomainIdentity,
) -> Envelope {
    make_event(event_type, payload, identity.as_str())
}

/// Derive a child envelope from one or more parent envelopes (ENV-I4/I3):
///
/// * every distinct parent id is recorded in `provenance.chain` as a
///   `caused_by:<event_id>` entry;
/// * the frozen correlation (`work_request_id`) is carried unchanged from the
///   parents when they agree on one, and a caller-supplied correlation that
///   disagrees is a [`DeriveError::CorrelationMismatch`].
pub fn derive_event(
    parents: &[&Envelope],
    event_type: &str,
    payload: Map<String, Value>,
    hash: &str,
    correlation: Option<&str>,
) -> Result<Envelope, DeriveError> {
    if parents.is_empty() {
        return Err(DeriveError::NoParents);
    }
    let mut seen: Vec<&str> = Vec::with_capacity(parents.len());
    for p in parents {
        if seen.contains(&p.event_id.as_str()) {
            return Err(DeriveError::DuplicateParent {
                event_id: p.event_id.clone(),
            });
        }
        seen.push(&p.event_id);
    }
    // ENV-I3: correlation stability. All agreeing parents fix the child's
    // work_request_id; disagreement between given and parents is fatal.
    let mut parent_correlation: Option<Option<&str>> = None;
    for p in parents {
        let w = p.work_request_id();
        match parent_correlation {
            None => parent_correlation = Some(w),
            Some(prev) if prev != w => {
                return Err(DeriveError::CorrelationMismatch {
                    parent_work_request_id: w.map(str::to_string),
                    given: correlation.map(str::to_string),
                })
            }
            _ => {}
        }
    }
    let fixed = parent_correlation.flatten();
    if let (Some(given), Some(fixed)) = (correlation, fixed) {
        if given != fixed {
            return Err(DeriveError::CorrelationMismatch {
                parent_work_request_id: Some(fixed.to_string()),
                given: Some(given.to_string()),
            });
        }
    }
    let effective_correlation = fixed.or(correlation);

    let mut child = make_event(event_type, payload, hash);
    if let (Some(obj), Some(w)) = (child.payload.as_object_mut(), effective_correlation) {
        obj.insert("work_request_id".into(), Value::String(w.to_string()));
    }
    // Record causality after construction (make_event built the base chain).
    if let Some(prov) = child.provenance.as_mut() {
        if let Some(chain) = prov.get_mut("chain").and_then(Value::as_array_mut) {
            for id in seen {
                chain.push(Value::String(format!("{CAUSALITY_PREFIX}{id}")));
            }
        }
    }
    Ok(child)
}

impl Envelope {
    /// The envelope's `domain_model_hash` (from its payload), if present.
    pub fn domain_model_hash(&self) -> Option<&str> {
        self.payload
            .get("domain_model_hash")
            .and_then(Value::as_str)
    }

    /// The envelope's `namespace`, carried inside `payload` per v1
    /// (`additionalProperties:false` rejects a top-level namespace). The
    /// signing canonical string sources its namespace header line from here.
    pub fn namespace(&self) -> Option<&str> {
        self.payload.get("namespace").and_then(Value::as_str)
    }

    /// Inject the tamper-evident trace chain root into the payload. This is the
    /// value Phase B signs with Ed25519 and SEA-Forge verifies; carrying it in
    /// the envelope is the federation hook (spec 0011).
    pub fn with_trace_chain_root(&mut self, chain_root: &str) -> Result<(), &'static str> {
        let Some(obj) = self.payload.as_object_mut() else {
            return Err("cannot attach trace_chain_root to non-object envelope payload");
        };
        obj.insert("trace_chain_root".into(), Value::String(chain_root.into()));
        Ok(())
    }

    /// The carried `trace_chain_root`, if any.
    pub fn trace_chain_root(&self) -> Option<&str> {
        self.payload.get("trace_chain_root").and_then(Value::as_str)
    }

    /// The frozen cycle correlation (`work_request_id`), if carried.
    pub fn work_request_id(&self) -> Option<&str> {
        self.payload.get("work_request_id").and_then(Value::as_str)
    }

    /// Causal parent envelope ids recorded in `provenance.chain`
    /// (`caused_by:<event_id>` entries), in declaration order (ENV-I4).
    pub fn causal_parents(&self) -> Vec<&str> {
        let mut out = Vec::new();
        if let Some(chain) = self
            .provenance
            .as_ref()
            .and_then(|p| p.get("chain"))
            .and_then(Value::as_array)
        {
            for entry in chain {
                if let Some(s) = entry.as_str() {
                    if let Some(id) = s.strip_prefix(CAUSALITY_PREFIX) {
                        out.push(id);
                    }
                }
            }
        }
        out
    }
}
