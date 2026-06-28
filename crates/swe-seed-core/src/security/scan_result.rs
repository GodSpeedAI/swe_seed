//! Scan results (spec 0007). `Pending` = not yet verified (the only honest
//! state when SkillSpector is unavailable); `Critical` blocks projection.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ScanStatus {
    /// Not yet verified. Never treated as a pass.
    Pending,
    /// Scanned, no blocking findings.
    Clean,
    /// Scanned with advisory findings (non-blocking).
    Warning,
    /// Blocking finding — projection is prevented.
    Critical,
    /// Scanner failed to produce a usable result.
    Error,
}

impl ScanStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, ScanStatus::Clean | ScanStatus::Warning | ScanStatus::Critical)
    }
    pub fn is_blocking(&self) -> bool {
        matches!(self, ScanStatus::Critical | ScanStatus::Error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFinding {
    pub id: String,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub skill_id: String,
    pub status: ScanStatus,
    pub findings: Vec<ScanFinding>,
    pub detail: String,
}

impl ScanResult {
    pub fn pending(skill_id: &str, detail: &str) -> Self {
        Self {
            skill_id: skill_id.into(),
            status: ScanStatus::Pending,
            findings: Vec::new(),
            detail: detail.into(),
        }
    }
}
