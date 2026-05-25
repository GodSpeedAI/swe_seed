# Imports

Store raw imported skill sources or import notes here.

Imported skills are untrusted until normalized, reviewed, and promoted to active Skill IR.

## Reviewed Sources

- `thananon/9arm-skills`: used as a process reference for Claude-style skill shape and engineering workflows. Imported invariants:
  - debugging requires reliable reproduction, fail path tracing, hypothesis disproof, and breadcrumb ledger;
  - post-mortem writing requires known root cause, identified fix, and validation coverage;
  - review starts by questioning intent, checking for a simpler alternative, tracing actual paths, and separating claim vs verification.
- `anthropics/skills` `skill-creator`: used as a skill-authoring process reference. Imported invariants:
  - skill descriptions must say both what the skill does and when it should trigger;
  - reusable scripts, references, and assets should be planned explicitly instead of rediscovered every run;
  - realistic evaluation prompts should exist before claiming a skill is complete.

Do not copy source text wholesale. Normalize durable behavior into Skill IR, route cards, playbooks, and validation.
