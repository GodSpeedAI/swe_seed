//! Trace record format (port of the Python trace JSON). Field order matches the
//! captured records. `events` and `verification` are heterogeneous, so they are
//! stored as `serde_json::Value` (constructed in the lifecycle module).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::route::RouteResult;
use anyhow::Context;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionClaim {
    pub at: String,
    pub claim: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRecord {
    pub trace_id: String,
    pub created_at: String,
    pub task: String,
    pub route_decision_record: String,
    pub route: RouteResult,
    pub events: Vec<Value>,
    #[serde(default)]
    pub verification: Vec<Value>,
    #[serde(default)]
    pub unresolved_risks: Vec<String>,
    #[serde(default)]
    pub completion_claim: Option<CompletionClaim>,
}

impl TraceRecord {
    pub fn load(path: &std::path::Path) -> anyhow::Result<TraceRecord> {
        let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        Ok(serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?)
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).with_context(|| format!("create {}", p.display()))?;
        }
        if std::fs::symlink_metadata(path)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
        {
            anyhow::bail!(
                "refusing to write trace record through symlink: {}",
                path.display()
            );
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("trace");
        let temporary = path.with_file_name(format!(".{name}.{}.tmp", uuid::Uuid::new_v4()));
        std::fs::write(
            &temporary,
            format!("{}\n", serde_json::to_string_pretty(self)?),
        )
        .with_context(|| format!("write {}", temporary.display()))?;
        std::fs::rename(&temporary, path).with_context(|| format!("replace {}", path.display()))?;
        Ok(())
    }

    pub fn latest_checkpoint(&self) -> Option<&Value> {
        self.events
            .iter()
            .rev()
            .find(|e| e.get("type").and_then(Value::as_str) == Some("trace.checkpoint"))
    }
}
