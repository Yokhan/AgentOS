#!/usr/bin/env bash
# AgentOS compatibility shim: delegate project bootstrap to the canonical template repo.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

resolve_template_root() {
  local candidate=""
  for candidate in \
    "$SCRIPT_DIR/../agent-project-template" \
    "$SCRIPT_DIR/agent-project-template"
  do
    if [ -f "$candidate/setup.sh" ] && [ -f "$candidate/CLAUDE.md" ]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

TEMPLATE_ROOT="$(resolve_template_root || true)"
if [ -z "$TEMPLATE_ROOT" ]; then
  echo "Error: canonical agent-project-template repo not found next to AgentOS." >&2
  echo "Expected one of: ../agent-project-template or ./agent-project-template" >&2
  exit 1
fi

exec bash "$TEMPLATE_ROOT/setup.sh" "$@"