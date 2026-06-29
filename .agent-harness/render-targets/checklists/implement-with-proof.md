Generated from Skill IR: implement-with-proof@1
Do not edit this generated file directly; update the Skill IR source instead.

# Implement With Proof Checklist

## Use this when

Use this checklist when triggers match: implement, build, add behavior, feature, change behavior.

## What to do

- [ ] Add or update the narrowest validation that should fail for the missing behavior.
- [ ] Run the check and confirm the expected failure signal.
- [ ] Make the smallest implementation change that satisfies the requirement.
- [ ] Update docs or specs only when the behavior contract changed.
- [ ] Run the proof commands and read the output before reporting completion.

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
