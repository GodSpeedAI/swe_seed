#!/usr/bin/env bash
set -euo pipefail

if ! command -v sops >/dev/null 2>&1; then
  echo "sops is required to encrypt secrets" >&2
  exit 1
fi

find secrets -type f \( -name '*.plain.yaml' -o -name '*.plain.yml' -o -name '*.plain.json' -o -name '*.plain.env' \) -print0 2>/dev/null |
  while IFS= read -r -d '' file; do
    output="${file/.plain./.}"
    sops --encrypt "$file" > "$output"
    echo "Wrote $output"
  done
