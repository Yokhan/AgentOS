#!/bin/bash
# verify-check.sh — Automated verification checklist
# Usage: bash scripts/verify-check.sh [--size XS|S|M|L|XL]

SIZE="M"
if [ "$1" = "--size" ] && [ -n "$2" ]; then SIZE="$2"; fi

echo "=== VERIFICATION (size: $SIZE) ==="
echo ""

PASS=0
FAIL=0
MANUAL=0

check() {
  local label="$1" result="$2"
  if [ "$result" = "PASS" ]; then
    echo "  [AUTO] ✓ $label"
    PASS=$((PASS + 1))
  elif [ "$result" = "FAIL" ]; then
    echo "  [AUTO] ✗ $label"
    FAIL=$((FAIL + 1))
  else
    echo "  [MANUAL] → $label"
    MANUAL=$((MANUAL + 1))
  fi
}

# Modified files
MODIFIED=$(git diff --name-only HEAD 2>/dev/null)
STAGED=$(git diff --cached --name-only 2>/dev/null)
ALL_CHANGED=$(echo "$MODIFIED $STAGED" | tr ' ' '\n' | sort -u | grep -v "^$")

if [ -z "$ALL_CHANGED" ]; then
  echo "  No modified files detected."
else
  echo "Modified files:"
  echo "$ALL_CHANGED" | while read -r f; do echo "    $f"; done
  echo ""
fi

# Gate 0: File sizes
echo "--- Gate 0: File sizes ---"
for f in $ALL_CHANGED; do
  [ -f "$f" ] || continue
  lines=$(wc -l < "$f" | tr -d ' ')
  if [ "$lines" -gt 375 ]; then
    check "$f: $lines lines (limit 375)" "FAIL"
  else
    check "$f: $lines/375 lines" "PASS"
  fi
done

# Gate 0: Syntax
echo ""
echo "--- Gate 0: Syntax ---"
for f in $ALL_CHANGED; do
  [ -f "$f" ] || continue
  case "$f" in
    *.sh) bash -n "$f" 2>/dev/null && check "bash syntax: $f" "PASS" || check "bash syntax: $f" "FAIL" ;;
    *.json) node -e "JSON.parse(require('fs').readFileSync('$f','utf8'))" 2>/dev/null && check "JSON: $f" "PASS" || check "JSON: $f" "FAIL" ;;
  esac
done

# Gate 1: Intent check (S+)
if [ "$SIZE" != "XS" ]; then
  echo ""
  echo "--- Gate 1: Intent ---"
  check "Does this match the user's actual request?" "MANUAL"
  if [ -f tasks/current.md ]; then
    check "Plan in tasks/current.md updated?" "MANUAL"
  fi
fi

# Gate 2: Quality (M+)
if [ "$SIZE" = "M" ] || [ "$SIZE" = "L" ] || [ "$SIZE" = "XL" ]; then
  echo ""
  echo "--- Gate 2: Quality ---"

  # Tests exist for modified source files
  for f in $ALL_CHANGED; do
    case "$f" in
      *.test.*|*.spec.*|test_*) continue ;;
      src/*|lib/*|app/*)
        test_file=$(echo "$f" | sed 's/\.\([^.]*\)$/.test.\1/')
        [ -f "$test_file" ] && check "Tests exist: $test_file" "PASS" || check "Tests for $f?" "MANUAL"
        ;;
    esac
  done

  # Tool registry check
  if [ -f _reference/tool-registry.md ]; then
    check "New shared utils registered in tool-registry?" "MANUAL"
  fi

  check "What is the WEAKEST part of this solution?" "MANUAL"
  check "What alternative did you consider and reject?" "MANUAL"
fi

# Gate 3-4: Full (L/XL)
if [ "$SIZE" = "L" ] || [ "$SIZE" = "XL" ]; then
  echo ""
  echo "--- Gate 3-4: Full verification ---"
  check "Pre-mortem: if this fails, why?" "MANUAL"
  check "User checkpoint reached?" "MANUAL"
fi

# Summary
echo ""
echo "=== RESULT ==="
echo "Auto checks: $PASS passed, $FAIL failed"
echo "Manual checks: $MANUAL remaining"
if [ "$FAIL" -gt 0 ]; then
  echo "STATUS: FIX $FAIL auto-check failure(s) before proceeding."
else
  echo "STATUS: Auto checks PASS. Complete $MANUAL manual check(s)."
fi
