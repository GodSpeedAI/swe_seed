# 9arm Skills Normalization

## Source skill

The local harness incorporates process lessons from `thananon/9arm-skills`, especially the debugging shape: reproduce reliably, trace the fail path, disprove hypotheses, keep a breadcrumb ledger, fix the root cause, and validate against the original failure.

## Imported invariant

- Debugging begins with reliable reproduction.
- Diagnosis traces the fail path before proposing fixes.
- Hypotheses must be falsifiable and tested with small experiments.
- Breadcrumbs preserve what each run proved or ruled out.
- Completion requires proof against the original symptom.
- Reusable lessons become skills, memory, evals, or route-card changes only when they improve outcome production.

## Local artifact

The imported behavior is normalized into local artifacts instead of copied into many divergent files:

- `.agent-harness/skills/40-debug/debug-discipline.json` is the canonical Skill IR.
- `.agent-harness/playbooks/30-debug-from-symptom.md` is the workflow playbook.
- `.agent-harness/render-targets/**/debug-discipline*` are generated projections.
- `.agent-harness/evals/negative-conformance.md` and `.agent-harness/evals/route-conflicts.md` define conformance pressure.

## Validation

`python scripts/harness.py validate` checks for the core 9arm-derived phrases in the debug skill, debug playbook, review playbook, and learning playbook. If the imported process changes, update the canonical Skill IR and regenerate render targets.
