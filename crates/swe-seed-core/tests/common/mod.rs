//! Shared test helpers. Included per test binary with `mod common;`.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// A path under the system temp dir that is unique per process and per call
/// (`<prefix>-<pid>-<seq><suffix>`). Nothing is created on disk. A clock-only
/// name can collide on coarse timers (macOS, some VMs), letting parallel tests
/// share and delete each other's files.
pub fn unique_temp_path(prefix: &str, suffix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}-{nanos}{suffix}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ))
}

/// Like [`unique_temp_path`] with no suffix.
pub fn unique_temp_dir(prefix: &str) -> PathBuf {
    unique_temp_path(prefix, "")
}
