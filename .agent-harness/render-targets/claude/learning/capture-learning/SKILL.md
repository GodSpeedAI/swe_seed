---
name: capture-learning
description: A proposal-oriented learning procedure that captures durable lessons, chooses one destination, and updates memory or harness artifacts only when the note changes future behavior.
---

Generated from Skill IR: capture-learning@1
Do not edit this generated file directly; update the Skill IR source instead.

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

## Forbidden behavior

- copying transcripts into memory
- recording one-off preferences as durable cognition
- changing stable instructions without evidence or explicit direction
- writing a note that does not change future behavior

## Done when

- the next agent would make a better decision because of the note
- the note is durable and concise
- validation is rerun when stable harness artifacts changed
