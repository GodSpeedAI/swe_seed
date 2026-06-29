Generated from Skill IR: verify-before-completion@1
Do not edit this generated file directly; update the Skill IR source instead.

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

## Forbidden behavior

- claiming completion from effort or intent
- using stale proof output
- ignoring unresolved risks that contradict the claim
- substituting summary language for observed verification

## Done when

- completion claims are tied to fresh proof
- missing proof is surfaced as a gap, not hidden by summary
- unresolved risks are either cleared or disclosed
