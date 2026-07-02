# Memory System

Durable memory lives under `.agent-harness/memory/`.

Memory files should be concise and cite evidence when they record material lessons. They must not store secrets, raw chat dumps, or unbounded logs.

Learning-oriented memory candidates should carry provenance from the originating trace or proof surface. A future implementation may index those packets in a lightweight local database, but database-backed recall is optional. A conforming baseline remains filesystem-first.

If an optional local database layer is added, structured mirroring should come first. In this repository that means an exact or metadata-oriented store before semantic vector retrieval. Vector retrieval should be adopted only after the distilled learning corpus is large enough that structured recall no longer covers the recovery and lookup failures being observed.

When the optional structured mirror exists, it should expose a read-only query surface for operators and agents. That query surface should support exact and metadata-oriented recovery over distilled summaries, unresolved risks, and candidate memory, skill, or harness updates so the structured mirror is directly useful before any vector layer is considered.

Required memory files are listed in `.agent-harness/config.yaml`.

Local agent work memory lives under `.agents/`. It is ignored by git and supports handoff, not canonical harness behavior. Agents use `.agents/CURRENT_STATUS.md` for the latest outcome and next action, `.agents/OPEN_QUESTIONS.md` for human decisions that cannot be answered from the repository, `.agents/DEBT.md` for out-of-scope issues noticed during work, and `.agents/lessons/` for generalizable lessons that may later be promoted into stable project instructions or harness memory.
