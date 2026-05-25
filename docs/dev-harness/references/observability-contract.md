# Observability Contract

Status: source-of-truth reference for the observability layer defined in [SWE_SEED_SPEC_v0.2.0.md](../../../SWE_SEED_SPEC_v0.2.0.md).

## Architecture

```text
agent hook or command wrapper
-> router or capture shim
-> normalized event
-> append-only JSONL event log
-> optional artifact files
-> optional rusql index
-> optional vector index
-> dashboards / replay / doctor / trace
```

The JSONL event ledger is the durable record. rusql, FTS, Tantivy-style search, and vector search are derived layers that can be rebuilt from the files.

## Required Event Fields

Each normalized event envelope must include:

- `schema_version`
- `event_id`
- `trace_id`
- `span_id`
- `parent_span_id`
- `timestamp`
- `agent`
- `agent_version`
- `native_event`
- `event`
- `session_id`
- `turn_id`
- `cwd`
- `repo_root`
- `profile`
- `hook_id`
- `script`
- `status`
- `duration_ms`
- `exit_code`
- `stdout_ref`
- `stderr_ref`
- `native_payload_ref`
- `normalized_payload_ref`
- `result_ref`

## Preferred File Layout

```text
.agent-hooks/
  config.yaml
  logs/
    events-YYYY-MM-DD.jsonl
  payloads/
    YYYY-MM-DD/
      <event_id>.native.json
      <event_id>.normalized.json
      <event_id>.result.json
  artifacts/
    YYYY-MM-DD/
      <event_id>.stdout.txt
      <event_id>.stderr.txt
  index/
    hooks.rusql
    vectors/
```

## Required Behaviors

- Redact sensitive data before persistence.
- Rotate logs by date and size.
- Preserve raw payloads and stdout or stderr as separate artifacts when useful.
- Support replay from captured artifacts.
- Support inspecting one event directly.
- Keep the observability layer usable from any language that can read stdin and write stdout or stderr.
- Keep indexes rebuildable from the filesystem ledger.

## Required Command Surface

Preferred CLI namespace:

```bash
agent-hooks trace --last
agent-hooks trace --session <id>
agent-hooks replay --event <event_id>
agent-hooks inspect --event <event_id>
agent-hooks doctor --observability
agent-hooks compact-logs
agent-hooks index rebuild
agent-hooks export otel
agent-hooks export junit
```

This repository currently exposes the local CLI as `scripts/agent-hooks ...` and mirrors the main actions through `just` recipes. The underlying behavior must remain stable and replay-focused.

## Upgrade Thresholds

Stay file-only while:

- fewer than 50,000 events exist,
- queries are mostly by date, session, or recent failure,
- `grep`, `jq`, and simple trace tooling remain enough,
- dashboard-style indexed lookups are not yet needed.

Add rusql when:

- filtering by agent, session, hook, or status becomes common,
- `doctor` or `trace` need fast lookups,
- concurrent agent activity makes file scans too slow.

Add vector retrieval when:

- semantic matching of similar failures, prompts, or outputs becomes a real recovery need,
- exact and structured search stop being enough on representative tasks.

## Config Files

- [SWE_SEED_SPEC_v0.2.0.md](../../../SWE_SEED_SPEC_v0.2.0.md)
- `tests/validate-harness.sh`
- `.agent-hooks/config.yaml`
- `scripts/agent-hooks`
- `scripts/agent_hooks.py`

## Ownership Boundaries

The dev harness owns command, CI, and adapter observability. The agent harness may emit compatible identifiers and payloads, but it should not become a runtime dependency of the dev harness observability layer.

## Proof

Current proof for this contract:

```bash
bash tests/validate-harness.sh
just ci
```

The contract is enforced with executable checks for capture, trace, inspect, replay, doctor, compaction, index rebuild, and both export surfaces.
