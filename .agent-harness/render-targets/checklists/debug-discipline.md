Generated from Skill IR: debug-discipline@1
Do not edit this generated file directly; update the Skill IR source instead.

# Debug Discipline Checklist

## Use this when

Use this checklist when triggers match: failing test, regression, unexpected behavior.

## What to do

- [ ] Establish a reliable reproduction or stop and document what artifact or access is missing.
- [ ] Trace the fail path end-to-end until expected behavior diverges from observed behavior.
- [ ] Form ranked falsifiable hypotheses.
- [ ] Try to disprove the leading hypothesis with the smallest experiment before treating it as root cause.
- [ ] Maintain a breadcrumb ledger of each run, what changed, what happened, and what it ruled in or out.
- [ ] Implement the smallest fix that explains the failure.
- [ ] Validate against the original reproduction.
- [ ] Add regression protection when appropriate.

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
