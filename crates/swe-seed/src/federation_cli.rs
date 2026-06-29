use std::process::ExitCode;

use anyhow::Result;
use clap::{Subcommand, ValueEnum};

#[derive(Subcommand)]
pub enum FederationAction {
    /// Show resolved federation flags + domain_model_hash
    Status,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FederationFlag {
    Off,
    On,
}

fn federation_config_from(
    flag: Option<FederationFlag>,
) -> swe_seed_core::federation::FederationConfig {
    use swe_seed_core::federation::{FederationConfig, SettlementMode};
    let mut cfg = FederationConfig::default();
    match flag {
        Some(FederationFlag::On) => {
            cfg.enabled = true;
            cfg.emit_envelope = true;
            cfg.consume_envelope = true;
            // Planes stay local/off by default; `on` only enables envelope I/O.
            cfg.settlement.mode = SettlementMode::Emit;
        }
        Some(FederationFlag::Off) | None => {}
    }
    cfg
}

pub fn run_run(
    root: &std::path::Path,
    task: Option<String>,
    federation: Option<FederationFlag>,
) -> Result<ExitCode> {
    use swe_seed_core::federation::{dispatch, emit_work_requested, resolve_from_root, Dispatch};

    let cfg = federation_config_from(federation);
    let resolved = resolve_from_root(root);
    let task = task.as_deref().unwrap_or("inner-stack smoke");

    // Build a WorkRequested envelope in memory (never dispatched when off).
    let envelope = emit_work_requested(
        &resolved.hash,
        "run-smoke",
        "swe-seed",
        "run",
        task,
        "low",
        None,
    );
    let dispatched = dispatch(&cfg, &envelope, None);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "task": task,
            "root": root.display().to_string(),
            "federation": {
                "enabled": cfg.enabled,
                "emits": cfg.emits(),
                "consumes": cfg.consumes(),
                "standalone": cfg.is_standalone(),
            },
            "domain_model_hash": resolved.hash,
            "hash_source": format!("{:?}", resolved.source),
            "hash_warned": resolved.warned,
            "dispatched": match dispatched { Dispatch::Written(_) => "written", Dispatch::Suppressed => "suppressed" },
        }))?
    );
    Ok(ExitCode::SUCCESS)
}

pub fn run_federation(root: &std::path::Path, action: FederationAction) -> Result<ExitCode> {
    use swe_seed_core::federation::{fallback_hash, resolve_from_root};
    let FederationAction::Status = action;
    let resolved = resolve_from_root(root);
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "config_root": root.join(".swe-seed").display().to_string(),
            "domain_model_hash": resolved.hash,
            "fallback_hash": fallback_hash(),
            "hash_source": format!("{:?}", resolved.source),
            "hash_warned": resolved.warned,
            "namespace": swe_seed_core::federation::NAMESPACE,
        }))?
    );
    Ok(ExitCode::SUCCESS)
}
