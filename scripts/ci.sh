#!/usr/bin/env bash
set -euo pipefail

mode="${1:-all}"

run_format() {
  if command -v pnpm >/dev/null 2>&1 && [[ -f package.json ]]; then
    pnpm exec prettier --check .
  else
    echo "Skipping format: pnpm is not installed"
  fi
}

run_lint() {
  if command -v uv >/dev/null 2>&1 && [[ -f pyproject.toml ]]; then
    uv run ruff check .
  else
    echo "Skipping Python lint: uv is not installed"
  fi
}

run_test() {
  # Rust-backed harness structure validation + the full Rust test suite.
  # Replaces the former `python scripts/harness.py validate` + Python CLI
  # parity smoke (`tests/validate-harness.sh`) after the Phase 10 cutover.
  cargo run -q -p swe-seed -- harness
  cargo test -q
  cargo build --release -q -p swe-seed
  SWE_SEED_BIN=target/release/swe-seed cargo test -q -p swe-seed --test cli_golden
}

case "$mode" in
format)
  run_format
  ;;
lint)
  run_lint
  ;;
test)
  run_test
  ;;
all)
  bash scripts/doctor.sh
  run_format
  run_lint
  run_test
  ;;
*)
  echo "Unknown CI mode: $mode" >&2
  echo "Expected one of: all, format, lint, test" >&2
  exit 2
  ;;
esac
