# Debug From Symptom

## Use this when

Use this when the task begins with a failure, regression, unexpected behavior, or suspicious validation output. The goal is to explain the symptom before changing the system.

This incorporates the 9arm debugging process: reliable reproduction first, fail path tracing second, hypothesis disproof third, and a breadcrumb ledger across every run.

## Composes with

Use after `00-orient-and-route.md`. Use before `20-change-with-proof.md` once root cause is understood. Use before `50-capture-learning.md` if the failure pattern should be remembered.

## Steps

1. Capture the exact failing command or symptom.
2. Establish reliable reproduction. If the failure is flaky, raise the failure rate before diagnosing. If reproduction is unavailable, stop and name the missing artifact or access.
3. Trace the fail path end-to-end. Find the boundary where expected behavior diverges from observed behavior.
4. Start a breadcrumb ledger. Record each run, what changed, what happened, and what it ruled in or out.
5. Form 3-5 ranked hypotheses after the fail path is known.
6. Try to disprove the leading hypothesis with the smallest experiment before treating it as root cause.
7. Fix only the root cause that explains the symptom and every breadcrumb.
8. Validate against the original reproduction, then run the route proof command.

## Stop when

Stop when the original symptom no longer reproduces, the breadcrumb ledger is consistent with the root cause, and the proof command passes. If blocked, stop when you can state the missing repro, missing access, or contradictory breadcrumb with evidence.

## Avoid

Do not patch likely causes without reliable reproduction. Do not rewrite broad areas before identifying the fail path. Do not trust one plausible hypothesis until you have tried to disprove it.
