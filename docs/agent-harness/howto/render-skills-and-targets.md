# Render Skills and Targets

Use this when Skill IR changed or when a generated agent surface looks stale.

The canonical source for reusable behavior is JSON Skill IR under `.agent-harness/skills/`. Render targets are generated projections for different agent surfaces. They should not be edited directly.

The repository ships a small built-in core skill set that covers planning, implementation with proof, testing, debugging, review, completion gating, and learning capture. If any of those canonical JSON skills change, rerender before claiming the harness behavior changed.

## Render the targets

Run:

```bash
python scripts/harness.py render-skills
```

Or use:

```bash
just harness-render-skills
```

This rewrites the generated targets from canonical Skill IR.

## Validate freshness

After rendering, run:

```bash
python scripts/harness.py validate
```

This catches stale or invalid render targets and enforces the canonical-file rule.

## Know what is generated

Common generated surfaces live under `.agent-harness/render-targets/` and include:

- Claude skill projections,
- Copilot instruction projections,
- hook prompt fragments,
- checklists.

When a Skill IR includes authoring metadata such as `bundled_resources` or `evaluation_prompts`, the generated targets should surface those sections instead of forcing operators to rediscover them manually.

If two targets would need identical bytes, they should be symlinked instead of copied as divergent regular files.

## Keep in mind

- Edit Skill IR, not generated targets.
- Rerender before claiming a skill change is complete.
- Use validation to prove the projections are current.
