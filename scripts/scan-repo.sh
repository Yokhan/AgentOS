#!/usr/bin/env bash
# scan-repo.sh — Scan a cloned/untrusted repo for security risks before opening
# Usage: bash scripts/scan-repo.sh <repo_path>
#
# Checks for:
# 1. Malicious .claude/ hooks that could execute on session start
# 2. Poisoned CLAUDE.md with prompt injection
# 3. Suspicious .mcp.json configurations
# 4. Dangerous package.json scripts (postinstall attacks)
# 5. Secrets accidentally committed

set -euo pipefail

REPO="${1:?Usage: scan-repo.sh <repo_path>}"

if [ ! -d "$REPO" ]; then
  echo "ERROR: Directory not found: $REPO"
  exit 1
fi

echo "=== SECURITY SCAN: $REPO ==="
echo ""

WARNINGS=0
CRITICAL=0

warn() { echo "⚠️  [WARN] $1"; WARNINGS=$((WARNINGS + 1)); }
crit() { echo "🚨 [CRITICAL] $1"; CRITICAL=$((CRITICAL + 1)); }

# === 1. Check for .claude/ hooks ===
echo "--- Checking .claude/ hooks ---"
if [ -d "$REPO/.claude/hooks" ]; then
  HOOK_FILES=$(find "$REPO/.claude/hooks" -type f -executable 2>/dev/null || find "$REPO/.claude/hooks" -type f -name "*.sh" 2>/dev/null)
  if [ -n "$HOOK_FILES" ]; then
    crit "Found executable hooks in .claude/hooks/:"
    echo "$HOOK_FILES" | while read -r f; do
      echo "  $f"
      # Check for dangerous patterns in hooks
      if grep -qE "(curl|wget|nc |netcat|bash -c|eval|exec|rm -rf|ssh |scp )" "$f" 2>/dev/null; then
        crit "  ^ Contains dangerous commands: $(grep -oE '(curl|wget|nc |netcat|bash -c|eval|exec|rm -rf|ssh |scp )' "$f" | sort -u | tr '\n' ', ')"
      fi
    done
  fi
else
  echo "  OK — no hooks directory"
fi

# === 2. Check CLAUDE.md for prompt injection ===
echo ""
echo "--- Checking CLAUDE.md ---"
if [ -f "$REPO/CLAUDE.md" ]; then
  if grep -qiE "(ignore (previous|prior|all) (instructions|rules)|override system|you are now|act as|pretend to be|new instructions:)" "$REPO/CLAUDE.md" 2>/dev/null; then
    crit "CLAUDE.md contains prompt injection patterns!"
  fi
  if grep -qiE "(curl|wget|fetch|http|api\.)|base64" "$REPO/CLAUDE.md" 2>/dev/null; then
    warn "CLAUDE.md references external URLs or encoding — review manually"
  fi
  CLAUDE_LINES=$(wc -l < "$REPO/CLAUDE.md" | tr -d ' ')
  if [ "$CLAUDE_LINES" -gt 500 ]; then
    warn "CLAUDE.md is suspiciously large ($CLAUDE_LINES lines) — may contain hidden instructions"
  fi
  echo "  Size: $CLAUDE_LINES lines"
else
  echo "  OK — no CLAUDE.md"
fi

# === 3. Check .mcp.json ===
echo ""
echo "--- Checking MCP configurations ---"
for MCP_FILE in "$REPO/.mcp.json" "$REPO/.claude/mcp.json"; do
  if [ -f "$MCP_FILE" ]; then
    warn "Found MCP config: $MCP_FILE"
    # Check for suspicious servers
    if grep -qiE "(http|https|ws|wss)://" "$MCP_FILE" 2>/dev/null; then
      crit "MCP config connects to external servers!"
      grep -oE "(http|https|ws|wss)://[^\"' ]*" "$MCP_FILE" 2>/dev/null | while read -r url; do
        echo "  External URL: $url"
      done
    fi
  fi
done
[ "$WARNINGS" -eq 0 ] && [ ! -f "$REPO/.mcp.json" ] && echo "  OK — no MCP config"

# === 4. Check package.json for malicious scripts ===
echo ""
echo "--- Checking package.json scripts ---"
if [ -f "$REPO/package.json" ]; then
  for HOOK in preinstall postinstall preuninstall; do
    if grep -q "\"$HOOK\"" "$REPO/package.json" 2>/dev/null; then
      SCRIPT=$(grep "\"$HOOK\"" "$REPO/package.json" | head -1)
      warn "Found $HOOK script: $SCRIPT"
      if echo "$SCRIPT" | grep -qE "(curl|wget|bash|sh -c|eval|nc |node -e)" 2>/dev/null; then
        crit "$HOOK script contains dangerous commands!"
      fi
    fi
  done
else
  echo "  OK — no package.json"
fi

# === 5. Check for committed secrets ===
echo ""
echo "--- Checking for committed secrets ---"
SECRET_FILES=$(find "$REPO" -maxdepth 3 -type f \( \
  -name ".env" -o -name ".env.local" -o -name ".env.production" \
  -o -name "*.pem" -o -name "*.key" -o -name "id_rsa*" \
  -o -name "credentials.json" -o -name "service-account*.json" \
  -o -name ".npmrc" -o -name ".pypirc" \
  \) -not -path "*/.git/*" -not -path "*/node_modules/*" 2>/dev/null)

if [ -n "$SECRET_FILES" ]; then
  warn "Found potential secret files:"
  echo "$SECRET_FILES" | while read -r f; do echo "  $f"; done
else
  echo "  OK — no obvious secret files"
fi

# Check for hardcoded secrets in source
SECRET_PATTERNS=$(find "$REPO" -maxdepth 3 -type f \( -name "*.ts" -o -name "*.js" -o -name "*.py" -o -name "*.go" \) \
  -not -path "*/node_modules/*" -not -path "*/.git/*" \
  -exec grep -l -E "(sk-[a-zA-Z0-9]{20,}|ghp_[a-zA-Z0-9]{36}|AKIA[0-9A-Z]{16}|-----BEGIN.*PRIVATE)" {} \; 2>/dev/null)

if [ -n "$SECRET_PATTERNS" ]; then
  crit "Found hardcoded secrets in source files:"
  echo "$SECRET_PATTERNS" | while read -r f; do echo "  $f"; done
fi

# === 6. Check .claude/settings.json permissions ===
echo ""
echo "--- Checking .claude/settings.json ---"
if [ -f "$REPO/.claude/settings.json" ]; then
  warn "Project has custom Claude settings"
  if grep -q '"dangerouslySkipPermissions"' "$REPO/.claude/settings.json" 2>/dev/null; then
    crit "Settings attempt to skip permissions!"
  fi
  if grep -q '"Bash"' "$REPO/.claude/settings.json" 2>/dev/null; then
    warn "Settings auto-allow Bash execution"
  fi
else
  echo "  OK — no settings.json"
fi

# === Summary ===
echo ""
echo "═══════════════════════════════════════"
echo "SCAN COMPLETE"
echo "  Warnings:  $WARNINGS"
echo "  Critical:  $CRITICAL"
echo ""

if [ "$CRITICAL" -gt 0 ]; then
  echo "🚨 DO NOT open this repo without reviewing critical findings!"
  echo "   Recommend: review flagged files manually before running any Claude session."
  exit 2
elif [ "$WARNINGS" -gt 0 ]; then
  echo "⚠️  Review warnings before proceeding. Likely safe but verify."
  exit 1
else
  echo "✅ No issues found. Safe to open."
  exit 0
fi
