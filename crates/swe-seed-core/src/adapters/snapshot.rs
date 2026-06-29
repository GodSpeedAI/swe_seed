use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::host::HostId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionSnapshot {
    pub host_id: HostId,
    pub entries: Vec<SnapshotEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotEntry {
    pub rel_path: String,
    pub existed: bool,
    pub bytes: Vec<u8>,
}

pub fn snapshot_path(root: &Path, host: HostId) -> PathBuf {
    root.join(".swe-seed")
        .join("host-snapshots")
        .join(format!("{}.json", host.as_str()))
}

pub fn read_snapshot(root: &Path, host: HostId) -> Result<Option<ProjectionSnapshot>> {
    let path = snapshot_path(root, host);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    let snapshot =
        serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(snapshot))
}

pub fn write_snapshot(root: &Path, snapshot: &ProjectionSnapshot) -> Result<()> {
    let path = snapshot_path(root, snapshot.host_id);
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    std::fs::write(
        &path,
        format!("{}\n", serde_json::to_string_pretty(snapshot)?),
    )
    .with_context(|| format!("write {}", path.display()))
}

pub fn delete_snapshot(root: &Path, host: HostId) -> Result<()> {
    let path = snapshot_path(root, host);
    if path.exists() {
        std::fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
    }
    Ok(())
}
