# 0004 — Host Adapter Contract

> **Reconciled by [0012](0012-existing-harness-reconciliation.md) (authoritative).** The unit
> of routing is the **RouteCard** (0014); skills project as **SkillIR** via `render-skills`
> (0007). "Projection" here = host-file rendering (a legitimate, distinct concept; not the
> routing sense). Adapters project the assembled **SeedPackageManifest** (0018).

## Purpose

Define how SWE_Seed projects the host-neutral registry (0003) into each coding-agent
tool's native config, and how adapters degrade honestly when a host lacks a capability
type. Adapters are the only component allowed to write host files.

## Non-goals

- Adapters do not normalize or scan capabilities (that is 0003/0007); they only project.
- Not a claim that all hosts share one lifecycle — they do not (0005).
- v0.1 ships Claude + Codex + OpenCode adapters; others are stubs that report support
  honestly.

## Prior-art evidence (sources removed, unnamed)

| Prior-art (removed) | File/path | Lines or section | Pattern observed | SWE_Seed interpretation |
|---|---|---|---|---|
| multi-host runtime prior-art | `(prior-art path redacted)` | `(prior-art loader)`, `command-loader`, `mcp-loader`, `plugin-loader` | One loader per format/category | One adapter per host; each declares which kinds it handles |
| multi-host runtime prior-art | `(prior-art path redacted)/` vs `(prior-art path redacted)/` | dirs | Separate runtime packages per host | Host adapters are separated, not a shared abstraction pretending uniformity |
| multi-host runtime prior-art | `(prior-art path redacted)`, `skill-mcp-manager` | files | Per-client config-dir + scoped MCP management | Adapter owns host config-dir paths + scoped MCP files |
| gateway prior-art | `src/commands/config_export/mod.rs`, `watch.rs` | files | Export gateway config to client formats; watch to resync | Adapter `generate()` + drift resync via doctor |

## SWE_Seed requirements

`HostAdapter` trait (conceptual):

```
trait HostAdapter {
    fn host_id(&self) -> &str;
    fn supported_kinds(&self) -> &[CapabilityKind];     // honest capability matrix
    fn unsupported_kinds(&self) -> &[CapabilityKind];
    fn plan(&self, reg: &Registry, profile: &Profile) -> ProjectionPlan; // dry-run, no writes
    fn generate(&self, plan: &ProjectionPlan) -> Result<Vec<Projection>>;
    fn rollback(&self, projections: &[Projection]) -> Result<()>;
    fn partial_support(&self) -> Vec<PartialSupport>;   // what it cannot do + why
}
```

Rules every adapter obeys:
- **Own-only writes**: write only files/regions the adapter declares as managed. Never
  touch user-authored host config outside managed markers.
- **Managed marker**: each generated file carries `# managed-by: swe-seed <hash>` header;
  for files SWE_Seed must merge into, use begin/end sentinel comments around the managed
  region.
- **Plan before write**: `plan()` is pure and used by `doctor` and `--dry-run`.
- **Honest degradation**: unsupported kinds produce a recorded `PartialSupport` warning,
  never a silent drop.
- **Rollback**: every `Projection` is reversible; rollback restores prior content or
  removes the file if SWE_Seed created it.

### Per-host adapter matrix

| Host | Supported kinds | Unsupported kinds | Files generated | Files to avoid | Symlink vs generated | Partial-support reporting |
|---|---|---|---|---|---|---|
| **Claude / Claude Code** | skill, command, agent, hook, mcp, rule | — (broad) | `.claude/skills/*`, `.claude/commands/*`, `.claude/agents/*`, `.claude/settings.json` (hooks/mcp region), `CLAUDE.md`/`AGENTS.md` refs | user's hand-written `CLAUDE.md` body outside managed region | generated files (skills/commands); merge region for settings | full support; warn only on conflicting hooks |
| **Codex CLI / Codex CI** | mcp, rule, command(limited) | hook(lifecycle), agent | `~/.codex/config.toml` (mcp region), `AGENTS.md` | other config.toml sections | generated/merged region | report `hook`,`agent` unsupported; map rules→AGENTS.md |
| **OpenCode** | skill, command, agent, mcp, rule, hook(subset) | hooks not in subset | OpenCode config dir (per `(prior-art)` config-dir concept), `AGENTS.md` | user runtime config | generated | report unsupported hook events |
| **VS Code / GitHub Copilot** | mcp, rule(instructions) | hook, agent, slash command | `.vscode/mcp.json`, `.github/copilot-instructions.md` | workspace settings unrelated keys | generated | report skills/agents/hooks unsupported |
| **Antigravity / Gemini-style** | mcp, rule | hook, agent, skill(varies) | tool-specific MCP config + instructions file | unrelated config | generated | **Open question**: exact file paths/format unverified → mark unsupported until confirmed |
| **GitHub Copilot CLI** | mcp(if supported), rule | hook, agent, skill, command | CLI config + instructions | unrelated config | generated | report most kinds unsupported; rules only |
| **Generic CI** | rule, mcp(headless), scan-gate enforcement | hook(interactive), agent, command, skill(interactive) | CI workflow snippet, `AGENTS.md`, scan-gate step | existing CI steps | generated snippet, merge-marked | report interactive kinds unsupported |

### Drift avoidance

- Each `Projection` stores `content_hash`. Doctor (0008) recomputes and flags drift.
- Adapters are deterministic: same registry+profile → byte-identical output.
- Stale files (in projection record but no longer in registry) are flagged and
  removable via `swe-seed sync --prune`.

### Rollback

- `swe-seed sync` snapshots prior managed regions before writing.
- `swe-seed rollback [--host H]` restores the last snapshot; created-from-scratch files
  are deleted, merged regions are reverted to the captured pre-state.

## CLI behavior, if applicable

`swe-seed sync [--host H] [--dry-run] [--prune]`, `swe-seed rollback [--host H]`,
`swe-seed hosts` (prints support matrix).

## Generated files, if applicable

See per-host matrix. All carry managed markers + hash.

## Rust module boundaries

`swe_seed::adapters` with `mod claude; mod codex; mod opencode; mod vscode_copilot;
mod antigravity; mod copilot_cli; mod ci;`. Shared `projection`, `marker`, `plan` helpers.

## Security and provenance considerations

Adapters refuse to project capabilities lacking provenance/scan status when the active
profile requires it. Generated MCP config inherits approval flags (0006).

## Tests

- Determinism test per adapter (golden-file, hash-stable).
- Honest-degradation test: unsupported kind → `PartialSupport`, not silent.
- Rollback test: generate → rollback → tree byte-identical to pre-state.
- Own-only test: adapter never modifies bytes outside its managed region.

## Decisions

- **Antigravity / Gemini-style & Copilot CLI**: ship as **honest stubs** that report their
  kinds `unsupported` until a human verifies real config paths/formats. Wrong generated
  config is worse than none — never guess a host's file format.
- **Codex hooks**: assume **none**; realize lifecycle as static AGENTS.md content. Revisit
  only if Codex adds a hook system.

## Acceptance criteria

- [ ] Every target host has a declared supported/unsupported matrix.
- [ ] No adapter claims uniform lifecycle support.
- [ ] All generated files reversible via rollback with a passing round-trip test.
