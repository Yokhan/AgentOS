#!/bin/bash
# Drift Detection Script (v2)
# Run before major sessions or weekly to catch stale docs and violations
# Usage: bash scripts/check-drift.sh

# shellcheck source=lib/platform.sh
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
# Node.js is used for JSON parsing (Python removed)
if [ -z "${NODE:-}" ]; then
  command -v node &>/dev/null && NODE="node" || NODE=""
fi

# Template version check
TEMPLATE_VERSION="4.9.0"
CLAUDE_VERSION=$(sed -n 's/.*Template Version: \([0-9.]*\).*/\1/p' CLAUDE.md 2>/dev/null || true)
if [ -z "$CLAUDE_VERSION" ]; then
    CLAUDE_VERSION="unknown"
fi
if [ "$CLAUDE_VERSION" = "unknown" ]; then
    echo "INFO: Template version not found in CLAUDE.md"
elif [ "$CLAUDE_VERSION" != "$TEMPLATE_VERSION" ]; then
    echo "WARNING: Template version mismatch. CLAUDE.md=$CLAUDE_VERSION, script expects=$TEMPLATE_VERSION"
fi

echo "=== Drift Detection ==="
echo ""
WARNINGS=0
ERRORS=0

# 1. Check if docs are stale (>30 days)
echo "[1/12] Checking document freshness..."
if [ -d docs ]; then
  for doc in docs/*.md; do
    [ -f "$doc" ] || continue
    mtime=$(_stat_mtime "$doc" 2>/dev/null || date +%s)
    age=$(( ($(date +%s) - mtime) / 86400 ))
    if [ "$age" -gt 30 ]; then
      echo "  ⚠️  $doc not updated in $age days"
      WARNINGS=$((WARNINGS + 1))
    fi
  done
fi

if [ -d src ] && [ -d docs ]; then
    echo "Checking src/ vs docs/ freshness..."
    # Check if src has files newer than docs (simplified check)
    newest_src=$(find src -name "*.ts" -o -name "*.py" -o -name "*.go" -o -name "*.rs" 2>/dev/null | head -1)
    if [ -n "$newest_src" ] && [ -d docs ]; then
        echo "  INFO: Remember to update docs/ when src/ changes significantly"
    fi
fi

# 2. Check CLAUDE.md size
echo "[2/12] Checking CLAUDE.md size..."
if [ -f CLAUDE.md ]; then
  lines=$(wc -l < CLAUDE.md)
  if [ "$lines" -gt 300 ]; then
    echo "  ⚠️  CLAUDE.md: $lines lines (limit 300)"
    WARNINGS=$((WARNINGS + 1))
  else
    echo "  ✅ CLAUDE.md: $lines lines (OK)"
  fi
fi

# 3. Check lessons.md size (>50 = time to promote)
echo "[3/12] Checking lessons.md..."
if [ -f tasks/lessons.md ]; then
  entries=$(grep -c "^### " tasks/lessons.md 2>/dev/null) || entries=0
  if [ "$entries" -gt 50 ]; then
    echo "  ⚠️  tasks/lessons.md has $entries entries — run /weekly to promote"
    WARNINGS=$((WARNINGS + 1))
  else
    echo "  ✅ tasks/lessons.md: $entries entries"
  fi
fi

# 4. Check for files > 375 lines in src/
echo "[4/12] Checking file sizes in src/..."
if [ -d src ]; then
  find src -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.py" -o -name "*.rs" -o -name "*.go" -o -name "*.js" -o -name "*.jsx" \) | while read -r file; do
    lines=$(wc -l < "$file")
    if [ "$lines" -gt 375 ]; then
      echo "  ⚠️  $file: $lines lines (limit 375)"
    fi
  done
fi

# 5. Check module entry points exist
echo "[5/12] Checking module entry points..."
if [ -d src/features ]; then
  find src/features -mindepth 1 -maxdepth 1 -type d | while read -r dir; do
    found=0
    for entry in "$dir"/index.* "$dir"/__init__.py "$dir"/mod.rs; do
      [ -f "$entry" ] && found=1 && break
    done
    if [ "$found" -eq 0 ]; then
      echo "  ❌ Missing entry point: $dir"
    fi
  done
fi

# 6. Check architecture boundaries
echo "[6/12] Checking architecture boundaries..."
if command -v npx &> /dev/null && [ -f .dependency-cruiser.js ]; then
  npx dependency-cruiser src --output-type err 2>/dev/null || echo "  ⚠️  Boundary violations detected"
else
  echo "  ℹ️  dependency-cruiser not configured (optional)"
fi

# 7. Check for secrets in tracked files
echo "[7/12] Scanning for potential secrets..."
if git rev-parse --git-dir > /dev/null 2>&1; then
  secrets=$(grep -rlE '(sk-[a-zA-Z0-9]{20,}|ghp_[a-zA-Z0-9]{36}|-----BEGIN.*(RSA|EC|DSA))' src/ 2>/dev/null || true)
  if [ -n "$secrets" ]; then
    echo "  ❌ Potential secrets found in: $secrets"
    ERRORS=$((ERRORS + 1))
  else
    echo "  ✅ No secrets detected"
  fi
fi

# 8. Check text and platform policy
echo "[8/12] Checking text and platform policy..."
if node scripts/validate-text-policy.js >/dev/null 2>&1; then
  echo "  OK: Text policy validates"
else
  echo "  ERROR: Text policy validation failed"
  ERRORS=$((ERRORS + 1))
fi

# 9. Check template manifest integrity
echo "[9/12] Checking template manifest..."
MANIFEST=".template-manifest.json"
if [ -f "$MANIFEST" ]; then
  # Validate JSON
  if _json_valid "$MANIFEST"; then
    echo "  ✅ $MANIFEST: valid JSON"
  else
    echo "  ❌ $MANIFEST: invalid JSON"
    ERRORS=$((ERRORS + 1))
  fi

  # Report template version
  tpl_ver=$(_node -e "console.log(JSON.parse(require('fs').readFileSync('$MANIFEST','utf8')).template_version||'unknown')" 2>/dev/null || echo "unknown")
  echo "  Template version (manifest): $tpl_ver"

  drift_result=$(_node -e "
const fs=require('fs');
const crypto=require('crypto');
const m=JSON.parse(fs.readFileSync('$MANIFEST','utf8'));
let drift=0;
let total=0;
for (const [file, info] of Object.entries(m.files || {})) {
  if (info.category === 'project') continue;
  total += 1;
  if (!fs.existsSync(file) || !fs.statSync(file).isFile()) {
    drift += 1;
    continue;
  }
  const actual=crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
  if (actual !== (info.hash || '')) drift += 1;
}
console.log(drift + '|' + total);
" 2>/dev/null || echo "1|0")
  drift_count=${drift_result%%|*}
  drift_total=${drift_result##*|}

  if [ "$drift_count" -gt 0 ]; then
    echo "  ⚠️  $drift_count/$drift_total template files have drifted from manifest hashes"
    WARNINGS=$((WARNINGS + 1))
  else
    echo "  ✅ All $drift_total template files match manifest hashes"
  fi
else
  echo "  ℹ️  No $MANIFEST found (sync not configured)"
fi

# 10. Check template rules not modified locally (read-only enforcement)
echo "[10/12] Checking template rules integrity..."
if [ -f "$MANIFEST" ]; then
  RULE_DRIFT_OUTPUT=$(_node -e "
const fs=require('fs');
const crypto=require('crypto');
const path=require('path');
const m=JSON.parse(fs.readFileSync('$MANIFEST','utf8'));
const patterns=[
  /^\\.claude\\/rules\\/[^/]+\\.md$/,
  /^\\.claude\\/library\\/[^/]+\\/[^/]+\\.md$/,
  /^\\.claude\\/agents\\/[^/]+\\.md$/,
  /^\\.agents\\/skills\\/[^/]+\\/SKILL\\.md$/,
  /^\\.agents\\/skills\\/[^/]+\\/agents\\/openai\\.yaml$/,
  /^\\.agents\\/skills\\/[^/]+\\/references\\/[^/]+\\.md$/,
  /^\\.codex\\/agents\\/[^/]+\\.toml$/,
];
let drift=0;
for (const [file, info] of Object.entries(m.files || {})) {
  const base=path.basename(file);
  if (base.startsWith('project-') || file.startsWith('.agents/skills/project-') || file.startsWith('.codex/agents/project-')) continue;
  if (!patterns.some((pattern) => pattern.test(file))) continue;
  if (!fs.existsSync(file) || !fs.statSync(file).isFile()) continue;
  const actual=crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
  if (actual !== (info.hash || '')) {
    console.log(file);
    drift += 1;
  }
}
console.error(drift);
" 2> .check-drift-rule-count.tmp)
  RULE_DRIFT=$(cat .check-drift-rule-count.tmp 2>/dev/null || echo 0)
  rm -f .check-drift-rule-count.tmp
  if [ -n "$RULE_DRIFT_OUTPUT" ]; then
    printf '%s\n' "$RULE_DRIFT_OUTPUT" | while IFS= read -r rule_file; do
      [ -n "$rule_file" ] && echo "  WARNING: $rule_file modified locally (template file should be read-only)"
    done
  fi
  for rule_file in; do
    [ -f "$rule_file" ] || continue
    # Skip project-* files (those ARE meant to be local)
    basename_f=$(basename "$rule_file")
    case "$basename_f" in project-*) continue ;; esac
    case "$rule_file" in .agents/skills/project-*/*) continue ;; esac
    case "$rule_file" in .codex/agents/project-*) continue ;; esac
    # Check if file is in manifest and hash matches
    EXPECTED_HASH=$(_node -e "
const m=JSON.parse(require('fs').readFileSync('$MANIFEST','utf8'));
const h=(m.files||{})['$rule_file']?.hash||'';console.log(h);
" 2>/dev/null || echo "")
    if [ -n "$EXPECTED_HASH" ]; then
      ACTUAL_HASH=$(_get_hash "$rule_file")
      if [ "$ACTUAL_HASH" != "$EXPECTED_HASH" ]; then
        echo "  ⚠️  $rule_file modified locally (template file — should be read-only)"
        RULE_DRIFT=$((RULE_DRIFT + 1))
      fi
    fi
  done
  if [ "$RULE_DRIFT" -gt 0 ]; then
    echo "  ⚠️  $RULE_DRIFT template rule(s) modified locally. Run /update-template to restore."
    WARNINGS=$((WARNINGS + RULE_DRIFT))
  else
    echo "  ✅ All template rules/agents match manifest"
  fi
else
  echo "  ℹ️  No manifest — cannot check rule integrity"
fi

# 11. Check tool registry health
echo "[11/12] Checking tool registry..."
REGISTRY="_reference/tool-registry.md"
if [ -f "$REGISTRY" ]; then
  # Check for stale entries (referenced paths that don't exist)
  STALE_TOOLS=$(_node -e "
const fs=require('fs');
const registry=fs.readFileSync('$REGISTRY','utf8').split(/\\r?\\n/);
let section=false;
let stale=0;
for (const line of registry) {
  if (/^## (Template-Level|Project-Level|Helpers & Utilities)/.test(line)) {
    section=true;
    continue;
  }
  if (/^## /.test(line)) {
    section=false;
    continue;
  }
  if (!section || !line.startsWith('|')) continue;
  const cols=line.split('|').map((item) => item.trim());
  const value=cols[2] || '';
  if (!value || value === 'Path' || value === 'Signature' || /^-+$/.test(value) || value.startsWith('_')) continue;
  if (!fs.existsSync(value)) stale += 1;
}
console.log(stale);
" 2>/dev/null || echo 0)
  : <<'SKIP_TOOL_REGISTRY_BASH_LOOP'
  while IFS='|' read -r kind value; do
    kind=$(printf "%s" "$kind" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
    value=$(printf "%s" "$value" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
    [ "$kind" = "PATH" ] || continue
    [ -z "$value" ] && continue
    if [ ! -e "$value" ]; then
      STALE_TOOLS=$((STALE_TOOLS + 1))
    fi
  done < <(
    awk '
      /^## Template-Level/ { section="tool"; next }
      /^## Project-Level/ { section="tool"; next }
      /^## Helpers & Utilities/ { section="tool"; next }
      /^## / { section=""; next }
      section != "tool" { next }
      /^\|/ {
        split($0, cols, "|")
        path = cols[3]
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", path)
        if (path == "" || path == "Path" || path == "Signature") {
          next
        }
        if (path ~ /^-+$/) {
          next
        }
        if (path ~ /^_/) {
          next
        }
        print "PATH|" path
      }
    ' "$REGISTRY" 2>/dev/null
  )
SKIP_TOOL_REGISTRY_BASH_LOOP

  if [ "$STALE_TOOLS" -gt 0 ]; then
    echo "  ⚠️  $STALE_TOOLS stale entries in tool registry (files deleted). Run: bash scripts/audit-reuse.sh"
    WARNINGS=$((WARNINGS + 1))
  else
    echo "  ✅ Tool registry: no stale entries"
  fi

  # Check if registry is too empty for a project with src/
  if [ -d src ]; then
    ENTRIES=$(grep -cE "^\| [^_|]" "$REGISTRY" 2>/dev/null) || ENTRIES=0
    if [ "$ENTRIES" -lt 8 ]; then
      echo "  ⚠️  Tool registry has only $ENTRIES entries. Run: bash scripts/scan-project.sh"
      WARNINGS=$((WARNINGS + 1))
    fi
  fi
else
  if [ -d src ]; then
    echo "  ⚠️  No tool registry found. Run: bash scripts/scan-project.sh"
    WARNINGS=$((WARNINGS + 1))
  else
    echo "  ℹ️  No tool registry (no src/ directory)"
  fi
fi

echo "[12/12] Checking trust defaults..."
if git ls-files --error-unmatch .claude/settings.local.json >/dev/null 2>&1; then
  echo "  ❌ .claude/settings.local.json is tracked. It must stay local-only."
  ERRORS=$((ERRORS + 1))
elif [ -f ".claude/settings.local.json" ]; then
  echo "  ℹ️  .claude/settings.local.json exists locally but is not tracked"
else
  echo "  ✅ No tracked project-local Claude settings in template root"
fi

if grep -Eq '^(model|model_reasoning_effort|approval_policy|sandbox_mode)\s*=' .codex/config.toml 2>/dev/null; then
  echo "  ❌ .codex/config.toml contains IDE/user-owned defaults"
  ERRORS=$((ERRORS + 1))
else
  echo "  ✅ .codex/config.toml keeps only template-safe settings"
fi

if [ -f "scripts/validate-codex-skills.js" ]; then
  if node scripts/validate-codex-skills.js >/dev/null 2>&1; then
    echo "  ✅ Codex skills validate"
  else
    echo "  ❌ Codex skill validation failed"
    ERRORS=$((ERRORS + 1))
  fi
else
  echo "  ⚠️  Codex skill validator missing"
  WARNINGS=$((WARNINGS + 1))
fi

if [ -f "scripts/validate-codex-agents.js" ]; then
  if node scripts/validate-codex-agents.js >/dev/null 2>&1; then
    echo "  ✅ Codex agents validate"
  else
    echo "  ❌ Codex agent validation failed"
    ERRORS=$((ERRORS + 1))
  fi
else
  echo "  ⚠️  Codex agent validator missing"
  WARNINGS=$((WARNINGS + 1))
fi

if [ -f "scripts/test-codex-routing.js" ]; then
  if node scripts/test-codex-routing.js >/dev/null 2>&1; then
    echo "  ✅ Codex route smoke passes"
  else
    echo "  ❌ Codex route smoke failed"
    ERRORS=$((ERRORS + 1))
  fi
else
  echo "  ⚠️  Codex route smoke missing"
  WARNINGS=$((WARNINGS + 1))
fi

if [ -f "scripts/validate-agent-sot.js" ]; then
  if node scripts/validate-agent-sot.js >/dev/null 2>&1; then
    echo "  ✅ Agent SOT validates"
  else
    echo "  ❌ Agent SOT validation failed"
    ERRORS=$((ERRORS + 1))
  fi
else
  echo "  ⚠️  Agent SOT validator missing"
  WARNINGS=$((WARNINGS + 1))
fi

if [ -f "scripts/validate-spec-kit.js" ]; then
  if node scripts/validate-spec-kit.js >/dev/null 2>&1; then
    echo "  ✅ Spec Kit snapshot validates"
  else
    echo "  ❌ Spec Kit snapshot validation failed"
    ERRORS=$((ERRORS + 1))
  fi
else
  echo "  ⚠️  Spec Kit snapshot validator missing"
  WARNINGS=$((WARNINGS + 1))
fi

if [ -f "docs/PRODUCT_BOUNDARY.md" ] && [ -f "docs/SAFE_DEFAULTS.md" ] && [ -f "docs/SUPPORTED_ENVIRONMENTS.md" ]; then
  echo "  ✅ Trust and environment docs present"
else
  echo "  ⚠️  Trust/environment docs missing. Re-run release hardening slice."
  WARNINGS=$((WARNINGS + 1))
fi

USER_NAME="$(basename "${HOME:-}")"
PATH_LEAKS=""
for pattern in "$HOME" "$PWD" "C:/Users/$USER_NAME" "C:\\Users\\$USER_NAME" "/Users/$USER_NAME" "/home/$USER_NAME"; do
  [ -n "$pattern" ] || continue
  MATCHES=$(git grep -n -I -F "$pattern" -- . ':(exclude).git' 2>/dev/null || true)
  if [ -n "$MATCHES" ]; then
    PATH_LEAKS="${PATH_LEAKS}${MATCHES}"$'\n'
  fi
done

if [ -n "$PATH_LEAKS" ]; then
  echo "  ❌ Personal machine paths found in tracked files:"
  printf '%s' "$PATH_LEAKS" | head -5
  ERRORS=$((ERRORS + 1))
else
  echo "  ✅ No personal machine paths in tracked files"
fi

echo ""
echo "=== Summary: $WARNINGS warnings, $ERRORS errors ==="
[ "$ERRORS" -gt 0 ] && echo "❌ Fix errors before proceeding" && exit 1
echo "✅ Done"
