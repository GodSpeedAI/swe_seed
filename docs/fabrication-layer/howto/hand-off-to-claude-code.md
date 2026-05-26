# Hand Off To Claude Code

Prepare the bounded handoff packet with:

```bash
python scripts/fabricate.py handoff <run_id>
```

The handoff directory contains:

- `AGENT_TASK.md`
- `CONTEXT_PACK.md`
- `PRD.md`
- `SDS.md`
- `TDD.md`
- `EVAL_SPEC.yaml`
- `MANIFEST.json`

An execution agent should be able to continue from this packet without needing broader repository
context.
