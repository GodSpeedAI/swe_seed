# Change With Proof

## Use this when

Use this for planned implementation, documentation, validation, or harness changes. It keeps change and proof coupled so completion is based on evidence rather than confidence.

## Composes with

Use after `10-frame-outcome.md`. Use with `30-debug-from-symptom.md` when a defect requires a fix. Use before `50-capture-learning.md` when the work reveals a reusable pattern.

## Steps

1. Add or update the narrowest validation that should fail for the missing behavior.
2. Run it and confirm the expected failure.
3. Make the smallest change that satisfies the route card.
4. Update docs, memory, or specs only when the behavior contract changed.
5. Run the proof commands from the route card.
6. Report changed files and observed proof.

## Stop when

Stop when proof commands pass or when a blocker is supported by command output. If proof fails, continue from the failing output rather than summarizing intentions.

## Avoid

Do not duplicate work already handled by `just ci`, `just harness-validate`, route-card validation, or formatter commands. Do not add ceremony that cannot fail usefully.
