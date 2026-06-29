//! Envelope emit builders + gated dispatch (spec 0011 §3). Each builder mirrors
//! the Python `emit_*` payload keys 1:1. `dispatch` writes only when the master
//! switch AND `emit_envelope` are on — otherwise it is a no-op (the standalone
//! invariant: zero external calls / no sink touched).

use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use super::envelope::{make_event, Envelope};
use super::flags::FederationConfig;

/// Per-event proof type (live vs simulation; simulation never activates).
pub const PROOF_TYPE_LIVE: &str = "live";
pub const PROOF_TYPE_SIMULATION: &str = "simulation";

fn payload_from(value: Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}

/// `WorkRequested` — SWE_SEED received a new task.
pub fn emit_work_requested(
    hash: &str,
    work_request_id: &str,
    actor_id: &str,
    operation: &str,
    resource: &str,
    risk_level: &str,
    repo: Option<&str>,
) -> Envelope {
    let payload = payload_from(json!({
        "work_request_id": work_request_id,
        "actor_id": actor_id,
        "operation": operation,
        "resource": resource,
        "risk_level": risk_level,
        "repo": repo,
    }));
    make_event("WorkRequested", payload, hash)
}

/// `ContextRequired` — SWE_SEED needs context from the Context Kernel.
pub fn emit_context_required(
    hash: &str,
    work_request_id: &str,
    context_requirement_id: &str,
    corpus_id: &str,
    query: Option<&str>,
    max_results: i64,
    is_private: bool,
    scope_claim: Option<&str>,
) -> Envelope {
    let payload = payload_from(json!({
        "work_request_id": work_request_id,
        "context_requirement_id": context_requirement_id,
        "corpus_id": corpus_id,
        "query": query,
        "max_results": max_results,
        "is_private": is_private,
        "scope_claim": scope_claim,
    }));
    make_event("ContextRequired", payload, hash)
}

/// `RouteSelected` — SWE_SEED selected a harness route card.
pub fn emit_route_selected(
    hash: &str,
    work_request_id: &str,
    route_id: &str,
    route_name: &str,
    proof_command_id: &str,
    selected_by: Option<&str>,
) -> Envelope {
    let payload = payload_from(json!({
        "work_request_id": work_request_id,
        "route_id": route_id,
        "route_name": route_name,
        "proof_command_id": proof_command_id,
        "selected_by": selected_by,
    }));
    make_event("RouteSelected", payload, hash)
}

/// `ProofStarted` — SWE_SEED begins running the proof command.
pub fn emit_proof_started(
    hash: &str,
    proof_result_id: &str,
    work_request_id: &str,
    route_id: &str,
    proof_command: Option<&str>,
    started_at: &str,
) -> Envelope {
    let payload = payload_from(json!({
        "proof_result_id": proof_result_id,
        "work_request_id": work_request_id,
        "route_id": route_id,
        "proof_command": proof_command,
        "started_at": started_at,
    }));
    make_event("ProofStarted", payload, hash)
}

/// `ProofCompleted` — the proof command finished.
pub fn emit_proof_completed(
    hash: &str,
    proof_result_id: &str,
    work_request_id: &str,
    result: &str,
    exit_code: Option<i64>,
    output_ref: Option<&str>,
    proof_type: &str,
    completed_at: &str,
) -> Envelope {
    let payload = payload_from(json!({
        "proof_result_id": proof_result_id,
        "work_request_id": work_request_id,
        "result": result,
        "exit_code": exit_code,
        "output_ref": output_ref,
        "proof_type": proof_type,
        "completed_at": completed_at,
    }));
    make_event("ProofCompleted", payload, hash)
}

/// Dispatch outcome: whether the envelope reached its sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dispatch {
    /// Envelope written to this path.
    Written(PathBuf),
    /// Suppressed: federation off / emit disabled → no external call made.
    Suppressed,
}

/// Write an envelope to a file sink iff `cfg.emits()`. When suppressed, no
/// filesystem touch occurs (the standalone invariant).
pub fn dispatch(cfg: &FederationConfig, envelope: &Envelope, sink: Option<&Path>) -> Dispatch {
    if !cfg.emits() {
        return Dispatch::Suppressed;
    }
    let body = format!(
        "{}\n",
        serde_json::to_string_pretty(envelope).unwrap_or_default()
    );
    match sink {
        Some(path) => {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            // ponytail: a failed sink write is logged to stderr but never panics
            // — a broken federation transport must not break the inner loop.
            if std::fs::write(path, &body).is_ok() {
                Dispatch::Written(path.to_path_buf())
            } else {
                Dispatch::Suppressed
            }
        }
        None => {
            // stdout sink
            print!("{body}");
            Dispatch::Written(PathBuf::from("-"))
        }
    }
}
