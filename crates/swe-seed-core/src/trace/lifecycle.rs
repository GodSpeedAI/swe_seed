//! Trace lifecycle: start / append / checkpoint / resume / distill / finish.
//! Ports `scripts/harness.py` trace_* so Rust records stay compatible.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use super::record::TraceRecord;
use crate::route::{build_route_result, write_route_decision};
use crate::util::{redact_secrets, slugify, utc_now, utc_stamp};

fn ledger_append_result(
    root: &Path,
    trace_id: &str,
    event_type: &str,
    payload_json: &str,
) -> Result<()> {
    let db = crate::trace_ledger::default_db_path(root);
    crate::trace_ledger::open(&db).and_then(|c| {
        crate::trace_ledger::append(
            &c,
            crate::trace_ledger::LedgerEntry {
                trace_id,
                event_type,
                payload_json,
                occurred_at: &utc_now(),
            },
        )
        .map(|_| ())
    })
}

/// Best-effort append to the tamper-evident trace ledger. Opens the ledger DB
/// at the default path and appends; failures are warned to stderr but never
/// break follow-up trace commands. The genesis append is handled separately
/// because routing evidence is required.
fn ledger_append(root: &Path, trace_id: &str, event_type: &str, payload_json: &str) {
    if let Err(e) = ledger_append_result(root, trace_id, event_type, payload_json) {
        eprintln!("trace ledger: best-effort append failed ({event_type}): {e}");
    }
}

fn records_dir(root: &Path) -> Result<PathBuf> {
    let mut dir = root
        .canonicalize()
        .with_context(|| format!("resolve repository root {}", root.display()))?;
    for component in [".agent-harness", "traces", "records"] {
        dir.push(component);
        match std::fs::symlink_metadata(&dir) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                bail!(
                    "trace records directory contains symlink: {}",
                    dir.display()
                );
            }
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => bail!("trace records path is not a directory: {}", dir.display()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(&dir)
                    .with_context(|| format!("create trace records directory {}", dir.display()))?;
            }
            Err(error) => return Err(error).with_context(|| format!("inspect {}", dir.display())),
        }
    }
    Ok(dir)
}

/// Resolve a generated trace id to its repository-local record file.
pub fn resolve_trace_path(root: &Path, trace: &str) -> Result<PathBuf> {
    let trace = trace.strip_suffix(".json").unwrap_or(trace);
    let Some((stamp, slug)) = trace.split_once('-') else {
        bail!("trace id must be a generated timestamp-slug");
    };
    let valid_stamp = stamp.len() == 16
        && stamp.as_bytes().get(8) == Some(&b'T')
        && stamp.as_bytes().get(15) == Some(&b'Z')
        && stamp
            .bytes()
            .enumerate()
            .all(|(index, byte)| matches!(index, 8 | 15) || byte.is_ascii_digit());
    if !valid_stamp
        || slug.is_empty()
        || !slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        bail!("trace id must be a generated timestamp-slug");
    }
    Ok(records_dir(root)?.join(format!("{trace}.json")))
}

/// `trace start`: route the task, write a route decision + trace record.
/// Returns the printed result `{ trace_id, trace_record, route_decision_record }`.
pub fn start(root: &Path, task: &str) -> Result<Value> {
    let route_result = build_route_result(root, task)?;
    let records_dir = records_dir(root)?;
    let trace_id = format!("{}-{}", utc_stamp(), slugify(&redact_secrets(task)));
    let route_decision_record = write_route_decision(root, task, &route_result)?;
    // Genesis: the routing decision is the first chain entry (tamper-evident).
    let genesis_payload = serde_json::json!({
        "task": redact_secrets(task),
        "job_type": route_result.job_type,
        "route_card": route_result.route_card,
    })
    .to_string();
    ledger_append_result(root, &trace_id, "RouteSelected", &genesis_payload)?;
    let record = TraceRecord {
        trace_id: trace_id.clone(),
        created_at: utc_now(),
        task: redact_secrets(task),
        route_decision_record: route_decision_record.clone(),
        route: route_result,
        events: vec![json!({
            "at": utc_now(),
            "type": "trace.start",
            "note": "Trace record created before implementation work.",
        })],
        verification: Vec::new(),
        unresolved_risks: Vec::new(),
        completion_claim: None,
    };
    let path = records_dir.join(format!("{trace_id}.json"));
    record.save(&path)?;
    Ok(json!({
        "trace_id": trace_id,
        "trace_record": rel(root, &path),
        "route_decision_record": route_decision_record,
    }))
}

fn load_for_update(root: &Path, trace: &str) -> Result<(PathBuf, TraceRecord)> {
    let path = resolve_trace_path(root, trace)?;
    let metadata = std::fs::symlink_metadata(&path)
        .with_context(|| format!("trace record not found: {trace}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("trace record not found: {trace}");
    }
    let rec = TraceRecord::load(&path)?;
    let trace_id = trace.strip_suffix(".json").unwrap_or(trace);
    if rec.trace_id != trace_id {
        bail!("trace record id does not match requested trace");
    }
    let db = crate::trace_ledger::default_db_path(root);
    let conn = crate::trace_ledger::open(&db)?;
    if !crate::trace_ledger::has_genesis(&conn, trace_id) {
        bail!("trace record has no routing genesis");
    }
    let genesis_payload: serde_json::Value = conn
        .query_row(
            "SELECT payload_json FROM chain_entries WHERE trace_id = ?1 AND seq = 0 AND event_type = 'RouteSelected'",
            rusqlite::params![trace_id],
            |row| row.get::<_, String>(0),
        )
        .context("load trace routing genesis")
        .and_then(|payload| serde_json::from_str(&payload).context("parse trace routing genesis"))?;
    if genesis_payload.get("task").and_then(Value::as_str) != Some(rec.task.as_str())
        || genesis_payload.get("job_type").and_then(Value::as_str)
            != Some(rec.route.job_type.as_str())
        || genesis_payload.get("route_card").and_then(Value::as_str)
            != Some(rec.route.route_card.as_str())
    {
        bail!("trace record does not match routing genesis");
    }
    Ok((path, rec))
}

/// `trace append <trace> <note>` → appends a `trace.note` event.
pub fn append(root: &Path, trace: &str, note: &str) -> Result<Value> {
    let (path, mut record) = load_for_update(root, trace)?;
    let note = redact_secrets(note);
    record.events.push(json!({
        "at": utc_now(),
        "type": "trace.note",
        "note": note,
    }));
    record.save(&path)?;
    ledger_append(
        root,
        &record.trace_id,
        "TraceNote",
        &json!({ "note": note }).to_string(),
    );
    Ok(json!({
        "trace_record": rel(root, &path),
        "events": record.events.len(),
    }))
}

/// `trace checkpoint` → appends a `trace.checkpoint` event; updates unresolved
/// risks when any are supplied.
pub fn checkpoint(
    root: &Path,
    trace: &str,
    stage: &str,
    summary: &str,
    next_action: Option<&str>,
    artifacts: &[String],
    unresolved_risks: &[String],
) -> Result<Value> {
    let (path, mut record) = load_for_update(root, trace)?;
    let stage = redact_secrets(stage);
    let summary = redact_secrets(summary);
    let next_action = next_action.map(redact_secrets);
    let artifacts: Vec<String> = artifacts
        .iter()
        .map(|value| redact_secrets(value))
        .collect();
    let unresolved_risks: Vec<String> = unresolved_risks
        .iter()
        .map(|value| redact_secrets(value))
        .collect();
    let checkpoint = json!({
        "at": utc_now(),
        "type": "trace.checkpoint",
        "stage": stage,
        "summary": summary,
        "next_action": next_action,
        "artifacts": artifacts,
        "unresolved_risks": unresolved_risks,
    });
    record.events.push(checkpoint.clone());
    if !unresolved_risks.is_empty() {
        record.unresolved_risks = unresolved_risks;
    }
    record.save(&path)?;
    ledger_append(
        root,
        &record.trace_id,
        "TraceCheckpoint",
        &checkpoint.to_string(),
    );
    Ok(json!({
        "trace_record": rel(root, &path),
        "checkpoint": checkpoint,
    }))
}

/// `trace resume` → print the resume summary (last checkpoint, risks, claim).
pub fn resume(root: &Path, trace: &str) -> Result<Value> {
    let (_path, record) = load_for_update(root, trace)?;
    let latest_checkpoint = record.latest_checkpoint().cloned();
    Ok(json!({
        "trace_id": record.trace_id,
        "task": record.task,
        "route": record.route,
        "latest_checkpoint": latest_checkpoint,
        "unresolved_risks": record.unresolved_risks,
        "completion_claim": record.completion_claim,
    }))
}

/// `trace finish` → requires a claim; sets completion_claim (+ optional verification).
pub fn finish(
    root: &Path,
    trace: &str,
    claim: &str,
    command: Option<&str>,
    result: Option<&str>,
) -> Result<Value> {
    let (path, mut record) = load_for_update(root, trace)?;
    let now = utc_now();
    let claim = redact_secrets(claim);
    record.completion_claim = Some(super::record::CompletionClaim {
        at: now.clone(),
        claim: claim.clone(),
    });
    if let Some(cmd) = command {
        let result = result.map(redact_secrets);
        record.verification.push(json!({
            "at": now,
            "command": redact_secrets(cmd),
            "result": result.unwrap_or_else(|| "not recorded".into()),
        }));
    }
    record.save(&path)?;
    ledger_append(
        root,
        &record.trace_id,
        "TraceFinished",
        &json!({ "claim": claim }).to_string(),
    );
    Ok(json!({
        "trace_record": rel(root, &path),
        "completion_claim": record.completion_claim,
    }))
}

/// `trace distill` → produce a learning review from the record.
pub fn distill(root: &Path, trace: &str) -> Result<Value> {
    let (path, record) = load_for_update(root, trace)?;
    let job_type = record.route.job_type.clone();
    let route_card = record.route.route_card.clone();
    let latest_checkpoint = record.latest_checkpoint().cloned().unwrap_or(json!({}));
    let completion_claim = record.completion_claim.clone().unwrap_or_default();
    let summary = latest_checkpoint
        .get("summary")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .or_else(|| Some(completion_claim.claim.clone()))
        .unwrap_or_else(|| record.task.clone());
    let verification_status = classify_verification_status(&record);
    let unresolved_risks: Vec<String> = record.unresolved_risks.clone();
    let checkpoint_artifacts: Vec<String> = latest_checkpoint
        .get("artifacts")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let checkpoint_next_action = latest_checkpoint
        .get("next_action")
        .and_then(Value::as_str)
        .map(|s| s.to_string());

    let mut evidence = vec![
        format!("trace:{}", record.trace_id),
        format!("route:{route_card}"),
    ];
    for a in &checkpoint_artifacts {
        evidence.push(format!("artifact:{a}"));
    }

    let candidate_memory_updates = if verification_status == "verified"
        && unresolved_risks.is_empty()
    {
        vec![json!({
            "destination": "successful-patterns",
            "note": format!("{job_type} trace reached proof-backed completion: {summary}"),
            "rationale": "Preserve the validated pattern without storing the whole transcript.",
            "evidence": evidence,
        })]
    } else {
        let destination = if unresolved_risks.is_empty() {
            "failure-patterns"
        } else {
            "open-questions"
        };
        let mut ev = evidence.clone();
        for r in &unresolved_risks {
            ev.push(format!("risk:{r}"));
        }
        vec![json!({
            "destination": destination,
            "note": format!("{job_type} trace ended with incomplete proof or unresolved risk: {summary}"),
            "rationale": "Carry forward the blocker so the next agent sees the gap without rereading the transcript.",
            "evidence": ev,
        })]
    };

    let required_skills = &record.route.required_skills;
    let candidate_skill_updates = if !required_skills.is_empty() {
        vec![json!({
            "priority": 1,
            "action": "patch_existing_skill",
            "target": required_skills[0],
            "summary": format!("Patch the governing skill first if this trace exposed a reusable lesson: {summary}"),
            "rationale": "Prefer updating the active governing skill before creating a new one.",
            "evidence": evidence,
        })]
    } else {
        vec![json!({
            "priority": 2,
            "action": "add_support_file",
            "target": "existing umbrella skill or future skill-authoring target",
            "summary": format!("If this trace revealed reusable detail, prefer a reference, template, or script before a new standalone skill: {summary}"),
            "rationale": "Support files capture durable operational detail with less library sprawl than creating a new skill.",
            "evidence": evidence,
        })]
    };

    let mut candidate_harness_updates = Vec::new();
    if job_type == "harness_improvement"
        || checkpoint_artifacts
            .iter()
            .any(|a| a.starts_with(".agent-harness/") || a == "HARNESS_SPEC.md")
    {
        let target = if checkpoint_artifacts.iter().any(|a| a == "HARNESS_SPEC.md") {
            "spec"
        } else {
            "validation"
        };
        candidate_harness_updates.push(json!({
            "target": target,
            "summary": format!("Review whether the trace lesson should be promoted into the harness contract: {summary}"),
            "rationale": "Harness-facing work should convert repeated friction into a validated contract rather than a one-off note.",
            "validation": [
                {"command": "just harness-validate", "expected": "passes with the promoted contract"},
                {"command": "just ci", "expected": "smoke checks cover the new behavior"},
            ],
        }));
    }

    let mut preserve = vec![summary.clone()];
    if let Some(na) = checkpoint_next_action {
        preserve.push(na);
    }
    for r in &unresolved_risks {
        preserve.push(format!("risk: {r}"));
    }

    let learning_review = json!({
        "trace_id": record.trace_id,
        "route": {"job_type": job_type, "route_card": route_card},
        "summary": summary,
        "verification_status": verification_status,
        "candidate_memory_updates": candidate_memory_updates,
        "candidate_skill_updates": candidate_skill_updates,
        "candidate_harness_updates": candidate_harness_updates,
        "compression_handoff": {"preserve_for_resume": preserve},
        "provenance": {
            "generated_by": "trace.distill",
            "generated_at": utc_now(),
            "trace_record": rel(root, &path),
            "route_decision_record": record.route_decision_record,
        },
    });
    Ok(json!({ "learning_review": learning_review }))
}

fn classify_verification_status(record: &TraceRecord) -> String {
    let verification = &record.verification;
    if verification.is_empty() {
        return "missing".to_string();
    }
    if !record.unresolved_risks.is_empty() {
        return "partial".to_string();
    }
    let text: String = verification
        .iter()
        .filter_map(|v| v.get("result").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let markers = ["exit 0", "passed", "success", "all checks passed"];
    if markers.iter().any(|m| text.contains(m)) {
        "verified".to_string()
    } else {
        "partial".to_string()
    }
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned())
}
