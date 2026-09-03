# SWE_SEED Architecture

The authoritative architecture documentation for SWE_SEED is maintained in [docs/architecture.md](docs/architecture.md).

## Quick Summary of Architectural Views

- **Logical Architecture**: 3-layer downward governance stack (`SweSeed` -> `Harness` -> `Fabricator`).
- **Runtime Architecture**: Single-binary Rust CLI (`swe-seed`), file-first local persistence (`.agent-harness/`, `.agent-hooks/`, `.swe-seed/`), zero background daemons.
- **Dependency Architecture**: Cargo workspace (`crates/swe-seed` bin, `crates/swe-seed-core` lib) with strict downward dependency flow and zero LLM runtime dependencies.
- **Data Architecture**: File-first storage, append-only JSON traces, SHA-256 cryptographic hash-chained SQLite ledger (`ledger.db`).
- **Control Flow**: Pre-execution work contract binding (Route -> Context -> Artifacts -> Proof -> Trace) before code modification.
- **Security & Trust**: Fail-closed routing gates, automated secret redaction, SOPS-encrypted Ed25519 federation keys.

For complete diagrams, source trails, and operational details, see the canonical [Architecture Guide](docs/architecture.md).
