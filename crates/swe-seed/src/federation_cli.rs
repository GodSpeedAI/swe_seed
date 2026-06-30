use std::process::ExitCode;

use anyhow::Result;
use clap::{Subcommand, ValueEnum};

#[derive(Subcommand)]
pub enum FederationAction {
    /// Show resolved federation flags + domain_model_hash
    Status,
    /// Generate an Ed25519 keypair (pub committed; private key to gitignored .swe-seed/)
    Keygen { key_id: String },
    /// Sign a trace's chain root → a signed ProofCompleted envelope (stdout)
    Sign {
        trace: String,
        /// The key id whose private key signs (under .swe-seed/federation/keys/)
        #[arg(long)]
        key: String,
    },
    /// Verify a signed envelope file against a public key (by key id or file)
    Verify {
        /// Path to a signed envelope JSON file
        file: String,
        /// Key id (pubkey read from .agent-harness/federation/keys/<id>.pub)
        #[arg(long)]
        key: Option<String>,
        /// ...or a direct path to a base64 raw public key
        #[arg(long)]
        pubkey: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FederationFlag {
    Off,
    On,
}

fn federation_config_from(
    flag: Option<FederationFlag>,
) -> swe_seed_core::federation::FederationConfig {
    use swe_seed_core::federation::{FederationConfig, SettlementMode};
    let mut cfg = FederationConfig::default();
    match flag {
        Some(FederationFlag::On) => {
            cfg.enabled = true;
            cfg.emit_envelope = true;
            cfg.consume_envelope = true;
            // Planes stay local/off by default; `on` only enables envelope I/O.
            cfg.settlement.mode = SettlementMode::Emit;
        }
        Some(FederationFlag::Off) | None => {}
    }
    cfg
}

pub fn run_run(
    root: &std::path::Path,
    task: Option<String>,
    federation: Option<FederationFlag>,
) -> Result<ExitCode> {
    use swe_seed_core::federation::{dispatch, emit_work_requested, resolve_from_root, Dispatch};

    let cfg = federation_config_from(federation);
    let resolved = resolve_from_root(root);
    let task = task.as_deref().unwrap_or("inner-stack smoke");

    // Build a WorkRequested envelope in memory (never dispatched when off).
    let envelope = emit_work_requested(
        &resolved.hash,
        "run-smoke",
        "swe-seed",
        "run",
        task,
        "low",
        None,
    );
    let dispatched = dispatch(&cfg, &envelope, None);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "task": task,
            "root": root.display().to_string(),
            "federation": {
                "enabled": cfg.enabled,
                "emits": cfg.emits(),
                "consumes": cfg.consumes(),
                "standalone": cfg.is_standalone(),
            },
            "domain_model_hash": resolved.hash,
            "hash_source": format!("{:?}", resolved.source),
            "hash_warned": resolved.warned,
            "dispatched": match dispatched { Dispatch::Written(_) => "written", Dispatch::Suppressed => "suppressed" },
        }))?
    );
    Ok(ExitCode::SUCCESS)
}

pub fn run_federation(root: &std::path::Path, action: FederationAction) -> Result<ExitCode> {
    use swe_seed_core::federation::{fallback_hash, resolve_from_root};
    match action {
        FederationAction::Status => {
            let resolved = resolve_from_root(root);
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "config_root": root.join(".swe-seed").display().to_string(),
                    "domain_model_hash": resolved.hash,
                    "fallback_hash": fallback_hash(),
                    "hash_source": format!("{:?}", resolved.source),
                    "hash_warned": resolved.warned,
                    "namespace": swe_seed_core::federation::NAMESPACE,
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
        FederationAction::Keygen { key_id } => {
            use swe_seed_core::federation::{generate_signing_key, public_key_b64, write_keypair};
            let secret = generate_signing_key();
            let (pub_path, priv_path) = write_keypair(root, &key_id, &secret)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "key_id": key_id,
                    "public_key_b64": public_key_b64(&secret.verifying_key()),
                    "public_key_path": pub_path.display().to_string(),
                    "private_key_path": priv_path.display().to_string(),
                    "private_key_note": "PRIVATE key written to gitignored .swe-seed/. Encrypt with SOPS/age before any committed use; the public key is safe to commit.",
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
        FederationAction::Sign { trace, key } => {
            use swe_seed_core::federation::{
                load_signing_key, private_key_path, resolve_from_root, sign_envelope,
            };
            use swe_seed_core::trace_ledger;
            let key_path = private_key_path(root, &key)?;
            let secret = match load_signing_key(&key_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("federation sign: {e}");
                    return Ok(ExitCode::from(1));
                }
            };
            let db = trace_ledger::default_db_path(root);
            let conn = trace_ledger::open(&db)?;
            if let Err(reason) = trace_ledger::verify_chain(&conn, &trace) {
                eprintln!("federation sign: trace '{trace}' chain verification failed: {reason}");
                return Ok(ExitCode::from(1));
            }
            let chain_root = match trace_ledger::chain_root(&conn, &trace) {
                Some(h) => h,
                None => {
                    eprintln!(
                        "federation sign: trace '{trace}' has no chain entries (route it first)"
                    );
                    return Ok(ExitCode::from(1));
                }
            };
            let trace_path = swe_seed_core::trace::lifecycle::resolve_trace_path(root, &trace);
            let trace_record = match swe_seed_core::trace::TraceRecord::load(&trace_path) {
                Ok(record) => record,
                Err(e) => {
                    eprintln!("federation sign: cannot load trace proof record: {e:#}");
                    return Ok(ExitCode::from(1));
                }
            };
            let proof = match verified_trace_proof(&trace_record) {
                Some(proof) => proof,
                None => {
                    eprintln!(
                        "federation sign: trace '{}' has no verified passing proof evidence",
                        trace_record.trace_id
                    );
                    return Ok(ExitCode::from(1));
                }
            };
            let resolved = resolve_from_root(root);
            // Build the envelope, attach the chain root, then sign + attach sig.
            let mut envelope = swe_seed_core::federation::emit_proof_completed(
                &resolved.hash,
                &proof.command_id,
                &trace_record.trace_id,
                "pass",
                Some(0),
                Some(&proof.summary),
                swe_seed_core::federation::PROOF_TYPE_LIVE,
                &swe_seed_core::util::utc_now(),
            );
            envelope
                .with_trace_chain_root(&chain_root)
                .map_err(anyhow::Error::msg)?;
            let sig = sign_envelope(&envelope, &secret, &key);
            if let Some(obj) = envelope.payload.as_object_mut() {
                obj.insert(
                    "signature".into(),
                    serde_json::to_value(&sig).unwrap_or(serde_json::Value::Null),
                );
            }
            println!("{}", serde_json::to_string_pretty(&envelope)?);
            Ok(ExitCode::SUCCESS)
        }
        FederationAction::Verify { file, key, pubkey } => {
            use swe_seed_core::federation::{
                public_key_from_b64, public_key_path, verify_envelope, Envelope,
            };
            let path = root.join(&file);
            let envelope: Envelope = match serde_json::from_slice(&std::fs::read(&path)?) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("federation verify: parse {}: {e}", path.display());
                    return Ok(ExitCode::from(1));
                }
            };
            let sig = match envelope
                .payload
                .get("signature")
                .and_then(|s| s.get("value"))
                .and_then(|v| v.as_str())
            {
                Some(s) => s.to_string(),
                None => {
                    eprintln!("federation verify: envelope has no payload.signature.value");
                    return Ok(ExitCode::from(1));
                }
            };
            let pub_b64 = if let Some(k) = key {
                std::fs::read_to_string(public_key_path(root, &k)?)?
                    .trim()
                    .to_string()
            } else if let Some(p) = pubkey {
                std::fs::read_to_string(root.join(&p))?.trim().to_string()
            } else {
                eprintln!("federation verify: provide --key <id> or --pubkey <path>");
                return Ok(ExitCode::from(1));
            };
            let pk = match public_key_from_b64(&pub_b64) {
                Ok(pk) => pk,
                Err(e) => {
                    eprintln!("federation verify: bad public key: {e}");
                    return Ok(ExitCode::from(1));
                }
            };
            match verify_envelope(&envelope, &sig, &pk) {
                Ok(()) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "verified": true,
                            "event_type": envelope.event_type,
                            "namespace": envelope.namespace,
                        }))?
                    );
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    eprintln!("federation verify: INVALID: {e}");
                    Ok(ExitCode::from(1))
                }
            }
        }
    }
}

struct VerifiedTraceProof {
    command_id: String,
    summary: String,
}

fn verified_trace_proof(record: &swe_seed_core::trace::TraceRecord) -> Option<VerifiedTraceProof> {
    if !record.unresolved_risks.is_empty() {
        return None;
    }
    let passing = record.verification.iter().rev().find(|entry| {
        entry
            .get("result")
            .and_then(|v| v.as_str())
            .map(is_passing_result)
            .unwrap_or(false)
    })?;
    let command = passing
        .get("command")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("trace-verification");
    let result = passing
        .get("result")
        .and_then(|v| v.as_str())
        .unwrap_or("pass");
    Some(VerifiedTraceProof {
        command_id: command.to_string(),
        summary: format!(
            "trace verification evidence {}/{}: {command}: {result}",
            record.verification.len(),
            record.trace_id
        ),
    })
}

fn is_passing_result(result: &str) -> bool {
    let result = result.to_lowercase();
    ["exit 0", "passed", "success", "all checks passed"]
        .iter()
        .any(|marker| result.contains(marker))
}
