#!/usr/bin/env bash
set -euo pipefail

file="${1:-}"

if [[ -z "$file" ]]; then
  echo "Usage: scripts/secrets-edit.sh <secret-file>" >&2
  exit 2
fi

if ! command -v sops >/dev/null 2>&1; then
  echo "sops is required to edit secrets" >&2
  exit 1
fi

if [[ ! -f "$file" ]]; then
  echo "error: file not found: $file" >&2
  exit 1
fi

sops "$file"
