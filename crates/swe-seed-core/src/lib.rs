//! swe-seed-core — the governance library for the SWE_Seed Rust rewrite.
//!
//! Phase 0: `.baml` contracts-as-data + config parsing (spec 0019).
//! Phase 1: SweSeed layer (specs 0003/0009/0018) — capability registry,
//! seed-package assembly, boundary validation, idempotent regeneration,
//! and fail-closed provenance verification.

pub mod config;
pub mod context;
pub mod contracts;
pub mod doctor;
pub mod eval;
pub mod hooks;
pub mod provenance;
pub mod route;
pub mod seed;
pub mod trace;
pub mod util;
