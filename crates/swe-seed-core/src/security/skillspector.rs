//! SkillSpector is an *external* scanner (spec 0007). It is never faked:
//!
//! - if the binary is not installed → `Pending` ("not installed");
//! - if it hangs → killed after a timeout → `Pending` ("timed out");
//! - on a non-zero exit → `Error` (blocking);
//! - on a zero exit, the documented contract below is parsed; any unrecognized
//!   output stays `Pending` rather than being promoted to `Clean`.
//!
//! Assumed v0.1 contract (to verify against real SkillSpector post-v0.1):
//! zero-exit stdout is a JSON object `{"status": "clean"|"warning"|"critical",
//! "findings": [{"id","severity","message"}]}`. Only a recognized `"clean"`
//! activates; everything unrecognized stays Pending.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::scan_result::{ScanFinding, ScanResult, ScanStatus};

/// The binary name looked up on `PATH`.
pub const SKILLSPECTOR_BIN: &str = "skillspector";
/// Hard ceiling on a single SkillSpector run; on expiry the child is killed.
pub const SKILLSPECTOR_TIMEOUT_SECS: u64 = 30;

/// True if a `skillspector` executable is resolvable on `PATH`.
pub fn skillspector_available() -> bool {
    which::which(SKILLSPECTOR_BIN).is_ok()
}

/// Run SkillSpector, gated by an explicit availability flag (so tests are
/// deterministic regardless of the host's PATH).
pub fn run_skillspector_with(skill_path: &Path, available: bool) -> ScanResult {
    let id = skill_id(skill_path);
    if !available {
        return ScanResult::pending(&id, "SkillSpector not installed; scan pending");
    }
    let mut cmd = Command::new(SKILLSPECTOR_BIN);
    cmd.arg(skill_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match run_with_timeout(cmd, SKILLSPECTOR_TIMEOUT_SECS) {
        None => ScanResult {
            skill_id: id,
            status: ScanStatus::Pending,
            findings: Vec::new(),
            detail: format!(
                "SkillSpector timed out after {SKILLSPECTOR_TIMEOUT_SECS}s; scan pending"
            ),
        },
        Some(output) => parse_skillspector_output(&id, &output),
    }
}

/// Run SkillSpector using the real host availability.
pub fn run_skillspector(skill_path: &Path) -> ScanResult {
    run_skillspector_with(skill_path, skillspector_available())
}

fn skill_id(skill_path: &Path) -> String {
    skill_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("skill")
        .to_string()
}

/// Run `cmd`, capturing output, killing the child if it exceeds `secs`.
fn run_with_timeout(mut cmd: Command, secs: u64) -> Option<std::process::Output> {
    let mut child = cmd.spawn().ok()?;
    let deadline = Instant::now() + Duration::from_secs(secs);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().ok(),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => {
                let _ = child.kill();
                return None;
            }
        }
    }
}

pub fn parse_skillspector_output(id: &str, output: &std::process::Output) -> ScanResult {
    // Non-zero exit is a blocking scanner error.
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return ScanResult {
            skill_id: id.into(),
            status: ScanStatus::Error,
            findings: vec![ScanFinding {
                id: "skillspector-nonzero".into(),
                severity: "high".into(),
                message: detail.clone(),
            }],
            detail,
        };
    }

    // Trusted zero-exit contract: JSON object on stdout with a known status.
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(stdout.trim()) {
        if let Some(status_str) = v.get("status").and_then(|s| s.as_str()) {
            let findings = v
                .get("findings")
                .and_then(|f| f.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|f| {
                            Some(ScanFinding {
                                id: f.get("id")?.as_str()?.to_string(),
                                severity: f
                                    .get("severity")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("medium")
                                    .to_string(),
                                message: f
                                    .get("message")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let detail = format!("SkillSpector status: {status_str}");
            let status = match status_str {
                "clean" => ScanStatus::Clean,
                "warning" => ScanStatus::Warning,
                "critical" => ScanStatus::Critical,
                _ => {
                    return ScanResult::pending(
                        id,
                        &format!("unrecognized SkillSpector status '{status_str}'; scan pending"),
                    )
                }
            };
            return ScanResult {
                skill_id: id.into(),
                status,
                findings,
                detail,
            };
        }
    }

    // Unrecognized output (including empty stdout) → never a faked pass.
    ScanResult::pending(
        id,
        "SkillSpector produced no trusted terminal signal; scan pending",
    )
}

// Minimal, dependency-free `which` lookup so we don't pull a crate for one call.
mod which {
    use std::path::PathBuf;
    pub fn which(bin: &str) -> Result<PathBuf, ()> {
        let path = std::env::var_os("PATH").ok_or(())?;
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(bin);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        Err(())
    }
}
