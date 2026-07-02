use std::process::ExitCode;

use anyhow::Result;

use crate::host_cli::{self, HostSelection};

pub fn run_doctor_cmd(
    root: &std::path::Path,
    json: bool,
    host: Option<HostSelection>,
) -> Result<ExitCode> {
    use swe_seed_core::adapters::DriftStatus;
    use swe_seed_core::doctor::{run_doctor, DoctorStatus};
    let report = run_doctor(root);
    let mut host_reports = Vec::new();
    let mut host_failed = false;
    if let Some(selection) = host {
        for drift in host_cli::host_drift_reports(root, selection)? {
            host_failed |= drift.status == DriftStatus::Drifted;
            host_reports.push(drift);
        }
    }
    let combined_overall = if report.overall == DoctorStatus::Fail || host_failed {
        DoctorStatus::Fail
    } else {
        report.overall.clone()
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "core": report,
                "hosts": host_reports,
                "overall": combined_overall,
            }))?
        );
    } else {
        for c in &report.checks {
            println!("{:?}\t{}\t{}", c.status, c.name, c.detail);
        }
        for host_report in &host_reports {
            println!(
                "{:?}\thost:{}\t{}",
                host_report.status,
                host_report.host_id,
                if host_report.drifted_files.is_empty() {
                    "clean".to_string()
                } else {
                    host_report.drifted_files.join(",")
                }
            );
        }
        println!("{:?}\toverall", combined_overall);
    }
    Ok(if combined_overall == DoctorStatus::Fail {
        // Plan 0008: exit non-zero iff any check fails. Warn is surfaced in the
        // report but does not fail the run.
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}
