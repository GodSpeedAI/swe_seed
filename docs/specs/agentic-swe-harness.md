# Agentic SWE Harness

Canonical source: [`HARNESS_SPEC.md`](../../HARNESS_SPEC.md).

This document exists so agents can discover the harness specification through the `docs/specs/` contract.

The implementation target for this repository is Core Conformance plus the production-readiness delta now captured in `HARNESS_SPEC.md`: executable route cards, Skill IR rendering, memory artifacts, trace templates, route decision ledgers, trace records, learning-review distillation packets, negative evals, route conflict evals, import normalization notes, validation, and a minimal harness CLI.

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
python scripts/harness.py trace checkpoint TRACE_ID --stage change --summary "what changed"
python scripts/harness.py trace resume TRACE_ID
```

The harness also includes a dependency-free context stewardship layer inspired by `mksglu/context-mode`, a narrow gstack-derived handoff delta, and a superpowers-derived bootstrap rule: route-required context first, tool-output containment, think-in-code analysis for bulk data, trace-backed session continuity, compact trace checkpoints that let a later route resume from stage, next action, and unresolved risk instead of the full transcript, and a spec-first entry point for broad fresh-session build prompts.
