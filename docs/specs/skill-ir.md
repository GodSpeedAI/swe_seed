# Skill IR

Skill IR is the canonical representation of reusable agent behavior.

This scaffold stores Skill IR as JSON under `.agent-harness/skills/` so validation and rendering can run without extra dependencies.

Required fields:

- `id`
- `version`
- `category`
- `jtbd`
- `triggers`
- `procedure`
- `evidence_required`
- `forbidden_behaviors`
- `outputs`
- `success_criteria`

Rendered artifacts must include source metadata and preserve evidence requirements, forbidden behavior, and success criteria.
