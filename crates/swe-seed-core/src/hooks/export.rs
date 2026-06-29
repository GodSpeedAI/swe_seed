//! OTEL-compatible JSON + JUnit XML export of the hook event log (spec 0005).

use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

use super::runtime::iter_events;

/// Build an OpenTelemetry-compatible logs JSON from the event log.
pub fn export_otel(log_dir: &Path, root: &Path) -> Result<Value> {
    let records = iter_events(log_dir)?;
    let log_records: Vec<Value> = records
        .iter()
        .map(|(env, _)| {
            json!({
                "timeUnixNano": env.get("timestamp"),
                "severityText": env.get("status").cloned().unwrap_or(Value::String("ok".into())),
                "traceId": env.get("trace_id").cloned().unwrap_or(Value::Null),
                "spanId": env.get("span_id").cloned().unwrap_or(Value::Null),
                "body": { "stringValue": env.get("event").cloned().unwrap_or(Value::Null) },
                "attributes": [
                    { "key": "event_id", "value": { "stringValue": env.get("event_id").cloned().unwrap_or(Value::Null) } },
                    { "key": "hook_id", "value": { "stringValue": env.get("hook_id").cloned().unwrap_or(Value::Null) } },
                    { "key": "session_id", "value": { "stringValue": env.get("session_id").cloned().unwrap_or(Value::Null) } },
                ],
            })
        })
        .collect();
    Ok(json!({
        "resource": {
            "attributes": {
                "service.name": "swe-seed-dev-harness",
                "service.namespace": "dev-harness",
                "service.version": "0.2.0",
                "repo.root": root.to_string_lossy(),
            }
        },
        "scopeLogs": [
            { "scope": { "name": "agent-hooks" }, "logRecords": log_records },
        ],
    }))
}

/// Build a JUnit XML suite from the event log. Non-`ok` events become failures.
pub fn export_junit(log_dir: &Path) -> Result<String> {
    let records = iter_events(log_dir)?;
    let total = records.len();
    let mut failures = 0usize;
    let mut cases = String::new();
    for (env, _) in &records {
        let status = env.get("status").and_then(|v| v.as_str()).unwrap_or("ok");
        let classname = env
            .get("hook_id")
            .and_then(|v| v.as_str())
            .unwrap_or("agent-hooks");
        let name = env
            .get("event_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown-event");
        let dur = env
            .get("duration_ms")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            / 1000.0;
        if status != "ok" {
            failures += 1;
            let msg = xml_escape(
                env.get("event")
                    .and_then(|v| v.as_str())
                    .unwrap_or("failed-event"),
            );
            let detail = xml_escape(&format!(
                "status={} exit_code={}",
                status,
                env.get("exit_code")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "null".into())
            ));
            cases.push_str(&format!(
                "    <testcase classname=\"{classname}\" name=\"{name}\" time=\"{dur}\">\n      <failure message=\"{msg}\">{detail}</failure>\n    </testcase>\n"
            ));
        } else {
            cases.push_str(&format!(
                "    <testcase classname=\"{classname}\" name=\"{name}\" time=\"{dur}\" />\n"
            ));
        }
    }
    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuite name=\"agent-hooks-observability\" tests=\"{total}\" failures=\"{failures}\">\n{cases}</testsuite>\n"
    ))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
