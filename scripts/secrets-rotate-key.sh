#!/usr/bin/env bash
set -euo pipefail

if ! command -v sops >/dev/null 2>&1; then
  echo "sops is required to rotate secrets keys" >&2
  exit 1
fi

if [[ ! -d secrets ]]; then
  echo "No secrets/ directory found. Nothing to rotate."
  exit 0
fi

find secrets -type f \( -name '*.yaml' -o -name '*.yml' -o -name '*.json' -o -name '*.env' \) -print0 2>/dev/null |
  while IFS= read -r -d '' file; do
    sops updatekeys --yes "$file"
    echo "Rotated keys for $file"
  done
