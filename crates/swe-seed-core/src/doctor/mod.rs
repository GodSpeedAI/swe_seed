//! Doctor + drift detection (spec 0008). Aggregates boundary validation, the
//! core eval run, frozen-integrity, and manifest drift into one report. Exits
//! non-zero iff any check `Fail`.

pub mod check;
pub mod drift;
pub mod report;

pub use report::{run_doctor, DoctorCheck, DoctorReport, DoctorStatus, CORE_EVAL_SPEC};
