//! Scan-finding exceptions / waivers (spec 0007). A waived finding id does not
//! block projection; approvals are persisted to `.swe-seed/skill-exceptions.json`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct Exceptions {
    waived_findings: HashSet<String>,
}

impl Exceptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn waive(&mut self, finding_id: impl Into<String>) {
        self.waived_findings.insert(finding_id.into());
    }
    pub fn waives(&self, finding_id: &str) -> bool {
        self.waived_findings.contains(finding_id)
    }
    pub fn len(&self) -> usize {
        self.waived_findings.len()
    }
    pub fn is_empty(&self) -> bool {
        self.waived_findings.is_empty()
    }
}

/// Where waived finding ids are persisted.
pub fn store_path(root: &Path) -> PathBuf {
    root.join(".swe-seed").join("skill-exceptions.json")
}

impl Exceptions {
    /// Load waived finding ids from the JSON store; a missing store is empty.
    pub fn load(path: &Path) -> Self {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(_) => return Self::new(),
        };
        let arr: Vec<String> = serde_json::from_slice(&bytes).unwrap_or_default();
        let mut ex = Self::new();
        for id in arr {
            ex.waive(id);
        }
        ex
    }

    /// Persist the waived finding ids as a JSON array.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let mut ids: Vec<&String> = self.waived_findings.iter().collect();
        ids.sort();
        std::fs::write(path, format!("{}\n", serde_json::to_string_pretty(&ids)?))?;
        Ok(())
    }
}
