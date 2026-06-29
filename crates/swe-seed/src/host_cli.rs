use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;
use clap::ValueEnum;
use swe_seed_core::adapters::{
    adapter_for, all_host_ids, detect_host_drift, rollback_host, sync_host, HostDriftReport,
    HostId, SyncOptions,
};

#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum HostSelection {
    Claude,
    Codex,
    Opencode,
    GithubCopilot,
    Antigravity,
    Ci,
    All,
}

pub fn host_ids(selection: HostSelection) -> Vec<HostId> {
    match selection {
        HostSelection::Claude => vec![HostId::Claude],
        HostSelection::Codex => vec![HostId::Codex],
        HostSelection::Opencode => vec![HostId::OpenCode],
        HostSelection::GithubCopilot => vec![HostId::GithubCopilot],
        HostSelection::Antigravity => vec![HostId::Antigravity],
        HostSelection::Ci => vec![HostId::Ci],
        HostSelection::All => all_host_ids(),
    }
}

pub fn host_drift_reports(root: &Path, selection: HostSelection) -> Result<Vec<HostDriftReport>> {
    host_ids(selection)
        .into_iter()
        .map(|host| detect_host_drift(root, host))
        .collect()
}

pub fn run_sync(
    root: &Path,
    selection: HostSelection,
    dry_run: bool,
    prune: bool,
) -> Result<ExitCode> {
    let mut reports = Vec::new();
    for host in host_ids(selection) {
        reports.push(sync_host(root, host, SyncOptions { dry_run, prune })?);
    }
    println!("{}", serde_json::to_string_pretty(&reports)?);
    Ok(ExitCode::SUCCESS)
}

pub fn run_rollback(root: &Path, selection: HostSelection) -> Result<ExitCode> {
    let mut reports = Vec::new();
    for host in host_ids(selection) {
        let files = rollback_host(root, host)?;
        reports.push(serde_json::json!({
            "host_id": host,
            "files": files,
        }));
    }
    println!("{}", serde_json::to_string_pretty(&reports)?);
    Ok(ExitCode::SUCCESS)
}

pub fn run_hosts() -> Result<ExitCode> {
    let rows = all_host_ids()
        .into_iter()
        .map(|host| {
            let adapter = adapter_for(host);
            serde_json::json!({
                "host_id": host,
                "capabilities": adapter.capabilities(),
            })
        })
        .collect::<Vec<_>>();
    println!("{}", serde_json::to_string_pretty(&rows)?);
    Ok(ExitCode::SUCCESS)
}
