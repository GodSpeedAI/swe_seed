Generated from Skill IR: implement-with-proof@1
Do not edit this generated file directly unless this repository intentionally allows generated-surface edits.
Update the Skill IR source instead.

# Implement With Proof

## Use this when

Use this when the request or observed state matches these triggers: implement, build, add behavior, feature, change behavior.

Job to be done: change behavior with the smallest implementation delta that is tied to failing and passing proof.

## What to do

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

## Bundled resources

- `.agent-harness/playbooks/20-change-with-proof.md` (reference): extend the compact implementation procedure with the full red-green-proof loop Use when the task changes behavior and needs a disciplined validation sequence.
- `.agent-harness/traces/traceability-template.yaml` (asset): capture changed artifacts, validation, and next actions when the implementation spans multiple steps Use when the work needs durable traceability or handoff.

## Evaluation prompts

- `small-feature-proof`: Implement a small harness behavior change and prove it with the narrowest available failing and passing check. Check for: adds or updates a falsifiable check first; keeps the implementation delta focused; reads the proof output before claiming completion.

## Forbidden behavior

- editing before there is a falsifiable check when one is available
- broad rewrites that are not justified by the requirement
- claiming completion from intent instead of proof output
- updating unrelated surfaces while the touched slice is still unverified

## Done when

- the implementation delta is traceable to the requirement
- proof commands pass and are read
- unresolved risks are either absent or explicitly stated

If evidence is missing, do not claim completion. State the gap and the next proof command.
