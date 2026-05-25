#!/usr/bin/env bash
set -euo pipefail

event="${1:-}"

emit() {
  local event_name="$1"
  local purpose="$2"
  local action="$3"
  local boundary="$4"

  printf 'event: %s\n' "$event_name"
  printf 'purpose: %s\n' "$purpose"
  printf 'action: %s\n' "$action"
  printf 'boundary: %s\n' "$boundary"
}

case "$event" in
  session.start)
    emit \
      "session.start" \
      "Orient the agent before it spends attention." \
      "Read AGENTS.md, .agent-harness/context/README.md, .agent-harness/memory/repo-map.md, .agent-harness/memory/constraints.md, and the selected route card when known." \
      "Do not perform project edits from this hook."
    ;;
  prompt.submit)
    emit \
      "prompt.submit" \
      "Convert user intent into an executable route plan." \
      "Run or mirror python scripts/harness.py route '<task>'; if the route lacks next action, treat the router as defective." \
      "Ask for clarification only when route choice materially changes the work."
    ;;
  tool.pre)
    emit \
      "tool.pre" \
      "Prevent high-cost mistakes before tools run." \
      "Check workspace, command intent, context budget, secret risk, and destructive-operation risk before execution." \
      "Do not block ordinary read, validation, or formatting commands."
    ;;
  tool.post)
    emit \
      "tool.post" \
      "Preserve evidence while it is still fresh." \
      "Record changed files, command result, proof relevance, and tool-output containment decisions in the trace draft when the command affects completion." \
      "Do not write secrets, raw environment dumps, or noisy logs into traces."
    ;;
  turn.stop)
    emit \
      "turn.stop" \
      "Stop false completion claims." \
      "Compare the route card's proof and done_when fields against observed evidence before final response." \
      "If proof is missing, report the gap instead of claiming completion."
    ;;
  session.end)
    emit \
      "session.end" \
      "Capture reusable learning without bloating memory." \
      "Write a reflection or improvement proposal only when there is evidence of repeated friction or a changed harness contract." \
      "Do not auto-apply material harness changes while learning.auto_apply is false."
    ;;
  *)
    echo "usage: $0 <session.start|prompt.submit|tool.pre|tool.post|turn.stop|session.end>" >&2
    exit 2
    ;;
esac
