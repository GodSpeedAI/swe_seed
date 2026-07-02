# Decisions

## Use this when

Read this when changing architecture, routing, validation, CI, command names, or memory. It records choices that should not be reopened casually.

## Keep in mind

The harness exists to make agents and developers move toward verified outcomes. Prefer contracts that change behavior over documents that only explain concepts.

## Current Decisions

- `just` is the stable command API for humans, agents, and CI.
- `just ci` is the default proof command for completion claims.
- GitHub Actions should call `just ci` instead of duplicating check logic.
- `AGENTS.md` is a dispatcher, not the place for long procedures.
- `.agent-harness/routes/` contains executable route cards. A job type without a route card is not useful.
- `swe-seed route` must return a route plan with next action, context, skills, artifacts, proof, and done conditions.
- Skill IR is stored as JSON to keep validation and rendering dependency-light.
- Material harness self-improvement should update the spec first, then validation, then implementation.
- Memory should stay compact. Add durable lessons, not session transcripts.

## Change Test

Before changing one of these decisions, identify the friction it creates, the evidence for changing it, and the command that proves the new behavior.
