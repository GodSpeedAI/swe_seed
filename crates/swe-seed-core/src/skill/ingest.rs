//! Ingest pipeline (spec 0007): `discover → fetch → scan → normalize →
//! project → verify`. A blocking scan prevents projection; activation requires
//! `source_hash` + a terminal, non-blocking scan.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

use super::ir::SkillIR;
use crate::provenance::content_hash;
use crate::security::{
    exceptions::Exceptions, gate::scan_blocks_projection, scan_result::ScanResult,
    skillspector::run_skillspector,
};

/// An ingested skill: its normalized IR, provenance hash, scan result, source path.
#[derive(Debug, Clone)]
pub struct SkillRecord {
    pub ir: SkillIR,
    pub source_hash: String,
    pub scan: ScanResult,
    pub path: PathBuf,
}

pub fn skills_dir(root: &Path) -> PathBuf {
    root.join(".agent-harness").join("skills")
}

/// `discover`: all `skills/**/*.json`, sorted.
pub fn discover(root: &Path) -> Result<Vec<PathBuf>> {
    let dir = skills_dir(root);
    let mut out = Vec::new();
    walk_json(&dir, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_json(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(anyhow::Error::from(e).context(format!("read {}", dir.display()))),
    };
    for entry in entries {
        let p = entry?.path();
        if p.is_dir() {
            walk_json(&p, out)?;
        } else if p.extension().is_some_and(|x| x == "json") {
            out.push(p);
        }
    }
    Ok(())
}

/// `fetch`: read a skill file as raw JSON.
pub fn fetch(path: &Path) -> Result<Value> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?)
}

/// `normalize`: coerce raw JSON into a baml-faithful `SkillIR`.
pub fn normalize(raw: &Value) -> Result<SkillIR> {
    Ok(serde_json::from_value(raw.clone())?)
}

/// Full ingest of one skill: scan (SkillSpector) runs *before* normalization,
/// then read → normalize → attach provenance `source_hash`.
pub fn ingest_one(path: &Path) -> Result<SkillRecord> {
    // Scan immediately after the file is confirmed readable, before normalize.
    let readable = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let scan = run_skillspector(path);
    ingest_with_scan_bytes(path, readable, scan)
}

/// Ingest with an externally-supplied scan result (dependency injection for
/// deterministic tests). The scan is provided so it is always computed before
/// any normalization/projection, and tests don't depend on the host PATH.
pub fn ingest_with_scan(path: &Path, scan: ScanResult) -> Result<SkillRecord> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    ingest_with_scan_bytes(path, bytes, scan)
}

fn ingest_with_scan_bytes(path: &Path, bytes: Vec<u8>, scan: ScanResult) -> Result<SkillRecord> {
    let raw: Value = serde_json::from_slice(&bytes)?;
    let ir = normalize(&raw)?;
    Ok(SkillRecord {
        source_hash: content_hash(&bytes),
        scan,
        ir,
        path: path.to_path_buf(),
    })
}

/// Ordered pipeline over all discovered skills, using the real scanner.
pub fn pipeline(root: &Path, exceptions: &Exceptions) -> Result<(Vec<SkillRecord>, Vec<String>)> {
    pipeline_scanning(root, exceptions, run_skillspector)
}

/// Pipeline with an injected scanner (`scanner(&path) -> ScanResult`) so tests
/// are deterministic regardless of whether SkillSpector is installed.
pub fn pipeline_scanning<S>(
    root: &Path,
    exceptions: &Exceptions,
    scanner: S,
) -> Result<(Vec<SkillRecord>, Vec<String>)>
where
    S: Fn(&Path) -> ScanResult,
{
    let mut records = Vec::new();
    let mut blocked = Vec::new();
    for path in discover(root)? {
        let scan = scanner(&path);
        let rec = ingest_with_scan(&path, scan)?;
        if scan_blocks_projection(&rec.scan, exceptions) {
            blocked.push(rec.ir.id.clone());
        }
        records.push(rec);
    }
    Ok((records, blocked))
}
