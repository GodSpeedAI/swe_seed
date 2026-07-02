//! Context plane (spec 0015). `ContextBudget` bounds what a route may load;
//! `build_pack` enforces it (excludes excluded files, caps raw output, warns
//! stale context) and produces a `ContextPack`.

pub mod budget;
pub mod pack;
pub mod policy;

pub use budget::ContextBudget;
pub use pack::{build_pack, cap_lines, parse_line_cap, ContextPack, DEFAULT_STALE_THRESHOLD};
pub use policy::{build_budget_from_policy, context_plan};
