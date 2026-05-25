---
name: verify-before-completion
description: A completion gate that compares the current claim against required proof, unresolved risks, and observed validation output before allowing the work to be called done. Use this skill when the request mentions the agent is about to make a completion claim, release or finishing work that depends on proof output, turn-stop checks that need to block completion theater.
---

Generated from Skill IR: verify-before-completion@1
Do not edit this generated file directly unless this repository intentionally allows generated-surface edits.
Update the Skill IR source instead.

# Verify Before Completion

## Use this when

Use this when triggers match: complete, done, finish, ready, ship.

Job to be done: block premature completion claims until the stated proof obligations have been run and read.

## What to do

1. List the proof commands or evidence required by the current route or contract.
2. Check whether each required proof surface has fresh observed output.
3. Compare unresolved risks against the intended completion claim.
4. If evidence is missing, report the gap instead of claiming completion.
5. Only allow completion language once the required proof has been run and read.

## Evidence required

- required-proof list
- fresh proof output
- unresolved risk note
- completion gate decision

## Bundled resources

- `docs/specs/verification-system.md` (reference): tie completion decisions to the repository proof model rather than to conversational confidence Use when the operator needs to confirm what proof is required for the current claim.
- `.agent-harness/routes/release.json` (reference): reuse the release route's explicit proof-before-release discipline for other high-stakes completion claims Use when the current work is release-adjacent or otherwise sensitive to false completion.

## Evaluation prompts

- `block-false-completion`: The agent is about to say the work is done, but proof output is missing. Apply a completion gate and return the correct next action. Check for: names the missing proof surface; blocks premature completion language; uses observed evidence instead of confidence.

## Forbidden behavior

- claiming completion from effort or intent
- using stale proof output
- ignoring unresolved risks that contradict the claim
- substituting summary language for observed verification

## Done when

- completion claims are tied to fresh proof
- missing proof is surfaced as a gap, not hidden by summary
- unresolved risks are either cleared or disclosed

This generated Claude-style skill follows the 9arm-skills shape: one skill directory, one SKILL.md, YAML frontmatter, and behavior-preserving body generated from Skill IR.
