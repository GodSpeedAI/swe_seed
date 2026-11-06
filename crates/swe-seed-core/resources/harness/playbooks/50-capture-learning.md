# Capture Learning

## Use this when

Use this at the end of meaningful work when the session revealed a reusable decision, failure pattern, successful pattern, open question, or route defect. The goal is to improve future attention without creating memory bloat.

For fixed defects, use the 9arm post-mortem gate: do not write an engineering record unless reliable reproduction, known root cause, identified fix, and validation coverage all exist.

## Composes with

Use after `20-change-with-proof`, `30-debug-from-symptom`, or `40-review-for-risk`. Use with the `harness_improvement` route when the learning changes a contract.

## Steps

1. Identify whether the lesson is durable and likely to recur.
2. If it is a fixed bug, check the post-mortem gate: reliable reproduction, root cause, fix, and validation coverage.
3. Choose exactly one destination: decisions, failure patterns, successful patterns, open questions, constraints, glossary, route card, spec, or post-mortem.
4. Write the smallest note that changes future behavior.
5. Include the evidence or command that supports it when useful.
6. State validation coverage honestly. If only one configuration was checked, say that.
7. Run validation if a harness artifact changed.

## Stop when

Stop when the next agent would make a better decision because of the note. If the note only records what happened, leave it out or put it in a trace/reflection instead of memory. For bugs, stop if the post-mortem inputs are missing.

## Avoid

Do not copy session transcripts into memory. Do not update memory for one-off preferences. Do not change stable instructions without evidence or explicit user direction.
