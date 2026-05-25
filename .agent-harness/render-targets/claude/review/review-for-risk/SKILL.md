---
name: review-for-risk
description: A review procedure that looks for simpler alternatives, traces the actual path, and separates claim from verification before reporting findings. Use this skill when the request mentions requests to review changes for behavioral risk, audit work on harness contracts or proof surfaces, code review situations where findings must be evidence-backed.
---

Generated from Skill IR: review-for-risk@1
Do not edit this generated file directly unless this repository intentionally allows generated-surface edits.
Update the Skill IR source instead.

# Review For Risk

## Use this when

Use this when triggers match: review, audit, inspect, critique, risk.

Job to be done: evaluate a change for bugs, regressions, proof gaps, and contract drift using actual-path review.

## What to do

1. Identify the changed contract or intended outcome in one sentence.
2. Ask whether there is a simpler alternative before line-level review.
3. Trace the actual path rather than only the diff.
4. Separate claim versus verification for each material claim.
5. Prioritize correctness bugs, missing validation, security risks, and workflow regressions with evidence.
6. State residual risk even when no finding is found.

## Evidence required

- actual path traced
- claim-versus-verification notes
- findings with concrete evidence or an explicit no-findings statement
- residual risk note

## Bundled resources

- `.agent-harness/playbooks/40-review-for-risk.md` (reference): extend the compact review procedure with the longer actual-path and claim-vs-verification workflow Use when the review needs the full scrutinize process.
- `.agent-harness/memory/failure-patterns.md` (reference): bias the review toward known harness failure modes instead of cosmetic commentary Use when the change touches routing, proof, memory, hooks, or validation.

## Evaluation prompts

- `harness-risk-review`: Review a harness change for behavioral regressions and missing proof. Findings must be evidence-backed and ordered by severity. Check for: names the actual path reviewed; keeps findings separate from optional improvements; states residual risk or test gaps.

## Forbidden behavior

- reviewing only the diff when behavior depends on surrounding code
- leading with summary or praise instead of findings
- blurring claims and verification
- suggesting stylistic changes that do not affect outcome, maintainability, or correctness

## Done when

- findings are actionable and evidence-backed
- the actual path reviewed is named
- residual risk is explicit

This generated Claude-style skill follows the 9arm-skills shape: one skill directory, one SKILL.md, YAML frontmatter, and behavior-preserving body generated from Skill IR.
