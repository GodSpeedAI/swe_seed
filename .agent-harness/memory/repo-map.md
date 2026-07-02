# Repo Map

## Use this when

Read this before implementation, bugfix, refactor, review, documentation, or harness-improvement work. Its job is to orient attention quickly so the agent changes the right layer instead of wandering through the tree.

## Keep in mind

This repository has two harness layers:

- CI/CD dev harness: `justfile`, `scripts/`, `.github/workflows/ci.yml`, tool configs, and `docs/dev-harness/`.
- Agentic SWE harness: `AGENTS.md`, `HARNESS_SPEC.md`, `docs/specs/`, and `.agent-harness/`.

The dev harness proves commands and environment behavior. The agentic harness governs how agents route work, load context, use skills, preserve traceability, and learn from friction.

## Attention Map

- Start with `AGENTS.md` when the task is about how agents should behave.
- Start with `HARNESS_SPEC.md` when the task changes the harness contract.
- Start with `.agent-harness/routes/` when the task changes routing or agent movement.
- Start with `crates/swe-seed` when validation, rendering, or route output is wrong.
- Start with `.agent-harness/memory/` when the task concerns durable lessons or cognitive ergonomics.
- Start with `docs/dev-harness/` when the task concerns human operation of local CI, tools, or secrets.

Default proof command: `just ci`.
