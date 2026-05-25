---
name: capture-learning
description: A proposal-oriented learning procedure that captures durable lessons, chooses one destination, and updates memory or harness artifacts only when the note changes future behavior. Use this skill when the request mentions the end of meaningful work that revealed a reusable lesson, sessions that exposed a route defect or repeated friction, trace distillation and proposal-oriented memory updates.
---

Generated from Skill IR: capture-learning@1
Do not edit this generated file directly unless this repository intentionally allows generated-surface edits.
Update the Skill IR source instead.

# Capture Learning

## Use this when

Use this when triggers match: learning, lesson, postmortem, memory update, reflection.

Job to be done: record only durable, evidence-backed lessons that improve future agent decisions without creating memory bloat.

## What to do

1. Identify whether the lesson is durable and likely to recur.
2. If it is a fixed bug, check the post-mortem gate: reproduction, root cause, fix, and validation coverage.
3. Choose exactly one destination for the lesson.
4. Write the smallest note that changes future behavior.
5. Include supporting evidence when useful and state validation coverage honestly.
6. Run validation when a harness artifact changed.

## Evidence required

- durability decision
- chosen destination
- small evidence-backed note
- validation coverage statement

## Bundled resources

- `.agent-harness/playbooks/50-capture-learning.md` (reference): extend the compact learning procedure with the full post-mortem gate and destination rules Use when a session reveals a reusable lesson, route defect, or durable pattern.
- `.agent-harness/reflections/learning-review-template.yaml` (asset): shape proposal-oriented memory, skill, and harness updates from finished traces Use when learning needs to be carried forward as a structured packet rather than a direct memory edit.

## Evaluation prompts

- `durable-lesson-only`: A session revealed a recurring harness failure pattern. Capture only the smallest evidence-backed lesson that improves future behavior. Check for: chooses one destination; avoids transcript-style memory; states validation coverage honestly.

## Forbidden behavior

- copying transcripts into memory
- recording one-off preferences as durable cognition
- changing stable instructions without evidence or explicit direction
- writing a note that does not change future behavior

## Done when

- the next agent would make a better decision because of the note
- the note is durable and concise
- validation is rerun when stable harness artifacts changed

This generated Claude-style skill follows the 9arm-skills shape: one skill directory, one SKILL.md, YAML frontmatter, and behavior-preserving body generated from Skill IR.
