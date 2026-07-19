#!/usr/bin/env bash
# Initialize Spec Kit in a project using the pinned local snapshot ref.

set -euo pipefail

PROJECT_DIR="."
INTEGRATION="codex"
SCRIPT_TYPE="sh"
REF=""
IGNORE_AGENT_TOOLS=0
DRY_RUN=0
USE_INSTALLED=0

usage() {
  cat <<'USAGE'
Usage: bash scripts/init-spec-kit.sh [options]

Options:
  --project-dir <dir>       Project directory to initialize (default: .)
  --integration <key>       Spec Kit integration key (default: codex)
  --script <sh|ps>          Script type for generated helpers (default: sh)
  --ref <ref>               Override Spec Kit ref from manifest
  --ignore-agent-tools      Pass --ignore-agent-tools to specify init
  --use-installed           Use installed specify CLI instead of pinned uvx ref
  --dry-run                 Print command without running it
  -h, --help                Show help
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --project-dir)
      PROJECT_DIR="${2:-}"
      [ -n "$PROJECT_DIR" ] || { echo "ERROR: --project-dir requires a value" >&2; exit 1; }
      shift 2
      ;;
    --integration)
      INTEGRATION="${2:-}"
      [ -n "$INTEGRATION" ] || { echo "ERROR: --integration requires a value" >&2; exit 1; }
      shift 2
      ;;
    --script)
      SCRIPT_TYPE="${2:-}"
      [ "$SCRIPT_TYPE" = "sh" ] || [ "$SCRIPT_TYPE" = "ps" ] || {
        echo "ERROR: --script must be sh or ps" >&2
        exit 1
      }
      shift 2
      ;;
    --ref)
      REF="${2:-}"
      [ -n "$REF" ] || { echo "ERROR: --ref requires a value" >&2; exit 1; }
      shift 2
      ;;
    --ignore-agent-tools)
      IGNORE_AGENT_TOOLS=1
      shift
      ;;
    --use-installed)
      USE_INSTALLED=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

[ -d "$PROJECT_DIR" ] || {
  echo "ERROR: Project directory not found: $PROJECT_DIR" >&2
  exit 1
}

if [ -z "$REF" ]; then
  [ -f "_reference/spec-kit/manifest.json" ] || {
    echo "ERROR: Missing _reference/spec-kit/manifest.json" >&2
    exit 1
  }
  command -v node >/dev/null 2>&1 || {
    echo "ERROR: node is required to read Spec Kit manifest" >&2
    exit 1
  }
  REF="$(node -e "const m=require('./_reference/spec-kit/manifest.json'); console.log(m.ref || '')")"
fi

[ -n "$REF" ] || {
  echo "ERROR: Cannot determine Spec Kit ref" >&2
  exit 1
}

EXTRA_ARGS=()
if [ "$IGNORE_AGENT_TOOLS" -eq 1 ]; then
  EXTRA_ARGS+=(--ignore-agent-tools)
fi

if [ "$USE_INSTALLED" -eq 0 ] && command -v uvx >/dev/null 2>&1; then
  CMD=(uvx --from "git+https://github.com/github/spec-kit.git@$REF" specify init "$PROJECT_DIR" --integration "$INTEGRATION" --script "$SCRIPT_TYPE" "${EXTRA_ARGS[@]}")
elif [ "$USE_INSTALLED" -eq 1 ] && command -v specify >/dev/null 2>&1; then
  CMD=(specify init "$PROJECT_DIR" --integration "$INTEGRATION" --script "$SCRIPT_TYPE" "${EXTRA_ARGS[@]}")
else
  echo "ERROR: Install uv/uvx for pinned Spec Kit ref, or pass --use-installed to use an existing specify CLI." >&2
  echo "See integrations/spec-kit/README.md" >&2
  exit 1
fi

printf 'Spec Kit command:'
printf ' %q' "${CMD[@]}"
printf '\n'

if [ "$DRY_RUN" -eq 0 ]; then
  "${CMD[@]}"
fi
