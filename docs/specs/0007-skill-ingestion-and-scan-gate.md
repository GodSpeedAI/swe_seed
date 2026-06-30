# 0007 — Skill Ingestion and Scan Gate

> **Reconciled by [0012](0012-existing-harness-reconciliation.md) (authoritative).** The
> normalized skill type is **SkillIR** (`SkillPack` → SkillIR); ingestion ends in
> `render-skills` projecting SkillIR to host `render_targets` (0004). SkillSpector is an
> external **EvalCheck**/gate (0013). Promotion follows the live-pass-only proof rule (0013).
> SkillIR shape is canonical in `.baml` ([0019](0019-baml-contracts-as-data.md)).

## Purpose

Define how SWE_Seed ingests external skills (e.g. `addyosmani/agent-skills`) through a
deterministic pipeline that scans them for risk before activation, with SkillSpector as the
first `SecurityGate`. Specifies the interface and flow only — no scanner is implemented yet.

## Non-goals

- Not implementing SkillSpector or any scanner in v0.1.
- Not bundling skills; SWE_Seed ingests them from sources.
- Not designing skill *authoring*; only ingestion + governance.

## Prior-art evidence (sources removed, unnamed)

| Prior-art (removed) | File/path | Lines or section | Pattern observed | SWE_Seed interpretation |
|---|---|---|---|---|
| gateway prior-art | `src/skills/{installer.rs,parser.rs,registry.rs,renderer.rs,watcher.rs}` | files | install → parse → register → render pipeline | SWE_Seed discover→fetch→scan→normalize→project→verify (scan added) |
| gateway prior-art | `src/capability/hash.rs` | file | Content hashing | Source hash in provenance |
| multi-host runtime prior-art | `(prior-art path redacted)` | lines | `SkillDefinition`, `SkillSource` union, `RuntimeSkillConfig` | `SkillPack` model + portable/full runtime (0003) |
| multi-host runtime prior-art | `(prior-art path redacted)` | dirs | builtin vs runtime skill loading paths | normalize step distinguishes portable vs host-runtime skills |

## SWE_Seed requirements

### Pipeline

```
discover → fetch/register → scan → normalize → project → verify
```

1. **discover**: enumerate skills in a `CapabilitySource` (e.g. each `SKILL.md` dir).
2. **fetch/register**: materialize into a content-addressed cache; create a draft
   `SkillPack` + `ProvenanceRecord` (source + `source_hash`). Status = `pending-scan`.
3. **scan**: run applicable `SecurityGate`s. SkillSpector first. Produce a `ScanResult`.
4. **normalize**: map to host-neutral `SkillPack` metadata; classify `portable|full`.
5. **project**: only if scan permits — host adapters (0004) generate host files.
6. **verify**: doctor (0008) confirms files present, hashes match, scan status recorded.

### Scanning rules

- **When required**: whenever the active `CapabilityProfile.require_scan = true`, or for any
  skill from an untrusted/unpinned source. `readonly`/`coding` default require scan;
  `dangerous` always requires scan + explicit approval.
- **Where results stored**: `.swe-seed/scans/<skill-id>/<scan-id>.json` plus a pointer in
  the `SkillPack` (`scan_ref`) and `ProvenanceRecord` (`scan_ref`).
- **Risk scores → activation**:
  - Severity mapping is normative: `low = score < 0.25`, `medium = 0.25 ≤ score < 0.50`,
    `high = 0.50 ≤ score < 0.75`, `critical = score ≥ 0.75`.
  - `low_threshold` defaults to `0.25`; `block_threshold` defaults to `0.75`.
  - `score < low_threshold` → auto-activate.
  - `low ≤ score < block_threshold` → activate only with recorded approval (exception).
  - `score ≥ block_threshold` → **blocked**; cannot project until remediated/excepted.
  - Any scanner `error`/timeout → treated as fail-closed under `dangerous`, warn otherwise.
- **Output formats**: scanners may emit SARIF, JSON, or Markdown. SWE_Seed stores the raw
  artifact, parses SARIF/JSON into a normalized `ScanResult{score, findings[], format}`,
  and renders Markdown for human reading. SARIF is the preferred machine format.
- **Failed scans block installation**: a blocking result prevents `project`; the skill
  stays `blocked` in the registry and no host files are generated.
- **Exceptions/approvals**: a human records an exception in
  `.swe-seed/scans/exceptions.toml` (`skill_id`, `scan_id`, `approver`, `reason`,
  `expires`). Exceptions are scoped to a specific scan hash — re-scan invalidates them.
- **Provenance + source hash**: recorded at fetch; activation requires both
  `source_hash` and a terminal `scan_status` (`pass|exception`).

### SecurityGate interface (conceptual)

```
trait SecurityGate {
    fn id(&self) -> &str;                       // e.g. "skillspector"
    fn applies_to(&self) -> &[CapabilityKind];  // [Skill] for v0.1
    fn scan(&self, pack: &SkillPackDraft) -> Result<ScanResult>;
}
struct ScanResult { score: f32, findings: Vec<Finding>, format: ScanFormat, raw_ref: PathBuf }
enum ScanStatus { Pending, Pass, Blocked, Exception, Error }
```

## CLI behavior, if applicable

`swe-seed skill add <source>`, `swe-seed scan <skill-id>`, `swe-seed scan --all`,
`swe-seed skill approve <skill-id> --reason "..."`, `swe-seed skill list --status blocked`.

## Generated files, if applicable

`.swe-seed/scans/<skill-id>/*.json|*.sarif|*.md`, `.swe-seed/scans/exceptions.toml`, plus
host skill files via adapters (0004) only after pass/exception.

## Rust module boundaries

`swe_seed::security` with `mod gate` (trait), `mod scan_result`, `mod skillspector`
(adapter shelling out to the external tool — no scanner logic reimplemented), `mod
exceptions`. Ingestion pipeline in `swe_seed::capability::skill::ingest`. **No EE-derived
scanning logic.**

## Security and provenance considerations

The scan gate is the primary supply-chain control for skills. Source hashing prevents
silent upstream changes; exceptions are hash-scoped and expiring; blocking is fail-closed
for dangerous profiles.

**Local-first authority + optional federation (0011).** The scan gate is the *local*
authority for activation and works fully standalone. With `federation.authority =
delegate|hybrid`, a passing scan may additionally require an external `AuthorityChecked`
before projection; an external `deny` blocks, `escalate` requires human approval. External
authority can only make activation **stricter**, never override a local block. The proof
rule unifies with the loop (0011): a capability is **activated/promoted only on a live
proof** (`proof_type=live`, `result=pass`) — never on a simulation.

## Tests

- Pipeline stages run in order; a blocked scan prevents projection.
- SARIF + JSON parsed into normalized `ScanResult`; Markdown rendered.
- Exception scoped to scan hash; re-scan invalidates it.
- Missing `source_hash` or `scan_status` blocks activation.

## Decisions

- **SkillSpector is an external tool**: shell out, parse SARIF (preferred) + exit code;
  never reimplement scanner logic. Until its exact CLI/output is verified, the adapter
  returns `Pending` (does not fake a pass). *Open, human action:* confirm SkillSpector's
  CLI contract before wiring M5.
- **Thresholds (v0.1 defaults)**: any `high`/`critical` finding → `block`; `medium` →
  `exception-required`; `low`/none → auto-pass. Fail-closed for the `dangerous` profile.
  Values tunable in profile config; revisit once real scan data exists.

## Acceptance criteria

- [ ] Pipeline discover→fetch→scan→normalize→project→verify specified.
- [ ] Blocking, exception, and provenance rules defined without implementing a scanner.
- [ ] SkillSpector modeled as an external `SecurityGate`, not reimplemented.
