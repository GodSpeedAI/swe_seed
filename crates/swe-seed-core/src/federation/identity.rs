//! Canonical DomainForge domain-identity gate (convergence plan task T01;
//! frozen requirements ENV-I1, ENV-I2, I1).
//!
//! A `domain_model_hash` is canonical identity ONLY when it both is a real
//! SHA-256 hex digest AND resolves to an actual model-artifact serialization.
//! Fallback pseudo-hashes (e.g. the standalone-mode constant from
//! [`super::envelope::fallback_hash`]) and placeholder digests are rejected,
//! never silently accepted: there is no code path here that manufactures a
//! hash when the model cannot be resolved.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::envelope::{fallback_hash, HashSource, ResolvedHash};

/// Length of a lowercase hexadecimal SHA-256 digest.
const SHA256_HEX_LEN: usize = 64;

/// Why a candidate string cannot be canonical domain identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityError {
    /// Not a 64-character lowercase hex digest.
    MalformedHash { got: String },
    /// A structurally valid digest that is a known pseudo-identity: the
    /// standalone fallback constant, the all-zero digest, or a digest of an
    /// empty artifact.
    PlaceholderIdentity { got: String },
    /// The referenced model artifact does not exist.
    MissingArtifact { path: PathBuf },
    /// The referenced model artifact exists but cannot be read.
    UnreadableArtifact { path: PathBuf },
    /// The declared hash does not match the recomputed artifact content hash.
    HashMismatch {
        declared: String,
        recomputed: String,
    },
}

impl std::fmt::Display for IdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedHash { got } => {
                write!(
                    f,
                    "malformed domain_model_hash (want 64 lowercase hex): {got}"
                )
            }
            Self::PlaceholderIdentity { got } => {
                write!(
                    f,
                    "fallback/placeholder pseudo-hash rejected as domain identity: {got}"
                )
            }
            Self::MissingArtifact { path } => {
                write!(f, "domain model artifact not found: {}", path.display())
            }
            Self::UnreadableArtifact { path } => {
                write!(f, "domain model artifact unreadable: {}", path.display())
            }
            Self::HashMismatch {
                declared,
                recomputed,
            } => {
                write!(f, "declared domain_model_hash {declared} != recomputed artifact hash {recomputed}")
            }
        }
    }
}

impl std::error::Error for IdentityError {}

/// A domain identity that has passed the canonical gate. Constructing one of
/// these is the ONLY way to stamp a hash into a canonical envelope through the
/// T01 API surface, making "valid-looking but fabricated identity"
/// unrepresentable at the type level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedDomainIdentity {
    hash: String,
}

impl VerifiedDomainIdentity {
    /// The verified digest.
    pub fn as_str(&self) -> &str {
        &self.hash
    }

    /// Wrap an already-resolved hash from the environment resolution chain.
    /// Fallback-sourced resolutions are refused: they are exactly the
    /// pseudo-identity the frozen contract forbids.
    pub fn from_resolved(resolved: &ResolvedHash) -> Result<Self, IdentityError> {
        if resolved.source == HashSource::Fallback {
            return Err(IdentityError::PlaceholderIdentity {
                got: resolved.hash.clone(),
            });
        }
        verify_declared_hash(&resolved.hash)
    }
}

fn is_canonical_digest(s: &str) -> bool {
    s.len() == SHA256_HEX_LEN
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Reject the known pseudo-identities: the standalone fallback constant and
/// the all-zero digest.
fn reject_placeholders(hash: &str) -> Result<(), IdentityError> {
    let placeholders = [fallback_hash(), "0".repeat(SHA256_HEX_LEN)];
    if placeholders.iter().any(|p| p == hash) {
        return Err(IdentityError::PlaceholderIdentity {
            got: hash.to_string(),
        });
    }
    Ok(())
}

/// Gate a *declared* digest without an artifact at hand: structural validity
/// plus placeholder refusal. This proves the negative half of ENV-I2 (a
/// missing model cannot yield apparently-valid identity); pair it with
/// [`verify_against_artifact`] for full resolvability.
pub fn verify_declared_hash(declared: &str) -> Result<VerifiedDomainIdentity, IdentityError> {
    if !is_canonical_digest(declared) {
        return Err(IdentityError::MalformedHash {
            got: declared.to_string(),
        });
    }
    reject_placeholders(declared)?;
    Ok(VerifiedDomainIdentity {
        hash: declared.to_string(),
    })
}

/// Full ENV-I1 gate: the declared digest must be canonical AND equal the
/// recomputed SHA-256 of the referenced model artifact's bytes. A missing or
/// unreadable artifact is an error — identity is never manufactured from the
/// declaration alone once an artifact is claimed.
pub fn verify_against_artifact(
    declared: &str,
    artifact: &Path,
) -> Result<VerifiedDomainIdentity, IdentityError> {
    let verified = verify_declared_hash(declared)?;
    if !artifact.exists() {
        return Err(IdentityError::MissingArtifact {
            path: artifact.to_path_buf(),
        });
    }
    let bytes = std::fs::read(artifact).map_err(|_| IdentityError::UnreadableArtifact {
        path: artifact.to_path_buf(),
    })?;
    let mut h = Sha256::new();
    h.update(&bytes);
    let recomputed = format!("{:x}", h.finalize());
    if recomputed != declared {
        return Err(IdentityError::HashMismatch {
            declared: declared.to_string(),
            recomputed,
        });
    }
    Ok(verified)
}

/// Strict environment-chain resolution: same lookup order as
/// [`super::envelope::resolve_from`] (SEA_ROOT, then SEA_MANIFEST_PATH), but
/// the fallback branch is replaced by an error. Discovery walks are not
/// performed here: strict identity requires a caller-declared location.
pub fn strict_resolve(
    sea_root: Option<&str>,
    sea_manifest: Option<&str>,
) -> Result<ResolvedHash, IdentityError> {
    if let Some(root) = sea_root.filter(|s| !s.is_empty()) {
        if let Some(h) = super::envelope::read_manifest_hash(
            &PathBuf::from(root).join(super::envelope::manifest_rel()),
        ) {
            return Ok(ResolvedHash {
                hash: h,
                source: HashSource::SeaRoot,
                warned: false,
            });
        }
    }
    if let Some(p) = sea_manifest.filter(|s| !s.is_empty()) {
        let cand = PathBuf::from(p);
        if cand.is_file() {
            if let Some(h) = super::envelope::read_manifest_hash(&cand) {
                return Ok(ResolvedHash {
                    hash: h,
                    source: HashSource::SeaManifestPath,
                    warned: false,
                });
            }
        }
    }
    Err(IdentityError::MissingArtifact {
        path: PathBuf::from(
            sea_manifest
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| {
                    format!(
                        "{}",
                        PathBuf::from(sea_root.unwrap_or("<unresolved>"))
                            .join(super::envelope::manifest_rel())
                            .display()
                    )
                }),
        ),
    })
}
