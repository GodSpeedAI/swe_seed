# Start a Task

Use this flow when a task is not trivial.

## 1. Route the task

Run:

```bash
python scripts/harness.py route "task"
```

The result should name:

- `job_type`
- `route_card`
- `required_context`
- `proof`
- `next_action`

If the route does not move the next action forward, treat that as a harness defect.

## 2. Record the route when auditability matters

For implementation-like or harness-changing work, write the route decision first:

```bash
python scripts/harness.py route --record "task"
```

This creates a route decision ledger entry under `.agent-harness/traces/route-decisions/`.

## 3. Read only the required context

Start with the files named by the route. If the task still needs broader context, generate a context plan:

```bash
python scripts/harness.py context-plan "task"
```

Read only enough to make the next edit or validation step clear.

## 4. Start a trace before material changes

Run:

```bash
python scripts/harness.py trace start "task"
```

This links the task to a route decision and creates a working ledger for evidence.

## 5. Follow the route work loop and proof commands

Use the route card as the procedure contract. Do not skip to editing or completion claims just because the task seems obvious.

## Done when

The task has a governing route, a minimal context set, a trace when needed, and a clear proof command before edits begin.
