# Proof Before Completion

Fabrication is not done when `generate` finishes. It is done when:

- `validate` passes,
- `handoff` exists,
- `proof` writes a result,
- `PROOF_RECORD.md` is present,
- `REFLECTION.md` is present,
- the skill decision is recorded.

The reference implementation makes this visible in `fabricate status <run_id>` so agents have a
bounded next action instead of a vague completion claim.
