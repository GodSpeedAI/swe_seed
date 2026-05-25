# Frame Outcome

## Use this when

Use this after routing and before making changes when the desired outcome, acceptance criteria, or proof is not concrete enough. This playbook prevents “doing activity” instead of producing the requested result.

## Composes with

Use after `00-orient-and-route.md`. Use before `20-change-with-proof.md` for implementation. Use before `40-review-for-risk.md` when review scope is fuzzy.

## Steps

1. Write the outcome in one sentence.
2. Identify the artifact that must change: spec, code, docs, route card, memory, hook, or validation.
3. Identify proof: command output, route result, doc existence, or explicit reviewed finding.
4. Identify the smallest useful delta.
5. Decide whether the spec must change before implementation.

## Stop when

Stop when the next edit and proof command are clear. If you cannot name the artifact or proof, ask one focused question or record a conservative assumption.

## Avoid

Do not invent scope to make the work feel complete. Do not create abstractions unless they reduce a real decision or proof burden.
