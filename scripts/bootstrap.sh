#!/usr/bin/env bash
set -euo pipefail

echo "Bootstrapping SWE_SEED development harness"

if command -v mise >/dev/null 2>&1; then
  mise install
fi

if command -v pnpm >/dev/null 2>&1 && [[ -f pnpm-lock.yaml ]]; then
  pnpm install --frozen-lockfile
fi

if command -v uv >/dev/null 2>&1; then
  uv sync
fi

echo "Bootstrap complete. Run: just doctor"
