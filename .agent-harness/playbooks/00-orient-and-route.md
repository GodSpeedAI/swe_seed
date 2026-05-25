# Orient And Route

## Use this when

Use this at the start of any non-trivial task, especially when the user asks for implementation, bugfixing, review, or harness improvement. The goal is to spend attention on the right layer before touching files.

## Composes with

Use before `10-frame-outcome.md` for new work. Use before `30-debug-from-symptom.md` when the task starts with a failure. Use before `40-review-for-risk.md` when asked to review.

## Steps

1. Read `AGENTS.md`.
2. Run or mentally apply `python scripts/harness.py route "<task>"`.
3. Read the selected route card.
4. Read only the route card's required context unless evidence says more is needed.
5. Name the next concrete action from the route result.

## Stop when

Stop when you have a selected route card, required context, proof commands, and first action. If routing yields only a label or feels decorative, switch to the `harness_improvement` route because the harness failed to move the work.

## Avoid

Do not read the whole repository for orientation. Do not ask a question when a safe route and reversible first step are available.
