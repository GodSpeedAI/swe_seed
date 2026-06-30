//! Canonical SEA envelope + `domain_model_hash` resolution (spec 0011).
//! Pure port of `agentic_capability_loop/adapters.py` `_load_hash` / `_event`.
//! In standalone mode the envelope is never constructed; this module only
//! provides the boundary artifact for `emit`/`consume`.

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

/// The fallback hash input — `sha256("agentic_capability_loop")` (Python parity).
const FALLBACK_INPUT: &[u8] = b"agentic_capability_loop";

/// Canonical event envelope. Field names match the Python adapter 1:1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub event_id: String,
    pub event_type: String,
    pub namespace: String,
    pub occurred_at: String,
    pub payload: Value,
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

fn read_manifest_hash(path: &Path) -> Option<String> {
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

/// Build a canonical envelope, injecting `domain_model_hash` as the first
/// payload key (matches Python `{"domain_model_hash": ..., **payload}`).
pub fn make_event(event_type: &str, payload: Map<String, Value>, hash: &str) -> Envelope {
    let mut full = Map::new();
    full.insert("domain_model_hash".into(), Value::String(hash.into()));
    for (k, v) in payload {
        full.insert(k, v);
    }
    Envelope {
        event_id: Uuid::new_v4().to_string(),
        event_type: event_type.into(),
        namespace: NAMESPACE.into(),
        occurred_at: utc_now(),
        payload: Value::Object(full),
    }
}

impl Envelope {
    /// The envelope's `domain_model_hash` (from its payload), if present.
    pub fn domain_model_hash(&self) -> Option<&str> {
        self.payload
            .get("domain_model_hash")
            .and_then(Value::as_str)
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
}
