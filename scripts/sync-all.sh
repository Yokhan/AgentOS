#!/bin/bash
# sync-all.sh — Sync template to all projects with .template-manifest.json
# Usage: bash scripts/sync-all.sh [search_directory] [--dry-run]

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

TEMPLATE_DIR="$(normalize_drive_path "$(cd "$(dirname "$0")/.." && pwd)")"
SEARCH_DIR="${1:-$HOME/Documents}"
DRY_RUN=""

# Parse args
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN="--dry-run" ;;
    /*|~*) SEARCH_DIR="$arg" ;;
  esac
done

# Colors (if terminal supports)
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "=== Template Sync All ==="
echo "Template: $TEMPLATE_DIR"
echo "Scanning: $SEARCH_DIR"
[ -n "$DRY_RUN" ] && echo "Mode: DRY RUN"
echo ""

UPDATED=0
CURRENT=0
CONFLICTS=0
FAILED=0
TOTAL=0

# Find all projects with template manifest
while IFS= read -r -d '' manifest; do
  PROJECT_DIR="$(dirname "$manifest")"
  PROJECT_NAME="$(basename "$PROJECT_DIR")"

  # Skip the template itself
  if [ "$PROJECT_DIR" = "$TEMPLATE_DIR" ]; then
    continue
  fi

  # Skip archived projects
  if echo "$PROJECT_DIR" | grep -q "_archive"; then
    continue
  fi

  TOTAL=$((TOTAL + 1))

  # Read current version
  CURRENT_VER=$(grep -o '"template_version"[[:space:]]*:[[:space:]]*"[^"]*"' "$manifest" 2>/dev/null | head -1 | sed 's/.*"\([^"]*\)"/\1/' || echo "unknown")

  # Read template version from CLAUDE.md (template itself doesn't have .template-manifest.json)
  TEMPLATE_VER=$(sed -n 's/.*Template Version: \([0-9.]*\).*/\1/p' "$TEMPLATE_DIR/CLAUDE.md" 2>/dev/null || echo "unknown")

  if [ "$CURRENT_VER" = "$TEMPLATE_VER" ]; then
    printf "${GREEN}✓${NC} %-30s v%s (current)\n" "$PROJECT_NAME" "$CURRENT_VER"
    CURRENT=$((CURRENT + 1))
  else
    if [ -f "$TEMPLATE_DIR/scripts/sync-template.sh" ]; then
      if [ -n "$DRY_RUN" ]; then
        printf "${YELLOW}~${NC} %-30s v%s → v%s (would update)\n" "$PROJECT_NAME" "$CURRENT_VER" "$TEMPLATE_VER"
        UPDATED=$((UPDATED + 1))
      else
        printf "  %-30s v%s → v%s ... " "$PROJECT_NAME" "$CURRENT_VER" "$TEMPLATE_VER"
        if bash "$TEMPLATE_DIR/scripts/sync-template.sh" "$TEMPLATE_DIR" --project-dir "$PROJECT_DIR" 2>/dev/null; then
          printf "${GREEN}✓ updated${NC}\n"
          UPDATED=$((UPDATED + 1))
        else
          printf "${RED}✗ failed${NC}\n"
          FAILED=$((FAILED + 1))
        fi
      fi
    else
      printf "${RED}✗${NC} %-30s sync-template.sh not found\n" "$PROJECT_NAME"
      FAILED=$((FAILED + 1))
    fi
  fi
done < <(find "$SEARCH_DIR" -maxdepth 3 -name ".template-manifest.json" -print0 2>/dev/null)

echo ""
echo "=== Summary ==="
echo "Total: $TOTAL projects"
echo "Updated: $UPDATED"
echo "Current: $CURRENT"
echo "Failed: $FAILED"
