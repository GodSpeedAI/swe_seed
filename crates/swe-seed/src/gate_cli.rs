//! `swe-seed gate <trace>` — routing enforcement gate. Exits 0 iff the trace
//! has a `RouteSelected` genesis in the tamper-evident ledger; non-zero
//! (block) otherwise. With `--verify`, also recomputes the full chain to confirm
//! tamper-evidence. Used by the PreToolUse hook and the CI merge gate.

use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;

pub fn run_gate(root: &Path, trace: &str, verify: bool) -> Result<ExitCode> {
    match swe_seed_core::routing_gate::route_gate(root, trace) {
        Ok(swe_seed_core::routing_gate::RouteGate::Allow) => {
            if verify {
                // Full tamper-evidence check: recompute the chain to the root.
                let db = swe_seed_core::trace_ledger::default_db_path(root);
                let conn = swe_seed_core::trace_ledger::open(&db)?;
                if let Err(reason) = swe_seed_core::trace_ledger::verify_chain(&conn, trace) {
                    eprintln!("block: chain verification failed: {reason}");
                    return Ok(ExitCode::from(1));
                }
            }
            println!(
                "allow: trace '{trace}' routed{}",
                if verify { " + chain verifies" } else { "" }
            );
            Ok(ExitCode::SUCCESS)
        }
        Ok(swe_seed_core::routing_gate::RouteGate::Block(reason)) => {
            eprintln!("block: {reason}");
            Ok(ExitCode::from(1))
        }
        Err(e) => {
            eprintln!("gate error: {e:#}");
            Ok(ExitCode::from(2))
        }
    }
}
