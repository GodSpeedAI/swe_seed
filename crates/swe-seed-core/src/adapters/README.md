# adapters

## Overview

Host adapter subsystem for the SWE_Seed governance library. It projects canonical
hook policies into per-host configuration files for six supported agent hosts
(Claude, Codex, OpenCode, GitHub Copilot, Antigravity, CI), snapshots the
pre-projection state for rollback, and detects drift between on-disk files and
the last projection. The central dispatch is `adapter_for`, and every adapter
implements the `HostAdapter` trait.

## Interface

- `adapter_for(host: HostId) -> Box<dyn HostAdapter>`: factory returning one of
  `ClaudeAdapter`, `CodexAdapter`, `OpenCodeAdapter`, `GithubCopilotAdapter`,
  `AntigravityAdapter`, `CiAdapter` [mod.rs:adapter_for]
- `project_host(host: HostId) -> ProjectionPlan`: adapter-delegated plan builder
  [mod.rs:project_host]
- `hook_command_for(event: CanonicalHookEvent) -> String`: `PreToolUse` projects
  the enforcing gate command `swe-seed agent-hooks route-gate`; every other event
  projects the logging command `swe-seed agent-hooks capture <event>`
  [mod.rs:hook_command_for]
- `sync_host(root: &Path, host: HostId, options: SyncOptions) -> Result<SyncReport>`:
  snapshots (first sync only), then writes projected files unless `dry_run`
  [mod.rs:sync_host]
- `rollback_host(root: &Path, host: HostId) -> Result<Vec<String>>`: restores
  snapshotted bytes, removes files that did not pre-exist, deletes the snapshot
  [mod.rs:rollback_host]
- `detect_host_drift(root: &Path, host: HostId) -> Result<HostDriftReport>`:
  compares on-disk bytes with the snapshot-derived expected projection and flags
  stale snapshot-only paths [mod.rs:detect_host_drift]
- `support_entries(events, support_for) -> Vec<EventSupport>`: drops
  `AdapterSupport::Unsupported` entries from support tables [mod.rs:support_entries]
- `pub trait HostAdapter { host_id, capabilities, support_for, project }`
  [host.rs:HostAdapter]
- `pub enum HostId { Claude, Codex, OpenCode, GithubCopilot, Antigravity, Ci }`
  with serde renames `"claude"`, `"codex"`, `"opencode"`, `"github-copilot"`,
  `"antigravity"`, `"ci"` and `FromStr` [host.rs:HostId]
- `all_host_ids() -> Vec<HostId>` [host.rs:all_host_ids]
- `pub enum AdapterSupport { Full, Partial { reason }, Unsupported { reason } }`
  with `is_full()` [host.rs:AdapterSupport]
- `pub enum FailureMode { FailOpen, FailClosed, RuntimeSpecific, Unknown }`
  [capabilities.rs:FailureMode]
- `pub struct HostCapabilities` (block/rewrite/modify/inject/stop/subagent/watch
  flags plus `pre_tool_failure_mode`) and const constructor
  `HostCapabilities::command_hooks()` [capabilities.rs:HostCapabilities]
- `pub struct ProjectedFile { rel_path, content, owned_whole_file }`,
  `ProjectionPlan`, `EventSupport`, `SyncReport`, `SyncOptions` with const
  constructors `SyncOptions::write()` / `SyncOptions::dry_run()`, `DriftStatus`,
  `HostDriftReport` [projection.rs:ProjectedFile], [projection.rs:ProjectionPlan],
  [projection.rs:EventSupport], [projection.rs:SyncReport],
  [projection.rs:SyncOptions], [projection.rs:DriftStatus],
  [projection.rs:HostDriftReport]
- `ProjectionSnapshot`, `SnapshotEntry { rel_path, existed, bytes }`,
  `snapshot_path(root, host) -> <root>/.swe-seed/host-snapshots/<host>.json`,
  `read_snapshot`, `write_snapshot`, `delete_snapshot` [snapshot.rs:ProjectionSnapshot],
  [snapshot.rs:SnapshotEntry], [snapshot.rs:snapshot_path], [snapshot.rs:read_snapshot],
  [snapshot.rs:write_snapshot], [snapshot.rs:delete_snapshot]
- `MARKER_BEGIN` / `MARKER_END` = `"BEGIN SWE_SEED MANAGED"` / `"END SWE_SEED MANAGED"`,
  `block(stable_id, body)`, `merge_json(existing, projected)` with managed-key
  stamping under `swe_seed_managed.managed_keys` [marker.rs:MARKER_BEGIN],
  [marker.rs:block], [marker.rs:merge_json]
- `ClaudeAdapter`: 15 fully supported canonical events; projects
  `.claude/settings.json` and `.claude/settings.local.json` (merged, not owned)
  [claude.rs:ClaudeAdapter], [claude.rs:project]
- `CodexAdapter`: 9 fully supported events; projects `.codex/hooks.json`; other
  events are `Partial` with reason "Codex CLI projection is limited to command
  hooks" [codex.rs:CodexAdapter], [codex.rs:support_for]
- `OpenCodeAdapter`: 7 events mapped to native plugin names
  (`tool.execute.before`, `tool.execute.after`, `permission.asked`,
  `session.idle`, `session.error`, `session.compacted`, `file.edited`); projects
  `.opencode/plugins/swe_seed.ts` as an owned marker block whose body enforces
  the route gate on `PreToolUse` [opencode.rs:OpenCodeAdapter],
  [opencode.rs:project]
- `GithubCopilotAdapter`: 12 fully supported events; projects
  `.github/hooks/swe-seed.json` with `cloud_agent_constraints` entry
  [github_copilot.rs:GithubCopilotAdapter], [github_copilot.rs:project]
- `AntigravityAdapter`: only `PreToolUse` is `Full` (matchers `run_command`,
  `shell`, `bash`, `write`, `edit`); `Stop` is `Partial`; others `Unsupported`;
  projects `.agents/hooks.json` [antigravity.rs:support_for],
  [antigravity.rs:project]
- `CiAdapter`: `PreToolUse`/`PostToolUse` are `Partial` (post-hoc enforcement);
  projects `.agents/ci/swe-seed-policy-check.sh`,
  `.agents/ci/swe-seed-policy-check.ps1`, and
  `.github/workflows/swe-seed-policy.yml`, all owned whole files running
  `swe-seed doctor --host all` and `swe-seed harness` [ci.rs:support_for],
  [ci.rs:project]

## Invariants

- First-write snapshot: `write_snapshot` is a no-op when the snapshot file
  already exists and `ensure_snapshot` skips when a snapshot is readable, so the
  first `sync_host` wins and rollback always restores the pre-projection state
  [snapshot.rs:write_snapshot], [mod.rs:ensure_snapshot]
- `PreToolUse` must project the enforcing route gate; all other canonical events
  project the non-enforcing capture command [mod.rs:hook_command_for]
- Files whose `rel_path` ends in `.json` are merged via `marker::merge_json`
  (existing keys are preserved, previously managed keys that are no longer
  projected are removed); all other files are written byte-for-byte from the
  projection [mod.rs:projected_bytes], [marker.rs:merge_object]
- `HostId::from_str` fails closed with `bail!("unsupported host: {other}")` for
  any name outside the six supported hosts [host.rs:FromStr]
- `rollback_host` removes projected files whose snapshot entry has
  `existed == false`, restoring the exact pre-projection file set
  [mod.rs:rollback_host]
- Drift status is `Clean` only when every projected file matches the
  snapshot-derived expected bytes and no stale snapshot-only path exists on disk
  [mod.rs:detect_host_drift]

## Side Effects

- Writes host configuration files under the target root: `.claude/settings.json`,
  `.claude/settings.local.json`, `.codex/hooks.json`,
  `.opencode/plugins/swe_seed.ts`, `.github/hooks/swe-seed.json`,
  `.agents/hooks.json`, `.agents/ci/swe-seed-policy-check.sh`,
  `.agents/ci/swe-seed-policy-check.ps1`, `.github/workflows/swe-seed-policy.yml`
  [claude.rs:project], [codex.rs:project], [opencode.rs:project],
  [github_copilot.rs:project], [antigravity.rs:project], [ci.rs:project]
- Creates, reads, and deletes snapshot state under
  `<root>/.swe-seed/host-snapshots/<host>.json` [snapshot.rs:snapshot_path]
- `sync_host`/`rollback_host` create parent directories, overwrite file contents,
  and remove files via `std::fs::write` / `std::fs::remove_file`
  [mod.rs:write_projected], [mod.rs:rollback_host]
- Projected CI scripts and workflow execute `swe-seed doctor --host all` and
  `swe-seed harness` at CI runtime (effect of the generated files, not of this
  module) [ci.rs:project]

## Verification

- Run: `cargo test -p swe-seed-core`
