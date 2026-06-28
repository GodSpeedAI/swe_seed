//! Scan gate + activation gate (spec 0007). A blocking scan prevents
//! projection; activation requires `source_hash` and a terminal, non-blocking
//! scan status.

use crate::skill::SkillRecord;
use super::exceptions::Exceptions;
use super::scan_result::ScanResult;

/// A blocking scan (Critical/Error, after applying exceptions) prevents
/// projection/render of a skill. Fail closed on the status: a blocking status
/// blocks even when `findings` is empty; an empty findings list never bypasses
/// the gate. When findings are present, the block is lifted only if every
/// finding is explicitly waived.
pub fn scan_blocks_projection(scan: &ScanResult, exceptions: &Exceptions) -> bool {
    if !scan.status.is_blocking() {
        return false;
    }
    if scan.findings.is_empty() {
        return true;
    }
    // Block unless ALL findings are waived.
    !scan.findings.iter().all(|f| exceptions.waives(&f.id))
}

/// A skill may activate only when it carries provenance (`source_hash`) AND its
/// scan reached a terminal, non-blocking status (Clean/Warning). `Pending` and
/// `Critical` never activate.
pub fn can_activate(record: &SkillRecord, exceptions: &Exceptions) -> bool {
    !record.source_hash.is_empty()
        && record.scan.status.is_terminal()
        && !record.scan.status.is_blocking()
        && !scan_blocks_projection(&record.scan, exceptions)
}
