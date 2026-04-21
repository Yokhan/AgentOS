#!/usr/bin/env bash
# AgentOS compatibility shim: delegate sync to the canonical template repo.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

normalize_path() {
  local value="$1"
  case "$value" in
    [A-Z]:/*|[A-Z]:\\*)
      local drive
      local rest
      drive="$(printf '%s' "${value:0:1}" | tr 'A-Z' 'a-z')"
      rest="${value:2}"
      rest="${rest//\\//}"
      printf '/%s%s\n' "$drive" "$rest"
      ;;
    *)
      printf '%s\n' "${value//\\//}"
      ;;
  esac
}

resolve_template_root() {
  local candidate=""
  for candidate in \
    "$ROOT_DIR/../agent-project-template" \
    "$ROOT_DIR/agent-project-template"
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

ARGS=("$@")
if [ ${#ARGS[@]} -gt 0 ]; then
  case "$(normalize_path "${ARGS[0]}")" in
    "$(normalize_path "$ROOT_DIR")")
      ARGS[0]="$TEMPLATE_ROOT"
      ;;
  esac
fi

exec bash "$TEMPLATE_ROOT/scripts/sync-template.sh" "${ARGS[@]}"