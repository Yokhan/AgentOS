#!/usr/bin/env bash
# generate-project-spec.sh - Build a lightweight PROJECT_SPEC.md from repo state
# Usage:
#   bash scripts/generate-project-spec.sh         # print markdown to stdout
#   bash scripts/generate-project-spec.sh --write # overwrite PROJECT_SPEC.md

set -euo pipefail

normalize_drive_path() {
  local path="$1"
  case "$path" in
    /[A-Z]/*)
      printf '/%s%s\n' "$(printf '%s' "${path:1:1}" | tr 'A-Z' 'a-z')" "${path:2}"
      ;;
    *)
      printf '%s\n' "$path"
      ;;
  esac
}

SCRIPT_DIR="$(normalize_drive_path "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)")"
[ -f "$SCRIPT_DIR/lib/platform.sh" ] && source "$SCRIPT_DIR/lib/platform.sh"

OUTPUT_PATH="PROJECT_SPEC.md"
WRITE_MODE=false

while [ $# -gt 0 ]; do
  case "$1" in
    --write) WRITE_MODE=true ;;
    --help|-h)
      echo "Usage: bash scripts/generate-project-spec.sh [--write]"
      exit 0
      ;;
    *)
      echo "ERROR: Unknown argument '$1'" >&2
      exit 1
      ;;
  esac
  shift
done

project_name="$(basename "$(pwd)")"
scan_date="$(date +%Y-%m-%d)"

detect_project_type() {
  if [ -f "setup.sh" ] && [ -f "setup.bat" ] && [ -f "AGENTS.md" ] && [ -f "CLAUDE.md" ]; then
    printf 'template repository\n'
    return
  fi
  if [ -f "package.json" ] && [ -d "src" ]; then
    printf 'application/service repo\n'
    return
  fi
  if [ -f "package.json" ]; then
    printf 'JavaScript/TypeScript repo\n'
    return
  fi
  if [ -f "requirements.txt" ] || [ -f "pyproject.toml" ]; then
    printf 'Python repo\n'
    return
  fi
  if [ -f "Cargo.toml" ]; then
    printf 'Rust repo\n'
    return
  fi
  printf 'project repo\n'
}

append_unique() {
  local value="$1"
  local current="$2"
  case " $current " in
    *" $value "*) printf '%s\n' "$current" ;;
    *) printf '%s %s\n' "$current" "$value" | awk '{$1=$1; print}' ;;
  esac
}

has_project_file() {
  if git rev-parse --git-dir >/dev/null 2>&1; then
    [ -n "$(git ls-files "$@" 2>/dev/null | sed -n '1p')" ]
    return
  fi
  local pattern=""
  local find_args=()
  for pattern in "$@"; do
    find_args+=( -name "$pattern" -o )
  done
  unset "find_args[$((${#find_args[@]} - 1))]"
  [ -n "$(find . \( -path './.git' -o -path './node_modules' -o -path './.session-cache' -o -path './.codebase-memory' \) -prune -o -type f \( "${find_args[@]}" \) -print 2>/dev/null | sed -n '1p')" ]
}

detect_stack_summary() {
  local languages=""
  local manifests=""
  local nested_package_json_count=0

  [ -f "package.json" ] && manifests="$(append_unique "package.json" "$manifests")"
  [ -f "package-lock.json" ] && manifests="$(append_unique "package-lock.json" "$manifests")"
  [ -f "tsconfig.json" ] && manifests="$(append_unique "tsconfig.json" "$manifests")"
  [ -f "requirements.txt" ] && manifests="$(append_unique "requirements.txt" "$manifests")"
  [ -f "pyproject.toml" ] && manifests="$(append_unique "pyproject.toml" "$manifests")"
  [ -f "Cargo.toml" ] && manifests="$(append_unique "Cargo.toml" "$manifests")"
  [ -f "go.mod" ] && manifests="$(append_unique "go.mod" "$manifests")"
  if git rev-parse --git-dir >/dev/null 2>&1; then
    nested_package_json_count=$(git ls-files '*/package.json' 2>/dev/null | grep -vc '^package\.json$' 2>/dev/null) || nested_package_json_count=0
  else
    nested_package_json_count=$(find . -path './node_modules' -prune -o -path './.git' -prune -o -type f -name 'package.json' ! -path './package.json' -print | wc -l | tr -d ' ')
  fi

  if has_project_file '*.ts' '*.tsx'; then
    languages="$(append_unique "TypeScript" "$languages")"
  fi
  if has_project_file '*.js' '*.jsx'; then
    languages="$(append_unique "JavaScript" "$languages")"
  fi
  if has_project_file '*.py'; then
    languages="$(append_unique "Python" "$languages")"
  fi
  if has_project_file '*.rs'; then
    languages="$(append_unique "Rust" "$languages")"
  fi
  if has_project_file '*.go'; then
    languages="$(append_unique "Go" "$languages")"
  fi
  if has_project_file '*.sh'; then
    languages="$(append_unique "Shell" "$languages")"
  fi
  if [ -z "$languages" ]; then
    languages="Unknown"
  fi

  printf -- "- Languages: %s\n" "$languages"
  if [ -n "$manifests" ]; then
    printf -- "- Key manifests: %s\n" "$manifests"
  else
    printf -- "- Key manifests: none detected\n"
  fi
  if [ "$nested_package_json_count" -gt 0 ]; then
    printf -- "- Nested package manifests: %s\n" "$nested_package_json_count"
  fi

  if [ -f "package.json" ]; then
    dep_summary=$(node -e "try{const d=require('./package.json'); const deps=Object.keys(d.dependencies||{}); const dev=Object.keys(d.devDependencies||{}); const top=[...deps.slice(0,4), ...dev.slice(0,2)].slice(0,6); console.log(`- Node packages: deps=${deps.length}, devDeps=${dev.length}${top.length?`; sample=${top.join(', ')}`:''}`)}catch{console.log('- Node packages: unreadable package.json')}" 2>/dev/null || true)
    [ -n "${dep_summary:-}" ] && printf '%s\n' "$dep_summary"
  fi
  if [ -f "requirements.txt" ]; then
    req_count=$(grep -cEv '^[[:space:]]*($|#)' requirements.txt 2>/dev/null) || req_count=0
    printf -- "- Python requirements: %s entries\n" "$req_count"
  fi
}

describe_top_level_dir() {
  case "$1" in
    .claude) printf 'agent instructions, hooks, commands, and shared rule library\n' ;;
    .codex) printf 'Codex-specific configuration and hook bridge\n' ;;
    .github) printf 'CI/release automation\n' ;;
    .vscode) printf 'editor workspace recommendations\n' ;;
    _reference) printf 'cross-project registries and reusable references\n' ;;
    brain) printf 'knowledge vault, decisions, and templates\n' ;;
    docs) printf 'user-facing and architecture documentation\n' ;;
    integrations) printf 'integration guides and external tooling notes\n' ;;
    mcp-servers) printf 'local MCP server implementations\n' ;;
    n8n) printf 'n8n integration assets and workflow exports\n' ;;
    scripts) printf 'automation, validation, and migration tooling\n' ;;
    src) printf 'application source code\n' ;;
    tasks) printf 'task handoff, lessons, and audit artifacts\n' ;;
    tests) printf 'rule and smoke test coverage\n' ;;
    public) printf 'static public assets\n' ;;
    app) printf 'application entrypoints or routes\n' ;;
    design-system) printf 'shared UI system artifacts\n' ;;
    templates) printf 'starter and scaffold templates shipped to child projects\n' ;;
    *) printf 'project directory\n' ;;
  esac
}

build_file_structure() {
  local dir=""
  while IFS= read -r dir; do
    [ -n "$dir" ] || continue
    printf -- "- %s: %s\n" "$dir" "$(describe_top_level_dir "$dir")"
  done < <(
    if git rev-parse --git-dir >/dev/null 2>&1; then
      git ls-files 2>/dev/null | awk -F/ 'NF>1 {print $1}' | sort -u
    else
      find . -mindepth 1 -maxdepth 1 -type d ! -name '.git' ! -name 'node_modules' ! -name '.idea' ! -name '.next' | sed 's#^\./##' | sort
    fi
  )
}

build_provides() {
  local script_count=0
  local script_list=""
  if [ -d "scripts" ]; then
    script_count=$(find scripts -maxdepth 1 -type f -name '*.sh' | wc -l | tr -d ' ')
    script_list=$(find scripts -maxdepth 1 -type f -name '*.sh' | sed 's#^scripts/##' | sort | head -6 | awk 'BEGIN{first=1} {if(!first) printf ", "; printf "%s", $0; first=0}')
  fi

  if [ -f "setup.sh" ] || [ -f "setup.bat" ]; then
    printf -- "- Bootstrap entrypoints: %s%s\n" \
      "$([ -f setup.sh ] && printf 'setup.sh' || printf '')" \
      "$([ -f setup.bat ] && printf ', setup.bat' || printf '')"
  fi
  if [ -d "scripts" ]; then
    printf -- "- Automation scripts: %s shell entrypoints" "$script_count"
    [ -n "$script_list" ] && printf ' (sample: %s)' "$script_list"
    printf '\n'
  fi
  if [ -d "mcp-servers" ]; then
    printf -- "- Local MCP servers: %s\n" "$(find mcp-servers -mindepth 1 -maxdepth 1 -type d | sed 's#^mcp-servers/##' | paste -sd ', ' -)"
  fi
  if [ -f "AGENTS.md" ] && [ -f "CLAUDE.md" ]; then
    printf -- "- Dual-agent instruction surface: CLAUDE.md + AGENTS.md\n"
  elif [ -f "CLAUDE.md" ]; then
    printf -- "- Agent instruction surface: CLAUDE.md\n"
  fi
}

build_depends_on() {
  if [ -f "package.json" ] || [ -d "mcp-servers" ]; then
    printf -- "- Node.js for JSON parsing, MCP tooling, and JavaScript/TypeScript tasks\n"
  fi
  printf -- "- Git for template sync, bootstrap history, and versioned workflows\n"
  if [ -f "setup.bat" ]; then
    printf -- "- PowerShell/cmd + Git Bash on Windows for full bootstrap and hook smoke coverage\n"
  fi
  if [ -f ".mcp.json" ]; then
    printf -- "- Local MCP configuration from .mcp.json\n"
  fi
}

build_current_state() {
  if [ -f "tasks/current.md" ] && [ -f "scripts/task-brief.sh" ]; then
    brief_json="$(bash scripts/task-brief.sh --json 2>/dev/null || true)"
    if [ -n "$brief_json" ]; then
      _node -e "
const fs=require('fs');
const data=JSON.parse(fs.readFileSync(0,'utf8'));
const phaseLine=(Array.isArray(data.roadmap)?data.roadmap:[]).find((item)=>/^Current milestone:/i.test(item));
console.log('- Goal: ' + (data.goal || 'not set'));
if (phaseLine) {
  console.log('- Phase: ' + phaseLine.replace(/^Current milestone:\s*/i, '').trim());
}
if (Array.isArray(data.status) && data.status.length > 0) {
  console.log('- Status: ' + data.status[0]);
}
if (Array.isArray(data.next_steps) && data.next_steps.length > 0) {
  console.log('- Next: ' + data.next_steps[0]);
}
" < <(printf '%s\n' "$brief_json") 2>/dev/null
      return
    fi
  fi
  if [ -f "tasks/current.md" ]; then
    head -8 tasks/current.md | awk 'NF {print "- " $0}'
    return
  fi
  printf -- "- No tasks/current.md found\n"
}

generate_markdown() {
  cat <<EOF
# Project Spec

> Generated by \`scripts/generate-project-spec.sh\`
> Updated: $scan_date

## Identity
- Name: $project_name
- Type: $(detect_project_type)

## Stack & Dependencies
$(detect_stack_summary)

## File Structure
$(build_file_structure)

## Provides (APIs, URLs, exports)
$(build_provides)

## Depends On (services, projects)
$(build_depends_on)

## Current State
$(build_current_state)

## Last Scan
$scan_date
EOF
}

if [ "$WRITE_MODE" = true ]; then
  generate_markdown > "$OUTPUT_PATH"
  echo "Wrote $OUTPUT_PATH"
else
  generate_markdown
fi
