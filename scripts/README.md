# Scripts

## Overview

Operational bash entry points for the SWE_SEED repository, dispatched through recipes in the root [justfile](../justfile). The directory holds eleven standalone scripts covering four concerns: environment bootstrap and toolchain checks (`bootstrap.sh`, `doctor.sh`), CI modes (`ci.sh`), SOPS-based secrets management (`secrets-decrypt.sh`, `secrets-edit.sh`, `secrets-encrypt.sh`, `secrets-rotate-key.sh`), and the optional learning-store SQLite mirror built from harness trace records (`sync-learning-store.sh`, `query-learning-store.sh`, `eval-learning-retrieval.sh`, `plan-learning-store.sh`). The learning-store scripts embed Python through `python - ... <<'PY'` heredocs and use the standard-library `sqlite3` module; this directory contains no standalone Python files and no subdirectories.

## Interface

- `bootstrap.sh` (no arguments): installs toolchain dependencies through mise, pnpm, and uv when each tool is present on PATH. Entry point for `just bootstrap`. `[bootstrap.sh:mise install]` `[bootstrap.sh:pnpm install --frozen-lockfile]` `[bootstrap.sh:uv sync]`
- `ci.sh [all|format|lint|test]` (default `all`): dispatches the prettier format check, the ruff Python lint, and the Rust harness and test pipeline; unknown modes print usage and exit 2. Entry point for `just ci`, `just format`, `just lint`, `just test`. `[ci.sh:case "$mode"]` `[ci.sh:run_format]` `[ci.sh:run_lint]` `[ci.sh:run_test]`
- `doctor.sh` (no arguments): verifies required commands (git, just, cargo) and warns on recommended commands (age, direnv, devbox, mise, pnpm, sops). Entry point for `just doctor`, also run by `ci.sh` in `all` mode. `[doctor.sh:required_commands]` `[doctor.sh:recommended_commands]`
- `sync-learning-store.sh [db_path]` (default `.agent-harness/traces/learning-store.sqlite3`): creates or incrementally updates the SQLite learning-store mirror from `.agent-harness/traces/records/*.json`. Entry point for `just harness-sync-learning-store`. `[sync-learning-store.sh:CREATE TABLE IF NOT EXISTS trace_records]` `[sync-learning-store.sh:INSERT OR REPLACE INTO trace_records]`
- `query-learning-store.sh [mode] [limit] [status] [job_type] [db_path]` (defaults `summaries`, `10`, `any`, `any`): read-only JSON queries joining `trace_records` and `learning_reviews`; modes are `summaries`, `risks`, `memory-updates`, `skill-updates`, `harness-updates`. Entry point for `just harness-query-learning-store`. `[query-learning-store.sh:valid_modes]` `[query-learning-store.sh:base_query]`
- `eval-learning-retrieval.sh [db_path]`: prints a JSON retrieval-readiness verdict (`defer` or `benchmark`) derived from corpus counts and thresholds. Entry point for `just harness-eval-learning-retrieval`. `[eval-learning-retrieval.sh:criteria]` `[eval-learning-retrieval.sh:thresholds_met]`
- `plan-learning-store.sh [rusql|ruvector|both]` (default `both`): prints the phased adoption plan for the optional SQLite mirror (rusql) and semantic retrieval layer (ruvector). Entry point for `just harness-plan-learning-store`. `[plan-learning-store.sh:print_rusql_plan]` `[plan-learning-store.sh:print_ruvector_plan]` `[plan-learning-store.sh:print_rollout_plan]`
- `secrets-decrypt.sh` (no arguments): decrypts every `secrets/*.{yaml,yml,json,env}` into a sibling `<file>.dec`. Entry point for `just secrets-decrypt`. `[secrets-decrypt.sh:sops --decrypt]`
- `secrets-encrypt.sh` (no arguments): encrypts every `secrets/*.plain.*` into the name with `.plain.` stripped. Entry point for `just secrets-encrypt`. `[secrets-encrypt.sh:sops --encrypt]`
- `secrets-edit.sh <file>`: opens one secret file in the sops editor. Entry point for `just secrets-edit`. `[secrets-edit.sh:sops "$file"]`
- `secrets-rotate-key.sh` (no arguments): runs `sops updatekeys --yes` over every `secrets/*.{yaml,yml,json,env}`. Entry point for `just secrets-rotate-key`. `[secrets-rotate-key.sh:sops updatekeys --yes]`

## Invariants

- Every script in this directory runs under `set -euo pipefail`; unset variables, command failures, and pipeline failures abort the script. `[bootstrap.sh:set -euo pipefail]` (identical guard in all 11 scripts)
- Filesystem artifacts under `.agent-harness/` remain the source of truth; the SQLite store is a derived, rebuildable mirror. `[sync-learning-store.sh:source_of_truth]` `[query-learning-store.sh:source_of_truth]`
- Incremental sync processes only trace files whose mtime is newer than the stored `MAX(synced_at)`; all other files are skipped. `[sync-learning-store.sh:SELECT MAX(synced_at) FROM trace_records]` `[sync-learning-store.sh:file_mtime <= last_sync]`
- A `trace distill` failure or timeout (60 seconds) skips that trace with a stderr warning instead of aborting the sync. `[sync-learning-store.sh:subprocess.check_output]` `[sync-learning-store.sh:timeout=60]` `[sync-learning-store.sh:except (subprocess.CalledProcessError, subprocess.TimeoutExpired)]`
- `trace_id` is the primary key of both mirror tables, and `learning_reviews.trace_id` declares a foreign key into `trace_records(trace_id)`. `[sync-learning-store.sh:FOREIGN KEY(trace_id) REFERENCES trace_records(trace_id)]`
- The retrieval-readiness verdict is `defer` unless the corpus holds at least 25 learning reviews, 15 verified reviews, and 3 distinct job types; otherwise it can reach `benchmark`. `[eval-learning-retrieval.sh:criteria]` `[eval-learning-retrieval.sh:thresholds_met]`
- `query-learning-store.sh` accepts only non-negative integer limits and modes in the fixed set. `[query-learning-store.sh:^[0-9]+$]` `[query-learning-store.sh:valid_modes]`
- Encryption is refused while `.sops.yaml` still contains the placeholder age key. `[secrets-encrypt.sh:age1replacewith]`
- Secrets scripts exit 1 when `sops` is not installed. `[secrets-decrypt.sh:command -v sops]` (same guard in `secrets-edit.sh`, `secrets-encrypt.sh`, `secrets-rotate-key.sh`)
- `ci.sh` skips the format check when pnpm or `package.json` is absent and the lint when uv or `pyproject.toml` is absent, with a notice instead of a failure. `[ci.sh:command -v pnpm]` `[ci.sh:command -v uv]`
- `doctor.sh` exits 1 when any required command is missing; missing recommended commands only warn. `[doctor.sh:required_commands]` `[doctor.sh:recommended_commands]`
- The printed learning-store plan requires keeping the filesystem-first baseline intact, making database indexing optional, and adopting rusql before ruvector. `[plan-learning-store.sh:print_common_header]`

## Side Effects

- `bootstrap.sh` runs package managers that download and install dependencies when the respective tool exists. `[bootstrap.sh:mise install]` `[bootstrap.sh:pnpm install --frozen-lockfile]` `[bootstrap.sh:uv sync]`
- `ci.sh` writes build artifacts to `target/` through cargo and executes the `swe-seed` harness binary and the `cli_golden` integration test with `SWE_SEED_BIN=target/release/swe-seed`. `[ci.sh:cargo run -q -p swe-seed -- harness]` `[ci.sh:cargo test -q]` `[ci.sh:cargo build --release -q -p swe-seed]` `[ci.sh:SWE_SEED_BIN=target/release/swe-seed cargo test -q -p swe-seed --test cli_golden]`
- `sync-learning-store.sh` creates the database parent directory, creates the `trace_records` and `learning_reviews` tables plus four indexes, enables SQLite WAL journaling, and upserts rows with `INSERT OR REPLACE`. `[sync-learning-store.sh:db_path.parent.mkdir(parents=True, exist_ok=True)]` `[sync-learning-store.sh:PRAGMA journal_mode=WAL]` `[sync-learning-store.sh:CREATE INDEX IF NOT EXISTS idx_traces_job_type]` `[sync-learning-store.sh:INSERT OR REPLACE INTO learning_reviews]`
- `sync-learning-store.sh` spawns `cargo run -q -p swe-seed -- trace distill <trace>` once per new or updated trace record. `[sync-learning-store.sh:subprocess.check_output]`
- `eval-learning-retrieval.sh` and `query-learning-store.sh` auto-run `bash scripts/sync-learning-store.sh "$db_path"` when the database file does not exist, creating the store on first use. `[eval-learning-retrieval.sh:bash scripts/sync-learning-store.sh "$db_path"]` `[query-learning-store.sh:bash scripts/sync-learning-store.sh "$db_path"]`
- `secrets-decrypt.sh` writes decrypted plaintext to `<file>.dec` beside each source file and warns on stderr to clean the `.dec` files up. `[secrets-decrypt.sh:output="${file}.dec"]` `[secrets-decrypt.sh:Wrote $output]`
- `secrets-encrypt.sh` writes encrypted outputs named by stripping `.plain.` from the input name. `[secrets-encrypt.sh:output="${file/.plain./.}"]`
- `secrets-rotate-key.sh` rewrites key metadata in place inside each secret file. `[secrets-rotate-key.sh:sops updatekeys --yes]`
- `secrets-edit.sh` mutates the target file only through the sops editing session. `[secrets-edit.sh:sops "$file"]`
- `doctor.sh` and `plan-learning-store.sh` perform no disk, network, or state mutations; they write to stdout/stderr and set an exit status only.

## Verification

- Run: `just ci`
