//! Routing enforcement gate (Phase A). Blocks work on a trace that was never
//! routed — i.e. has no `RouteSelected` genesis in the tamper-evident chain.
//! This is the "teeth" complement to the ledger's "evidence": the chain records
//! that routing happened, and this gate refuses to proceed when it didn't.
//! Spec 0011 (routing mandatory) + the audit's enforcement recommendation.

use std::path::Path;

use anyhow::Result;

use crate::trace_ledger::{self, default_db_path};

/// Outcome of the routing gate for a trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteGate {
    /// A `RouteSelected` genesis exists in the chain — routing happened.
    Allow,
    /// No genesis for this trace (never routed) — block.
    Block(String),
}

/// Check whether `trace_id` has a routing genesis in the ledger. Ledger-absent
/// is treated as Block (fail-closed): if there's no ledger, there's no proof of
/// routing.
pub fn route_gate(root: &Path, trace_id: &str) -> Result<RouteGate> {
    let db = default_db_path(root);
    if !db.is_file() {
        return Ok(RouteGate::Block(format!(
            "no trace ledger at {} — routing cannot be verified (fail-closed)",
            db.display()
        )));
    }
    let conn = trace_ledger::open(&db)?;
    if let Err(reason) = trace_ledger::verify_chain(&conn, trace_id) {
        return Ok(RouteGate::Block(format!(
            "trace '{trace_id}' ledger verification failed: {reason}"
        )));
    }
    if trace_ledger::has_genesis(&conn, trace_id) {
        Ok(RouteGate::Allow)
    } else {
        Ok(RouteGate::Block(format!(
            "trace '{trace_id}' has no RouteSelected genesis — routing is mandatory before work"
        )))
    }
}
