# Capture Run Learning

Run:

```bash
python scripts/fabricate.py reflect <run_id>
```

This refreshes the reflection template so reviewers can record what survived the semantic chain,
what proof is still weak, and whether a reusable lesson exists.

If there is no justified reusable lesson yet, keep `NO_SKILL_PROPOSED.md`. If reflection supports a
real reusable behavior, replace it with `SKILL_PROPOSAL.yaml` using the provided template.
