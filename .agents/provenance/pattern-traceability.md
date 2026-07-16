# Pattern Traceability Matrix

> **HUMAN-ONLY LEGAL RECORD — NOT A BUILD INPUT.** Names the (now-deleted) reference
> sources for clean-room audit. Implementing agents must not fetch or browse any project
> named here; build from `.agents/specs/` only.


Every observed pattern, its source, the SWE_Seed requirement it informs, and explicit
confirmation that no code was copied. EE-licensed mcp-gateway files are intentionally
absent (not inspected).

| Source repo | Source path | Pattern observed | SWE_Seed requirement | Code copied? | License concern? | Notes |
|---|---|---|---|---|---|---|
| mcp-gateway | `ARCHITECTURE.md` (System Overview) | Single `/mcp` endpoint front of many backends; meta-tools for discover-then-invoke; ~95% context savings | MCP gateway integration: one SWE_Seed endpoint → backend MCPs (spec 0006) | No | No (MIT, docs) | Concept only |
| mcp-gateway | `ARCHITECTURE.md` (Meta-Tools table) | `gateway_list_servers/list_tools/search_tools/invoke/get_stats` compact surface | Compact tool-discovery surface, base meta-tools (spec 0006) | No | No | Re-named for SWE_Seed |
| mcp-gateway | `src/capability/` (`definition/`, `loader.rs`, `parser.rs`, `hash.rs`, `validator/`, `schema_validator/`) | Declarative capability definitions, loader+parser, content hashing, schema validation | Capability registry + normalized metadata + source hash (spec 0003) | No | No | `hash.rs` informs provenance hashing concept |
| mcp-gateway | `src/discovery/` (`config_scanner.rs`, `process_scanner.rs`) | Scan existing client configs and running processes to discover backends | Doctor + import: detect existing MCP/host config (specs 0006, 0008) | No | No | Concept only |
| mcp-gateway | `src/config_reload/mod.rs`, `src/capability/watcher.rs` | Hot reload / file-watch of config & capabilities | Hot-reload concept for registry (spec 0003, deferred past v0.1) | No | No | v0.1 = restart-to-apply |
| mcp-gateway | `src/projection/` (`engine.rs`, `mode.rs`, `role.rs`, `schema.rs`) | Projection engine with modes/roles deciding what surface a client sees | `Projection` concept + capability profiles `readonly/coding/dangerous` (specs 0003, 0006) | No | No | Profiles re-derived |
| mcp-gateway | `src/commands/doctor.rs:21-80` | `CheckStatus{Pass,Fail,Warn}` enum, `CheckResult` struct w/ label/detail/hint, `run_doctor_command(fix, config_path)` | DoctorCheck model + `swe-seed doctor` (spec 0008) | No | No (MIT) | Status/severity shape re-derived as Rust |
| mcp-gateway | `src/commands/doctor.rs:137-289` | Checks: config present, port free, backend env vars, HTTP/stdio backend reachability, AI-client points-to-gateway | Doctor check catalog (spec 0008) | No | No | Adapted + extended to SWE_Seed checks |
| mcp-gateway | `src/commands/config_export/` | Export gateway config to client-specific formats; `watch.rs` keeps it synced | Host adapter generated-file projection + drift detection (specs 0004, 0008) | No | No | Concept only |
| mcp-gateway | `src/skills/` (`installer.rs`, `parser.rs`, `registry.rs`, `renderer.rs`, `watcher.rs`) | Skill install→parse→register→render pipeline | Skill ingestion flow discover→fetch→scan→normalize→project→verify (spec 0007) | No | No | Scan stage is SWE_Seed-added |
| mcp-gateway | `LICENSE-EE.md` | SPDX-tagged EE files for security (firewall, tool_integrity, message_signing…) | Mark these out-of-bounds; derive security from public specs (spec 0009) | No | **Yes — EE PolyForm Noncommercial** | Files NOT opened |
| oh-my-openagent | `packages/claude-code-compat-core/src/features/` (`claude-code-agent-loader`, `command-loader`, `mcp-loader`, `plugin-loader`) | Per-format compatibility loaders translating one tool's formats | Host adapter contract; compatibility loaders per host (specs 0004, 0002) | No | **Yes — Sustainable Use (noncommercial)** | Architecture observation only |
| oh-my-openagent | `packages/omo-codex/` vs `packages/omo-opencode/` | Separate host packages for Codex vs OpenCode runtimes | Host adapter separation; per-host modules (spec 0004) | No | Yes | Names not reused |
| oh-my-openagent | `packages/skills-loader-core/src/{hooks,features,types.ts}` | Skill loader with hooks (`auto-slash-command`), runtime vs builtin skill features, typed `SkillDefinition`/`SkillSource`/`SkillsConfig` | SkillPack/CapabilitySource data model; runtime-vs-portable distinction (specs 0003, 0007) | No | Yes | Types re-derived independently |
| oh-my-openagent | `packages/rules-engine/src/` (`agents-md.ts`, `engine`, `finder.ts`, `frontmatter-corpus`) | Rules driven by AGENTS.md + frontmatter, distance/matching engine | `Rule` concept; AGENTS.md as doctrine (specs 0002, 0003) | No | Yes | Concept only |
| oh-my-openagent | `packages/agents-md-core/src/` (`finder.ts`, `injector.ts`, `injection-cache.ts`, `formatter.ts`) | Find AGENTS.md hierarchy, inject into context, cache injection | Doctrine projection + ContextBuild hook (specs 0002, 0005) | No | Yes | Concept only |
| oh-my-openagent | `packages/mcp-client-core/src/` (`config-dir.ts`, `plugin-identity.ts`, `skill-mcp-manager`) | MCP client config dir handling, plugin identity, skill-scoped MCP management | Host MCP config generation + provenance identity (specs 0004, 0006) | No | Yes | Concept only |
| oh-my-openagent | `LICENSE.md` Sustainable Use License | Noncommercial source-available license | Clean-room: no code reuse; legal review (spec 0009) | No | **Yes** | Drives conservative posture |
