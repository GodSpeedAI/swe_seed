# 0008 — Doctor and Drift Detection

> **Reconciled by [0012](0012-existing-harness-reconciliation.md) (authoritative).** Doctor =
> `validate` + `eval run` ([0013](0013-eval-and-proof.md)) + `validate-boundaries`
> ([0018](0018-layer-boundary-governance.md)). The proof artifact is **ProofRecord**/
> **EvalResult** (0013); `DoctorCheck` is the local check wrapper that runs them. Live-pass-only
> promotion (0013/0011) applies.

## Purpose

Define a first-class `swe-seed doctor` that verifies the centralization layer is healthy
and that generated host config matches the registry (drift detection). Doctor is read-only
by default, with opt-in `--fix` for auto-fixable checks.

## Non-goals

- Doctor does not mutate state unless `--fix` is passed.
- Not a replacement for host tools' own diagnostics.

## Prior-art evidence (sources removed, unnamed)

| Prior-art (removed) | File/path | Lines or section | Pattern observed | SWE_Seed interpretation |
|---|---|---|---|---|
| gateway prior-art | `src/commands/doctor.rs:21-31` | `enum CheckStatus { Pass, Fail, Warn }` | Three-state status | `DoctorStatus` (re-derived) |
| gateway prior-art | `src/commands/doctor.rs:32-78` | `struct CheckResult{label,detail,status,hint}` + `pass/fail/warn/with_hint` | Result shape w/ remediation hint | `DoctorCheck` result + remediation |
| gateway prior-art | `src/commands/doctor.rs:80-135` | `run_doctor_command(fix, config_path)`; collect, print, exit on fails | Doctor runner + exit code semantics |
| gateway prior-art | `:137-167` config present/parseable; `:173-180` port free; `:182-202` backend env set; `:204-262` http/stdio backend reachable; `:265-288` client points at gateway | Concrete check catalog | SWE_Seed check catalog below (adapted/extended) |
| gateway prior-art | `src/commands/config_export/watch.rs` | file | Watch/resync exported config | Drift detection via projection hashes |

## SWE_Seed requirements

### Commands

```bash
swe-seed doctor                 # all hosts in this project
swe-seed doctor --host claude
swe-seed doctor --host codex
swe-seed doctor --host opencode
swe-seed doctor --project .
swe-seed doctor --fix           # apply auto-fixable remediations
```

Exit code: `0` if no `Fail`; non-zero if any `Fail` (warnings do not fail). Output groups
results and prints remediation hints, mirroring the runner shape observed at
`doctor.rs:289-311`.

### DoctorCheck model

```
struct DoctorCheck {
  name: String,
  severity: Severity,          // Info | Warn | Error
  run: fn(&Ctx) -> CheckOutcome // Pass | Warn | Fail
  remediation: String,
  auto_fixable: bool,
}
```

### Check catalog

| Check | Severity | Pass/Warn/Fail meaning | Remediation | Auto-fixable |
|---|---|---|---|---|
| active-skills | Info | registry skills resolve to cached content | re-fetch source | yes |
| active-mcp-servers | Error | each registered MCP defined + transport valid | fix registry entry | no |
| active-hooks | Warn | hooks map to supported events on target host | drop/relocate unsupported hook | partial |
| generated-host-files | Error | every `Projection` file exists on disk | run `swe-seed sync` | yes |
| stale-generated-files | Warn | no managed file lacks a registry source | `swe-seed sync --prune` | yes |
| broken-symlinks | Warn | managed symlinks resolve | recreate via sync | yes |
| unsupported-host-features | Info | report kinds the host cannot accept | none (informational) | no |
| duplicate-instructions | Warn | no capability projected twice into same host | dedupe registry | partial |
| conflicting-agents-md | Error | no contradictory AGENTS.md rules at same scope | resolve doctrine conflict | no |
| multiple-memory-systems | Warn | not >1 memory system active in project | pick one | no |
| multiple-context-retrieval | Warn | not >1 context-retrieval system active | pick one | no |
| mcp-backend-health | Error | stdio bin on PATH / http reachable (cf. `doctor.rs:204-262`) | start/install backend | no |
| skillspector-scan-status | Error | every projected skill has terminal scan status | run `swe-seed scan` | no |
| provenance-completeness | Error | every active capability has source+hash+license | re-register source | no |

### Drift detection

- For each `Projection`, recompute the managed file's content hash and compare to the
  stored hash. Mismatch → `generated-host-files` Fail (drifted) with a diff hint.
- A managed file present on disk with no matching `Projection` → `stale-generated-files`.
- `--fix` regenerates drifted files (after confirming the on-disk change wasn't a
  deliberate user edit outside the managed marker — if outside marker, warn, don't clobber).

## CLI behavior, if applicable

As above. `--json` for machine-readable output (CI). `--host all` default.

## Generated files, if applicable

None (read-only) unless `--fix`, which delegates to adapters (0004) / sync.

## Rust module boundaries

`swe_seed::doctor` with `mod check` (trait + registry of checks), `mod drift`, `mod report`.
Checks consume registry + projections read-only; `--fix` calls `swe_seed::adapters`.

## Federation (optional, 0011)

Doctor is SWE_Seed's **local proof engine**. With `federation.emit_envelope = true`, a
doctor/verify run emits `ProofStarted` / `ProofCompleted` envelopes (`result = pass|fail`,
`proof_type = live|simulation`) into the SEA loop. This is purely additive output — doctor
runs identically and fully with federation off. Per the loop's promotion rule, only a
**live** passing proof may activate/promote a capability (0007/0011); a `simulation` proof
verifies but never activates.

## Security and provenance considerations

`provenance-completeness` and `skillspector-scan-status` are Error-severity gates: a
project failing them is governance-non-compliant. Doctor never weakens a gate to pass.

## Tests

- Each check has a passing + failing fixture.
- Drift: tamper a managed file → `Fail`; `--fix` restores; edit outside marker → `Warn`,
  no clobber.
- Exit code non-zero iff any `Fail`.
- `--json` schema stable.

## Decisions

- **Multiple memory/context-retrieval detection**: use a **known-signatures list** (file/
  dir markers per system), reported as a heuristic `Warn`. Cheap and honest; extend the
  signature list over time rather than attempting generic inference.
- **`--fix` never installs software**: it only regenerates/prunes SWE_Seed-owned files.
  MCP backend health is reported, never auto-remediated (no `apt`/`npm install` from
  doctor).

## Acceptance criteria

- [ ] All 14 required checks present with severity, status meaning, remediation, fixability.
- [ ] Drift detection via projection hashes implemented and tested.
- [ ] Status/result shape independently derived (not copied from `doctor.rs`).
