Replaying 2 AI prompts from your last review on registry-provenance-core.

────────────────────────────────────────────────────────────────────────
  ~~minor [Functional Correctness]~~
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed/src/cli.rs:95crates/swe-seed/src/cli.rs:95-100]8;;

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed/src/cli.rs around lines 95 - 100, The Run CLI option
  for federation currently treats any value other than "on" as
  standalone, which allows invalid inputs to pass silently. Update the
  argument handling for federation in the Run command and the related
  parsing path around the referenced CLI logic so it only accepts the
  documented off|on values. If an unknown value is provided, return a
  validation error instead of falling back to standalone, and apply the same
  validation in the other referenced federation parsing code path.~~

────────────────────────────────────────────────────────────────────────
  ~~major [Data Integrity & Integration]~~
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/federation/consume.rs:36crates/swe-seed-core/src/federation/consume.rs:36-58]8;;

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed-core/src/federation/consume.rs around lines 36 - 58,
  Tighten federation boundary validation in check_drift and require so
  malformed envelopes cannot pass silently. Update check_drift to reject
  envelopes when domain_model_hash is missing as well as when it differs
  from the expected hash, and update require to also verify
  envelope.namespace matches the expected local namespace before accepting
  the payload. Use the existing ConsumeError variants (or add a specific one
  if needed) and keep the validation centralized in these helpers so
  downstream callers don’t have to enforce boundary identity themselves.~~
