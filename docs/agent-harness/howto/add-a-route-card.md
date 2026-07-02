# Add a Route Card

Use this when the harness needs a new job type route or when an existing task class needs its own executable contract.

Do not add a route card just to make the route list feel complete. Add one only when a distinct kind of work needs a different context set, work loop, proof contract, or done condition.

## 1. Confirm the work deserves a route

A new route is justified when one of these is true:

- the task class has a repeatable work loop that differs from existing routes,
- the proof contract differs materially from nearby routes,
- the agent keeps misrouting the task because no route card describes it well,
- the route needs its own artifacts or failure modes.

If the change only adjusts one existing route example, scoring rule, or proof command, update the current route instead.

## 2. Check the governing contract

If the route introduces a new required job type or changes what the harness promises, update `HARNESS_SPEC.md` first.

If it only adds repository-specific coverage within the existing spec, the route card can come first.

## 3. Create the route card

Add a JSON file under `.agent-harness/routes/`.

Use an existing route as the template. Every route card needs:

- `id`
- `job_type`
- `purpose`
- `semantic_triggers`
- `positive_examples`
- `negative_examples`
- `required_context`
- `required_skills`
- `work_loop`
- `required_artifacts`
- `proof`
- `done_when`
- `failure_modes`
- `fallback_policy`

Keep examples concrete. Avoid generic verbs that overlap with unrelated routes.

## 4. Make the route executable

The first work-loop step must move the task forward. A route card is not a label. It is a procedure contract.

Check that the route names:

- the right playbooks,
- the right memory or spec files,
- the proof commands that actually gate completion,
- artifacts that can be observed after the work.

## 5. Add deterministic coverage

Add at least one route check.

Common places:

- `.agent-harness/evals/core-conformance.md` for a named canonical prompt,
- `.agent-harness/evals/route-conflicts.md` if the new route could be confused with another route,
- `just harness-validate` for a deterministic CLI assertion.

If the route is meant to win against another plausible route, add a route-conflict eval instead of only a happy-path example.

## 6. Update operator docs when needed

If humans will need to use or interpret the route, update:

- `docs/agent-harness/references/route-map.md`
- `docs/agent-harness/references/artifact-map.md` if a new artifact class appears
- any relevant how-to guide

## 7. Run proof

```bash
just harness-validate
just harness-validate
just ci
```

## Done when

The route has a concrete contract, deterministic validation proves it can be selected, and the proof chain passes.
