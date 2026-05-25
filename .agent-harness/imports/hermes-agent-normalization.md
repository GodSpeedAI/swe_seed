# Hermes Agent Normalization

## Source project

The local harness incorporates selected process lessons from `NousResearch/hermes-agent`, especially its constrained post-task review loop for memory and skill maintenance, provenance-aware memory handling, and compaction-safe carry-forward of distilled learning.

## Imported invariant

- Finished work should be distilled into candidate memory, skill, and harness updates before context is discarded.
- Learning updates need provenance: trace, route, evidence, and unresolved risk should stay attached to the candidate change.
- Skill learning should prefer patching an existing governing skill before creating a new one, and support files are often a better fit than a brand-new skill.
- Compression or restart should preserve the distilled learning packet instead of forcing transcript archaeology.

## Local artifact

The imported behavior is normalized into local artifacts instead of runtime dependencies:

- `python scripts/harness.py trace distill <trace>` emits a structured learning review packet.
- `.agent-harness/reflections/learning-review-template.yaml` defines the packet shape.
- `HARNESS_SPEC.md` and `docs/specs/memory-system.md` define provenance and learning-review requirements.
- `tests/validate-harness.sh` enforces the command surface and packet availability.

## Validation

`bash tests/validate-harness.sh` checks that the learning review template exists, the trace distillation command is wired into the CLI surface, and a smoke trace can be distilled into a packet that includes `learning_review` and `provenance`.
