Generated from Skill IR: plan-and-frame@1
Do not edit this generated file directly; update the Skill IR source instead.

# Plan And Frame Hook Prompt

## Use this when

Use this hook prompt when lifecycle context matches: plan, scope, requirements, acceptance criteria, unclear target.

## What to do

Require the agent to pursue: turn a routed task into a concrete, proof-bearing next action before editing.

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

## Forbidden behavior

- starting implementation without naming the proof surface
- inventing scope to make the work feel complete
- creating abstractions that do not remove a real decision burden
- treating a route label as sufficient guidance

## Done when

- the next edit and proof command are explicit
- the chosen artifact aligns with the requested outcome
- implementation does not begin before the task is framed

Hook should warn or block when evidence is skipped or completion is claimed without proof.
