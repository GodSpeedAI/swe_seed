//! Durable idempotent-envelope admission (convergence plan task T01; frozen
//! requirement ENV-I6: every durable envelope write MUST be idempotent under
//! stable envelope identity).
//!
//! [`IdempotencyLedger`] is a crash-conservative, append-only set of accepted
//! idempotency keys backed by one JSONL file. `admit` is the single write
//! gate: a key is either admitted exactly once (First, appended + fsynced) or
//! reported as Duplicate with no write. Reopening the ledger reloads prior
//! keys from disk, so dedupe survives restart — the property replay/redelivery
//! tests in later edge tasks build on.

use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Outcome of offering an idempotency key for admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    /// The key had not been seen; it is now durably recorded.
    First,
    /// The key was already recorded; nothing was written.
    Duplicate,
}

/// Why an idempotency key cannot be admitted.
#[derive(Debug)]
pub enum AdmitError {
    Io(std::io::Error),
    MalformedKey { got: String },
}

impl std::fmt::Display for AdmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "idempotency ledger io error: {e}"),
            Self::MalformedKey { got } => {
                write!(
                    f,
                    "malformed idempotency key (want non-empty, no whitespace): {got:?}"
                )
            }
        }
    }
}

impl std::error::Error for AdmitError {}

impl From<std::io::Error> for AdmitError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Append-only idempotency-key set persisted as one key per line.
#[derive(Debug)]
pub struct IdempotencyLedger {
    path: PathBuf,
    seen: HashSet<String>,
}

fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.bytes().all(|b| b.is_ascii_hexdigit())
}

impl IdempotencyLedger {
    /// Open (or create) the ledger at `path`, loading previously admitted
    /// keys. A missing file is an empty ledger; a malformed line is a hard
    /// error — a corrupt ledger must fail closed, not silently shrink the
    /// dedupe set.
    pub fn open(path: &Path) -> Result<Self, AdmitError> {
        let mut seen = HashSet::new();
        if path.exists() {
            let f = std::fs::File::open(path)?;
            for line in BufReader::new(f).lines() {
                let line = line?;
                let key = line.trim();
                if !valid_key(key) {
                    return Err(AdmitError::MalformedKey {
                        got: format!("ledger line {key:?}"),
                    });
                }
                seen.insert(key.to_string());
            }
        }
        Ok(Self {
            path: path.to_path_buf(),
            seen,
        })
    }

    /// True when the key was already admitted (no I/O).
    pub fn contains(&self, key: &str) -> bool {
        self.seen.contains(key)
    }

    /// Number of durably recorded keys.
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    /// Whether the ledger records no keys.
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }

    /// Offer `key` for admission. First occurrence appends one line and
    /// fsyncs before returning [`Admission::First`]; any later occurrence
    /// returns [`Admission::Duplicate`] without touching storage.
    pub fn admit(&mut self, key: &str) -> Result<Admission, AdmitError> {
        if !valid_key(key) {
            return Err(AdmitError::MalformedKey {
                got: key.to_string(),
            });
        }
        if self.seen.contains(key) {
            return Ok(Admission::Duplicate);
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        f.write_all(key.as_bytes())?;
        f.write_all(b"\n")?;
        f.sync_all()?;
        self.seen.insert(key.to_string());
        Ok(Admission::First)
    }
}
