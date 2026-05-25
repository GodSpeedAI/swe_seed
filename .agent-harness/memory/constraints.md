# Constraints

## Use this when

Read this before adding dependencies, broadening scope, changing validation, touching secrets, or expanding memory. It protects the harness from becoming heavy or performative.

## Keep in mind

The harness should make verified work easier. Do not add structure unless it improves routing, proof, recovery, or developer experience.

## Operating Constraints

- Keep the scaffold dependency-light. Prefer Python standard library, shell, `just`, and existing toolchain support.
- Do not store secrets in memory, traces, rendered instruction files, examples, or logs.
- Do not auto-apply material harness changes while `learning.auto_apply` is false.
- Prefer validation of stable contracts over brittle wording checks.
- Keep `AGENTS.md` compact. It should dispatch to route cards, not duplicate every procedure.
- Keep memory files short enough to read under pressure.
- Prefer small, reversible changes with clear proof commands.
- A clever nudge has no value if it does not improve outcome production.
- Prefer a cheap lever before a heavy mechanism: config, defaults, route wording, validation, generated artifacts, or proof gates.
- Do not add optional architecture such as SQLite, vector search, dashboards, or package signing until actual usage creates pressure for it.
- Preserve local/remote parity: CI should call local commands where practical.

## Scope Control

When tempted to add a new abstraction, ask: does this make the next agent action clearer, reduce failure recovery time, or make a completion claim easier to verify? If not, leave it out.

When tempted to exploit an agent tendency, ask: what failure mode does this prevent, what outcome does it improve, and how will we know?
