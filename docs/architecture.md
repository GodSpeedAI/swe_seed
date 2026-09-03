# Architecture Guide

This document presents the canonical architectural model for SWE_SEED. It covers the system across six distinct architectural views: logical, runtime, dependency, data, control flow, and security/trust boundaries.

---

## 1. Logical Architecture

The system is organized into three strictly governed layers, where authority and ownership flow downward only.

```mermaid
graph TD
    subgraph SweSeedLayer["SweSeed Layer (Outer Governance)"]
        SeedManifest["SeedPackageManifest (.swe-seed/manifest.toml)"]
        BoundaryVal["Boundary Validator (ValidateLayerBoundaries)"]
        ProvVerify["Provenance & License Verifier (SHA-256)"]
        RegenPlan["Regeneration Planner (Idempotent Plans)"]
    end

    subgraph HarnessLayer["Harness Layer (Execution & Proof)"]
        Router["Semantic Router (11 RouteCards)"]
        ContextPlane["Context Budget & Packing"]
        TraceEngine["Trace Engine & Cryptographic Ledger"]
        EvalEngine["Deterministic Eval & Proof Runner"]
        HostAdapters["Host Projection Adapters (6 Hosts)"]
        HookEngine["Normalized Hook Runtime (.agent-hooks/)"]
        LearnLoop["Reflection & Learning Loop"]
    end

    subgraph FabricatorLayer["Fabricator Layer (Inner Prototyping)"]
        ProductSeed["ProductSeed Intake"]
        SemanticChain["Semantic Spec Chain (10-Node EARS/Gherkin/ADR)"]
        PrototypeHandoff["Prototype Handoff & EvalSpec"]
    end

    SweSeedLayer -->|governs & validates| HarnessLayer
    HarnessLayer -->|governs & evaluates| FabricatorLayer
```

### What to Notice in this Diagram
- **Downward Governance**: SweSeed governs Harness; Harness governs Fabricator. An inner layer cannot govern an outer layer concern.
- **Clear Separation**: Outer layers handle governance and cross-host integration; middle layers handle routing, proof, and execution integrity; inner layers handle product-to-prototype synthesis.

### What This Diagram Omits
This diagram omits physical file paths, runtime processes, and external network services (which are covered in subsequent views).

---

## 2. Runtime Architecture

SWE_SEED is architected as a **single-binary, file-first, local-first system**.

```mermaid
graph LR
    User["CLI / Developer / CI"] -->|Invokes| Bin["swe-seed binary\n(Rust target/release/swe-seed)"]

    subgraph LocalFilesystem["Project Local Filesystem"]
        HarnessDir[".agent-harness/\n(routes, memory, baml, traces)"]
        HooksDir[".agent-hooks/\n(events.jsonl, hooks.db)"]
        SweSeedDir[".swe-seed/\n(manifest.toml, provenance)"]
        HostFiles["Host Configurations\n(.claude/, .github/, CODEX.md)"]
    end

    Bin -->|Reads / Writes| HarnessDir
    Bin -->|Appends Events| HooksDir
    Bin -->|Validates| SweSeedDir
    Bin -->|Projects / Audits| HostFiles
```

### Key Runtime Characteristics
- **No Background Daemon**: SWE_SEED does not require a background daemon, long-running service, or local server for core workflows.
- **Fast Execution**: Written in Rust, commands execute in milliseconds, allowing integration into pre-commit hooks and interactive shell sessions.
- **SQLite Ledgers**: High-throughput event indexing and cryptographic hash validation utilize embedded SQLite 3 databases (`ledger.db`, `hooks.db`) without external database servers.

---

## 3. Dependency Architecture

### Crate Structure
The codebase is structured as a Cargo workspace with two members:
- **`swe-seed` (`crates/swe-seed`)**: The binary CLI crate. It owns argument parsing (`clap`), CLI dispatch, terminal formatting, and top-level exit code resolution.
- **`swe-seed-core` (`crates/swe-seed-core`)**: The core library crate. It owns all domain logic, BAML parsing, cryptographic verification, routing algorithms, host adapters, and trace management.

```mermaid
graph TD
    CLI["swe-seed (bin crate)"] --> Core["swe-seed-core (lib crate)"]

    subgraph ExternalDeps["Major External Dependencies"]
        Clap["clap (CLI parsing)"]
        Serde["serde / serde_json / toml"]
        Rusqlite["rusqlite (bundled SQLite)"]
        Ed25519["ed25519-dalek (Cryptographic signing)"]
        Sha2["sha2 (Hash chaining)"]
        Regex["regex (Pattern parsing)"]
    end

    CLI --> Clap
    Core --> Serde
    Core --> Rusqlite
    Core --> Ed25519
    Core --> Sha2
    Core --> Regex
```

### Invariants:
1. `swe-seed-core` contains zero CLI presentation logic; it is fully testable as a standalone library.
2. No LLM runtime libraries exist in either crate. All contract interactions occur through deserialization of BAML schemas (`contracts-as-data`).

---

## 4. Data Architecture

SWE_SEED treats repository files as the canonical source of truth:

```mermaid
flowchart TD
    subgraph Inputs["Canonical Inputs"]
        Specs["Markdown Specs\n(docs/specs/)"]
        BAML["BAML Schemas\n(.agent-harness/baml/baml_src/)"]
        RouteCards["Route Cards\n(.agent-harness/routes/*.json)"]
    end

    subgraph RuntimeState["Runtime & Generated State"]
        Traces["Trace Records\n(.agent-harness/traces/records/*.json)"]
        Ledger["Trace Ledger\n(.agent-harness/traces/ledger.db)"]
        Projections["Projected Configs\n(Managed comment blocks)"]
    end

    subgraph Verification["Verification Output"]
        EvalOut["Eval Results (EvalResult.json)"]
        GateOut["Routing Gate (Allow / Block)"]
    end

    Inputs -->|Evaluated by swe-seed| RuntimeState
    RuntimeState -->|Verified by swe-seed| Verification
```

### Data Lifecycles:
- **Canonical Schemas**: Immutable during execution; updated only through RFCs and versioned Git commits.
- **Trace Records**: Append-only. Every transition produces a new immutable record or state append.
- **Hash Chains**: Each event in `ledger.db` contains `sha256(previous_hash + current_event_payload)`. Altering any historical event invalidates the ledger.
- **Projected Blocks**: Re-derived idempotently via `swe-seed sync`.

---

## 5. Control Flow: The End-to-End Task Lifecycle

The following sequence illustrates the control flow when executing a task under SWE_SEED governance:

```mermaid
sequenceDiagram
    autonumber
    actor User as Engineer / Agent
    participant CLI as swe-seed CLI
    participant Router as Semantic Router
    participant Ledger as Trace Ledger
    participant Agent as Host Coding Agent
    participant Hooks as Hook Runtime
    participant CI as Proof / CI Engine
    participant Gate as Routing Gate

    User->>CLI: swe-seed route "task description" --record
    CLI->>Router: Match task against RouteCards
    Router-->>CLI: Selected RouteCard (e.g. bugfix)
    CLI->>Ledger: Record RouteSelected genesis event (SHA-256)
    CLI-->>User: Emit Work Contract JSON

    User->>CLI: swe-seed context-plan "task description"
    CLI-->>User: Emit Bounded ContextPack

    User->>Agent: Begin work loop using Bounded Context
    loop Execution Iterations
        Agent->>Hooks: Lifecycle events (PreToolUse, PostToolUse)
        Hooks->>Hooks: Redact secrets and tokens
        Hooks->>Ledger: Append verified event
    end

    Agent->>CI: Run pre-bound proof (just ci)
    CI-->>Agent: Exit code 0 (All tests passed)

    User->>CLI: swe-seed trace finish <id> --claim "Fix verified" --command "just ci"
    CLI->>Ledger: Append ProofRecord and close trace

    User->>Gate: swe-seed gate <id> --verify
    Gate->>Ledger: Verify genesis + cryptographic chain
    Ledger-->>Gate: Chain unbroken & verified
    Gate-->>User: Exit code 0 (Permitted to merge)
```

---

## 6. Trust and Security Boundaries

SWE_SEED operates under explicit security boundaries:

```mermaid
graph TD
    subgraph UntrustedZone["Untrusted / Partially Trusted Zone"]
        Agent["Autonomous Agent Generations\n(Unverified code, LLM summaries)"]
        ExternalSkills["External Skills / Remote MCPs"]
    end

    subgraph SecurityPerimeter["SWE_SEED Security Perimeter"]
        HookRedact["Secret & Token Redaction Engine"]
        ProvGate["SHA-256 Provenance & Scan Gate"]
        MCPPolicy["MCPGate Authorization & Permission Policy"]
    end

    subgraph TrustedZone["Protected Repository State"]
        RepoSource["Committed Source Tree"]
        CryptLedger["Cryptographic Trace Ledger"]
        ProtectedKeys["SOPS Encrypted Private Keys"]
    end

    Agent -->|Tool Invocations| HookRedact
    ExternalSkills -->|Ingestion| ProvGate
    Agent -->|MCP Calls| MCPPolicy

    HookRedact --> CryptLedger
    ProvGate --> RepoSource
    MCPPolicy --> TrustedZone
```

### Security Principles:
1. **No Secrets in State**: Hook and trace runtimes scrub API keys, bearer tokens, and credentials before writing records to disk.
2. **Fail-Closed Gates**: If a trace ledger is missing, corrupted, or lacks a routing genesis, `swe-seed gate` fails closed with an exit code of 1, blocking merges.
3. **Encrypted Keys at Rest**: Ed25519 federation keys in `.swe-seed/federation/keys/` are encrypted using SOPS and age.
4. **Clean-Room Boundaries**: External skills must pass structural scan gates before registration into the manifest.

---

## Source Trail

- `crates/swe-seed/src/main.rs`: Entry point and process exit code mapping.
- `crates/swe-seed/src/cli.rs`: Top-level CLI command definitions and dispatching.
- `crates/swe-seed-core/src/route/mod.rs`: Semantic router and work contract generation.
- `crates/swe-seed-core/src/trace_ledger.rs`: SHA-256 hash chaining and SQLite ledger persistence.
- `crates/swe-seed-core/src/routing_gate.rs`: `route_gate` enforcement logic.
- `crates/swe-seed-core/src/adapters/`: Multi-host projection adapters and marker block engine.
- `crates/swe-seed-core/src/hooks/runtime.rs`: Hook event interception and secret redaction.
- `crates/swe-seed-core/src/gateway/serve.rs`: MCPGate JSON-RPC 2.0 proxy and policy engine.
- `.agent-harness/baml/baml_src/`: Canonical schema definitions (`swe_seed.baml`, `harness.baml`, `fabricator.baml`).
