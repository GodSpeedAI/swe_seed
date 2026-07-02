use serde::{Deserialize, Serialize};

use super::host::{AdapterSupport, HostId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectedFile {
    pub rel_path: String,
    pub content: String,
    pub owned_whole_file: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionPlan {
    pub host_id: HostId,
    pub files: Vec<ProjectedFile>,
    pub support: Vec<EventSupport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSupport {
    pub event: String,
    pub support: AdapterSupport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncReport {
    pub host_id: HostId,
    pub dry_run: bool,
    pub files: Vec<String>,
    pub partial_support: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncOptions {
    pub dry_run: bool,
    pub prune: bool,
}

impl SyncOptions {
    pub const fn write() -> Self {
        Self {
            dry_run: false,
            prune: false,
        }
    }

    pub const fn dry_run() -> Self {
        Self {
            dry_run: true,
            prune: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftStatus {
    Clean,
    Drifted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostDriftReport {
    pub host_id: HostId,
    pub status: DriftStatus,
    pub drifted_files: Vec<String>,
}
