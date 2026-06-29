use std::fmt;
use std::str::FromStr;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::hooks::CanonicalHookEvent;

use super::capabilities::HostCapabilities;
use super::projection::ProjectionPlan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HostId {
    #[serde(rename = "claude")]
    Claude,
    #[serde(rename = "codex")]
    Codex,
    #[serde(rename = "opencode")]
    OpenCode,
    #[serde(rename = "github-copilot")]
    GithubCopilot,
    #[serde(rename = "antigravity")]
    Antigravity,
    #[serde(rename = "ci")]
    Ci,
}

impl HostId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
            Self::GithubCopilot => "github-copilot",
            Self::Antigravity => "antigravity",
            Self::Ci => "ci",
        }
    }
}

impl fmt::Display for HostId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for HostId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            "opencode" => Ok(Self::OpenCode),
            "github-copilot" => Ok(Self::GithubCopilot),
            "antigravity" => Ok(Self::Antigravity),
            "ci" => Ok(Self::Ci),
            other => bail!("unsupported host: {other}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdapterSupport {
    Full,
    Partial { reason: String },
    Unsupported { reason: String },
}

impl AdapterSupport {
    pub const fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }
}

pub trait HostAdapter {
    fn host_id(&self) -> HostId;
    fn capabilities(&self) -> HostCapabilities;
    fn support_for(&self, event: CanonicalHookEvent) -> AdapterSupport;
    fn project(&self) -> ProjectionPlan;
}

pub fn all_host_ids() -> Vec<HostId> {
    vec![
        HostId::Claude,
        HostId::Codex,
        HostId::OpenCode,
        HostId::GithubCopilot,
        HostId::Antigravity,
        HostId::Ci,
    ]
}
