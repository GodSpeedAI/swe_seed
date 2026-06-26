//! `.baml` contracts-as-data (spec 0019).
//!
//! Parses the canonical `.baml` schema into [`BamlType`] and offers the
//! [`BamlParity`] trait so hand-written Rust structs/enum can assert 1:1 parity
//! with their `.baml` counterpart. No LLM runtime; the files are read-only inputs.

pub mod baml_parse;
pub mod parity;

pub use baml_parse::{parse_baml_dir, parse_baml_src, BamlKind, BamlType};
pub use parity::{BamlParity, BamlShape};
