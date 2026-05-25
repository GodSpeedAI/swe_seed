#!/usr/bin/env bash
set -euo pipefail

if ! command -v sops >/dev/null 2>&1; then
  echo "sops is required to decrypt secrets" >&2
  exit 1
fi

find secrets -type f \( -name '*.yaml' -o -name '*.yml' -o -name '*.json' -o -name '*.env' \) -print0 2>/dev/null |
  while IFS= read -r -d '' file; do
    output="${file}.dec"
    sops --decrypt "$file" > "$output"
    echo "Wrote $output"
  done
