//! Shape validation for the chain: EARS requirements + Gherkin scenarios
//! (spec 0017 §3). Shape, not prose quality.

use super::artifacts::{EARSPattern, EARSRequirement, GherkinScenario};

/// Validate an EARS requirement's shape against its declared pattern.
///
/// EARS (Easy Approach to Requirements Syntax):
/// - `Ubiquitous`: "The `<system>` shall `<response>`." — no clause.
/// - `EventDriven`: "When `<trigger>`, the `<system>` shall `<response>`."
/// - `StateDriven`: "While `<state>`, the `<system>` shall `<response>`."
/// - `OptionalFeature`: "Where `<feature>`, the `<system>` shall `<response>`."
/// - `UnwantedBehavior`: "If `<unwanted_condition>`, then the `<system>`
///   shall `<response>`."
///
/// Every pattern needs a non-empty `response` and `ears_text`; each non-
/// Ubiquitous pattern needs its named clause. A requirement marked not
/// `testable` must say why.
pub fn validate_ears_requirement(req: &EARSRequirement) -> Result<(), String> {
    if req.system_name.trim().is_empty() {
        return Err(format!(
            "EARS requirement '{}' has an empty system_name",
            req.id
        ));
    }
    if req.response.trim().is_empty() {
        return Err(format!(
            "EARS requirement '{}' has an empty response",
            req.id
        ));
    }
    if req.ears_text.trim().is_empty() {
        return Err(format!(
            "EARS requirement '{}' has an empty ears_text",
            req.id
        ));
    }
    let clause_err = |field: &str| {
        format!(
            "EARS requirement '{}' uses pattern {:?} but its {field} is empty",
            req.id, req.pattern
        )
    };
    match req.pattern {
        EARSPattern::Ubiquitous => {}
        EARSPattern::EventDriven => {
            if req
                .trigger
                .as_deref()
                .map(str::trim)
                .unwrap_or("")
                .is_empty()
            {
                return Err(clause_err("trigger"));
            }
        }
        EARSPattern::StateDriven => {
            if req.state.as_deref().map(str::trim).unwrap_or("").is_empty() {
                return Err(clause_err("state"));
            }
        }
        EARSPattern::OptionalFeature => {
            if req
                .feature
                .as_deref()
                .map(str::trim)
                .unwrap_or("")
                .is_empty()
            {
                return Err(clause_err("feature"));
            }
        }
        EARSPattern::UnwantedBehavior => {
            if req
                .unwanted_condition
                .as_deref()
                .map(str::trim)
                .unwrap_or("")
                .is_empty()
            {
                return Err(clause_err("unwanted_condition"));
            }
        }
    }
    if !req.testable
        && req
            .non_testable_reason
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .is_empty()
    {
        return Err(format!(
            "EARS requirement '{}' is not testable but gives no non_testable_reason",
            req.id
        ));
    }
    Ok(())
}

/// Validate every EARS requirement in a PRD; return all errors (not just the
/// first) so a caller sees every shape defect at once.
pub fn validate_ears_requirements(reqs: &[EARSRequirement]) -> Result<(), Vec<String>> {
    let errs: Vec<String> = reqs
        .iter()
        .filter_map(|r| validate_ears_requirement(r).err())
        .collect();
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

/// Validate a Gherkin scenario's shape: non-empty name and at least one
/// Given/When/Then step (the irreducible scenario skeleton).
pub fn validate_gherkin_scenario(scn: &GherkinScenario) -> Result<(), String> {
    if scn.name.trim().is_empty() {
        return Err(format!("Gherkin scenario '{}' has an empty name", scn.id));
    }
    if scn.given_steps.iter().all(|s| s.trim().is_empty()) {
        return Err(format!("Gherkin scenario '{}' has no Given step", scn.id));
    }
    if scn.when_steps.iter().all(|s| s.trim().is_empty()) {
        return Err(format!("Gherkin scenario '{}' has no When step", scn.id));
    }
    if scn.then_steps.iter().all(|s| s.trim().is_empty()) {
        return Err(format!("Gherkin scenario '{}' has no Then step", scn.id));
    }
    Ok(())
}

/// Validate every Gherkin scenario; return all errors.
pub fn validate_gherkin_scenarios(scns: &[GherkinScenario]) -> Result<(), Vec<String>> {
    let errs: Vec<String> = scns
        .iter()
        .filter_map(|s| validate_gherkin_scenario(s).err())
        .collect();
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}
