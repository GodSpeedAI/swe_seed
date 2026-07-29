//! Gateway stage-6 proof (spec 0020 §7, §8): session ids (>=128-bit entropy),
//! per-session/client/tool counters, max-call + duration enforcement, durable
//! journal replay after "crash", and a concurrent edge test proving counters
//! never overrun. Run: `cargo test gateway_governance`.

use std::collections::HashSet;
use std::fs;
use std::sync::Arc;
use std::time::Duration;

use swe_seed_core::gateway::{generate_session_id, GatewayEffectiveLimits, Governance};

fn temp_root(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-gw-gov-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn limits(session: u64, client: u64, tool: u64) -> GatewayEffectiveLimits {
    GatewayEffectiveLimits {
        session_max_calls: session,
        session_max_duration_secs: 0,
        client_max_requests: client,
        tool_max_requests: tool,
    }
}

#[test]
fn session_id_is_128_bit_unique() {
    let mut seen = HashSet::new();
    for _ in 0..256 {
        let id = generate_session_id();
        assert_eq!(id.len(), 32, "session id must be 32 hex chars (128 bits)");
        assert!(id.bytes().all(|b| b.is_ascii_hexdigit()));
        assert!(seen.insert(id), "session id collision");
    }
}

#[test]
fn check_and_increment_enforces_session_max_calls() {
    let root = temp_root("maxcalls");
    let gov = Governance::load(&root, limits(3, 100, 100)).unwrap();
    let session = gov.open_session("cli");
    for _ in 0..3 {
        gov.check_and_increment(&session.session_id, "cli", "fs.read").unwrap();
    }
    let err = gov
        .check_and_increment(&session.session_id, "cli", "fs.read")
        .unwrap_err();
    assert_eq!(err.reason_code(), "session_limit_exceeded");
    assert_eq!(gov.snapshot().session_calls.get(&session.session_id), Some(&3));
    fs::remove_dir_all(&root).ok();
}

#[test]
fn check_and_increment_enforces_client_budget() {
    let root = temp_root("clientbudget");
    let gov = Governance::load(&root, limits(100, 2, 100)).unwrap();
    let s1 = gov.open_session("cli");
    let s2 = gov.open_session("cli");
    gov.check_and_increment(&s1.session_id, "cli", "t").unwrap();
    gov.check_and_increment(&s2.session_id, "cli", "t").unwrap();
    // client budget exhausted
    let s3 = gov.open_session("cli");
    let err = gov.check_and_increment(&s3.session_id, "cli", "t").unwrap_err();
    assert_eq!(err.reason_code(), "budget_exceeded");
    fs::remove_dir_all(&root).ok();
}

#[test]
fn check_and_increment_enforces_tool_budget() {
    let root = temp_root("toolbudget");
    let gov = Governance::load(&root, limits(100, 100, 1)).unwrap();
    let s = gov.open_session("cli");
    gov.check_and_increment(&s.session_id, "cli", "fs.write").unwrap();
    let err = gov.check_and_increment(&s.session_id, "cli", "fs.write").unwrap_err();
    assert_eq!(err.reason_code(), "budget_exceeded");
    fs::remove_dir_all(&root).ok();
}

#[test]
fn journal_replays_counters_after_restart() {
    let root = temp_root("replay");
    {
        let gov = Governance::load(&root, limits(100, 100, 100)).unwrap();
        let s = gov.open_session("cli");
        gov.check_and_increment(&s.session_id, "cli", "fs.read").unwrap();
        gov.check_and_increment(&s.session_id, "cli", "fs.read").unwrap();
        gov.check_and_increment(&s.session_id, "cli", "fs.write").unwrap();
        // "crash": drop governance without graceful shutdown.
    }
    // restart: load replays the durable journal → counters restored.
    let gov = Governance::load(&root, limits(100, 100, 100)).unwrap();
    let snap = gov.snapshot();
    // one session, one client, two tools (fs.read x2, fs.write x1)
    assert_eq!(snap.session_calls.values().sum::<u64>(), 3);
    assert_eq!(snap.client_counters.get("cli"), Some(&3));
    assert_eq!(snap.tool_counters.get("fs.read"), Some(&2));
    assert_eq!(snap.tool_counters.get("fs.write"), Some(&1));
    fs::remove_dir_all(&root).ok();
}

#[test]
fn replayed_counter_is_enforced_after_restart() {
    // Pre-crash work counts against the post-restart limit (no undercount).
    // client limit = 3; do 3 calls under client "cli" pre-crash; on restart a
    // NEW session's first call must hit the carried-over client budget.
    let root = temp_root("replaylimit");
    {
        let gov = Governance::load(&root, limits(100, 3, 100)).unwrap();
        let s = gov.open_session("cli");
        for _ in 0..3 {
            gov.check_and_increment(&s.session_id, "cli", "t").unwrap();
        }
    }
    let gov = Governance::load(&root, limits(100, 3, 100)).unwrap();
    let snap = gov.snapshot();
    assert_eq!(snap.client_counters.get("cli"), Some(&3), "client counter carried over");
    let new_session = gov.open_session("cli");
    let err = gov
        .check_and_increment(&new_session.session_id, "cli", "t")
        .unwrap_err();
    assert_eq!(
        err.reason_code(),
        "budget_exceeded",
        "replayed client counter must still be enforced after restart"
    );
    fs::remove_dir_all(&root).ok();
}

#[test]
fn concurrent_calls_never_overrun_session_limit() {
    let root = temp_root("concurrency");
    let gov = Arc::new(Governance::load(&root, limits(64, 1_000_000, 1_000_000)).unwrap());
    let session = gov.open_session("cli");
    let session_id = session.session_id.clone();
    let gov = Arc::clone(&gov);

    let ok = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let blocked = Arc::new(std::sync::atomic::AtomicU64::new(0));
    std::thread::scope(|s| {
        for _ in 0..256 {
            let gov = Arc::clone(&gov);
            let ok = Arc::clone(&ok);
            let blocked = Arc::clone(&blocked);
            let sid = session_id.clone();
            s.spawn(move || {
                match gov.check_and_increment(&sid, "cli", "fs.read") {
                    Ok(()) => {
                        ok.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                    Err(e) => {
                        assert_eq!(e.reason_code(), "session_limit_exceeded");
                        blocked.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            });
        }
    });

    let ok = ok.load(std::sync::atomic::Ordering::SeqCst);
    let blocked = blocked.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(ok, 64, "exactly the session limit may succeed, no overrun");
    assert_eq!(blocked, 256 - 64);
    assert_eq!(
        gov.snapshot().session_calls.get(&session_id),
        Some(&64),
        "counter matches the limit exactly"
    );
    fs::remove_dir_all(&root).ok();
}

#[test]
fn zero_limit_means_unlimited() {
    let root = temp_root("unlimited");
    let gov = Governance::load(&root, limits(0, 0, 0)).unwrap();
    let s = gov.open_session("cli");
    for _ in 0..50 {
        gov.check_and_increment(&s.session_id, "cli", "t").unwrap();
    }
    assert_eq!(gov.snapshot().session_calls.get(&s.session_id), Some(&50));
    fs::remove_dir_all(&root).ok();
}

#[test]
fn duration_limit_enforced() {
    let root = temp_root("duration");
    let mut l = limits(100, 100, 100);
    l.session_max_duration_secs = 1;
    let gov = Governance::load(&root, l).unwrap();
    let s = gov.open_session("cli");
    gov.check_and_increment(&s.session_id, "cli", "t").unwrap();
    std::thread::sleep(Duration::from_millis(1100));
    let err = gov.check_and_increment(&s.session_id, "cli", "t").unwrap_err();
    assert_eq!(err.reason_code(), "session_limit_exceeded");
    fs::remove_dir_all(&root).ok();
}
