Generated from Skill IR: review-for-risk@1
Do not edit this generated file directly; update the Skill IR source instead.

# Review For Risk Checklist

## Use this when

Use this checklist when triggers match: review, audit, inspect, critique, risk.

## What to do

- [ ] Identify the changed contract or intended outcome in one sentence.
- [ ] Ask whether there is a simpler alternative before line-level review.
- [ ] Trace the actual path rather than only the diff.
- [ ] Separate claim versus verification for each material claim.
- [ ] Prioritize correctness bugs, missing validation, security risks, and workflow regressions with evidence.
- [ ] State residual risk even when no finding is found.

## Evidence required

- actual path traced
- claim-versus-verification notes
- findings with concrete evidence or an explicit no-findings statement
- residual risk note

## Forbidden behavior

- reviewing only the diff when behavior depends on surrounding code
- leading with summary or praise instead of findings
- blurring claims and verification
- suggesting stylistic changes that do not affect outcome, maintainability, or correctness

## Done when

- findings are actionable and evidence-backed
- the actual path reviewed is named
- residual risk is explicit
