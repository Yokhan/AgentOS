#!/usr/bin/env bash
# sync-agents.sh — Validate CLAUDE.md and AGENTS.md reference the same shared rules
# Usage: bash scripts/sync-agents.sh
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
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

CLAUDE_MD="$PROJECT_DIR/CLAUDE.md"
AGENTS_MD="$PROJECT_DIR/AGENTS.md"
ISSUES=0
WARNINGS=0

echo "=== Agent Sync Check ==="

# 1. Check both files exist
if [[ ! -f "$CLAUDE_MD" ]]; then
  echo "ERROR: CLAUDE.md not found"
  ISSUES=$((ISSUES + 1))
fi
if [[ ! -f "$AGENTS_MD" ]]; then
  echo "ERROR: AGENTS.md not found"
  ISSUES=$((ISSUES + 1))
fi

if (( ISSUES > 0 )); then
  echo "FATAL: Missing instruction files. Cannot proceed."
  exit 1
fi

# 2. Check AGENTS.md size (Codex limit: 32KB)
AGENTS_SIZE=$(wc -c < "$AGENTS_MD" | tr -d ' ')
AGENTS_LIMIT=32768
echo ""
echo "--- Size Check ---"
echo "AGENTS.md: ${AGENTS_SIZE} bytes (limit: ${AGENTS_LIMIT})"
if (( AGENTS_SIZE > AGENTS_LIMIT )); then
  echo "ERROR: AGENTS.md exceeds Codex 32KB limit!"
  ISSUES=$((ISSUES + 1))
elif (( AGENTS_SIZE > AGENTS_LIMIT * 80 / 100 )); then
  echo "WARNING: AGENTS.md is >80% of limit"
  WARNINGS=$((WARNINGS + 1))
else
  echo "OK"
fi

# 3. Check template version match
echo ""
echo "--- Version Check ---"
CLAUDE_VER=$(grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' "$CLAUDE_MD" 2>/dev/null | head -1 || echo "unknown")
AGENTS_VER=$(grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' "$AGENTS_MD" 2>/dev/null | head -1 || echo "unknown")
echo "CLAUDE.md version: $CLAUDE_VER"
echo "AGENTS.md version: $AGENTS_VER"
if [[ "$CLAUDE_VER" != "$AGENTS_VER" ]]; then
  echo "ERROR: Version mismatch!"
  ISSUES=$((ISSUES + 1))
else
  echo "OK"
fi

# 4. Check shared library references
echo ""
echo "--- Library Reference Check ---"
CLAUDE_REFS=$(grep -oE '\.claude/library/[^ \`\)\"]+' "$CLAUDE_MD" 2>/dev/null | sort -u || true)
AGENTS_REFS=$(grep -oE '\.claude/library/[^ \`\)\"]+' "$AGENTS_MD" 2>/dev/null | sort -u || true)

# Rules in AGENTS.md but not CLAUDE.md (OK — Codex needs more context)
AGENTS_ONLY=$(comm -23 <(echo "$AGENTS_REFS") <(echo "$CLAUDE_REFS") 2>/dev/null || true)
if [[ -n "$AGENTS_ONLY" ]]; then
  echo "INFO: Rules in AGENTS.md only (OK — Codex needs more context):"
  echo "$AGENTS_ONLY" | sed 's/^/  /'
fi

# Check that referenced files exist
ALL_REFS=$(echo -e "${CLAUDE_REFS}\n${AGENTS_REFS}" | sort -u | grep -v '^$' || true)
for ref in $ALL_REFS; do
  if [[ ! -f "$PROJECT_DIR/$ref" ]]; then
    echo "ERROR: Referenced file missing: $ref"
    ISSUES=$((ISSUES + 1))
  fi
done
echo "Library files checked: $(echo "$ALL_REFS" | wc -l | tr -d ' ')"

# 5. Check SHARED_CONVENTIONS.md reference
echo ""
echo "--- Shared Conventions Check ---"
CONV_FILE="$PROJECT_DIR/docs/SHARED_CONVENTIONS.md"
if [[ ! -f "$CONV_FILE" ]]; then
  echo "ERROR: docs/SHARED_CONVENTIONS.md missing"
  ISSUES=$((ISSUES + 1))
else
  CLAUDE_HAS_CONV=$(grep -c "SHARED_CONVENTIONS" "$CLAUDE_MD" 2>/dev/null || true)
  AGENTS_HAS_CONV=$(grep -c "SHARED_CONVENTIONS" "$AGENTS_MD" 2>/dev/null || true)
  CLAUDE_HAS_CONV="${CLAUDE_HAS_CONV:-0}"
  AGENTS_HAS_CONV="${AGENTS_HAS_CONV:-0}"
  if (( CLAUDE_HAS_CONV == 0 )); then
    echo "WARNING: CLAUDE.md does not reference SHARED_CONVENTIONS.md"
    WARNINGS=$((WARNINGS + 1))
  fi
  if (( AGENTS_HAS_CONV == 0 )); then
    echo "WARNING: AGENTS.md does not reference SHARED_CONVENTIONS.md"
    WARNINGS=$((WARNINGS + 1))
  fi
  if (( CLAUDE_HAS_CONV > 0 && AGENTS_HAS_CONV > 0 )); then
    echo "OK: Both files reference SHARED_CONVENTIONS.md"
  fi
fi

# 6. Check DON'T section parity
echo ""
echo "--- DON'T Section Check ---"
CLAUDE_DONTS=$(sed -n "/^## DON'T/,/^##/p" "$CLAUDE_MD" 2>/dev/null | grep "^-" | wc -l | tr -d ' ')
AGENTS_DONTS=$(sed -n "/^## DON'T/,/^##/p" "$AGENTS_MD" 2>/dev/null | grep "^-" | wc -l | tr -d ' ')
echo "CLAUDE.md DON'T rules: $CLAUDE_DONTS"
echo "AGENTS.md DON'T rules: $AGENTS_DONTS"
DIFF=$((CLAUDE_DONTS - AGENTS_DONTS))
if (( DIFF < 0 )); then DIFF=$((-DIFF)); fi
if (( DIFF > 2 )); then
  echo "WARNING: DON'T sections differ by $DIFF rules — check for drift"
  WARNINGS=$((WARNINGS + 1))
else
  echo "OK (within tolerance)"
fi

# 7. Check Codex infrastructure
echo ""
echo "--- Codex Infrastructure Check ---"
for file in ".codex/config.toml" ".codex/hooks.json"; do
  if [[ -f "$PROJECT_DIR/$file" ]]; then
    echo "OK: $file exists"
  else
    echo "WARNING: $file missing"
    WARNINGS=$((WARNINGS + 1))
  fi
done

# Summary
echo ""
echo "=== Summary ==="
echo "Issues (must fix): $ISSUES"
echo "Warnings (should fix): $WARNINGS"

if (( ISSUES == 0 && WARNINGS == 0 )); then
  echo "Status: GREEN"
  exit 0
elif (( ISSUES == 0 )); then
  echo "Status: YELLOW"
  exit 0
else
  echo "Status: RED"
  exit 1
fi
