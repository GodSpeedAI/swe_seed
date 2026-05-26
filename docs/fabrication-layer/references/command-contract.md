# Command Contract

Reference commands:

- `fabricate new <seed>`
- `fabricate generate <run_id>`
- `fabricate validate <run_id>`
- `fabricate handoff <run_id>`
- `fabricate proof <run_id>`
- `fabricate reflect <run_id>`
- `fabricate status <run_id>`

The SWE Seed reference packaging also exposes matching `just fabricate-*` aliases.

## Proof Command Map

| Claim                       | Proof command or check                                                     |
| --------------------------- | -------------------------------------------------------------------------- |
| Fabrication scaffold exists | `fabricate validate <run_id>`                                              |
| Artifact packet complete    | `fabricate validate <run_id>`                                              |
| Traceability passes         | `fabricate validate <run_id>`                                              |
| Context pack is ready       | `fabricate handoff <run_id>`                                               |
| Agent task is ready         | `fabricate handoff <run_id>`                                               |
| Prototype exists            | `.fabricator/runs/<run_id>/prototype/index.html` exists                    |
| Prototype runs              | `fabricate proof <run_id>`                                                 |
| HTML5 game opens locally    | `fabricate proof <run_id>` plus optional manual browser review             |
| Proof recorded              | `generated/PROOF_RECORD.md` exists and reflects proof output               |
| Reflection captured         | `generated/REFLECTION.md` exists                                           |
| Skill decision captured     | `generated/SKILL_PROPOSAL.yaml` or `generated/NO_SKILL_PROPOSED.md` exists |

Serialization note: the reference loader is dependency-free and accepts JSON or a small YAML subset.
For complex arrays of objects, `.yaml` artifact files may contain JSON object text so another agent
can regenerate and parse them without adding a YAML runtime.
