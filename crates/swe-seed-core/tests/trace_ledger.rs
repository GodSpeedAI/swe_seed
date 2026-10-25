//! trace_ledger: hash-chained append-only log. A route-decision genesis,
//! tamper-evidence on partial edits, and a chain-root accessor (Phase A).

use std::sync::Once;

use swe_seed_core::trace_ledger::{
    adopt_legacy_anchors, append, append_with_crash_point, chain_root, open, verify_chain,
    verify_chain_with, AnchorPolicy, CrashPoint, LedgerEntry, ANCHOR_KEY_ENV, GENESIS_PREV,
};

/// Every test in this binary shares one anchor key, kept outside the ledger dirs.
/// The env var is set once, before any ledger call, so no test reads it mid-write.
static KEY_ENV: Once = Once::new();

fn use_test_anchor_key() {
    KEY_ENV.call_once(|| {
        let key =
            std::env::temp_dir().join(format!("swe-seed-anchor-test-{}.key", std::process::id()));
        std::fs::remove_file(&key).ok();
        std::env::set_var(ANCHOR_KEY_ENV, &key);
    });
}

fn temp_db() -> std::path::PathBuf {
    use_test_anchor_key();
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

#[test]
fn append_after_trailing_row_deleted_is_refused_and_truncation_stays_visible() {
    // Deleting the last row then appending must not reuse the seq and overwrite
    // the head anchor, which would hide the truncation.
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-trunc-append");
    let anchor_before = std::fs::read_to_string(db.with_extension("anchors.json")).unwrap();
    conn.execute(
        "DELETE FROM chain_entries WHERE trace_id = 'T-trunc-append' AND seq = 2",
        [],
    )
    .unwrap();

    let err = append(
        &conn,
        swe_seed_core::trace_ledger::LedgerEntry {
            trace_id: "T-trunc-append",
            event_type: "TraceNote",
            payload_json: "{}",
            occurred_at: "t",
        },
    )
    .unwrap_err();
    assert!(
        format!("{err:#}").contains("head anchor mismatch before append"),
        "{err:#}"
    );

    // Nothing committed: still two rows, and the anchor file is untouched.
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM chain_entries WHERE trace_id = 'T-trunc-append'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);
    assert_eq!(
        std::fs::read_to_string(db.with_extension("anchors.json")).unwrap(),
        anchor_before
    );

    // Truncation is still reported by verification.
    let verr = verify_chain(&conn, "T-trunc-append").unwrap_err();
    assert!(verr.contains("head anchor mismatch"), "{verr}");
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}

fn entry<'a>(trace_id: &'a str, event_type: &'a str) -> LedgerEntry<'a> {
    LedgerEntry {
        trace_id,
        event_type,
        payload_json: "{}",
        occurred_at: "t",
    }
}

fn row_count(conn: &rusqlite::Connection, trace_id: &str) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM chain_entries WHERE trace_id = ?1",
        rusqlite::params![trace_id],
        |r| r.get(0),
    )
    .unwrap()
}

fn read_anchor_json(db: &std::path::Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(db.with_extension("anchors.json")).unwrap())
        .unwrap()
}

#[test]
fn crash_after_anchor_before_commit_self_heals() {
    // The pending anchor reached disk, COMMIT never ran: the row rolls back and the
    // committed state still matches the ledger.
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-pre");
    let err = append_with_crash_point(&conn, entry("T-pre", "TraceNote"), CrashPoint::BeforeCommit)
        .unwrap_err();
    assert!(format!("{err:#}").contains("simulated crash before COMMIT"));
    assert_eq!(row_count(&conn, "T-pre"), 3);
    let json = read_anchor_json(&db);
    assert_eq!(json["anchors"]["T-pre"]["length"], 3);
    assert_eq!(json["anchors"]["T-pre"]["pending"]["length"], 4);
    assert!(verify_chain(&conn, "T-pre").is_ok());

    // The next append discards the stale pending state and extends the chain.
    let next = append(&conn, entry("T-pre", "TraceNote")).unwrap();
    assert_eq!(next.seq, 3);
    assert_eq!(row_count(&conn, "T-pre"), 4);
    let json = read_anchor_json(&db);
    assert_eq!(json["anchors"]["T-pre"]["length"], 4);
    assert!(json["anchors"]["T-pre"].get("pending").is_none());
    assert!(verify_chain(&conn, "T-pre").is_ok());
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}

#[test]
fn crash_after_commit_before_finalize_self_heals() {
    // COMMIT landed, the anchor was never promoted: the pending state matches the
    // ledger and verifies. The next append promotes it.
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-post");
    let err = append_with_crash_point(
        &conn,
        entry("T-post", "TraceNote"),
        CrashPoint::BeforeFinalize,
    )
    .unwrap_err();
    assert!(format!("{err:#}").contains("simulated crash before anchor finalize"));
    assert_eq!(row_count(&conn, "T-post"), 4);
    let json = read_anchor_json(&db);
    assert_eq!(json["anchors"]["T-post"]["length"], 3);
    assert_eq!(json["anchors"]["T-post"]["pending"]["length"], 4);
    assert!(verify_chain(&conn, "T-post").is_ok());

    let next = append(&conn, entry("T-post", "TraceNote")).unwrap();
    assert_eq!(next.seq, 4);
    let json = read_anchor_json(&db);
    assert_eq!(json["anchors"]["T-post"]["length"], 5);
    assert!(json["anchors"]["T-post"].get("pending").is_none());
    assert!(verify_chain(&conn, "T-post").is_ok());
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}

#[test]
fn tampered_anchor_with_stale_signature_fails() {
    // Truncate the trailing row and rewrite the anchor's length/head to match the
    // shorter chain. The signature was made over the original anchors, so it fails.
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-forge");
    let prev_head: String = conn
        .query_row(
            "SELECT hash FROM chain_entries WHERE trace_id = 'T-forge' AND seq = 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    conn.execute(
        "DELETE FROM chain_entries WHERE trace_id = 'T-forge' AND seq = 2",
        [],
    )
    .unwrap();
    let mut json = read_anchor_json(&db);
    json["anchors"]["T-forge"]["length"] = serde_json::json!(2);
    json["anchors"]["T-forge"]["head"] = serde_json::json!(prev_head);
    std::fs::write(
        db.with_extension("anchors.json"),
        serde_json::to_string_pretty(&json).unwrap(),
    )
    .unwrap();
    let err = verify_chain(&conn, "T-forge").unwrap_err();
    assert!(err.contains("signature does not verify"), "{err}");
    assert!(verify_chain_with(&conn, "T-forge", AnchorPolicy::AllowUnanchored).is_err());
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}

#[test]
fn anchor_key_is_created_owner_only() {
    let db = temp_db();
    let conn = open(&db).unwrap();
    append(&conn, entry("T-key", "RouteSelected")).unwrap();
    let key = std::path::PathBuf::from(std::env::var_os(ANCHOR_KEY_ENV).unwrap());
    assert!(key.is_file(), "anchor key created at {}", key.display());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&key).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}

#[test]
fn legacy_unsigned_anchor_refuses_append_until_adopted() {
    // An unsigned legacy anchor is never signed by an append. The operator adopts it once.
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-legacy-signed");
    let legacy = read_anchor_json(&db)["anchors"].clone();
    std::fs::write(
        db.with_extension("anchors.json"),
        serde_json::to_string_pretty(&legacy).unwrap(),
    )
    .unwrap();

    let err = verify_chain(&conn, "T-legacy-signed").unwrap_err();
    assert!(err.contains("unsigned (legacy)"), "{err}");
    assert!(err.contains("adopt-anchors"), "{err}");
    assert!(verify_chain_with(&conn, "T-legacy-signed", AnchorPolicy::AllowUnanchored).is_ok());

    // Append refuses and writes nothing.
    let err = append(&conn, entry("T-legacy-signed", "TraceNote")).unwrap_err();
    assert!(format!("{err:#}").contains("adopt-anchors"), "{err:#}");
    assert_eq!(row_count(&conn, "T-legacy-signed"), 3);
    assert!(read_anchor_json(&db).get("signature").is_none());

    // The operator adoption signs the file, after which verify and append work.
    let adopted = adopt_legacy_anchors(&conn).unwrap();
    assert_eq!(adopted, vec![("T-legacy-signed".to_string(), 3)]);
    assert!(read_anchor_json(&db).get("signature").is_some());
    assert!(verify_chain(&conn, "T-legacy-signed").is_ok());
    let next = append(&conn, entry("T-legacy-signed", "TraceNote")).unwrap();
    assert_eq!(next.seq, 3);
    assert!(verify_chain(&conn, "T-legacy-signed").is_ok());
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}

#[test]
fn legacy_swap_after_truncation_cannot_be_appended() {
    // The downgrade attack: truncate the rows, replace the signed anchor file with an
    // unsigned one that matches the truncated chain, then append. The append must
    // refuse, so the attacker's history is never signed.
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-downgrade");
    let prev_head: String = conn
        .query_row(
            "SELECT hash FROM chain_entries WHERE trace_id = 'T-downgrade' AND seq = 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    conn.execute(
        "DELETE FROM chain_entries WHERE trace_id = 'T-downgrade' AND seq = 2",
        [],
    )
    .unwrap();
    let forged = serde_json::json!({"T-downgrade": {"length": 2, "head": prev_head}});
    std::fs::write(
        db.with_extension("anchors.json"),
        serde_json::to_string_pretty(&forged).unwrap(),
    )
    .unwrap();

    let err = append(&conn, entry("T-downgrade", "TraceNote")).unwrap_err();
    assert!(format!("{err:#}").contains("unsigned (legacy)"), "{err:#}");
    assert_eq!(row_count(&conn, "T-downgrade"), 2);
    assert!(read_anchor_json(&db).get("signature").is_none());
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}

#[test]
fn adopt_refuses_legacy_anchor_that_disagrees_with_chain() {
    // Adoption signs every anchored chain, so a mismatch must refuse and sign nothing.
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-legacy-trunc");
    let legacy = read_anchor_json(&db)["anchors"].clone();
    std::fs::write(
        db.with_extension("anchors.json"),
        serde_json::to_string_pretty(&legacy).unwrap(),
    )
    .unwrap();
    conn.execute(
        "DELETE FROM chain_entries WHERE trace_id = 'T-legacy-trunc' AND seq = 2",
        [],
    )
    .unwrap();
    let err = adopt_legacy_anchors(&conn).unwrap_err();
    assert!(format!("{err:#}").contains("refusing to adopt"), "{err:#}");
    assert!(read_anchor_json(&db).get("signature").is_none());
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}

#[test]
fn adopt_refuses_already_signed_file() {
    let db = temp_db();
    let conn = open(&db).unwrap();
    append_three(&conn, "T-already");
    let before = std::fs::read_to_string(db.with_extension("anchors.json")).unwrap();
    let err = adopt_legacy_anchors(&conn).unwrap_err();
    assert!(format!("{err:#}").contains("already signed"), "{err:#}");
    assert_eq!(
        std::fs::read_to_string(db.with_extension("anchors.json")).unwrap(),
        before
    );
    std::fs::remove_file(&db).ok();
    std::fs::remove_file(db.with_extension("anchors.json")).ok();
}
