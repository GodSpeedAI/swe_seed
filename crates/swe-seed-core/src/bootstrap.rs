//! Idempotent project bootstrap: installs the versioned harness resource
//! bundle into `<target>/.agent-harness/`.
//!
//! Rules (fail closed, never destructive):
//! - Only paths present in the embedded bundle are ever written.
//! - Missing file → created. Identical file → untouched.
//! - Existing file that differs from the bundle is never overwritten. If it
//!   still matches what the last `init` recorded, the bundle has moved on and
//!   it is reported as `UpdateAvailable`; otherwise it is a `Conflict`
//!   (a user-owned or locally modified file).
//! - The install manifest is written only when it would change.
//! - Writes are atomic (temp file + rename); a symlink in the harness path is refused.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

include!(concat!(env!("OUT_DIR"), "/bundle.rs"));

/// Version of the embedded resource bundle (tracks this crate's version).
pub const BUNDLE_VERSION: &str = env!("CARGO_PKG_VERSION");
const HARNESS_DIR: &str = ".agent-harness";
const MANIFEST_NAME: &str = "install-manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Create,
    Unchanged,
    UpdateAvailable,
    Conflict,
}

impl Action {
    pub fn label(self) -> &'static str {
        match self {
            Action::Create => "create",
            Action::Unchanged => "unchanged",
            Action::UpdateAvailable => "update-available",
            Action::Conflict => "conflict",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
struct Manifest {
    bundle_version: String,
    /// relative path (under `.agent-harness/`) → sha256 hex installed by `init`
    files: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct InitReport {
    pub dry_run: bool,
    pub bundle_version: String,
    pub entries: Vec<(String, Action)>,
}

impl InitReport {
    pub fn count(&self, a: Action) -> usize {
        self.entries.iter().filter(|(_, x)| *x == a).count()
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn refuse_symlink(path: &Path) -> Result<()> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            bail!("refusing to write through symlink: {}", path.display());
        }
    }
    Ok(())
}

fn read_manifest(path: &Path) -> Result<Manifest> {
    if !path.exists() {
        return Ok(Manifest::default());
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {} (fail closed)", path.display()))
}

fn write_atomic(dest: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    refuse_symlink(dest)?;
    let tmp = dest.with_extension("swe-seed-tmp");
    fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, dest).with_context(|| format!("rename into {}", dest.display()))?;
    Ok(())
}

/// Install the bundle into `target`. With `dry_run`, nothing is written.
pub fn init(target: &Path, dry_run: bool) -> Result<InitReport> {
    if !target.is_dir() {
        bail!("target is not a directory: {}", target.display());
    }
    let harness = target.join(HARNESS_DIR);
    refuse_symlink(&harness)?;
    let manifest_path = harness.join(MANIFEST_NAME);
    let mut manifest = read_manifest(&manifest_path)?;

    let mut entries = Vec::with_capacity(BUNDLE.len());
    let mut planned_writes: Vec<(PathBuf, &[u8], String, String)> = Vec::new();

    for (rel, bytes) in BUNDLE {
        let dest = harness.join(rel);
        let want = sha256_hex(bytes);
        let action = if !dest.exists() {
            planned_writes.push((dest.clone(), bytes, rel.to_string(), want.clone()));
            Action::Create
        } else {
            let have = sha256_hex(&fs::read(&dest)?);
            if have == want {
                // Already installed and identical: record ownership if untracked.
                if manifest.files.get(*rel) != Some(&want) {
                    manifest.files.insert(rel.to_string(), want.clone());
                }
                Action::Unchanged
            } else if manifest.files.get(*rel) == Some(&have) {
                Action::UpdateAvailable
            } else {
                Action::Conflict
            }
        };
        if action == Action::Create {
            manifest.files.insert(rel.to_string(), want.clone());
        }
        entries.push((rel.to_string(), action));
    }

    let manifest_changed =
        manifest.bundle_version != BUNDLE_VERSION || !planned_writes.is_empty() || {
            // any tracked entry changed (e.g., newly recorded Unchanged files)
            let before = read_manifest(&manifest_path)?;
            before
                != Manifest {
                    bundle_version: BUNDLE_VERSION.to_string(),
                    files: manifest.files.clone(),
                }
        };

    if !dry_run {
        for (dest, bytes, _rel, _sha) in &planned_writes {
            write_atomic(dest, bytes)?;
        }
        if manifest_changed {
            manifest.bundle_version = BUNDLE_VERSION.to_string();
            fs::create_dir_all(&harness)?;
            let json = serde_json::to_string_pretty(&manifest)?;
            write_atomic(&manifest_path, format!("{json}\n").as_bytes())?;
        }
    }

    Ok(InitReport {
        dry_run,
        bundle_version: BUNDLE_VERSION.to_string(),
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("swe-seed-init-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn fresh_init_creates_bundle_then_is_idempotent() {
        let t = tmp("fresh");
        let r1 = init(&t, false).unwrap();
        assert_eq!(r1.count(Action::Create), BUNDLE.len());
        let r2 = init(&t, false).unwrap();
        assert_eq!(r2.count(Action::Unchanged), BUNDLE.len());
        assert_eq!(r2.count(Action::Create), 0);
        let _ = fs::remove_dir_all(&t);
    }

    #[test]
    fn customized_route_is_preserved_as_conflict() {
        let t = tmp("custom");
        init(&t, false).unwrap();
        let route = t.join(HARNESS_DIR).join("routes/bugfix.json");
        fs::write(&route, b"{\"custom\":true}").unwrap();
        let r = init(&t, false).unwrap();
        assert_eq!(fs::read(&route).unwrap(), b"{\"custom\":true}");
        assert!(r
            .entries
            .iter()
            .any(|(p, a)| p == "routes/bugfix.json" && *a == Action::Conflict));
        let _ = fs::remove_dir_all(&t);
    }

    #[test]
    fn dry_run_writes_nothing() {
        let t = tmp("dry");
        let r = init(&t, true).unwrap();
        assert_eq!(r.count(Action::Create), BUNDLE.len());
        assert!(!t.join(HARNESS_DIR).exists());
        let _ = fs::remove_dir_all(&t);
    }

    #[test]
    fn corrupt_manifest_fails_closed() {
        let t = tmp("corrupt");
        fs::create_dir_all(t.join(HARNESS_DIR)).unwrap();
        fs::write(t.join(HARNESS_DIR).join(MANIFEST_NAME), b"not json").unwrap();
        assert!(init(&t, false).is_err());
        let _ = fs::remove_dir_all(&t);
    }
}
