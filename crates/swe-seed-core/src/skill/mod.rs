//! Skill ingestion + render (spec 0007). `SkillIR` mirrors `harness.baml`
//! (loaded tolerantly from the richer runtime JSON); the ingest pipeline gates
//! projection on a blocking scan and activation on source_hash + terminal scan.

pub mod ingest;
pub mod ir;
pub mod render;

pub use ingest::{
    discover, fetch, ingest_one, ingest_with_scan, normalize, pipeline, pipeline_scanning,
    SkillRecord,
};
pub use ir::{ArtifactStatus, SkillIR};
pub use render::{render_skill, render_all, RenderTarget};
