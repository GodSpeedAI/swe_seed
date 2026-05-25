#!/usr/bin/env bash
set -euo pipefail

if ! command -v sops >/dev/null 2>&1; then
  echo "sops is required to encrypt secrets" >&2
  exit 1
fi

if grep -q 'age1replacewith' .sops.yaml 2>/dev/null; then
  echo "error: .sops.yaml contains a placeholder age key. Replace it before encrypting." >&2
  exit 1
fi

if [[ ! -d secrets ]]; then
  echo "No secrets/ directory found. Nothing to encrypt."
  exit 0
fi

find secrets -type f \( -name '*.plain.yaml' -o -name '*.plain.yml' -o -name '*.plain.json' -o -name '*.plain.env' \) -print0 |
  while IFS= read -r -d '' file; do
    output="${file/.plain./.}"
    sops --encrypt "$file" >"$output"
    echo "Wrote $output"
  done
