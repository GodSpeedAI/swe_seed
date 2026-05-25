# Negative Conformance Evals

These evals define failures the harness must catch. They are intentionally negative because a harness that only proves happy paths will miss completion theater.

## Failure Eval 1: Missing Route Action

Breakage: `AGENTS.md` lists job types but does not tell the agent to run or apply the router.

Expected failure: `python scripts/harness.py validate` fails because the operating contract no longer makes job types executable.

Why it matters: Labels without next action make the agent know more while moving less.

## Failure Eval 2: Missing Proof Gate

Breakage: A route card has no proof command or done condition.

Expected failure: validation fails on the route card.

Why it matters: The agent can otherwise claim completion from effort instead of evidence.

## Failure Eval 3: Stale Render Target

Breakage: A generated render target is edited directly and no longer matches Skill IR.

Expected failure: validation reports a stale rendered artifact.

Why it matters: Divergent prompt surfaces create inconsistent behavior across agents.

## Failure Eval 4: Thin Memory

Breakage: a memory file contains headings but no actionable operating guidance.

Expected failure: validation reports the memory artifact as too thin or missing required phrases.

Why it matters: Memory should focus attention, not become decorative storage.

## Failure Eval 5: Missing Trace Guidance

Breakage: trace docs omit how route decisions and trace records are written.

Expected failure: validation reports missing trace README phrases.

Why it matters: Traceability must be an action the agent can take.

## Failure Eval 6: Completion Language Drift

Breakage: `AGENTS.md` omits prose constraints such as avoid em dashes, no praise before verification, or avoid should work.

Expected failure: validation reports missing prose constraint phrases.

Why it matters: Claim language shapes whether the agent stops early or keeps moving toward proof.
