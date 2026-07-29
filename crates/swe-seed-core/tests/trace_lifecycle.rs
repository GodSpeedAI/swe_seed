//! trace_lifecycle: start/append/checkpoint/resume/distill/finish work end to
//! end, and TraceSchema rejects records missing identifier fields.
//!
//! Uses a temp root with the real route cards copied in, so routing works and
//! nothing is written into the real `.agent-harness/traces/`.

use std::fs;
use std::path::PathBuf;

use swe_seed_core::route::RouteResult;
use swe_seed_core::trace::{self, lifecycle, record::TraceRecord};

fn real_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

/// A temp project root with `.agent-harness/routes/` populated from the repo.
fn temp_root() -> PathBuf {
    let real = real_root();
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-trace-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(dir.join(".agent-harness/routes")).unwrap();
    for entry in fs::read_dir(real.join(".agent-harness/routes")).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().is_some_and(|x| x == "json") {
            let name = p.file_name().unwrap();
            fs::copy(&p, dir.join(".agent-harness/routes").join(name)).unwrap();
        }
    }
    dir
}

fn fixture_record(trace_id: &str, task: &str) -> TraceRecord {
    TraceRecord {
        trace_id: trace_id.into(),
        created_at: "2026-06-17T00:23:16+00:00".into(),
        task: task.into(),
        route_decision_record: ".agent-harness/traces/route-decisions/x.json".into(),
        route: RouteResult {
            job_type: "test".into(),
            route_card: ".agent-harness/routes/test.json".into(),
            confidence: "low".into(),
            assumption: None,
            required_context: vec![],
            required_skills: vec![],
            work_loop: vec![],
            required_artifacts: vec![],
            proof: vec![],
            done_when: vec![],
            next_action: "n".into(),
        },
        events: vec![],
        verification: vec![],
        unresolved_risks: vec![],
        completion_claim: None,
    }
}

#[test]
fn full_trace_lifecycle_with_unresolved_risk() {
    let root = temp_root();
    let started = lifecycle::start(&root, "lifecycle probe").expect("start");
    let trace_id = started["trace_id"].as_str().unwrap().to_string();
    assert!(root
        .join(started["trace_record"].as_str().unwrap())
        .is_file());
    assert!(started["route_decision_record"]
        .as_str()
        .unwrap()
        .contains("route-decisions"));

    // append adds a note event (now 2 events).
    let appended = lifecycle::append(&root, &trace_id, "a note").expect("append");
    assert_eq!(appended["events"].as_i64(), Some(2));

    // checkpoint records stage/summary/risks.
    let cp = lifecycle::checkpoint(
        &root,
        &trace_id,
        "change",
        "spec delta captured",
        Some("run targeted validation"),
        &["HARNESS_SPEC.md".to_string()],
        &["proof not run".to_string()],
    )
    .expect("checkpoint");
    assert_eq!(cp["checkpoint"]["stage"], "change");

    // resume surfaces the latest checkpoint + risks.
    let r = lifecycle::resume(&root, &trace_id).expect("resume");
    assert_eq!(r["latest_checkpoint"]["summary"], "spec delta captured");
    assert_eq!(r["unresolved_risks"][0], "proof not run");
    assert!(r["completion_claim"].is_null());

    // finish sets the completion claim (claim is required by the CLI; here it is supplied).
    // A verification entry is recorded so distill classifies a real status.
    let fin = lifecycle::finish(
        &root,
        &trace_id,
        "done",
        Some("just ci"),
        Some("incomplete"),
    )
    .expect("finish");
    assert_eq!(fin["completion_claim"]["claim"], "done");

    // distill: verification present but unresolved risk remains → partial.
    let d = lifecycle::distill(&root, &trace_id).expect("distill");
    assert_eq!(d["learning_review"]["verification_status"], "partial");
    assert_eq!(
        d["learning_review"]["candidate_memory_updates"][0]["destination"],
        "open-questions"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn distill_verified_when_clean_passing_verification() {
    let root = temp_root();
    let started = lifecycle::start(&root, "clean run").expect("start");
    let trace_id = started["trace_id"].as_str().unwrap().to_string();
    // No checkpoint risks; finish with passing verification evidence.
    lifecycle::finish(
        &root,
        &trace_id,
        "shipped",
        Some("just ci"),
        Some("all checks passed"),
    )
    .expect("finish");
    let d = lifecycle::distill(&root, &trace_id).expect("distill");
    assert_eq!(d["learning_review"]["verification_status"], "verified");
    assert_eq!(
        d["learning_review"]["candidate_memory_updates"][0]["destination"],
        "successful-patterns"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn trace_schema_rejects_missing_identifiers() {
    let schema = trace::trace_schema();

    let mut rec = fixture_record("", "some task");
    assert_eq!(
        trace::missing_identifiers(&schema, &rec),
        vec!["trace_id".to_string()]
    );

    rec.trace_id = "20260617T002316Z-lifecycle-probe".into();
    rec.task = "".into();
    assert_eq!(
        trace::missing_identifiers(&schema, &rec),
        vec!["task".to_string()]
    );

    rec.task = "has task".into();
    assert!(trace::missing_identifiers(&schema, &rec).is_empty());
}

#[test]
fn missing_trace_record_errors() {
    let root = temp_root();
    let err = lifecycle::resume(&root, "20260728T000000Z-definitely-missing").unwrap_err();
    assert!(err.to_string().contains("trace record not found"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn trace_id_does_not_leak_secret_from_task() {
    // The trace_id and both filenames must be built from the redacted task, so
    // a secret in the task cannot leak into the id/filename (only the stored
    // task field carries the redacted form).
    let root = temp_root();
    let secret = "sk-abcdef01234567890123456789";
    let started = lifecycle::start(&root, &format!("{secret} do work")).expect("start");
    let trace_id = started["trace_id"].as_str().unwrap();
    let trace_rec = started["trace_record"].as_str().unwrap();
    let route_rec = started["route_decision_record"].as_str().unwrap();

    assert!(
        !trace_id.contains(secret),
        "trace_id leaks secret: {trace_id}"
    );
    assert!(
        !trace_rec.contains(secret),
        "trace_record path leaks secret: {trace_rec}"
    );
    assert!(
        !route_rec.contains(secret),
        "route_decision path leaks secret: {route_rec}"
    );
    assert!(
        trace_id.ends_with("redacted-do-work"),
        "trace_id not sanitized: {trace_id}"
    );

    // Stored task field is the redacted form, never the raw secret.
    let rec = TraceRecord::load(&root.join(trace_rec)).expect("load record");
    assert!(!rec.task.contains(secret));
    assert!(rec.task.contains("[REDACTED]"));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn trace_updates_stay_inside_the_repository() {
    let root = temp_root();
    let outside = std::env::temp_dir().join(format!(
        "swe-seed-external-trace-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fixture_record("external", "outside")
        .save(&outside)
        .unwrap();

    let err = lifecycle::append(&root, outside.to_str().unwrap(), "must not write").unwrap_err();
    assert!(err.to_string().contains("trace id"));
    assert!(TraceRecord::load(&outside).unwrap().events.is_empty());

    let _ = fs::remove_file(&outside);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn trace_updates_require_a_routed_trace_record() {
    let root = temp_root();
    let trace_id = "20260728T000000Z-manual";
    let path = lifecycle::resolve_trace_path(&root, trace_id).unwrap();
    fixture_record(trace_id, "manual").save(&path).unwrap();

    let err = lifecycle::append(&root, trace_id, "must not write").unwrap_err();
    assert!(err.to_string().contains("routing genesis"));
    assert!(TraceRecord::load(&path).unwrap().events.is_empty());

    let _ = fs::remove_dir_all(&root);
}

#[cfg(unix)]
#[test]
fn trace_updates_reject_record_symlinks() {
    use std::os::unix::fs::symlink;

    let root = temp_root();
    let started = lifecycle::start(&root, "safe trace").unwrap();
    let trace_id = started["trace_id"].as_str().unwrap();
    let path = root.join(started["trace_record"].as_str().unwrap());
    let outside = std::env::temp_dir().join(format!("swe-seed-symlink-target-{}.json", trace_id));
    fixture_record(trace_id, "outside").save(&outside).unwrap();
    fs::remove_file(&path).unwrap();
    symlink(&outside, &path).unwrap();

    assert!(lifecycle::append(&root, trace_id, "must not write").is_err());
    assert!(TraceRecord::load(&outside).unwrap().events.is_empty());

    let _ = fs::remove_file(&outside);
    let _ = fs::remove_dir_all(&root);
}

#[cfg(unix)]
#[test]
fn trace_updates_reject_symlinked_record_directories() {
    use std::os::unix::fs::symlink;

    let root = temp_root();
    let started = lifecycle::start(&root, "safe trace").unwrap();
    let trace_id = started["trace_id"].as_str().unwrap();
    let records = root.join(".agent-harness/traces/records");
    let outside = std::env::temp_dir().join(format!("swe-seed-record-dir-{}", trace_id));
    fs::create_dir_all(&outside).unwrap();
    fixture_record(trace_id, "outside")
        .save(&outside.join(format!("{trace_id}.json")))
        .unwrap();
    fs::remove_dir_all(&records).unwrap();
    symlink(&outside, &records).unwrap();

    assert!(lifecycle::append(&root, trace_id, "must not write").is_err());
    assert!(TraceRecord::load(&outside.join(format!("{trace_id}.json")))
        .unwrap()
        .events
        .is_empty());

    let _ = fs::remove_file(&records);
    let _ = fs::remove_dir_all(&outside);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn trace_updates_redact_user_supplied_text() {
    let root = temp_root();
    let secret = "sk-abcdef01234567890123456789";
    let started = lifecycle::start(&root, "clean task").unwrap();
    let trace_id = started["trace_id"].as_str().unwrap();

    lifecycle::append(&root, trace_id, secret).unwrap();
    lifecycle::checkpoint(
        &root,
        trace_id,
        secret,
        secret,
        Some(secret),
        &[secret.to_string()],
        &[secret.to_string()],
    )
    .unwrap();
    lifecycle::finish(&root, trace_id, secret, Some(secret), Some(secret)).unwrap();

    let record = fs::read_to_string(root.join(started["trace_record"].as_str().unwrap())).unwrap();
    assert!(!record.contains(secret));
    assert!(record.contains("[REDACTED]"));

    let _ = fs::remove_dir_all(&root);
}
