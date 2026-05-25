Generated from Skill IR: debug-discipline@1
Do not edit this generated file directly unless this repository intentionally allows generated-surface edits.
Update the Skill IR source instead.

# Debug Discipline Hook Prompt

## Use this when

Use this hook prompt when lifecycle context indicates: failing test, regression, unexpected behavior.

## What to do

Require the agent to pursue this outcome: diagnose and fix a defect without guessing.

1. Establish a reliable reproduction or stop and document what artifact or access is missing.
2. Trace the fail path end-to-end until expected behavior diverges from observed behavior.
3. Form ranked falsifiable hypotheses.
4. Try to disprove the leading hypothesis with the smallest experiment before treating it as root cause.
5. Maintain a breadcrumb ledger of each run, what changed, what happened, and what it ruled in or out.
6. Implement the smallest fix that explains the failure.
7. Validate against the original reproduction.
8. Add regression protection when appropriate.

## Evidence required

- reliable reproduction command or scenario
- observed failure
- fail path trace
- breadcrumb ledger
- hypothesis disproof result
- root cause explanation
- validation result

## Forbidden behavior

- speculative fix without reliable reproduction or explicit uncertainty
- broad rewrite before root cause
- single-hypothesis anchoring without disproof
- discarding breadcrumb evidence that contradicts the chosen hypothesis
- declaring done without rerunning proof

## Done when

- original failure no longer reproduces under the reliable reproduction
- root cause is connected to fix
- breadcrumb ledger is consistent with the final root cause
- relevant tests pass

Hook behavior should warn or block only when the agent is about to skip evidence, guess at root cause, or claim completion without proof.
