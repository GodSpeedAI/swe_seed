//! Security gate for skill ingestion (spec 0007): scan results, the external
//! SkillSpector gate (never a faked pass), the blocking projection gate, and
//! the activation gate (source_hash + terminal scan).

pub mod exceptions;
pub mod gate;
pub mod scan_result;
pub mod skillspector;

pub use exceptions::{store_path as exceptions_store_path, Exceptions};
pub use gate::{can_activate, scan_blocks_projection};
pub use scan_result::{ScanFinding, ScanResult, ScanStatus};
pub use skillspector::{run_skillspector, run_skillspector_with, skillspector_available};
