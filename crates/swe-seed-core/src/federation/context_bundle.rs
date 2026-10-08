//! Dual-read adjudication of the canonical CEP `godspeed.context_bundle`
//! envelope (profile `godspeed.context_bundle` v1.0.0, CEP-0008).
//!
//! The bundle is the ONE canonical cross-system context artifact; the legacy
//! `ContextPacketCreated` wire shape is read during a bounded migration window
//! (dual-read) but is never written as canonical. SWE_SEED consumes a bundle
//! only after verifying: envelope kind, profile declaration, pinned world,
//! work-request correlation, truthful complete/partial/none accounting, and
//! the CEP integrity hash (forged or modified bundles are refused).

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use super::proof_completed::canonical_json;

pub const PROFILE_ID: &str = "godspeed.context_bundle";
pub const PROFILE_VERSION: &str = "1.0.0";

/// The bundle facts downstream lineage binds to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleFacts {
    /// CEP `envelope_id` — occurrence identity.
    pub envelope_id: String,
    /// CEP `integrity.content_hash` — exact content identity.
    pub content_hash: String,
    /// The pinned immutable semantic world.
    pub world_ref: String,
    pub work_request_ref: String,
    /// `complete` | `partial` | `none` as CK stated it.
    pub retrieval_completeness: String,
}

/// The compact identity SWE_SEED carries forward into the governed lineage
/// (E4 payload key `context_bundle_ref`).
impl BundleFacts {
    pub fn to_ref(&self) -> Value {
        json!({
            "envelope_id": self.envelope_id,
            "content_hash": self.content_hash,
            "world_ref": self.world_ref,
            "retrieval_completeness": self.retrieval_completeness,
        })
    }
}

/// Recompute the CEP integrity hash: sha256 over the canonical JSON of the
/// envelope with the `integrity` block removed. Numbers are normalized to 6
/// decimal places (and negative zero flattened) first: JSON parsers may
/// drift 1 ULP on long float spellings, and the same envelope must hash
/// identically on both sides of the boundary. This is the SAME rule
/// `ck-mcp::cep_bundle::canonical_json` applies when the hash is produced.
pub fn bundle_content_hash(envelope: &Value) -> String {
    let mut bare = envelope.clone();
    if let Some(obj) = bare.as_object_mut() {
        obj.remove("integrity");
    }
    let normalized = normalize_numbers(&bare);
    format!(
        "sha256:{:x}",
        Sha256::digest(canonical_json(&normalized).as_bytes())
    )
}

/// Float/number normalization for hash stability (mirrors CK's rule):
/// every float to 6 decimal places, `-0.0` to `0.0`, integers untouched.
fn normalize_numbers(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(k.clone(), normalize_numbers(v));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(normalize_numbers).collect()),
        Value::Number(n) => match n.as_f64() {
            Some(f) => {
                let rounded = format!("{:.6}", f).parse::<f64>().unwrap_or(f);
                Value::Number(
                    serde_json::Number::from_f64(if rounded == 0.0 { 0.0 } else { rounded })
                        .unwrap_or_else(|| n.clone()),
                )
            }
            None => value.clone(),
        },
        other => other.clone(),
    }
}

/// Why a bundle was refused at the SWE_SEED boundary.
pub fn verify_context_bundle(
    bundle: &Value,
    work_request_id: &str,
    world_ref: &str,
    context_requirement_id: Option<&str>,
) -> Result<BundleFacts, String> {
    let obj = bundle
        .as_object()
        .ok_or_else(|| "context bundle must be an object".to_string())?;

    // 1. It IS a CEP envelope of the right kind.
    if obj.get("envelope_kind").and_then(Value::as_str) != Some("context_bundle") {
        return Err("wrong envelope kind: not a context_bundle".to_string());
    }
    let envelope_id = obj
        .get("envelope_id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "context bundle missing envelope_id".to_string())?;

    // 2. It declares the canonical GodSpeed profile at the known version.
    let profile = obj
        .get("extensions")
        .and_then(|e| e.get("cep.profile"))
        .ok_or_else(|| "context bundle missing cep.profile declaration".to_string())?;
    if profile.get("profile_id").and_then(Value::as_str) != Some(PROFILE_ID) {
        return Err(format!(
            "wrong profile id: expected {PROFILE_ID}, got {:?}",
            profile.get("profile_id")
        ));
    }
    if profile.get("profile_version").and_then(Value::as_str) != Some(PROFILE_VERSION) {
        return Err(format!(
            "wrong profile version: expected {PROFILE_VERSION}, got {:?}",
            profile.get("profile_version")
        ));
    }

    // 3. It is pinned to the world the request was made in.
    let bundle_world = obj
        .get("scope")
        .and_then(|s| s.get("world_ref"))
        .and_then(Value::as_str)
        .ok_or_else(|| "context bundle missing scope.world_ref".to_string())?;
    if bundle_world != world_ref {
        return Err(format!(
            "world mismatch: bundle {bundle_world:?}, request {world_ref:?}"
        ));
    }

    // 4. It correlates to THIS work request (and requirement).
    let ext = obj
        .get("extensions")
        .and_then(|e| e.get("godspeed.context_bundle"))
        .ok_or_else(|| "context bundle missing godspeed.context_bundle extension".to_string())?;
    let bundle_wr = ext
        .get("work_request_ref")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if bundle_wr != work_request_id {
        return Err(format!(
            "cross-wired context bundle: work_request_ref {bundle_wr:?} != {work_request_id:?}"
        ));
    }
    if let Some(expected_cr) = context_requirement_id {
        let got_cr = ext
            .get("context_requirement_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if got_cr != expected_cr {
            return Err(format!(
                "cross-wired context bundle: context_requirement_id {got_cr:?} != {expected_cr:?}"
            ));
        }
    }

    // 5. Truthful completeness accounting (partial is never complete; absence
    //    is never dressed up).
    let level = ext
        .get("retrieval_completeness")
        .and_then(Value::as_str)
        .ok_or_else(|| "context bundle missing retrieval_completeness".to_string())?;
    let status = obj
        .get("completeness_status")
        .and_then(Value::as_str)
        .unwrap_or("");
    let refs = obj.get("references").and_then(Value::as_array);
    let omissions = obj.get("omissions").and_then(Value::as_array);
    let questions = obj.get("questions").and_then(Value::as_array);
    match level {
        "complete" => {
            if status != "complete_for_context"
                || refs.map(|a| a.is_empty()).unwrap_or(true)
                || omissions.map(|a| !a.is_empty()).unwrap_or(false)
            {
                return Err("a complete bundle must be complete_for_context with references and no omissions".to_string());
            }
        }
        "partial" => {
            if status == "complete_for_context"
                || refs.map(|a| a.is_empty()).unwrap_or(true)
                || omissions.map(|a| a.is_empty()).unwrap_or(true)
            {
                return Err("a partial bundle must keep its references and omissions and never read complete".to_string());
            }
        }
        "none" => {
            if status == "complete_for_context"
                || refs.map(|a| !a.is_empty()).unwrap_or(false)
                || omissions.map(|a| a.is_empty()).unwrap_or(true)
                || questions.map(|a| a.is_empty()).unwrap_or(true)
            {
                return Err("an empty bundle must carry its unanswered question and omissions and never read complete".to_string());
            }
        }
        other => return Err(format!("unknown retrieval_completeness {other:?}")),
    }

    // 6. CEP integrity: the exact content must match its own hash. Forged or
    //    modified bundles are refused here.
    let declared = obj
        .get("integrity")
        .and_then(|i| i.get("content_hash"))
        .and_then(Value::as_str)
        .ok_or_else(|| "context bundle missing integrity.content_hash".to_string())?;
    let computed = bundle_content_hash(bundle);
    if declared != computed {
        return Err(format!(
            "integrity mismatch: declared {declared}, computed {computed}"
        ));
    }

    Ok(BundleFacts {
        envelope_id: envelope_id.to_string(),
        content_hash: declared.to_string(),
        world_ref: bundle_world.to_string(),
        work_request_ref: bundle_wr.to_string(),
        retrieval_completeness: level.to_string(),
    })
}

/// Build the canonical-tool-response bundle facts from a raw response value,
/// or `None` when the response carries no bundle (bounded legacy window).
pub fn bundle_from_response(resp: &Value) -> Option<&Value> {
    resp.get("context_bundle").filter(|b| !b.is_null())
}

/// The compact `context_bundle_ref` entry SEA-Forge records on its authority
/// lineage, rebuilt from a stored reference value.
pub fn ref_matches(facts: &BundleFacts, stored: &Value) -> bool {
    let obj = match stored.as_object() {
        Some(o) => o,
        None => return false,
    };
    obj.get("envelope_id").and_then(Value::as_str) == Some(facts.envelope_id.as_str())
        && obj.get("content_hash").and_then(Value::as_str) == Some(facts.content_hash.as_str())
        && obj.get("world_ref").and_then(Value::as_str) == Some(facts.world_ref.as_str())
}

/// Map a bundle `retrieval_completeness` string onto the packet vocabulary.
pub fn completeness_of(facts: &BundleFacts) -> &'static str {
    match facts.retrieval_completeness.as_str() {
        "complete" => "complete",
        "partial" => "partial",
        "none" => "none",
        _ => "unknown",
    }
}

/// Keep Map import used (payload construction helper for tests/consumers).
pub fn empty_ext() -> Map<String, Value> {
    Map::new()
}
