//! Manifest drift detection. Records the assembled manifest hash on first
//! doctor run; later runs warn (non-fatal) when it changes.

use std::path::Path;

use crate::provenance::content_hash;
use crate::seed::assemble_default;
use super::report::{DoctorCheck, DoctorStatus};

/// Warn-level: a changed manifest is surfaced but does not fail doctor.
pub fn manifest_drift_check(root: &Path) -> DoctorCheck {
    let manifest = assemble_default();
    let json = match serde_json::to_string(&manifest) {
        Ok(s) => s,
        Err(e) => {
            return DoctorCheck {
                name: "manifest-drift".into(),
                status: DoctorStatus::Fail,
                detail: format!("cannot serialize manifest: {e}"),
            };
        }
    };
    let current = content_hash(json.as_bytes());
    let sidecar = root.join(".swe-seed").join("manifest.sha256");

    if sidecar.is_file() {
        match std::fs::read_to_string(&sidecar) {
            Ok(prev) if prev.trim() == current => DoctorCheck {
                name: "manifest-drift".into(),
                status: DoctorStatus::Pass,
                detail: "manifest unchanged since last doctor run".into(),
            },
            Ok(_) => {
                // Update the baseline to the current hash so the next run
                // compares against the latest state (the "since last doctor
                // run" message stays accurate), and still surface the change.
                let _ = std::fs::write(&sidecar, &current);
                DoctorCheck {
                    name: "manifest-drift".into(),
                    status: DoctorStatus::Warn,
                    detail: "manifest changed since last doctor run (baseline updated)".into(),
                }
            }
            Err(e) => DoctorCheck {
                name: "manifest-drift".into(),
                status: DoctorStatus::Warn,
                detail: format!("cannot read drift baseline: {e}"),
            },
        }
    } else if let Err(e) = std::fs::create_dir_all(sidecar.parent().unwrap_or(Path::new(".")))
        .and_then(|_| std::fs::write(&sidecar, &current))
    {
        DoctorCheck {
            name: "manifest-drift".into(),
            status: DoctorStatus::Warn,
            detail: format!("cannot record drift baseline: {e}"),
        }
    } else {
        DoctorCheck {
            name: "manifest-drift".into(),
            status: DoctorStatus::Pass,
            detail: "manifest drift baseline recorded".into(),
        }
    }
}
