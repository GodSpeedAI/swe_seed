Generated from Skill IR: review-for-risk@1
Do not edit this generated file directly; update the Skill IR source instead.

# Review For Risk Hook Prompt

## Use this when

Use this hook prompt when lifecycle context matches: review, audit, inspect, critique, risk.

## What to do

Require the agent to pursue: evaluate a change for bugs, regressions, proof gaps, and contract drift using actual-path review.

1. Identify the changed contract or intended outcome in one sentence.
2. Ask whether there is a simpler alternative before line-level review.
3. Trace the actual path rather than only the diff.
4. Separate claim versus verification for each material claim.
5. Prioritize correctness bugs, missing validation, security risks, and workflow regressions with evidence.
6. State residual risk even when no finding is found.

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

Hook should warn or block when evidence is skipped or completion is claimed without proof.
