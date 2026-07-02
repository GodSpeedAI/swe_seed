#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use swe_seed_core::adapters::{
    detect_host_drift, rollback_host, sync_host, HostDriftReport, HostId, SyncOptions, SyncReport,
};

pub fn temp_root(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("create temp root");
    dir
}

pub fn cleanup(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

pub fn read(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).expect("read projected file")
}

pub fn sync(root: &Path, host: HostId) -> SyncReport {
    sync_host(root, host, SyncOptions::write()).expect("sync host")
}

pub fn dry_run(root: &Path, host: HostId) -> SyncReport {
    sync_host(root, host, SyncOptions::dry_run()).expect("dry run host")
}

pub fn rollback(root: &Path, host: HostId) {
    rollback_host(root, host).expect("rollback host");
}

pub fn drift(root: &Path, host: HostId) -> HostDriftReport {
    detect_host_drift(root, host).expect("detect host drift")
}
