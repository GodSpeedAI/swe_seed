# Inspect the Harness

Use this when you need to answer “what governs this harness behavior?” before editing.

This is the right entry point for route confusion, uncertainty about generated targets, or questions about where a memory or eval rule comes from.

## Inspect a harness item

Run:

```bash
just harness-doctor
```

Replace `debug-discipline` with a route ID, skill ID, memory topic, or other harness term.

The result should help you locate:

- canonical source files,
- rendered targets,
- related routes,
- proof surfaces,
- nearby evals or memory artifacts.

Use this before broad repository search when the question is about the harness itself.

## Check harness health

Run:

```bash
just harness-doctor
```

Use doctor when you suspect a local environment or scaffolding issue rather than a contract bug.

Doctor checks the basic operator surface:

- required commands,
- entry instruction presence,
- trace directory availability.

## Escalate from inspect to source

Once `inspect` tells you the governing files, read those files directly. Common next stops are:

- `HARNESS_SPEC.md`
- `.agent-harness/routes/`
- `.agent-harness/evals/`
- `.agent-harness/memory/`
- `.agent-harness/hooks/hook-router.sh`

## Done when

You can name the artifact that governs the behavior before you edit or debug it.
