# Skill IR

Skill IR is the canonical representation of reusable agent behavior.

This scaffold stores Skill IR as JSON under `.agent-harness/skills/` so validation and rendering can run without extra dependencies.

Required fields:

- `id`
- `version`
- `category`
- `jtbd`
- `description`
- `triggers`
- `procedure`
- `evidence_required`
- `forbidden_behaviors`
- `outputs`
- `success_criteria`

Recommended authoring fields:

- `trigger_contexts`: phrases or contexts that should strengthen trigger matching in metadata-driven surfaces
- `bundled_resources`: reusable scripts, references, or assets with path, purpose, and when-to-use guidance
- `evaluation_prompts`: realistic prompts plus the checks each prompt should verify

Rendered artifacts must include source metadata and preserve evidence requirements, forbidden behavior, and success criteria.
