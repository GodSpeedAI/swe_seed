//! `AdaptationDecision` + the proof-gated adaptation policy (spec 0016).
//! Computes, from the four eval results (0013), which adaptations are allowed
//! vs blocked. **Adaptation eligibility is proof-gated:** a failing
//! `AdaptationEligibility` eval blocks promotion regardless of other results.

use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};
use crate::eval::{EvalClass, EvalResult, EvalStatus};

/// harness.baml `AdaptationDecision`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdaptationDecision {
    pub run_id: String,
    pub decision_id: String,
    pub product_result: EvalStatus,
    pub process_result: EvalStatus,
    pub learning_result: EvalStatus,
    pub adaptation_result: EvalStatus,
    pub allowed_adaptations: Vec<String>,
    pub blocked_adaptations: Vec<String>,
    pub reason: String,
    pub required_next_actions: Vec<String>,
    pub created_artifacts: Vec<String>,
}

impl BamlParity for AdaptationDecision {
    fn baml_name() -> &'static str {
        "AdaptationDecision"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "run_id",
                "decision_id",
                "product_result",
                "process_result",
                "learning_result",
                "adaptation_result",
                "allowed_adaptations",
                "blocked_adaptations",
                "reason",
                "required_next_actions",
                "created_artifacts",
            ],
            field_types: vec![
                "string",
                "string",
                "EvalStatus",
                "EvalStatus",
                "EvalStatus",
                "EvalStatus",
                "string[]",
                "string[]",
                "string",
                "string[]",
                "string[]",
            ],
        }
    }
}

/// All adaptation kinds the loop can grant. Promotion (skill or regression) is
/// the high-trust action and is gated hardest.
const ADAPTATIONS: &[&str] = &[
    "skill_promotion",
    "regression_registration",
    "adr_update",
    "lesson_publish",
];

/// Aggregate a set of eval results into one status per EvalClass. A class is
/// `Fail` if any of its checks failed; else `Pass` if any passed; else
/// `Inconclusive`. Missing classes default to `Inconclusive` (fail-closed: an
/// unknown gate never grants trust).
fn per_class(results: &[EvalResult]) -> [EvalStatus; 4] {
    let mut out = [
        EvalStatus::Inconclusive,
        EvalStatus::Inconclusive,
        EvalStatus::Inconclusive,
        EvalStatus::Inconclusive,
    ];
    let idx = |c: EvalClass| match c {
        EvalClass::ProductOutcome => 0,
        EvalClass::ProcessCompliance => 1,
        EvalClass::LearningQuality => 2,
        EvalClass::AdaptationEligibility => 3,
    };
    for r in results {
        for c in r.checks.iter() {
            let i = idx(c.eval_class);
            out[i] = combine(out[i], c.status);
        }
    }
    out
}

fn combine(a: EvalStatus, b: EvalStatus) -> EvalStatus {
    use EvalStatus::*;
    match (a, b) {
        // Any required failure dominates.
        (Fail, _) | (_, Fail) => Fail,
        // Waived is weaker than pass but stronger than inconclusive.
        (Pass, _) | (_, Pass) => Pass,
        (Waived, _) | (_, Waived) => Waived,
        _ => Inconclusive,
    }
}

impl AdaptationDecision {
    /// `true` iff the decision grants `skill_promotion` specifically. Regression
    /// registration is a distinct grant (`regression_registration`), not a
    /// proxy for skill promotion — `promotion_gate` uses this exact check.
    pub fn promotion_allowed(&self) -> bool {
        self.allowed_adaptations
            .iter()
            .any(|a| a == "skill_promotion")
    }

    /// `true` iff the decision grants `regression_registration` (distinct from
    /// skill promotion; a regression may be registered even when a skill is
    /// not being promoted).
    pub fn regression_registration_allowed(&self) -> bool {
        self.allowed_adaptations
            .iter()
            .any(|a| a == "regression_registration")
    }
}

/// Build an `AdaptationDecision` from a run's eval results. The four class
/// statuses are cited on the record. The policy:
/// - A non-`Pass` `AdaptationEligibility` result blocks **every** adaptation
///   (proof gate: promotion never overrides an eligibility failure).
/// - Otherwise each adaptation is allowed when its backing class(es) `Pass`.
pub fn build_adaptation_decision(run_id: &str, results: &[EvalResult]) -> AdaptationDecision {
    let [product, process, learning, adaptation] = per_class(results);

    let mut allowed = Vec::new();
    let mut blocked = Vec::new();
    let mut next_actions = Vec::new();

    // ponytail: the eligibility gate is absolute — short-circuit everything.
    if adaptation != EvalStatus::Pass {
        blocked.extend(ADAPTATIONS.iter().map(|s| s.to_string()));
        next_actions.push(format!(
            "AdaptationEligibility eval is {adaptation:?}; resolve before any promotion"
        ));
        return AdaptationDecision {
            run_id: run_id.into(),
            decision_id: format!("adapt-{run_id}"),
            product_result: product,
            process_result: process,
            learning_result: learning,
            adaptation_result: adaptation,
            allowed_adaptations: allowed,
            blocked_adaptations: blocked,
            reason: format!(
                "adaptation blocked: AdaptationEligibility={adaptation:?} (product={product:?} process={process:?} learning={learning:?})"
            ),
            required_next_actions: next_actions,
            created_artifacts: Vec::new(),
        };
    }

    // Eligibility passed: gate each adaptation on its backing class(es).
    let grant = |cond: bool, name: &str, allowed: &mut Vec<String>, blocked: &mut Vec<String>| {
        if cond {
            allowed.push(name.into());
        } else {
            blocked.push(name.into());
        }
    };

    // Promotion is the highest-trust action: needs product + learning quality.
    grant(
        product == EvalStatus::Pass && learning == EvalStatus::Pass,
        "skill_promotion",
        &mut allowed,
        &mut blocked,
    );
    grant(
        product == EvalStatus::Pass && learning == EvalStatus::Pass,
        "regression_registration",
        &mut allowed,
        &mut blocked,
    );
    grant(process == EvalStatus::Pass, "adr_update", &mut allowed, &mut blocked);
    grant(
        learning == EvalStatus::Pass,
        "lesson_publish",
        &mut allowed,
        &mut blocked,
    );

    if !blocked.is_empty() {
        next_actions.push(format!("blocked adaptations need a passing eval: {blocked:?}"));
    }

    AdaptationDecision {
        run_id: run_id.into(),
        decision_id: format!("adapt-{run_id}"),
        product_result: product,
        process_result: process,
        learning_result: learning,
        adaptation_result: adaptation,
        allowed_adaptations: allowed,
        blocked_adaptations: blocked,
        reason: format!(
            "adaptation eligible (AdaptationEligibility=Pass); product={product:?} process={process:?} learning={learning:?}"
        ),
        required_next_actions: next_actions,
        created_artifacts: Vec::new(),
    }
}

/// Promotion gate tying the decision to a proposal candidate: a proposal may
/// promote only when the decision grants `skill_promotion`. Used by the CLI's
/// `learn promote` so an eligibility-blocked run can never activate a skill.
pub fn promotion_gate(decision: &AdaptationDecision) -> Result<(), String> {
    if decision.promotion_allowed() {
        Ok(())
    } else {
        Err(format!(
            "promotion blocked by AdaptationDecision '{}': {}",
            decision.decision_id, decision.reason
        ))
    }
}
