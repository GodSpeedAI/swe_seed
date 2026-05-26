# Verify A Prototype

Run proof with:

```bash
python scripts/fabricate.py proof <run_id>
```

The reference implementation writes `proof/EVAL_RESULT.json` and refreshes:

- `generated/PROOF_RECORD.md`
- `generated/REFLECTION.md`
- `generated/ADAPTATION_DECISION.yaml`

The built-in proof is a deterministic static HTML scan for the HTML5 pilot. Release claims can add
manual browser proof on top of that, but must not replace the recorded proof artifacts.
