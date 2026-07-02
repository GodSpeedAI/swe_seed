#!/usr/bin/env bash
set -euo pipefail

required_commands=(git just cargo)
recommended_commands=(age direnv devbox mise pnpm sops)
missing_required=()
missing_recommended=()

for command_name in "${required_commands[@]}"; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    missing_required+=("$command_name")
  fi
done

for command_name in "${recommended_commands[@]}"; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    missing_recommended+=("$command_name")
  fi
done

if ((${#missing_required[@]} > 0)); then
  echo "Missing required commands: ${missing_required[*]}" >&2
  exit 1
fi

if ((${#missing_recommended[@]} > 0)); then
  echo "Missing recommended commands: ${missing_recommended[*]}"
fi

echo "Required development commands are available"
