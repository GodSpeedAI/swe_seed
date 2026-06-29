~~Replaying 10 AI prompts from your last review on registry-provenance-core.~~

────────────────────────────────────────────────────────────────────────
  ~~major [Functional Correctness]~~
  ~~→ ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/adapters/github_copilot.rs:44crates/swe-seed-core/src/adapters/github_copilot.rs:44-53]8;;~~

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,~~
  ~~skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed-core/src/adapters/github_copilot.rs around lines 44 -~~
  ~~53, The GitHub Copilot adapter is returning AdapterSupport::Partial for~~
  ~~every non-FULL event in support_for(), but the support() projection~~
  ~~only advertises FULL events, creating inconsistent host reporting.~~
  ~~Update support() in github_copilot.rs to include the same partial~~
  ~~events that support_for() treats as partial, or change support_for()~~
  ~~so it only returns supported events; keep the behavior aligned across the~~
  ~~support_for() and support() logic.~~


────────────────────────────────────────────────────────────────────────
  ~~major [Functional Correctness]~~
  ~~→ ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/adapters/mod.rs:99crates/swe-seed-core/src/adapters/mod.rs:99-120]8;;~~

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,~~
  ~~skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed-core/src/adapters/mod.rs around lines 99 - 120,~~
  ~~detect_host_drift currently only compares files in~~
  ~~project_host(host).files, so stale snapshot-managed files removed from the~~
  ~~current plan can be missed. Update detect_host_drift to also inspect the~~
  ~~snapshot’s managed file list (from snapshot::read_snapshot /~~
  ~~expected_bytes data) and mark any snapshot-only paths still present on~~
  ~~disk as drifted, then keep HostDriftReport.status in sync with the~~
  ~~combined drifted set.~~


────────────────────────────────────────────────────────────────────────
  ~~major [Functional Correctness]~~
  ~~→ ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/adapters/opencode.rs:38crates/swe-seed-core/src/adapters/opencode.rs:38-46]8;;~~

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,~~
  ~~skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed-core/src/adapters/opencode.rs around lines 38 - 46,~~
  ~~The OpenCode adapter’s projected plan is dropping Partial support~~
  ~~information, so support_for() and project() disagree. Update~~
  ~~project() in opencode.rs to preserve unmapped events from~~
  ~~support_for() as part of the generated ProjectionPlan instead of~~
  ~~serializing only the FULL mappings, and make sure the plan reflects both~~
  ~~AdapterSupport::Full and AdapterSupport::Partial consistently.~~


────────────────────────────────────────────────────────────────────────
  ~~minor [Functional Correctness]~~
  ~~→ ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed/src/cli.rs:567crates/swe-seed/src/cli.rs:567-585]8;;~~

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,~~
  ~~skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed/src/cli.rs around lines 567 - 585, The doctor summary~~
  ~~in cli.rs is printing only report.overall, which can disagree with the~~
  ~~actual exit status when a selected host is drifted. Update the summary~~
  ~~logic in the branch that prints per-check and per-host results so the~~
  ~~final “overall” line reflects the combined doctor outcome (report.overall~~
  ~~plus host_failed) using the existing doctor/report handling in cli.rs,~~
  ~~rather than always printing report.overall alone.~~


────────────────────────────────────────────────────────────────────────
  ~~major [Data Integrity & Integration]~~
  ~~→ ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/adapters/marker.rs:10crates/swe-seed-core/src/adapters/marker.rs:10-25]8;;~~

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,~~
  ~~skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed-core/src/adapters/marker.rs around lines 10 - 25,~~
  ~~merge_json currently only merges overlapping keys, so dropped managed~~
  ~~top-level keys remain in the persisted JSON and the user file never~~
  ~~converges. Update merge_json and the merge_object flow to track the~~
  ~~previously managed key set and remove any keys no longer present in the~~
  ~~new projected Value before writing the result, or switch to replacing a~~
  ~~dedicated managed subtree atomically so old managed fields cannot survive~~
  ~~across syncs.~~


────────────────────────────────────────────────────────────────────────
  ~~major [Functional Correctness]~~
  ~~→ ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/adapters/codex.rs:32crates/swe-seed-core/src/adapters/codex.rs:32-39]8;;~~

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,~~
  ~~skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed-core/src/adapters/codex.rs around lines 32 - 39, The~~
  ~~projected support table is inconsistent with CodexAdapter::support_for():~~
  ~~non-FULL events are marked Partial there, but ProjectionPlan.support~~
  ~~currently only lists Full events, so typed-plan consumers will treat the~~
  ~~partial events as unsupported. Update the ProjectionPlan construction in~~
  ~~codex.rs to include the same partial CanonicalHookEvent entries that~~
  ~~support_for() returns as AdapterSupport::Partial, keeping the plan aligned~~
  ~~with the adapter’s declared coverage.~~


────────────────────────────────────────────────────────────────────────
  ~~major [Data Integrity & Integration]~~
  ~~→ ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/adapters/mod.rs:131crates/swe-seed-core/src/adapters/mod.rs:131-141]8;;~~

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,~~
  ~~skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed-core/src/adapters/mod.rs around lines 131 - 141, The~~
  ~~snapshot read logic in the file handling branch is treating every~~
  ~~std::fs::read failure as if the file were missing, which hides real errors~~
  ~~and can mark existing files as absent. Update the read path in the~~
  ~~snapshot-building code (the match on std::fs::read in the adapter module,~~
  ~~including the related branch in the same flow) to only map NotFound to~~
  ~~SnapshotEntry with existed=false, and propagate all other I/O errors~~
  ~~upward instead of converting them into an empty entry. Keep the existing~~
  ~~SnapshotEntry construction for successful reads, but preserve error~~
  ~~details for non-missing failures so the baseline stays accurate.~~


────────────────────────────────────────────────────────────────────────
  ~~major [Functional Correctness]~~
  ~~→ ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/adapters/antigravity.rs:20crates/swe-seed-core/src/adapters/antigravity.rs:20-29]8;;~~

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,~~
  ~~skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed-core/src/adapters/antigravity.rs around lines 20 - 29,~~
  ~~Keep ProjectionPlan.support in sync with~~
  ~~AntigravityAdapter::support_for(): support_for() marks~~
  ~~CanonicalHookEvent::Stop as Partial, but project() currently only exposes~~
  ~~PreToolUse, so any constructed ProjectionPlan under-reports coverage.~~
  ~~Update AntigravityAdapter::project() to compute and publish support that~~
  ~~matches the adapter’s declared event support, using the same support_for()~~
  ~~logic or equivalent coverage calculation, and ensure the~~
  ~~ProjectionPlan.support field reflects all supported events.~~


────────────────────────────────────────────────────────────────────────
  ~~major [Data Integrity & Integration]~~
  ~~→ ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/adapters/host.rs:12crates/swe-seed-core/src/adapters/host.rs:12-20]8;;~~

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,~~
  ~~skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed-core/src/adapters/host.rs around lines 12 - 20,~~
  ~~HostId’s serde representation is inconsistent with its as_str and~~
  ~~FromStr contract, so JSON uses Rust variant names instead of the~~
  ~~expected wire values. Update the HostId enum in host.rs so~~
  ~~serialization/deserialization follows the same lowercase/kebab-case~~
  ~~strings used by as_str and FromStr, and ensure the affected~~
  ~~ProjectionPlan/ProjectionSnapshot/drift report paths keep using that~~
  ~~same format through the HostId type.~~


────────────────────────────────────────────────────────────────────────
  ~~major [Functional Correctness]~~
  ~~→ ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/adapters/ci.rs:28crates/swe-seed-core/src/adapters/ci.rs:28-40]8;;~~

  ~~▶ Prompt for AI agent~~
  ~~Verify each finding against current code. Fix only still-valid issues,~~
  ~~skip the rest with a brief reason, keep changes minimal, and validate.~~

  ~~In @crates/swe-seed-core/src/adapters/ci.rs around lines 28 - 40, The CI~~
  ~~adapter’s projection metadata is inconsistent with its support reporting:~~
  ~~support_for() in the CI adapter marks both CanonicalHookEvent::PreToolUse~~
  ~~and CanonicalHookEvent::PostToolUse as partially supported, but the~~
  ~~projected support list only advertises PreToolUse. Update the CI adapter’s~~
  ~~projection/support construction so ProjectionPlan.support includes the~~
  ~~same partial events that support_for() reports, keeping the metadata~~
  ~~contract aligned.~~

