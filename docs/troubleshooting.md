# Troubleshooting Guide

This guide provides diagnostic procedures, root causes, and recovery runbooks for common failures and edge cases encountered when operating SWE_SEED.

---

## Diagnostic Matrix

| Observable Symptom | Likely Root Cause | Subsystem | Diagnostic Command | Recovery Action |
|---|---|---|---|---|
| `gate <id> --verify` exits non-zero (`RouteGate::Block`) | Trace was never routed (missing genesis) or hash chain was tampered with | Trace Ledger / Gate | `swe-seed trace resume <id>` | Re-route the task with `--record` before executing work. Do not hand-edit trace files. |
| Router selects unexpected `job_type` | Semantic triggers or phrasing matched another route card | Routing Engine | `swe-seed route "<task>"` | Use more explicit action verbs (e.g. "fix regression" vs "analyze failure") or refine triggers in route cards. |
| `doctor --host <host>` reports `Drift detected` | Host configuration file was hand-edited inside managed block | Host Adapters / Doctor | `swe-seed doctor --host <host> --json` | Run `swe-seed sync --host <host>` to re-project canonical state, or `swe-seed rollback --host <host>` to restore snapshot. |
| `harness-validate` fails with `missing root spec` | A root specification or BAML schema file was deleted or renamed | Harness Validation | `just harness-validate` | Restore missing file from git history. Root specs and BAML contracts are mandatory. |
| `just doctor` fails with `Missing required commands` | `cargo`, `just`, or `git` is not available in `$PATH` | Environment / Bootstrap | `bash scripts/doctor.sh` | Install missing tools or add `/home/<user>/.cargo/bin` to your environment `$PATH`. |
| `context-plan` excludes a critical constraint file | File is not listed in `required_context` of selected route card | Context Budget Plane | `swe-seed context-plan "<task>"` | Update `.agent-harness/routes/<job_type>.json` to include the required file under `required_context`. |
| Federation envelope verification fails (`InvalidSignature`) | Public key mismatch or envelope payload was modified post-signing | Federation Subsystem | `swe-seed federation status` | Verify public key in `.agent-harness/federation/keys/` matches the private key used to sign. |
| Fabricator `validate-chain` reports broken link | An intermediate artifact (e.g. `JobStory` or `PRD`) is missing or has invalid `TraceabilityLink` | Fabricator Layer | `swe-seed fabricate validate-chain <run_id>` | Re-generate missing link using `swe-seed fabricate <run_id>` following the 10-node chain sequence. |
| Intermittent test failure in `gateway::serve` concurrency test | Multi-threaded log flush without newline synchronization | MCPGate Gateway | `cargo test -p swe-seed-core --lib gateway::serve` | Re-run test individually; isolated concurrency race documented in `.agents/DEBT.md`. |

---

## Detailed Recovery Runbooks

### 1. Recovering from a Blocked Routing Gate (`swe-seed gate`)

#### Symptom:
Running `just gate-merge` or `swe-seed gate <trace_id> --verify` fails with:
```text
error: trace 'trace-xyz' has no RouteSelected genesis — routing is mandatory before work
```
or:
```text
error: trace 'trace-xyz' ledger verification failed: hash mismatch at index 4
```

#### Cause:
1. The developer or agent modified code without first running `swe-seed route "<task>" --record`.
2. Someone manually edited `.agent-harness/traces/records/<trace_id>.json` or tampered with `.agent-harness/traces/ledger.db`.

#### Recovery:
1. If the task was never routed, establish a valid genesis:
   ```bash
   just harness-route-record "<original task description>"
   ```
2. Start the trace:
   ```bash
   just harness-trace-start "<original task description>"
   ```
3. Run the proof command declared in the route:
   ```bash
   just ci
   ```
4. Conclude the trace with the verified proof:
   ```bash
   just harness-trace-finish <new_trace_id> "Completion verified" "just ci" "pass"
   ```
5. Re-run gate verification:
   ```bash
   swe-seed gate <new_trace_id> --verify
   ```

---

### 2. Resolving Host Projection Drift

#### Symptom:
`just harness-doctor` or `swe-seed doctor --host claude` prints:
```text
Host 'claude' has drifted from canonical harness configuration
File: .claude/settings.json (managed block content mismatch)
```

#### Cause:
A developer or host IDE modified lines between `<!-- BEGIN SWE_SEED MANAGED BLOCK -->` and `<!-- END SWE_SEED MANAGED BLOCK -->`.

#### Recovery:
1. **Option A (Overwrite with canonical state)**:
   If the canonical specs in `.agent-harness/` are correct, regenerate the host file:
   ```bash
   swe-seed sync --host claude
   ```
2. **Option B (Restore from snapshot)**:
   If the sync produced unwanted changes, restore the prior snapshot:
   ```bash
   swe-seed rollback --host claude
   ```
3. **Option C (Adopt manual edits into canonical source)**:
   If the manual edits were intentional, update the source templates under `.agent-harness/` and run `swe-seed sync` so all hosts receive the update deterministically.

---

### 3. Diagnosing Environment / PATH Issues in WSL / Linux

#### Symptom:
Running `just` or `cargo` commands yields:
```text
bash: line 1: cargo: command not found
```

#### Cause:
In non-interactive subshells, user environment files (like `.bashrc`) may not be sourced, meaning `$HOME/.cargo/bin` is absent from `$PATH`.

#### Recovery:
Prepend the Cargo binary path explicitly when invoking subshells:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
just doctor
```
Ensure `$HOME/.cargo/bin` is exported in your login profile (`~/.profile` or `~/.bash_profile`).
