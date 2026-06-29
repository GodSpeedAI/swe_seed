Generated from Skill IR: test-with-proof@1
Do not edit this generated file directly; update the Skill IR source instead.

# Test With Proof Hook Prompt

## Use this when

Use this hook prompt when lifecycle context matches: test, coverage, validation, proof, regression protection.

## What to do

Require the agent to pursue: add or repair verification that catches the intended behavior and is included in the proof path.

1. Identify the behavior under test and the intended failure signal.
2. Write a failing or contract-revealing check that exercises the real behavior.
3. Run the check and confirm the expected signal.
4. Make the smallest change needed to make the check pass without unrelated behavior changes.
5. Run the proof path that includes the new check and read the output.

## Evidence required

- behavior-under-test note
- failing or contract-revealing check
- red-green evidence when applicable
- proof output

## Forbidden behavior

- tests that only validate mocks instead of real behavior
- tests that pass before they can fail
- folding unrelated production changes into the testing slice
- claiming the check works without rerunning proof

## Done when

- the new check catches the intended behavior
- the proof path includes the new check
- the testing slice does not smuggle in unrelated changes

Hook should warn or block when evidence is skipped or completion is claimed without proof.
