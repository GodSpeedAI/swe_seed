# Claude Render Targets

Generated Claude-style skill surfaces live here.

Shape:

- bucket directory, such as `debug/`
- one directory per skill
- `SKILL.md` with YAML frontmatter containing `name` and `description`

Do not edit generated `SKILL.md` files directly. Update the source Skill IR under `.agent-harness/skills/`, then run:

```bash
just harness-render-skills
```

## Skills

- [plan-and-frame](planning/plan-and-frame/SKILL.md)
- [implement-with-proof](implementation/implement-with-proof/SKILL.md)
- [test-with-proof](test/test-with-proof/SKILL.md)
- [debug-discipline](debug/debug-discipline/SKILL.md)
- [review-for-risk](review/review-for-risk/SKILL.md)
- [verify-before-completion](completion/verify-before-completion/SKILL.md)
- [capture-learning](learning/capture-learning/SKILL.md)
