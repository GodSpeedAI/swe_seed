//! Learning + adaptation loop (spec 0016). The self-improvement loop:
//! `finished trace → ReflectionTemplate → LearningRecord → (LearningCandidate)
//!    → AdaptationDecision (gated by 4 eval results)
//!    → SkillProposal | RegressionCase | HarnessADR | NoChange`.
//!
//! Depends on `eval` (0013) and `trace` (0014). No proposal promotes without
//! proof + rollback; regression cases make a lesson enforceable.

pub mod adaptation;
pub mod candidate;
pub mod proposal;
pub mod record;
pub mod reflection;
pub mod regression;

pub use adaptation::{build_adaptation_decision, promotion_gate, AdaptationDecision};
pub use candidate::{can_promote_candidate, LearningCandidate, PromotionOutcome};
pub use proposal::{validate_proposal, SkillProposal};
pub use record::{validate_learning_record, LearningDisposition, LearningRecord};
pub use reflection::{default_template, reflect, validate_reflection, ReflectionTemplate};
pub use regression::{eval_check_for_regression, validate_regression, RegressionCase};

// ValidationRequirement is the canonical harness type (contracts::harness); the
// re-export keeps `super::ValidationRequirement` resolving in submodules.
pub use crate::contracts::harness::ValidationRequirement;
