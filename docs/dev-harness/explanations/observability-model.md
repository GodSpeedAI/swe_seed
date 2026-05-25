# Observability Model

## Why This Design Exists

The dev harness needs observability for the same reason it needs `just ci`: contributors need a stable way to understand what happened, not only a command that failed. Setup scripts, CI checks, secrets workflows, and hook adapters all produce behavior that must be inspectable after the fact.

The chosen design is file-first, language-agnostic, and upgradeable. It keeps append-only JSONL events as the durable ledger, stores large payloads and stdout or stderr in separate artifact files, and adds indexes only when lookup pressure justifies them.

This model also keeps the dev harness and the agent harness aligned without coupling them. The dev harness can capture hook and command behavior from any agent or adapter, while the agent harness remains free to manage routing, skills, traces, and learning in its own structures.

## Tradeoff

The file-first design gives up some early query convenience in exchange for portability, replayability, and recovery. JSONL logs are slower than a purpose-built index for large filtered queries, but they are easier to inspect, diff, sync, back up, redact, replay, and rebuild from when adapters or indexes change.

That tradeoff is intentional. A rusql index, FTS layer, or vector store is useful only after the filesystem ledger stops being enough for the real recovery work the harness performs.

## What Should Not Change Casually

- Do not move the durable source of truth from append-only files into a database.
- Do not require shell, Python, Rust, or TypeScript scripts to import a logging SDK just to participate.
- Do not inline large payloads into every event when artifact references are sufficient.
- Do not add vector search before exact and structured lookup are measurably inadequate.
- Do not bind the dev harness observability layer to one agent implementation or the agent harness Skill IR.

## Upgrade Path

Use the smallest level that serves the recovery task:

- Level 0: JSONL files only.
- Level 1: rusql index for structured queries.
- Level 2: rusql FTS or Tantivy-style text search.
- Level 3: vector retrieval for semantic similarity across failures, prompts, and summaries.

The upgrade rule is simple: add the next layer only when the current layer no longer supports the outcome the harness is supposed to produce.
