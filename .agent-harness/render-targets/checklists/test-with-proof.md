Generated from Skill IR: test-with-proof@1
Do not edit this generated file directly unless this repository intentionally allows generated-surface edits.
Update the Skill IR source instead.

# Test With Proof Checklist

## Use this when

Use this checklist when triggers match: test, coverage, validation, proof, regression protection.

## What to do

- [ ] Identify the behavior under test and the intended failure signal.
- [ ] Write a failing or contract-revealing check that exercises the real behavior.
- [ ] Run the check and confirm the expected signal.
- [ ] Make the smallest change needed to make the check pass without unrelated behavior changes.
- [ ] Run the proof path that includes the new check and read the output.

## Evidence required

- behavior-under-test note
- failing or contract-revealing check
- red-green evidence when applicable
- proof output

## Bundled resources

- `.agent-harness/routes/test.json` (reference): keep the testing procedure aligned with the route card that governs verification-only work Use when the task is primarily about tests or validation.
- `docs/specs/verification-system.md` (reference): tie the new check to the repository proof model and completion rule Use when the operator needs to confirm that the verification surface supports the final claim.

## Evaluation prompts

- `contract-check`: Add a harness contract check for a missing behavior and prove it fails before the implementation change. Check for: captures a real failure signal; keeps the change scoped to verification when possible; proves the new check is part of the final proof path.

## Forbidden behavior

- tests that only validate mocks instead of real behavior
- tests that pass before they can fail
- folding unrelated production changes into the testing slice
- claiming the check works without rerunning proof

## Done when

- the new check catches the intended behavior
- the proof path includes the new check
- the testing slice does not smuggle in unrelated changes

Before final response, every checked item must be backed by observed evidence or a documented skipped-check reason.
