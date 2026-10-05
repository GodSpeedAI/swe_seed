//! Semantic-world identity gate (CEP-0008 `world_ref`, migration Stage 9).
//!
//! A `world_ref` names exactly one resolved semantic-world revision:
//! `world:<name>@sha256:<64 lowercase hex>`. DomainForge defines it and
//! SEA-Forge recomputes its digest. SWE_SEED checks SYNTAX and EQUALITY only
//! and never claims a digest was verified here; it also never derives a
//! `world_ref` from the legacy `domain_model_hash`. A mutable alias
//! (`world:<name>`) or a label is not an identity and is refused.

use serde_json::Value;

use super::envelope::Envelope;

/// A syntactically valid, pinned `world_ref`. Construction is the only gate:
/// holding one means the string had the right shape, nothing more.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldRef(String);

/// Why a candidate `world_ref` cannot enter the boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldRefError {
    /// Absent from the payload (or not a string).
    Missing,
    /// Present but not `world:<name>@sha256:<64 lowercase hex>`.
    Malformed { got: String },
    /// A downstream artifact names a different world than the originating
    /// work request.
    Mismatch { expected: String, got: String },
}

impl std::fmt::Display for WorldRefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => write!(f, "world_ref is missing"),
            Self::Malformed { got } => write!(
                f,
                "world_ref {got:?} is not world:<name>@sha256:<64 lowercase hex>"
            ),
            Self::Mismatch { expected, got } => {
                write!(f, "world_ref mismatch: expected {expected}, got {got}")
            }
        }
    }
}

impl std::error::Error for WorldRefError {}

fn is_lower_hex64(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// `[a-z][a-z0-9]*(?:[._-][a-z0-9]+)*`, at most 64 characters.
fn is_valid_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    let mut chars = name.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_lowercase()) {
        return false;
    }
    let mut prev_sep = false;
    for c in chars {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            prev_sep = false;
        } else if matches!(c, '.' | '_' | '-') && !prev_sep {
            prev_sep = true;
        } else {
            return false;
        }
    }
    !prev_sep
}

impl WorldRef {
    /// Parse and validate. Syntax only.
    pub fn parse(candidate: &str) -> Result<Self, WorldRefError> {
        let malformed = || WorldRefError::Malformed {
            got: candidate.to_string(),
        };
        let (name_part, digest) = candidate.split_once("@sha256:").ok_or_else(malformed)?;
        let name = name_part.strip_prefix("world:").ok_or_else(malformed)?;
        if is_valid_name(name) && is_lower_hex64(digest) {
            Ok(Self(candidate.to_string()))
        } else {
            Err(malformed())
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Fail unless `other` is exactly this world. A missing or malformed
    /// value is a mismatch, never a pass.
    pub fn require_same(&self, other: Option<&str>) -> Result<(), WorldRefError> {
        match other {
            Some(got) if got == self.0 => Ok(()),
            Some(got) => Err(WorldRefError::Mismatch {
                expected: self.0.clone(),
                got: got.to_string(),
            }),
            None => Err(WorldRefError::Mismatch {
                expected: self.0.clone(),
                got: "<missing>".into(),
            }),
        }
    }
}

impl std::fmt::Display for WorldRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Envelope {
    /// The envelope's raw `world_ref` string (from its payload), if present.
    /// Unvalidated: use [`Envelope::verified_world_ref`] to gate it.
    pub fn world_ref(&self) -> Option<&str> {
        self.payload.get("world_ref").and_then(Value::as_str)
    }

    /// The envelope's `world_ref`, required and syntax-checked.
    pub fn verified_world_ref(&self) -> Result<WorldRef, WorldRefError> {
        match self.payload.get("world_ref") {
            None | Some(Value::Null) => Err(WorldRefError::Missing),
            Some(Value::String(s)) => WorldRef::parse(s),
            Some(other) => Err(WorldRefError::Malformed {
                got: other.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const D: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn vectors_match_the_cep_pattern() {
        for ok in [
            format!("world:a@sha256:{D}"),
            format!("world:acme.core-v2_x@sha256:{D}"),
            format!("world:{}@sha256:{D}", "a".repeat(64)),
        ] {
            assert!(WorldRef::parse(&ok).is_ok(), "rejected {ok:?}");
        }
        for bad in [
            String::new(),
            "world:acme".into(),
            format!("acme@sha256:{D}"),
            format!("world:Acme@sha256:{D}"),
            format!("world:acme@sha256:{}", D.to_uppercase()),
            format!("world:acme@sha256:{D}0"),
            format!("world:{}@sha256:{D}", "a".repeat(65)),
            format!("world:a-@sha256:{D}"),
            format!("world:a..b@sha256:{D}"),
            format!("world:1a@sha256:{D}"),
        ] {
            assert!(WorldRef::parse(&bad).is_err(), "accepted {bad:?}");
        }
    }

    #[test]
    fn missing_or_different_world_is_a_mismatch() {
        let w = WorldRef::parse(&format!("world:a@sha256:{D}")).unwrap();
        assert!(w.require_same(Some(w.as_str())).is_ok());
        assert!(matches!(
            w.require_same(None),
            Err(WorldRefError::Mismatch { .. })
        ));
        assert!(matches!(
            w.require_same(Some(&format!("world:b@sha256:{D}"))),
            Err(WorldRefError::Mismatch { .. })
        ));
    }
}
