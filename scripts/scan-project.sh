#!/bin/bash
# scan-project.sh - Scan repo and populate _reference/tool-registry.md

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

MODE="full"
if [ "${1:-}" = "--report" ]; then
  MODE="report"
fi

REGISTRY="_reference/tool-registry.md"

sanitize_cell() {
  printf '%s' "$1" | tr '\r\n|' '   ' | sed 's/[[:space:]]\+/ /g; s/^ //; s/ $//'
}

extract_table_lines() {
  local heading="$1"
  if [ ! -f "$REGISTRY" ]; then
    return
  fi

  awk -v heading="$heading" '
    $0 == heading { capture=1; next }
    /^## / && capture { exit }
    capture && /^\|/ { print }
  ' "$REGISTRY"
}

echo "=== Project Scanner ==="

STACK="unknown"
STACK_FILE=""
if [ -f package.json ]; then
  STACK="js/ts"; STACK_FILE="package.json"
elif [ -f tsconfig.json ]; then
  STACK="js/ts"; STACK_FILE="tsconfig.json"
elif [ -f requirements.txt ]; then
  STACK="python"; STACK_FILE="requirements.txt"
elif [ -f pyproject.toml ]; then
  STACK="python"; STACK_FILE="pyproject.toml"
elif [ -f go.mod ]; then
  STACK="go"; STACK_FILE="go.mod"
elif [ -f Cargo.toml ]; then
  STACK="rust"; STACK_FILE="Cargo.toml"
fi
echo "Stack: $STACK ${STACK_FILE:+(detected via $STACK_FILE)}"

SRC_DIRS=()
for dir in src lib app packages; do
  if [ -d "$dir" ]; then
    SRC_DIRS+=("$dir")
  fi
done
if [ ${#SRC_DIRS[@]} -gt 0 ]; then
  echo "Source dirs: ${SRC_DIRS[*]}"
else
  echo "Source dirs: (none)"
fi

declare -A TEMPLATE_PATHS=()
while IFS= read -r line; do
  path="$(printf '%s\n' "$line" | awk -F'|' '{print $3}')"
  path="$(sanitize_cell "$path")"
  case "$path" in
    ""|Path|-----|_Run*) continue ;;
  esac
  TEMPLATE_PATHS["$path"]=1
done < <(extract_table_lines "## Template-Level (available in ALL projects)")

declare -A PROJECT_ROW_SEEN=()
declare -A HELPER_ROW_SEEN=()
PROJECT_ROWS=()
HELPER_ROWS=()

echo ""
echo "[1/2] Scanning scripts..."
for script_dir in scripts bin tools; do
  [ -d "$script_dir" ] || continue
  for file in "$script_dir"/*.sh "$script_dir"/*.py "$script_dir"/*.js; do
    [ -f "$file" ] || continue
    tool_name="${file##*/}"
    tool_name="${tool_name%.*}"
    purpose="$(awk 'NR<=5 && (/^#( |$)/ || /^\/\// || /^"""/) { sub(/^[# \/"]*/, ""); print substr($0, 1, 72); exit }' "$file" 2>/dev/null || true)"
    purpose="${purpose:-project script}"
    purpose="${purpose//|/ }"
    purpose="${purpose#"${purpose%%[![:space:]]*}"}"
    purpose="${purpose%"${purpose##*[![:space:]]}"}"
    echo "  $file: $purpose"

    if [ -z "${TEMPLATE_PATHS[$file]:-}" ] && [ -z "${PROJECT_ROW_SEEN[$file]:-}" ]; then
      PROJECT_ROWS+=("| $tool_name | $file | $purpose | agent/manual |")
      PROJECT_ROW_SEEN["$file"]=1
    fi
  done
done

echo ""
echo "[2/2] Scanning shared utilities..."
for shared_name in shared utils helpers common lib core; do
  SEARCH_ROOTS=("${SRC_DIRS[@]}" ".")
  for root in "${SEARCH_ROOTS[@]}"; do
    dir="$root/$shared_name"
    [ -d "$dir" ] || continue

    while IFS= read -r file; do
      [ -f "$file" ] || continue
      basename_f="$(basename "$file")"
      ext="${basename_f##*.}"
      exports=""

      case "$ext" in
        ts|tsx|js|jsx)
          exports=$(grep -E "^export (default )?(function|const|class|type|interface|enum) " "$file" 2>/dev/null | \
            sed -E 's/^export (default )?(function|const|let|class|type|interface|enum) ([a-zA-Z0-9_]+).*/\3/' | head -5)
          ;;
        py)
          exports=$(grep -E "^(def |class )" "$file" 2>/dev/null | \
            sed -E 's/^(def |class )([a-zA-Z0-9_]+).*/\2/' | head -5)
          ;;
        go)
          exports=$(grep -E "^func [A-Z]" "$file" 2>/dev/null | \
            sed -E 's/^func ([A-Za-z0-9_]+).*/\1/' | head -5)
          ;;
        rs)
          exports=$(grep -E "^pub (fn|struct|enum|trait) " "$file" 2>/dev/null | \
            sed -E 's/^pub (fn|struct|enum|trait) ([a-zA-Z0-9_]+).*/\2/' | head -5)
          ;;
      esac

      for export_name in $exports; do
        helper_key="$export_name|$file"
        [ -n "$export_name" ] || continue
        [ -z "${HELPER_ROW_SEEN[$helper_key]:-}" ] || continue

        importers=0
        case "$ext" in
          ts|tsx|js|jsx)
            if [ ${#SRC_DIRS[@]} -gt 0 ]; then
              importers=$(grep -rl "$export_name" "${SRC_DIRS[@]}" --include="*.ts" --include="*.tsx" --include="*.js" --include="*.jsx" 2>/dev/null | grep -cv "$file" || echo 0)
            fi
            ;;
          py)
            if [ ${#SRC_DIRS[@]} -gt 0 ]; then
              importers=$(grep -rl "import.*$export_name" "${SRC_DIRS[@]}" --include="*.py" 2>/dev/null | grep -cv "$file" || echo 0)
            fi
            ;;
        esac

        echo "  $file: $export_name (used by $importers files)"
        HELPER_ROWS+=("| $export_name | $file | export | $importers files |")
        HELPER_ROW_SEEN["$helper_key"]=1
      done
    done < <(find "$dir" -type f \( -name "*.ts" -o -name "*.js" -o -name "*.py" -o -name "*.go" -o -name "*.rs" \) -not -name "*.test.*" -not -name "*.spec.*" 2>/dev/null)
  done
done

echo ""
echo "=== Scan Summary ==="
echo "Stack: $STACK"
echo "Project-level scripts: ${#PROJECT_ROWS[@]}"
echo "Shared utilities: ${#HELPER_ROWS[@]}"

if [ "$MODE" = "report" ]; then
  echo ""
  echo "(report mode - no changes written)"
  exit 0
fi

if [ ! -f "$REGISTRY" ]; then
  echo "WARNING: $REGISTRY not found. Run setup-project first."
  exit 1
fi

TEMPLATE_TABLE="$(extract_table_lines "## Template-Level (available in ALL projects)")"
[ -n "$TEMPLATE_TABLE" ] || TEMPLATE_TABLE=$'| Tool | Path | Purpose |\n|------|------|---------|'

CANDIDATE_TABLE="$(extract_table_lines "## Candidates for Extraction (auto-detected by audit-reuse.sh)")"
[ -n "$CANDIDATE_TABLE" ] || CANDIDATE_TABLE=$'| Function | Found in | Count | Recommendation |\n|----------|----------|-------|----------------|\n| _Run `bash scripts/audit-reuse.sh` to detect_ | | | |'

DESIGN_TABLE="$(extract_table_lines "## Design Tokens & Components (Figma projects only)")"
[ -n "$DESIGN_TABLE" ] || DESIGN_TABLE=$'| Component | ID/Path | Variants | Used by |\n|-----------|---------|----------|---------|\n| _Populated by agents working with Figma MCP_ | | | |'

PROJECT_TABLE=$'| Tool | Path | Purpose | Used by |\n|------|------|---------|---------|'
if [ ${#PROJECT_ROWS[@]} -gt 0 ]; then
  PROJECT_TABLE+=$'\n'"$(printf '%s\n' "${PROJECT_ROWS[@]}" | sort)"
else
  PROJECT_TABLE+=$'\n| _No project-level tools detected_ | | | |'
fi

HELPER_TABLE=$'| Function | Path | Signature | Used by |\n|----------|------|-----------|---------|'
if [ ${#HELPER_ROWS[@]} -gt 0 ]; then
  HELPER_TABLE+=$'\n'"$(printf '%s\n' "${HELPER_ROWS[@]}" | sort)"
else
  HELPER_TABLE+=$'\n| _No shared utilities detected_ | | | |'
fi

cat > "$REGISTRY" <<EOF
# Tool Registry

> Searchable index of reusable utilities across this project.
> **Check HERE before writing new code.** See \`.claude/library/technical/atomic-reuse.md\`.
>
> Maintained by: agents (manual), \`scripts/scan-project.sh\` (project-level scan), \`scripts/audit-reuse.sh\` (ongoing).

## Template-Level (available in ALL projects)

$TEMPLATE_TABLE

## Project-Level (auto-populated by scan-project.sh, updated by agents)

$PROJECT_TABLE

## Helpers & Utilities (src/shared/ or lib/)

$HELPER_TABLE

## Candidates for Extraction (auto-detected by audit-reuse.sh)

$CANDIDATE_TABLE

## Design Tokens & Components (Figma projects only)

$DESIGN_TABLE

---

_Last scan: $(date +%Y-%m-%d)_
EOF

echo ""
echo "Updated $REGISTRY"
