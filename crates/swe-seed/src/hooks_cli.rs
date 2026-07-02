use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum HooksAction {
    /// Append a hook event to the JSONL log (payload from stdin)
    Capture {
        event: String,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        trace_id: Option<String>,
        #[arg(long)]
        hook_id: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Export the event log
    Export {
        #[command(subcommand)]
        format: ExportFormat,
    },
    /// Compact oversized logs (gzip)
    CompactLogs,
    /// Rebuild the SQLite index from the JSONL logs
    Index,
    /// Routing enforcement at the hook layer: evaluate the route gate for the
    /// active trace (from `--trace-id` or `SWE_SEED_TRACE`), log a PreToolUse
    /// event (allowed/blocked), and exit 0 (allow) / 1 (block). When no trace
    /// is active, allows (nothing to enforce). Wire this as the host's
    /// PreToolUse hook command.
    RouteGate {
        #[arg(long)]
        trace_id: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ExportFormat {
    /// OpenTelemetry-compatible JSON
    Otel {
        #[arg(long)]
        output: Option<String>,
    },
    /// JUnit XML
    Junit {
        #[arg(long)]
        output: Option<String>,
    },
}

pub fn run_agent_hooks(action: HooksAction, root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::config::{load_yaml, HooksConfig};
    use swe_seed_core::hooks;

    let cfg_path = root.join(".agent-hooks/config.yaml");
    let hooks_cfg: HooksConfig = load_yaml(&cfg_path)?;
    let log_dir = root.join(&hooks_cfg.paths.logs);
    let index_db = root.join(&hooks_cfg.paths.index_db);
    let redact_cfg = hooks::RedactionConfig {
        key_substrings: hooks_cfg.redaction.key_substrings.clone(),
        value_patterns: hooks_cfg.redaction.value_patterns.clone(),
    };

    match action {
        HooksAction::Capture {
            event,
            session_id,
            trace_id,
            hook_id,
            status,
        } => {
            let mut payload: serde_json::Value = serde_json::Value::Object(Default::default());
            if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                let mut buf = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
                    .map_err(|e| anyhow::anyhow!("read hook payload from stdin: {e}"))?;
                if !buf.trim().is_empty() {
                    payload = serde_json::from_str(&buf)
                        .map_err(|e| anyhow::anyhow!("parse hook payload JSON from stdin: {e}"))?;
                }
            }
            let mut envelope = serde_json::json!({
                "event": event,
                "hook_id": hook_id.unwrap_or_else(|| event.clone()),
                "status": status.unwrap_or_else(|| "ok".into()),
                "trace_id": trace_id,
                "session_id": session_id,
                "attributes": payload,
            });
            let (log, event_id) = match hooks::append_event(&log_dir, &mut envelope, &redact_cfg) {
                Ok(record) => record,
                Err(e) => {
                    eprintln!("route-gate: audit log append failed; blocking hook: {e:#}");
                    return Ok(ExitCode::from(1));
                }
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "event_id": event_id,
                    "log": log.strip_prefix(root).unwrap_or(&log).display().to_string(),
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
        HooksAction::Export { format } => match format {
            ExportFormat::Otel { output } => {
                let payload = hooks::export::export_otel(&log_dir, root)?;
                emit_export(&serde_json::to_string_pretty(&payload)?, root, output)
            }
            ExportFormat::Junit { output } => {
                let payload = hooks::export::export_junit(&log_dir)?;
                emit_export(&payload, root, output)
            }
        },
        HooksAction::CompactLogs => {
            let min = hooks_cfg.logging.compact_min_size_bytes;
            let compacted = hooks::compact_logs(&log_dir, min)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "compacted": compacted.iter().map(|p| p.strip_prefix(root).unwrap_or(p).display().to_string()).collect::<Vec<_>>(),
                    "count": compacted.len(),
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
        HooksAction::Index => {
            let n = hooks::index::rebuild_index(&index_db, &log_dir)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "index_db": index_db.strip_prefix(root).unwrap_or(&index_db).display().to_string(),
                    "indexed_events": n,
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
        HooksAction::RouteGate { trace_id } => {
            use swe_seed_core::routing_gate::{route_gate, RouteGate};
            // Resolve the active trace: explicit --trace-id, else SWE_SEED_TRACE env.
            let trace_id = trace_id.or_else(|| std::env::var("SWE_SEED_TRACE").ok());
            let (status, reason, allowed) = match &trace_id {
                None => {
                    // No active trace → nothing to enforce; allow.
                    ("allowed", "no active trace (SWE_SEED_TRACE unset)".into(), true)
                }
                Some(tid) => match route_gate(root, tid) {
                    Ok(RouteGate::Allow) => ("allowed", String::new(), true),
                    Ok(RouteGate::Block(r)) => ("blocked", r, false),
                    Err(e) => ("blocked", format!("gate error: {e:#}"), false),
                },
            };
            let mut envelope = serde_json::json!({
                "event": "PreToolUse",
                "hook_id": "route-gate",
                "status": status,
                "trace_id": trace_id,
                "attributes": { "reason": reason },
            });
            let (log, event_id) = hooks::append_event(&log_dir, &mut envelope, &redact_cfg)?;
            let _ = log;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "event_id": event_id,
                    "trace_id": trace_id,
                    "status": status,
                    "allowed": allowed,
                }))?
            );
            Ok(if allowed { ExitCode::SUCCESS } else { ExitCode::from(1) })
        }
    }
}

fn emit_export(content: &str, root: &std::path::Path, output: Option<String>) -> Result<ExitCode> {
    if let Some(out) = output {
        let p = root.join(&out);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&p, content)?;
    }
    print!("{content}");
    Ok(ExitCode::SUCCESS)
}
