//! trace_ledger: hash-chained append-only log. A route-decision genesis,
//! tamper-evidence on partial edits, and a chain-root accessor (Phase A).

use swe_seed_core::trace_ledger::{
    append, chain_root, open, verify_chain, verify_chain_with, AnchorPolicy, GENESIS_PREV,
};

fn temp_db() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "swe-seed-ledger-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn genesis_is_route_decision_and_chain_grows() {
    let db = temp_db();
    let conn = open(&db).unwrap();

    // Genesis = the route decision.
    let g = append(
        &conn,
        swe_seed_core::trace_ledger::LedgerEntry {
            trace_id: "T1",
            event_type: "RouteSelected",
            payload_json: r#"{"job_type":"test","route_card":"test.json"}"#,
            occurred_at: "t0",
        },
    )
    .unwrap();
    assert_eq!(g.seq, 0, "first append is genesis (seq 0)");
    assert_eq!(g.hash, chain_root(&conn, "T1").unwrap());

    // Subsequent events chain off it.
    let e1 = append(
        &conn,
        swe_seed_core::trace_ledger::LedgerEntry {
            trace_id: "T1",
            event_type: "ProofStarted",
            payload_json: r#"{"proof":"cargo test"}"#,
            occurred_at: "t1",
        },
    )
    .unwrap();
    assert_eq!(e1.seq, 1);
    assert_eq!(chain_root(&conn, "T1").unwrap(), e1.hash);
    assert_ne!(e1.hash, g.hash);

    // Clean chain verifies and returns the root.
    assert_eq!(verify_chain(&conn, "T1").unwrap(), e1.hash);
    std::fs::remove_file(&db).ok();
}

#[test]
fn tampered_payload_is_detected() {
    let db = temp_db();
    let conn = open(&db).unwrap();
    append(
        &conn,
        swe_seed_core::trace_ledger::LedgerEntry {
            trace_id: "T2",
            event_type: "RouteSelected",
            payload_json: r#"{"j":"a"}"#,
            occurred_at: "t",
        },
    )
    .unwrap();
    append(
        &conn,
        swe_seed_core::trace_ledger::LedgerEntry {
            trace_id: "T2",
            event_type: "ProofStarted",
            payload_json: r#"{"p":"x"}"#,
            occurred_at: "t",
        },
    )
    .unwrap();
    // Clean → verifies.
    assert!(verify_chain(&conn, "T2").is_ok());

    // Tamper: flip a payload byte in the genesis entry, leave its stored hash.
    conn.execute(
        "UPDATE chain_entries SET payload_json = ?1 WHERE trace_id = 'T2' AND seq = 0",
        rusqlite::params![r#"{"j":"FORGED"}"#],
    )
    .unwrap();
    let err = verify_chain(&conn, "T2").unwrap_err();
    assert!(
        err.contains("tampered") || err.contains("prev_hash broken"),
        "{err}"
    );
    std::fs::remove_file(&db).ok();
}

#[test]
fn tampered_event_type_is_detected() {
    let db = temp_db();
    let conn = open(&db).unwrap();
    append(
        &conn,
        swe_seed_core::trace_ledger::LedgerEntry {
            trace_id: "T-event",
            event_type: "TraceNote",
            payload_json: r#"{"j":"a"}"#,
            occurred_at: "t",
        },
    )
    .unwrap();
    conn.execute(
        "UPDATE chain_entries SET event_type = 'RouteSelected' WHERE trace_id = 'T-event' AND seq = 0",
        [],
    )
    .unwrap();
    let err = verify_chain(&conn, "T-event").unwrap_err();
    assert!(err.contains("tampered"), "{err}");
    std::fs::remove_file(&db).ok();
}

#[test]
fn severed_link_is_detected() {
    // If someone inserts a bogus prev_hash on entry 1, the chain links break.
    let db = temp_db();
    let conn = open(&db).unwrap();
    append(
        &conn,
        swe_seed_core::trace_ledger::LedgerEntry {
            trace_id: "T3",
            event_type: "RouteSelected",
            payload_json: r#"{"j":"a"}"#,
            occurred_at: "t",
        },
    )
    .unwrap();
    conn.execute(
        "INSERT INTO chain_entries (trace_id, seq, event_id, event_type, payload_json, prev_hash, hash, occurred_at)
         VALUES ('T3', 1, 'x', 'ProofStarted', '{}', 'bogusprev', 'bogushash', 't')",
        [],
    )
    .unwrap();
    let err = verify_chain(&conn, "T3").unwrap_err();
    assert!(err.contains("prev_hash broken"), "{err}");
    std::fs::remove_file(&db).ok();
}

#[test]
fn genesis_prev_is_zero_hash() {
    // The documented contract: genesis seeds off the all-zero hash.
    assert_eq!(GENESIS_PREV.len(), 64);
    assert!(GENESIS_PREV.chars().all(|c| c == '0'));
}

fn append_three(conn: &rusqlite::Connection, trace_id: &str) {
    for (i, event_type) in ["RouteSelected", "ProofStarted", "ProofFinished"]
        .iter()
        .enumerate()
    {
        append(
            conn,
            swe_seed_core::trace_ledger::LedgerEntry {
                trace_id,
                event_type,
                payload_json: &format!(r#"{{"step":{i}}}"#),
                occurred_at: "t",
            },
        )
        .unwrap();
    }
}

#[test]
fn truncated_trailing_entry_fails_verification() {
    // Deleting the last chain row leaves a shorter prefix that is internally
    // consistent; only a head/length anchor written outside the rows catches it.
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-trunc");
    assert!(verify_chain(&conn, "T-trunc").is_ok());
    conn.execute(
        "DELETE FROM chain_entries WHERE trace_id = 'T-trunc' AND seq = 2",
        [],
    )
    .unwrap();
    let err = verify_chain(&conn, "T-trunc").unwrap_err();
    assert!(err.contains("head anchor mismatch"), "{err}");
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}

#[test]
fn deleted_middle_entry_fails_verification() {
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-mid");
    conn.execute(
        "DELETE FROM chain_entries WHERE trace_id = 'T-mid' AND seq = 1",
        [],
    )
    .unwrap();
    let err = verify_chain(&conn, "T-mid").unwrap_err();
    assert!(err.contains("prev_hash broken"), "{err}");
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}

#[test]
fn unanchored_legacy_trace_is_reported_not_passed() {
    // A ledger written before anchors existed has rows but no sidecar anchor.
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-legacy");
    std::fs::remove_file(db.with_extension("anchors.json")).unwrap();
    let err = verify_chain(&conn, "T-legacy").unwrap_err();
    assert!(err.contains("unanchored"), "{err}");
    // The explicit legacy opt-in accepts it, structure only.
    assert!(verify_chain_with(&conn, "T-legacy", AnchorPolicy::AllowUnanchored).is_ok());
    std::fs::remove_file(&db).ok();
}

#[test]
fn anchor_is_checked_even_under_legacy_policy() {
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-forged");
    conn.execute(
        "DELETE FROM chain_entries WHERE trace_id = 'T-forged' AND seq = 2",
        [],
    )
    .unwrap();
    let err = verify_chain_with(&conn, "T-forged", AnchorPolicy::AllowUnanchored).unwrap_err();
    assert!(err.contains("head anchor mismatch"), "{err}");
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}

#[test]
fn append_does_not_vouch_for_legacy_rows() {
    // Appending to an unanchored trace must not write an anchor covering old rows.
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-legacy2");
    std::fs::remove_file(db.with_extension("anchors.json")).unwrap();
    append(
        &conn,
        swe_seed_core::trace_ledger::LedgerEntry {
            trace_id: "T-legacy2",
            event_type: "TraceNote",
            payload_json: "{}",
            occurred_at: "t",
        },
    )
    .unwrap();
    assert!(verify_chain(&conn, "T-legacy2")
        .unwrap_err()
        .contains("unanchored"));
    std::fs::remove_file(&db).ok();
}
