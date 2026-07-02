//! Trace + durable-decision records (spec 0014). Lifecycle mirrors the Python
//! harness so records stay compatible; `TraceSchema` rejects records missing
//! identifier fields.

pub mod lifecycle;
pub mod record;
pub mod schema;

pub use record::{CompletionClaim, TraceRecord};
pub use schema::{missing_identifiers, trace_schema, TraceSchema};
