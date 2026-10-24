# CLI Reference Manual

This reference manual documents the complete command-line interface for the `swe-seed` binary, including all subcommands, arguments, options, defaults, and exit codes.

---

## 1. Top-Level Usage

```bash
swe-seed [COMMAND]
```

### Global Exit Codes:
- `0`: Success / Passed.
- `1`: Domain or verification failure (e.g. gate blocked, boundary validation failed, check failed).
- `2`: Command line usage error / invalid arguments.

---

## 2. Command Index

### `swe-seed route`
Match a user prompt against canonical route cards and output the work contract.
```bash
swe-seed route <task> [--record]
```
- `<task>`: Text description of the task.
- `--record`: Also write a route-decision record to `.agent-harness/traces/route-decisions/<trace_id>.json`. This does **not** create a ledger genesis entry. Genesis is written only by `swe-seed trace start <task>`, which is the command that opens a trace you can later gate.

### `swe-seed context-plan`
Calculate the bounded context pack for a task.
```bash
swe-seed context-plan <task>
```
- `<task>`: Text description of the task.

### `swe-seed gate`
Verify trace genesis and tamper-evident hash chain integrity.
```bash
swe-seed gate <trace> [--verify]
```
- `<trace>`: The trace identifier string.
- `--verify`: Cryptographically recomputes all SHA-256 event hashes in `ledger.sqlite3`.

### `swe-seed trace`
Manage the execution trace lifecycle.
```bash
swe-seed trace start <task>
swe-seed trace append <trace_id> <note>
swe-seed trace checkpoint <trace_id> --stage <stage> --summary <summary> [--next-action <action>]
swe-seed trace resume <trace_id>
swe-seed trace distill <trace_id>
swe-seed trace finish <trace_id> --claim <claim> [--command <cmd>] [--result <result>]
```

### `swe-seed doctor`
Aggregate environment, harness, boundary, and host drift checks.
```bash
swe-seed doctor [--json] [--host <host>]
```
- `--json`: Emit a structured, machine-readable JSON report.
- `--host <host>`: Run drift checks for a specific host or all hosts. Accepted values: `claude`, `codex`, `opencode`, `github-copilot`, `antigravity`, `ci`, `all`. `copilot` is rejected; use `github-copilot`.

### `swe-seed sync`
Project canonical doctrine and skills into host-native tool files.
```bash
swe-seed sync --host <host> [--dry-run] [--prune]
```
- `--host <host>`: Target host tool (mandatory).
- `--dry-run`: Preview changes without modifying files.
- `--prune`: Remove stale projected files.

### `swe-seed rollback`
Restore host runtime files from the latest snapshot.
```bash
swe-seed rollback --host <host>
```

### `swe-seed hosts`
Print the supported host adapters and their capability matrices.
```bash
swe-seed hosts
```

### `swe-seed eval`
Execute deterministic evaluation specifications.
```bash
swe-seed eval run --spec <path> [--output <path>]
```
- `--spec <path>`: Path to the target eval spec. The loader accepts TOML or JSON, not YAML.
- `--trusted`: Allows `command_check` steps, which run spec-controlled shell commands. Off by default.
- `--output <path>`: Optional destination for the result JSON.

### `swe-seed validate`
Validate the structure and frozen hash integrity of an evaluation specification.
```bash
swe-seed validate [spec]
```

### `swe-seed harness`
Execute static structural validation over the repository harness (root specs, BAML files, route cards, active skills, render targets).
```bash
swe-seed harness
```

### `swe-seed seed`
Manage SweSeed outer layer capability assembly and boundary audits.
```bash
swe-seed seed assemble                 # Write .swe-seed/seed-package-manifest.json
swe-seed seed validate-boundaries      # Run layer boundary validation
swe-seed seed regenerate               # Compute idempotent regeneration plan
```

### `swe-seed provenance`
Verify cryptographic hashes and license metadata for all registered capabilities.
```bash
swe-seed provenance verify
swe-seed provenance list
swe-seed provenance show <capability-id>
```
- `verify`: Reads every `*.json` record under `.swe-seed/provenance/` and exits non-zero if any record fails.
- `list`: Prints the records and exits non-zero if any are corrupt.
- `show <capability-id>`: Prints one record from `.swe-seed/provenance/<capability-id>.json`.

### `swe-seed fabricate`
Execute bounded product-to-prototype synthesis runs.
```bash
swe-seed fabricate [need]
swe-seed fabricate validate-chain <run_id>
```
- `[need]`: Bounded product need description.
- `validate-chain <run_id>`: Check unbroken continuity of the 10-node semantic chain.

### `swe-seed gateway`
Manage the MCPGate capability and tool plane proxy.
```bash
swe-seed gateway doctor
swe-seed gateway list
swe-seed gateway proof [--routing] [--concurrency <N>] [--reload]
swe-seed gateway import --namespace <ns> openapi <path>
swe-seed gateway serve [--bind <addr>] [--port <port>] [--once]
```
- `proof`: Runs one of the three built-in proof targets. It takes flags, not a positional argument.
- `import`: Requires `--namespace` and two positionals: the target (currently `openapi`) and the source path.
- `serve`: Serves JSON-RPC over a loopback HTTP endpoint. `--bind` defaults to loopback; a non-loopback address requires TLS or a tunnel. `--port 0` (the default) lets the OS choose. The chosen address is printed to stdout before serving. `--once` serves exactly one connection and exits, which is what the tests use.

### `swe-seed federation`
Manage SEA-Loop federation event envelopes and Ed25519 signing.
```bash
swe-seed federation status
swe-seed federation keygen <key_id>
swe-seed federation sign <trace_id> --key <key_id>
swe-seed federation verify <envelope_file> --key <key_id>
```

### `swe-seed agent-hooks`
Run hook capture, maintenance, and log export routines.
```bash
swe-seed agent-hooks capture <EVENT> [--session-id <id>] [--trace-id <id>]
swe-seed agent-hooks compact-logs
swe-seed agent-hooks index
swe-seed agent-hooks export otel [--output <path>]
swe-seed agent-hooks export junit [--output <path>]
swe-seed agent-hooks route-gate [--trace-id <id>]
```
- `route-gate`: Evaluates the routing gate for the active trace, taken from `--trace-id` or `SWE_SEED_TRACE`. Logs a `PreToolUse` event and exits 0 to allow or 1 to block. When no trace is active it allows. This is the command to wire as a host `PreToolUse` hook.

### `swe-seed reflect` & `swe-seed learn`
Reflect upon completed traces and promote reviewed lessons.
```bash
swe-seed reflect <trace_id>
swe-seed learn promote <record> [--proposal <path>] [--regression <path>]
swe-seed adapt <run_id>
```
