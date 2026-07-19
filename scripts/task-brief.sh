#!/usr/bin/env bash
# task-brief.sh - Compact summary of tasks/current.md

set -euo pipefail

MODE="default"
TASK_FILE="tasks/current.md"

print_usage() {
  cat <<'EOF'
Usage: bash scripts/task-brief.sh [--brief|--json|--full] [path]

Summarize a task handoff markdown file with compact default output.
EOF
}

parse_args() {
  while [ $# -gt 0 ]; do
    case "$1" in
      --brief) MODE="brief" ;;
      --json) MODE="json" ;;
      --full) MODE="full" ;;
      -h|--help)
        print_usage
        exit 0
        ;;
      *)
        TASK_FILE="$1"
        ;;
    esac
    shift
  done
}

trim_line() {
  local value="$1"
  value="${value%$'\r'}"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s\n' "$value"
}

extract_section() {
  local heading="$1"
  awk -v heading="$heading" '
    $0 == "## " heading { capture=1; next }
    /^## / && capture { exit }
    capture { print }
  ' "$TASK_FILE"
}

collect_items() {
  local heading="$1"
  local line=""
  local item=""

  while IFS= read -r line; do
    line="$(trim_line "$line")"
    [ -z "$line" ] && continue

    if [[ "$line" =~ ^[-*][[:space:]]+(.*)$ ]]; then
      item="${BASH_REMATCH[1]}"
    elif [[ "$line" =~ ^[0-9]+\.[[:space:]]+(.*)$ ]]; then
      item="${BASH_REMATCH[1]}"
    else
      item="$line"
    fi

    printf '%s\n' "$item"
  done < <(extract_section "$heading")
}

first_item() {
  local heading="$1"
  local item=""

  while IFS= read -r item; do
    [ -z "$item" ] && continue
    printf '%s\n' "$item"
    return 0
  done < <(collect_items "$heading")

  return 0
}

collect_warnings() {
  local line_count=""
  line_count=$(wc -l < "$TASK_FILE" | tr -d ' ')

  if [ "$line_count" -gt 90 ]; then
    printf '%s\n' "handoff is $line_count lines; session-start may hide important details"
  fi
  if grep -q '^### Complexity Estimate' "$TASK_FILE" 2>/dev/null; then
    printf '%s\n' "planning scaffolding is still mixed into tasks/current.md"
  fi
  if grep -q '^### Implementation Order' "$TASK_FILE" 2>/dev/null; then
    printf '%s\n' "implementation checklist is still living in the handoff file"
  fi
  if grep -q '^## Immediate Next Step' "$TASK_FILE" 2>/dev/null; then
    return
  fi

  printf '%s\n' "Immediate Next Step section is missing"
}

print_list() {
  local title="$1"
  local heading="$2"
  local limit="$3"
  local count=0
  local total=0
  local item=""

  total=$(collect_items "$heading" | wc -l | tr -d ' ')
  echo "$title:"
  while IFS= read -r item; do
    [ -z "$item" ] && continue
    echo "- $item"
    count=$((count + 1))
    if [ "$count" -ge "$limit" ]; then
      break
    fi
  done < <(collect_items "$heading")

  if [ "$count" -eq 0 ]; then
    echo "- (none)"
  elif [ "$total" -gt "$limit" ]; then
    echo "- ... +$((total - limit)) more"
  fi
}

print_warning_list() {
  local count=0
  local limit="$1"
  local total=0
  local item=""

  total=$(collect_warnings | wc -l | tr -d ' ')
  echo "Warnings:"
  while IFS= read -r item; do
    [ -z "$item" ] && continue
    echo "- $item"
    count=$((count + 1))
    if [ "$count" -ge "$limit" ]; then
      break
    fi
  done < <(collect_warnings)

  if [ "$count" -eq 0 ]; then
    echo "- (none)"
  elif [ "$total" -gt "$limit" ]; then
    echo "- ... +$((total - limit)) more"
  fi
}

json_escape() {
  JSON_ESCAPE_VALUE="$1" node -e "const value = process.env.JSON_ESCAPE_VALUE || ''; process.stdout.write(JSON.stringify(value).slice(1, -1));"
}

print_json_array() {
  local key="$1"
  shift
  local first=1

  printf '  "%s": [' "$key"
  while [ $# -gt 0 ]; do
    if [ "$first" -eq 0 ]; then
      printf ', '
    fi
    printf '"%s"' "$(json_escape "$1")"
    first=0
    shift
  done
  printf ']'
}

print_brief() {
  local goal=""

  goal="$(first_item "Goal")"
  echo "=== Task Brief ==="
  echo "File: $TASK_FILE"
  echo "Goal: ${goal:-_Not set yet._}"
  print_list "User wants" "User Wants" 3
  print_list "Status" "Current Status" 4
  print_list "Next" "Immediate Next Step" 2
}

print_default() {
  local goal=""

  goal="$(first_item "Goal")"
  echo "=== Task Brief ==="
  echo "File: $TASK_FILE"
  echo "Goal: ${goal:-_Not set yet._}"
  print_list "Roadmap" "Active Roadmap" 3
  print_list "User wants" "User Wants" 3
  print_list "Status" "Current Status" 6
  print_list "Next" "Immediate Next Step" 3
  print_warning_list 3
}

print_full() {
  echo "=== Task Brief (full) ==="
  echo "File: $TASK_FILE"
  print_list "Goal" "Goal" 20
  print_list "User wants" "User Wants" 20
  print_list "Roadmap" "Active Roadmap" 20
  print_list "Status" "Current Status" 20
  print_list "Next" "Immediate Next Step" 20
}

print_json() {
  local goal=""
  local line_count=""

  mapfile -t roadmap < <(collect_items "Active Roadmap")
  mapfile -t user_wants < <(collect_items "User Wants")
  mapfile -t status_items < <(collect_items "Current Status")
  mapfile -t next_steps < <(collect_items "Immediate Next Step")
  mapfile -t warnings < <(collect_warnings)

  goal="$(first_item "Goal")"
  line_count=$(wc -l < "$TASK_FILE" | tr -d ' ')

  echo "{"
  printf '  "file": "%s",\n' "$(json_escape "$TASK_FILE")"
  printf '  "line_count": %s,\n' "$line_count"
  printf '  "goal": "%s",\n' "$(json_escape "${goal:-_Not set yet._}")"
  print_json_array "roadmap" "${roadmap[@]}"
  echo ","
  print_json_array "user_wants" "${user_wants[@]}"
  echo ","
  print_json_array "status" "${status_items[@]}"
  echo ","
  print_json_array "next_steps" "${next_steps[@]}"
  echo ","
  print_json_array "warnings" "${warnings[@]}"
  echo
  echo "}"
}

parse_args "$@"

if [ ! -f "$TASK_FILE" ]; then
  echo "ERROR: $TASK_FILE not found" >&2
  exit 1
fi

case "$MODE" in
  brief) print_brief ;;
  json) print_json ;;
  full) print_full ;;
  *) print_default ;;
esac
