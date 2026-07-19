#!/usr/bin/env bash
# downstream-census.sh - Compact migration matrix for downstream template projects
# Usage:
#   bash scripts/downstream-census.sh --brief
#   bash scripts/downstream-census.sh --json /path/to/project-one "/path/to/project two"

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
PROJECT_ROOT="$(normalize_drive_path "$(cd "$SCRIPT_DIR/.." && pwd)")"
[ -f "$SCRIPT_DIR/lib/platform.sh" ] && source "$SCRIPT_DIR/lib/platform.sh"

OUTPUT_MODE="default"
SEARCH_DIR="${HOME}/Documents"
RUN_SYNC=true
declare -a PROJECTS=()

usage() {
  cat <<'EOF'
Usage: bash scripts/downstream-census.sh [options] [project-dir ...]

Options:
  --brief         Compact table output
  --json          JSON array output
  --search DIR    Search for downstream projects under DIR when no explicit project dirs are given
  --no-sync       Skip dry-run sync and report manifest/spec metadata only
  -h, --help      Show this help
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --brief) OUTPUT_MODE="brief" ;;
    --json) OUTPUT_MODE="json" ;;
    --search)
      shift
      if [ $# -eq 0 ]; then
        echo "ERROR: --search requires a directory" >&2
        exit 1
      fi
      SEARCH_DIR="$1"
      ;;
    --no-sync) RUN_SYNC=false ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      PROJECTS+=("$1")
      ;;
  esac
  shift
done

if [ ${#PROJECTS[@]} -eq 0 ]; then
  while IFS= read -r -d '' manifest; do
    PROJECTS+=("$(dirname "$manifest")")
  done < <(find "$SEARCH_DIR" -maxdepth 3 -name ".template-manifest.json" -print0 2>/dev/null)
fi

if [ ${#PROJECTS[@]} -eq 0 ]; then
  echo "No downstream projects found." >&2
  exit 1
fi

TARGET_VERSION="$(sed -n 's/.*Template Version: \([0-9.]*\).*/\1/p' "$PROJECT_ROOT/CLAUDE.md" 2>/dev/null | head -1)"
[ -n "$TARGET_VERSION" ] || TARGET_VERSION="unknown"

json_escape() {
  printf '%s' "$1" | sed ':a;N;$!ba;s/\\/\\\\/g;s/"/\\"/g;s/\t/\\t/g;s/\r//g;s/\n/\\n/g'
}

format_bool() {
  if [ "$1" = true ]; then
    printf 'true'
  else
    printf 'false'
  fi
}

parse_metric() {
  local key="$1"
  local content="$2"
  local value=""
  value=$(printf '%s\n' "$content" | sed -n "s/^${key}:[[:space:]]*\\([0-9][0-9]*\\).*/\\1/p" | tail -1)
  printf '%s\n' "${value:-0}"
}

classify_project() {
  local manifest_present="$1"
  local sync_exit="$2"
  local conflicts="$3"

  if [ "$manifest_present" != true ]; then
    printf 'no-manifest\n'
    return
  fi
  if [ "$sync_exit" -ne 0 ]; then
    printf 'sync-error\n'
    return
  fi
  if [ "$conflicts" -gt 0 ]; then
    printf 'manual-merge\n'
    return
  fi
  printf 'clean-dry-run\n'
}

render_row() {
  local name="$1"
  local current_version="$2"
  local classification="$3"
  local conflicts="$4"
  local new_files="$5"
  local deprecated="$6"
  local spec_present="$7"
  local agents_present="$8"

  printf '%-28s  %-8s  %-14s  c=%-4s n=%-4s d=%-4s  spec=%-3s agents=%-3s\n' \
    "$name" "$current_version" "$classification" "$conflicts" "$new_files" "$deprecated" \
    "$([ "$spec_present" = true ] && printf yes || printf no)" \
    "$([ "$agents_present" = true ] && printf yes || printf no)"
}

declare -a RESULT_ROWS=()

for project in "${PROJECTS[@]}"; do
  [ -d "$project" ] || continue

  project="$(normalize_drive_path "$(cd "$project" && pwd)")"
  [ "$project" = "$PROJECT_ROOT" ] && continue

  manifest_path="$project/.template-manifest.json"
  manifest_present=false
  current_version="missing"
  if [ -f "$manifest_path" ]; then
    manifest_present=true
    current_version=$(cd "$project" && _node -e "try{const d=JSON.parse(require('fs').readFileSync('.template-manifest.json','utf8'));console.log(d.template_version||'unknown')}catch{console.log('invalid')}" 2>/dev/null || echo "invalid")
  fi

  agents_present=false
  spec_present=false
  [ -f "$project/AGENTS.md" ] && agents_present=true
  [ -f "$project/PROJECT_SPEC.md" ] && spec_present=true

  sync_exit=0
  updated=0
  new_files=0
  conflicts=0
  skipped=0
  preserved=0
  deprecated=0

  if [ "$RUN_SYNC" = true ] && [ "$manifest_present" = true ]; then
    sync_output=$(bash "$PROJECT_ROOT/scripts/sync-template.sh" "$PROJECT_ROOT" --project-dir "$project" --dry-run 2>&1) || sync_exit=$?
    updated=$(parse_metric "UPDATED" "$sync_output")
    new_files=$(parse_metric "NEW" "$sync_output")
    conflicts=$(parse_metric "CONFLICTS" "$sync_output")
    skipped=$(parse_metric "SKIPPED" "$sync_output")
    preserved=$(parse_metric "PRESERVED" "$sync_output")
    deprecated=$(parse_metric "DEPRECATED" "$sync_output")
  fi

  classification="$(classify_project "$manifest_present" "$sync_exit" "$conflicts")"
  RESULT_ROWS+=("$(printf '%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s' \
    "$project" "$(basename "$project")" "$current_version" "$TARGET_VERSION" "$classification" \
    "$sync_exit" "$updated" "$new_files" "$conflicts" "$skipped" "$preserved" "$deprecated" \
    "$spec_present" "$agents_present")")
done

if [ "$OUTPUT_MODE" = "json" ]; then
  echo "["
  first=1
  for row in "${RESULT_ROWS[@]}"; do
    IFS='|' read -r project_path project_name current_version target_version classification sync_exit updated new_files conflicts skipped preserved deprecated spec_present agents_present < <(printf '%s\n' "$row")
    if [ "$first" -eq 0 ]; then
      echo ","
    fi
    first=0
    printf '  {"project_name":"%s","project_path":"%s","current_version":"%s","target_version":"%s","classification":"%s","sync_exit":%s,"updated":%s,"new":%s,"conflicts":%s,"skipped":%s,"preserved":%s,"deprecated":%s,"has_project_spec":%s,"has_agents_md":%s}' \
      "$(json_escape "$project_name")" \
      "$(json_escape "$project_path")" \
      "$(json_escape "$current_version")" \
      "$(json_escape "$target_version")" \
      "$(json_escape "$classification")" \
      "$sync_exit" "$updated" "$new_files" "$conflicts" "$skipped" "$preserved" "$deprecated" \
      "$(format_bool "$spec_present")" "$(format_bool "$agents_present")"
  done
  echo
  echo "]"
  exit 0
fi

echo "=== Downstream Census ==="
echo "Template: $PROJECT_ROOT"
echo "Target version: $TARGET_VERSION"
echo ""
printf '%-28s  %-8s  %-14s  %-18s\n' "Project" "Current" "Class" "Notes"

for row in "${RESULT_ROWS[@]}"; do
  IFS='|' read -r _ project_name current_version _ classification _ _ new_files conflicts _ _ deprecated spec_present agents_present < <(printf '%s\n' "$row")
  if [ "$OUTPUT_MODE" = "brief" ]; then
    render_row "$project_name" "$current_version" "$classification" "$conflicts" "$new_files" "$deprecated" "$spec_present" "$agents_present"
  else
    notes="conflicts=$conflicts new=$new_files deprecated=$deprecated"
    printf '%-28s  %-8s  %-14s  %s\n' "$project_name" "$current_version" "$classification" "$notes"
  fi
done
