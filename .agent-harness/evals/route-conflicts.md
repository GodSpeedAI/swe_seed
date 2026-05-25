# Route Conflict Evals

These evals check whether the semantic router makes a useful choice when a prompt names multiple possible jobs.

## Conflict Eval 1: Bugfix Beats Test

Prompt: "fix a failing regression test"

Expected route: `bugfix`

Precedence rule: A reported failure selects diagnosis and repair before generic test authoring.

## Conflict Eval 2: Harness Spec Beats Implementation

Prompt: "implement HARNESS_SPEC.md semantic router"

Expected route: `harness_improvement`

Precedence rule: Work that changes harness behavior routes through harness improvement even if it includes implementation.

## Conflict Eval 3: Review Beats Refactor

Prompt: "review this refactor for behavioral risk"

Expected route: `review`

Precedence rule: A request to assess risk selects review before changing code.

## Conflict Eval 4: Documentation Beats Spec

Prompt: "document how to initialize the dev environment"

Expected route: `documentation`

Precedence rule: A how-to artifact selects documentation unless the user asks to change product requirements.

## Conflict Eval 5: Release Beats Documentation

Prompt: "prepare release notes and final checks"

Expected route: `release`

Precedence rule: Release readiness governs final checks and notes.

## Conflict Eval 6: Skill Authoring Beats General Harness Work

Prompt: "create a reusable debugging skill from this process"

Expected route: `skill_authoring`

Precedence rule: A portable behavioral contract selects skill authoring before broad harness improvement.

## Conflict Eval 7: Context Mechanism Beats Test

Prompt: "incorporate context-mode mechanisms"

Expected route: `harness_improvement`

Precedence rule: Work that changes routing, context budget, tool-output containment, or session continuity belongs to harness improvement even when validation must also change.
