# Debug Routing

Use this when the semantic router picks the wrong route or the route result is too weak to guide work.

## 1. Reproduce the route

Run the exact task text through the router:

```bash
just harness-route "task text"
```

Do not paraphrase until you have seen the current result.

## 2. Inspect the winning route card

Read the selected route card in `.agent-harness/routes/` and check:

- semantic triggers,
- positive examples,
- negative examples,
- proof commands,
- first work-loop step.

Routing errors usually come from one of three causes:

- generic token overlap in examples,
- missing positive examples for the intended prompt shape,
- a weak fallback for low-signal prompts.

## 3. Compare against the intended route

Read the intended route card and identify the smallest discriminating difference.

Good fixes are narrow:

- tighten example scoring,
- add a canonical conformance prompt,
- improve fallback behavior for a known prompt class.

Bad fixes are broad:

- adding many vague trigger words,
- rewriting unrelated route cards,
- hardcoding one-off prompts.

## 4. Add a deterministic check

If the route contract changes, add a conformance eval and a line in `just harness-validate`.

## 5. Rerun proof

At minimum:

```bash
just harness-validate
just harness-validate
```

Run `just ci` before claiming completion.
