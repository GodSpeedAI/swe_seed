use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum SkillAction {
    /// Add (validate/normalize) a skill by path
    Add { path: String },
    /// Scan a skill with the external SkillSpector gate
    Scan { id: String },
    /// Approve (waive) a scan finding so it no longer blocks
    Approve { finding_id: String },
    /// List discovered skills with scan status
    List,
}

pub fn run_render_skills(root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::security::{exceptions_store_path, Exceptions};
    use swe_seed_core::skill::render_all;
    let exceptions = Exceptions::load(&exceptions_store_path(root));
    let (rendered, blocked) = render_all(root, root, &exceptions)?;
    for b in &blocked {
        eprintln!("blocked (scan): {b}");
    }
    println!(
        "rendered {rendered} target file(s); {} blocked",
        blocked.len()
    );
    Ok(ExitCode::SUCCESS)
}

pub fn run_skill(action: SkillAction, root: &std::path::Path) -> Result<ExitCode> {
    use swe_seed_core::security::{exceptions_store_path, run_skillspector, Exceptions};
    use swe_seed_core::skill::{discover, fetch, ingest_one, normalize, SkillRecord};
    match action {
        SkillAction::Add { path } => {
            let p = root.join(&path);
            let rec = ingest_one(&p)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "id": rec.ir.id,
                    "version": rec.ir.version,
                    "status": format!("{:?}", rec.ir.status),
                    "source_hash": rec.source_hash,
                    "scan_status": format!("{:?}", rec.scan.status),
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
        SkillAction::Scan { id } => {
            // Match by SkillIR.id, not the filename stem.
            let mut found = None;
            for p in discover(root)? {
                let Ok(raw) = fetch(&p) else {
                    continue;
                };
                let Ok(ir) = normalize(&raw) else {
                    continue;
                };
                if ir.id == id {
                    found = Some(p);
                    break;
                }
            }
            let Some(path) = found else {
                eprintln!("skill not found: {id}");
                return Ok(ExitCode::from(1));
            };
            let scan = run_skillspector(&path);
            println!("{}", serde_json::to_string_pretty(&scan)?);
            Ok(ExitCode::SUCCESS)
        }
        SkillAction::Approve { finding_id } => {
            // Persist the waiver so it no longer blocks projection/render.
            let store = exceptions_store_path(root);
            let mut exceptions = Exceptions::load(&store);
            exceptions.waive(&finding_id);
            exceptions.save(&store)?;
            println!(
                "approved (waived) finding '{}' -> {}",
                finding_id,
                store.strip_prefix(root).unwrap_or(&store).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        SkillAction::List => {
            let mut rows = Vec::new();
            for p in discover(root)? {
                let rec: SkillRecord = match ingest_one(&p) {
                    Ok(r) => r,
                    Err(e) => {
                        rows.push(format!("?\t{}\tINVALID: {e}", p.display()));
                        continue;
                    }
                };
                rows.push(format!(
                    "{:?}\t{}\t{}",
                    rec.scan.status, rec.ir.id, rec.source_hash
                ));
            }
            println!("scan\tid\tsource_hash");
            for r in rows {
                println!("{r}");
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}
