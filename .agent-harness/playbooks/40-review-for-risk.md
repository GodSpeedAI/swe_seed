# Review For Risk

## Use this when

Use this when asked to review, audit, critique, or inspect changes. It keeps the review focused on defects, regressions, missing proof, and contract drift.

This incorporates the 9arm scrutinize process: question intent, look for a simpler alternative, trace the actual path, and separate claim vs verification.

## Composes with

Use after `00-orient-and-route.md`. Use `10-frame-outcome.md` first when review scope is unclear. Use `50-capture-learning.md` after review if a repeated failure pattern appears.

## Steps

1. Identify the changed contract or intended outcome in one sentence.
2. Ask whether there is a simpler alternative: no change, existing mechanism, smaller layer, or narrower scope.
3. Trace the actual path, not only the diff. Follow entry point, call sites, branches, state changes, and side effects.
4. For each claim, perform claim vs verification: state what the change claims, what path you traced, and whether evidence confirms it.
5. Look first for correctness bugs, missing validation, route/proof gaps, security risks, and broken developer workflow.
6. Cite concrete file and line evidence when available.
7. Separate findings from optional improvements.
8. State residual risk if no finding is found.

## Stop when

Stop when each material risk is either reported with evidence or ruled out by proof. A review with no findings should still name what actual path was traced and any remaining test gaps.

## Avoid

Do not lead with summaries or praise. Do not review only the diff when behavior depends on surrounding code. Do not blur claim vs verification. Do not request stylistic changes unless they affect correctness, maintainability, or the harness outcome.
