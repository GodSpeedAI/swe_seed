Generated from Skill IR: implement-with-proof@1
Do not edit this generated file directly; update the Skill IR source instead.

# Implement With Proof Hook Prompt

## Use this when

Use this hook prompt when lifecycle context matches: implement, build, add behavior, feature, change behavior.

## What to do

Require the agent to pursue: change behavior with the smallest implementation delta that is tied to failing and passing proof.

1. Add or update the narrowest validation that should fail for the missing behavior.
2. Run the check and confirm the expected failure signal.
3. Make the smallest implementation change that satisfies the requirement.
4. Update docs or specs only when the behavior contract changed.
5. Run the proof commands and read the output before reporting completion.

## Evidence required

- failing or contract-revealing check
- implementation delta tied to the requirement
- proof-command output
- changed-artifact summary

## Forbidden behavior

- editing before there is a falsifiable check when one is available
- broad rewrites that are not justified by the requirement
- claiming completion from intent instead of proof output
- updating unrelated surfaces while the touched slice is still unverified

## Done when

- the implementation delta is traceable to the requirement
- proof commands pass and are read
- unresolved risks are either absent or explicitly stated

Hook should warn or block when evidence is skipped or completion is claimed without proof.
