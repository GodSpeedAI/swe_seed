# Anthropic Skill Creator Normalization

## Source skill

The local harness incorporates authoring lessons from `anthropics/skills` `skills/skill-creator/SKILL.md`, especially the parts that improve whether a skill triggers correctly, carries its reusable support material, and can be evaluated with realistic prompts.

## Imported invariant

- A skill description must communicate both what the skill does and when it should trigger.
- Reusable support material should be planned explicitly as scripts, references, or assets instead of being rediscovered ad hoc.
- Skill completion is stronger when the author records realistic evaluation prompts before claiming the skill is finished.
- Imported skill-authoring behavior has value only when it improves the local harness outcome instead of adding ceremony.

## Local artifact

The imported behavior is normalized into local artifacts instead of copied as a dependency:

- `.agent-harness/routes/skill_authoring.json` points skill-authoring work at trigger metadata, resource planning, and evaluation prompts.
- `.agent-harness/skills/**` can store `description`, `trigger_contexts`, `bundled_resources`, and `evaluation_prompts` in canonical Skill IR.
- `.agent-harness/render-targets/**` projects those fields into generated skill surfaces.
- `docs/specs/skill-ir.md` and `HARNESS_SPEC.md` define the contract.

## Validation

`just harness-validate` and `just harness-validate` check that the normalized import exists, Skill IR includes the required metadata, and the generated Claude skill surface exposes bundled resources and evaluation prompts when present.
