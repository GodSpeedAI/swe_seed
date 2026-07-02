//! routing_gate: a trace with a RouteSelected genesis is allowed; a trace
//! without one (or no ledger at all) is blocked, fail-closed (Phase A gate).

use swe_seed_core::routing_gate::{route_gate, RouteGate};
use swe_seed_core::trace_ledger::{self, LedgerEntry};

fn temp_root() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-gate-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn no_ledger_is_fail_closed_block() {
    // No ledger at all → cannot prove routing → Block (fail-closed).
    let root = temp_root();
    match route_gate(&root, "T-none") {
        Ok(RouteGate::Block(msg)) => assert!(msg.contains("fail-closed"), "{msg}"),
        other => panic!("expected Block, got {other:?}"),
    }
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn trace_without_genesis_is_blocked() {
    let root = temp_root();
    let db = swe_seed_core::trace_ledger::default_db_path(&root);
    let conn = swe_seed_core::trace_ledger::open(&db).unwrap();
    // A trace that exists but was never routed (no genesis).
    let _ = trace_ledger::append(
        &conn,
        LedgerEntry {
            trace_id: "T-unrouted",
            event_type: "TraceNote",
            payload_json: r#"{"note":"x"}"#,
            occurred_at: "t",
        },
    )
    .unwrap();
    drop(conn);
    match route_gate(&root, "T-unrouted") {
        Ok(RouteGate::Block(msg)) => assert!(msg.contains("genesis"), "{msg}"),
        other => panic!("expected Block, got {other:?}"),
    }
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn trace_with_genesis_is_allowed() {
    let root = temp_root();
    let db = swe_seed_core::trace_ledger::default_db_path(&root);
    let conn = swe_seed_core::trace_ledger::open(&db).unwrap();
    let _ = trace_ledger::append(
        &conn,
        LedgerEntry {
            trace_id: "T-routed",
            event_type: "RouteSelected",
            payload_json: r#"{"job_type":"test"}"#,
            occurred_at: "t",
        },
    )
    .unwrap();
    drop(conn);
    assert_eq!(route_gate(&root, "T-routed").unwrap(), RouteGate::Allow);
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn tampered_ledger_is_fail_closed_block() {
    let root = temp_root();
    let db = swe_seed_core::trace_ledger::default_db_path(&root);
    let conn = swe_seed_core::trace_ledger::open(&db).unwrap();
    let _ = trace_ledger::append(
        &conn,
        LedgerEntry {
            trace_id: "T-tampered",
            event_type: "TraceNote",
            payload_json: r#"{"job_type":"test"}"#,
            occurred_at: "t",
        },
    )
    .unwrap();
    conn.execute(
        "UPDATE chain_entries SET event_type = 'RouteSelected' WHERE trace_id = 'T-tampered' AND seq = 0",
        [],
    )
    .unwrap();
    drop(conn);
    match route_gate(&root, "T-tampered") {
        Ok(RouteGate::Block(msg)) => assert!(msg.contains("verification failed"), "{msg}"),
        other => panic!("expected Block, got {other:?}"),
    }
    std::fs::remove_dir_all(&root).ok();
}
