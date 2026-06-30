set dotenv-load := true

# `swe-seed` binary, run through cargo (debug). Use `cargo build --release` once
# for the optimized binary if a recipe is hot.
swe := "cargo run -q -p swe-seed --"

default:
    @just --list

bootstrap:
    @bash scripts/bootstrap.sh

doctor:
    @bash scripts/doctor.sh

format:
    @bash scripts/ci.sh format

lint:
    @bash scripts/ci.sh lint

test:
    @bash scripts/ci.sh test

ci:
    @bash scripts/ci.sh

# Phase 10 final parity gate: release build + golden CLI parity (beyond `route`).
parity:
    @cargo build --release -p swe-seed
    @SWE_SEED_BIN=target/release/swe-seed cargo test -p swe-seed --test cli_golden

# ------------------------------ harness (Rust) ------------------------------

harness-validate:
    @{{swe}} harness

harness-doctor:
    @{{swe}} doctor

harness-render-skills:
    @{{swe}} render-skills

harness-route task:
    @{{swe}} route "{{task}}"

harness-route-record task:
    @{{swe}} route "{{task}}" --record

harness-context-plan task:
    @{{swe}} context-plan "{{task}}"

harness-trace-start task:
    @{{swe}} trace start "{{task}}"

harness-trace-append trace note:
    @{{swe}} trace append "{{trace}}" "{{note}}"

harness-trace-distill trace:
    @{{swe}} trace distill "{{trace}}"

harness-trace-finish trace claim:
    @{{swe}} trace finish "{{trace}}" --claim "{{claim}}"

harness-eval-run spec output="":
    @if [ -n "{{output}}" ]; then {{swe}} eval run --spec "{{spec}}" --output "{{output}}"; else {{swe}} eval run --spec "{{spec}}"; fi

harness-plan-learning-store backend="both":
    @bash scripts/plan-learning-store.sh "{{backend}}"

harness-sync-learning-store db_path="":
    @bash scripts/sync-learning-store.sh "{{db_path}}"

harness-query-learning-store mode="summaries" limit="10" status="any" job_type="any" db_path="":
    @bash scripts/query-learning-store.sh "{{mode}}" "{{limit}}" "{{status}}" "{{job_type}}" "{{db_path}}"

harness-eval-learning-retrieval db_path="":
    @bash scripts/eval-learning-retrieval.sh "{{db_path}}"

# ------------------------------ fabricator (Rust) ---------------------------

fabricate need:
    @{{swe}} fabricate "{{need}}"

fabricate-validate-chain run:
    @{{swe}} fabricate validate-chain "{{run}}"

# ------------------------------ hooks (Rust) --------------------------------

agent-hooks-compact-logs:
    @{{swe}} agent-hooks compact-logs

agent-hooks-index-rebuild:
    @{{swe}} agent-hooks index

agent-hooks-export-otel output="":
    @if [ -n "{{output}}" ]; then {{swe}} agent-hooks export otel --output "{{output}}"; else {{swe}} agent-hooks export otel; fi

agent-hooks-export-junit output="":
    @if [ -n "{{output}}" ]; then {{swe}} agent-hooks export junit --output "{{output}}"; else {{swe}} agent-hooks export junit; fi

# ------------------------------ federation (Rust) ---------------------------

federation-status:
    @{{swe}} federation status

# Routing enforcement: CI merge gate. Set SWE_SEED_TRACE to the merge's trace id;
# exits non-zero if it was never routed or the chain fails verification.
gate-merge:
    @trace="${SWE_SEED_TRACE:-}"; if [ -z "$$trace" ]; then echo "SWE_SEED_TRACE unset — no trace to gate" >&2; exit 1; fi; {{swe}} gate "$$trace" --verify

# SOPS-encrypt a federation private key at rest (idempotent if already encrypted).
federation-encrypt-key key_id:
    @sops --encrypt --in-place .swe-seed/federation/keys/{{key_id}}.key

# Decrypt a federation private key to stdout (e.g. for inspection).
federation-decrypt-key key_id:
    @sops -d .swe-seed/federation/keys/{{key_id}}.key

# ------------------------------ secrets -------------------------------------

secrets-encrypt:
    @bash scripts/secrets-encrypt.sh

secrets-decrypt:
    @bash scripts/secrets-decrypt.sh

secrets-edit file:
    @bash scripts/secrets-edit.sh "{{file}}"

secrets-rotate-key:
    @bash scripts/secrets-rotate-key.sh
