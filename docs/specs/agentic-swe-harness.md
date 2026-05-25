# Agentic SWE Harness

Canonical source: [`HARNESS_SPEC.md`](../../HARNESS_SPEC.md).

This document exists so agents can discover the harness specification through the `docs/specs/` contract.

The implementation target for this repository is Core Conformance plus the production-readiness delta now captured in `HARNESS_SPEC.md`: executable route cards, Skill IR rendering, memory artifacts, trace templates, route decision ledgers, trace records, negative evals, route conflict evals, import normalization notes, validation, and a minimal harness CLI.

Required local CLI surface:

```bash
python scripts/harness.py validate
python scripts/harness.py doctor
python scripts/harness.py render-skills
python scripts/harness.py route "task"
python scripts/harness.py route --record "task"
python scripts/harness.py inspect debug-discipline
python scripts/harness.py context-plan "task"
python scripts/harness.py trace start "task"
```

The harness also includes a dependency-free context stewardship layer inspired by `mksglu/context-mode`: route-required context first, tool-output containment, think-in-code analysis for bulk data, and trace-backed session continuity.
