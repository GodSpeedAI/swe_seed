# Artifact Map

The harness works by connecting tasks to durable artifacts.

| Artifact                | Path                                     | Purpose                                                                        |
| ----------------------- | ---------------------------------------- | ------------------------------------------------------------------------------ |
| Router contract         | `AGENTS.md`                              | Tells agents to route work before material changes                             |
| Governing specification | `HARNESS_SPEC.md`                        | Defines the harness contract and conformance requirements                      |
| Route cards             | `.agent-harness/routes/`                 | Executable per-job procedure contracts                                         |
| Playbooks               | `.agent-harness/playbooks/`              | Reusable human-readable procedures used by routes                              |
| Memory artifacts        | `.agent-harness/memory/`                 | Durable cognitive guidance such as repo map, constraints, and failure patterns |
| Context policy          | `.agent-harness/context/`                | Context budget, output containment, and continuity guidance                    |
| Skill IR                | `.agent-harness/skills/`                 | Canonical skill definitions used to generate render targets                    |
| Render targets          | `.agent-harness/render-targets/`         | Agent-facing projections of Skill IR                                           |
| Trace records           | `.agent-harness/traces/records/`         | Task ledger for evidence, checkpoints, and completion claims                   |
| Route decisions         | `.agent-harness/traces/route-decisions/` | Audit trail for route selection                                                |
| Evals                   | `.agent-harness/evals/`                  | Deterministic conformance pressure for the harness                             |
| Reflections             | `.agent-harness/reflections/`            | Improvement proposals and reusable learning                                    |
| Local operator docs     | `docs/agent-harness/`                    | Human-facing explanation, how-to, and reference material                       |
| Spec discovery docs     | `docs/specs/`                            | Lightweight spec mirrors for agent discovery                                   |

When a task changes the harness, identify which artifact actually governs that behavior before editing.
