use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result};

pub mod antigravity;
pub mod capabilities;
pub mod ci;
pub mod claude;
pub mod codex;
pub mod github_copilot;
pub mod host;
pub mod marker;
pub mod opencode;
pub mod projection;
pub mod snapshot;

pub use crate::hooks::CanonicalHookEvent;
pub use capabilities::{FailureMode, HostCapabilities};
pub use host::{all_host_ids, AdapterSupport, HostAdapter, HostId};
pub use projection::{DriftStatus, HostDriftReport, ProjectionPlan, SyncOptions, SyncReport};

use antigravity::AntigravityAdapter;
use ci::CiAdapter;
use claude::ClaudeAdapter;
use codex::CodexAdapter;
use github_copilot::GithubCopilotAdapter;
use host::AdapterSupport::{Partial, Unsupported};
use opencode::OpenCodeAdapter;
use projection::EventSupport;
use projection::ProjectedFile;
use snapshot::{ProjectionSnapshot, SnapshotEntry};

pub fn adapter_for(host: HostId) -> Box<dyn HostAdapter> {
    match host {
        HostId::Claude => Box::new(ClaudeAdapter),
        HostId::Codex => Box::new(CodexAdapter),
        HostId::OpenCode => Box::new(OpenCodeAdapter),
        HostId::GithubCopilot => Box::new(GithubCopilotAdapter),
        HostId::Antigravity => Box::new(AntigravityAdapter),
        HostId::Ci => Box::new(CiAdapter),
    }
}

pub fn project_host(host: HostId) -> ProjectionPlan {
    adapter_for(host).project()
}

pub fn sync_host(root: &Path, host: HostId, options: SyncOptions) -> Result<SyncReport> {
    let plan = project_host(host);
    if !options.dry_run {
        ensure_snapshot(root, &plan.files, host)?;
        for file in &plan.files {
            write_projected(root, file)?;
        }
    }
    Ok(SyncReport {
        host_id: host,
        dry_run: options.dry_run,
        files: plan
            .files
            .iter()
            .map(|file| file.rel_path.clone())
            .collect(),
        partial_support: plan
            .support
            .iter()
            .filter_map(|event| match &event.support {
                Partial { reason } | Unsupported { reason } => {
                    Some(format!("{}: {reason}", event.event))
                }
                AdapterSupport::Full => None,
            })
            .collect(),
    })
}

pub fn rollback_host(root: &Path, host: HostId) -> Result<Vec<String>> {
    let Some(snapshot) = snapshot::read_snapshot(root, host)? else {
        return Ok(Vec::new());
    };
    let mut restored = Vec::new();
    for entry in &snapshot.entries {
        let path = root.join(&entry.rel_path);
        if entry.existed {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            std::fs::write(&path, &entry.bytes)
                .with_context(|| format!("write {}", path.display()))?;
        } else if path.exists() {
            std::fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
        }
        restored.push(entry.rel_path.clone());
    }
    snapshot::delete_snapshot(root, host)?;
    Ok(restored)
}

pub fn detect_host_drift(root: &Path, host: HostId) -> Result<HostDriftReport> {
    let plan = project_host(host);
    let snapshot = snapshot::read_snapshot(root, host)?;
    let current_paths = plan
        .files
        .iter()
        .map(|file| file.rel_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut drifted = BTreeSet::new();
    for file in &plan.files {
        let path = root.join(&file.rel_path);
        let expected = expected_bytes(snapshot.as_ref(), file)?;
        match std::fs::read(&path) {
            Ok(actual) if actual == expected => {}
            _ => {
                drifted.insert(file.rel_path.clone());
            }
        }
    }
    if let Some(snapshot) = &snapshot {
        for entry in &snapshot.entries {
            if !current_paths.contains(entry.rel_path.as_str())
                && root.join(&entry.rel_path).exists()
            {
                drifted.insert(entry.rel_path.clone());
            }
        }
    }
    let status = if drifted.is_empty() {
        projection::DriftStatus::Clean
    } else {
        projection::DriftStatus::Drifted
    };
    Ok(HostDriftReport {
        host_id: host,
        status,
        drifted_files: drifted.into_iter().collect(),
    })
}

fn ensure_snapshot(root: &Path, files: &[ProjectedFile], host: HostId) -> Result<()> {
    if snapshot::read_snapshot(root, host)?.is_some() {
        return Ok(());
    }
    let entries = files
        .iter()
        .map(|file| {
            let path = root.join(&file.rel_path);
            match std::fs::read(&path) {
                Ok(bytes) => Ok(SnapshotEntry {
                    rel_path: file.rel_path.clone(),
                    existed: true,
                    bytes,
                }),
                Err(error) if error.kind() == ErrorKind::NotFound => Ok(SnapshotEntry {
                    rel_path: file.rel_path.clone(),
                    existed: false,
                    bytes: Vec::new(),
                }),
                Err(error) => Err(error).with_context(|| format!("read {}", path.display())),
            }
        })
        .collect::<Result<Vec<_>>>()?;
    snapshot::write_snapshot(
        root,
        &ProjectionSnapshot {
            host_id: host,
            entries,
        },
    )
}

fn write_projected(root: &Path, file: &ProjectedFile) -> Result<()> {
    let path = root.join(&file.rel_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let existing = std::fs::read(&path).ok();
    let bytes = projected_bytes(existing.as_deref(), file)?;
    std::fs::write(&path, bytes).with_context(|| format!("write {}", path.display()))
}

fn expected_bytes(snapshot: Option<&ProjectionSnapshot>, file: &ProjectedFile) -> Result<Vec<u8>> {
    let original = snapshot
        .and_then(|snap| {
            snap.entries
                .iter()
                .find(|entry| entry.rel_path == file.rel_path)
        })
        .and_then(|entry| entry.existed.then_some(entry.bytes.as_slice()));
    projected_bytes(original, file)
}

fn projected_bytes(existing: Option<&[u8]>, file: &ProjectedFile) -> Result<Vec<u8>> {
    if file.rel_path.ends_with(".json") {
        return marker::merge_json(existing, &file.content);
    }
    Ok(file.content.as_bytes().to_vec())
}

pub(crate) fn support_entries<F>(
    events: &[CanonicalHookEvent],
    mut support_for: F,
) -> Vec<EventSupport>
where
    F: FnMut(CanonicalHookEvent) -> AdapterSupport,
{
    events
        .iter()
        .filter_map(|event| match support_for(*event) {
            AdapterSupport::Full => Some(EventSupport {
                event: event.as_str().into(),
                support: AdapterSupport::Full,
            }),
            AdapterSupport::Partial { reason } => Some(EventSupport {
                event: event.as_str().into(),
                support: AdapterSupport::Partial { reason },
            }),
            AdapterSupport::Unsupported { .. } => None,
        })
        .collect()
}
