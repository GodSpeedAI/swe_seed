---
name: plan-and-frame
description: A framing procedure that names the outcome, the governing artifact, the proof surface, and the smallest useful delta before implementation starts. Use this skill when the request mentions broad requests that need an executable next action, implementation work where the acceptance criteria or proof are still fuzzy, spec or harness changes that need a smallest useful delta.
---

Generated from Skill IR: plan-and-frame@1
Do not edit this generated file directly unless this repository intentionally allows generated-surface edits.
Update the Skill IR source instead.

# Plan And Frame

## Use this when

Use this when triggers match: plan, scope, requirements, acceptance criteria, unclear target.

Job to be done: turn a routed task into a concrete, proof-bearing next action before editing.

## What to do

1. Write the intended outcome in one sentence.
2. Identify the governing artifact that must change, such as spec, code, docs, route card, validation, or hook.
3. Name the proof surface that can falsify the plan, such as a command, validation check, or reviewed artifact.
4. Choose the smallest useful delta that moves the task toward the intended outcome.
5. Decide whether the governing spec must change before implementation begins.

## Evidence required

- one-sentence outcome
- governing artifact
- proof surface
- smallest useful delta
- spec-first or implementation-first decision

## Bundled resources

- `.agent-harness/playbooks/10-frame-outcome.md` (reference): extend the compact framing procedure with the full outcome-and-proof workflow Use when the task is broad, the acceptance criteria are unclear, or the agent needs to decide whether spec changes come first.
- `.agent-harness/memory/constraints.md` (reference): keep planning tied to cheap levers and outcome production instead of decorative structure Use when the proposed change risks adding ceremony without improving proof quality or recovery.

## Evaluation prompts

- `broad-harness-request`: A user asks to improve the harness without naming a failing behavior. Frame the outcome and choose the smallest useful delta before editing. Check for: names a concrete outcome; identifies a falsifiable proof surface; chooses a minimal delta rather than broad exploration.

## Forbidden behavior

- starting implementation without naming the proof surface
- inventing scope to make the work feel complete
- creating abstractions that do not remove a real decision burden
- treating a route label as sufficient guidance

## Done when

- the next edit and proof command are explicit
- the chosen artifact aligns with the requested outcome
- implementation does not begin before the task is framed

This generated Claude-style skill follows the 9arm-skills shape: one skill directory, one SKILL.md, YAML frontmatter, and behavior-preserving body generated from Skill IR.
