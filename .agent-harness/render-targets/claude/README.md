# Claude Render Targets

Generated Claude-style skill surfaces live here.

Shape:

- bucket directory, such as `debug/`
- one directory per skill
- `SKILL.md` with YAML frontmatter containing `name` and `description`

Do not edit generated `SKILL.md` files directly. Update the source Skill IR under `.agent-harness/skills/`, then run:

```bash
python scripts/harness.py render-skills
```

## Skills

- [debug-discipline](debug/debug-discipline/SKILL.md)
