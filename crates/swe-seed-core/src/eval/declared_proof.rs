//! The proof an eval spec's `command_check` declares, exported for an independent verifier.
//!
//! A consumer (Cognate-Workbench, readiness P05) pins the spec's sha256 itself and asks SWE_SEED for
//! the proof, so SWE_SEED stays the only parser of its own format and the digest is not trusted on
//! first use. Nothing is executed and nothing is written.

use std::fmt;
use std::path::Path;

use serde::Serialize;

use super::spec::parse_eval_spec;
use crate::provenance::content_hash;

pub const DECLARED_PROOF_SCHEMA: &str = "swe_seed.eval_proof.v1";

/// `command_check` runs `sh -c <rule>` in the repository root. `/bin/sh` is POSIX's `sh`, made absolute
/// because an independent verifier may run the proof with a cleared environment (no `PATH`).
pub const PROOF_SHELL: &str = "/bin/sh";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeclaredProof {
    pub schema: &'static str,
    pub spec_id: String,
    pub spec_version: String,
    /// `sha256:<hex>` of the exact bytes that were parsed.
    pub spec_sha256: String,
    pub check_id: String,
    pub required: bool,
    pub argv: Vec<String>,
    /// Relative to the repository root the spec is evaluated in (`command_check` runs there).
    pub cwd: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclaredProofError {
    InvalidPin(String),
    Read(String),
    DigestMismatch {
        expected: String,
        actual: String,
    },
    Invalid(String),
    UnknownCheck(String),
    NotCommandCheck {
        check_id: String,
        check_type: String,
    },
    EmptyRule(String),
}

impl DeclaredProofError {
    /// Stable machine code for consumers.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidPin(_) => "invalid_pin",
            Self::Read(_) => "spec_unreadable",
            Self::DigestMismatch { .. } => "spec_digest_mismatch",
            Self::Invalid(_) => "spec_invalid",
            Self::UnknownCheck(_) => "unknown_check",
            Self::NotCommandCheck { .. } => "not_command_check",
            Self::EmptyRule(_) => "empty_rule",
        }
    }
}

impl fmt::Display for DeclaredProofError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPin(pin) => write!(
                f,
                "pin must be 64 hex digits, optionally prefixed sha256: (got {pin:?})"
            ),
            Self::Read(e) => write!(f, "cannot read spec: {e}"),
            Self::DigestMismatch { expected, actual } => {
                write!(f, "spec digest {actual} does not equal the pin {expected}")
            }
            Self::Invalid(e) => write!(f, "invalid spec: {e}"),
            Self::UnknownCheck(id) => write!(f, "spec has no check {id:?}"),
            Self::NotCommandCheck {
                check_id,
                check_type,
            } => write!(
                f,
                "check {check_id:?} is a {check_type}, not a command_check"
            ),
            Self::EmptyRule(id) => write!(f, "command_check {id:?} has an empty rule"),
        }
    }
}

impl std::error::Error for DeclaredProofError {}

/// Normalise a pin to `sha256:<lowercase hex>`.
fn normalise_pin(pin: &str) -> Result<String, DeclaredProofError> {
    let hex = pin.strip_prefix("sha256:").unwrap_or(pin);
    if hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(DeclaredProofError::InvalidPin(pin.to_string()));
    }
    Ok(format!("sha256:{}", hex.to_ascii_lowercase()))
}

/// The proof `check_id` declares in the spec at `path`, provided the spec's bytes hash to `pin`.
pub fn declared_proof(
    path: &Path,
    pin: &str,
    check_id: &str,
) -> Result<DeclaredProof, DeclaredProofError> {
    let expected = normalise_pin(pin)?;
    let bytes = std::fs::read(path)
        .map_err(|e| DeclaredProofError::Read(format!("{}: {e}", path.display())))?;
    let actual = content_hash(&bytes);
    if actual != expected {
        return Err(DeclaredProofError::DigestMismatch { expected, actual });
    }
    let spec =
        parse_eval_spec(&bytes, path).map_err(|e| DeclaredProofError::Invalid(format!("{e:#}")))?;
    let check = spec
        .checks
        .iter()
        .find(|c| c.id == check_id)
        .ok_or_else(|| DeclaredProofError::UnknownCheck(check_id.to_string()))?;
    if check.check_type != "command_check" {
        return Err(DeclaredProofError::NotCommandCheck {
            check_id: check.id.clone(),
            check_type: check.check_type.clone(),
        });
    }
    // Exactly what `command_check` runs (see `evaluate_check`).
    let rule = check.rule.trim();
    if rule.is_empty() {
        return Err(DeclaredProofError::EmptyRule(check.id.clone()));
    }
    Ok(DeclaredProof {
        schema: DECLARED_PROOF_SCHEMA,
        spec_id: spec.id.clone(),
        spec_version: spec.version.clone(),
        spec_sha256: actual,
        check_id: check.id.clone(),
        required: check.required,
        argv: vec![PROOF_SHELL.to_string(), "-c".to_string(), rule.to_string()],
        cwd: ".".to_string(),
    })
}
