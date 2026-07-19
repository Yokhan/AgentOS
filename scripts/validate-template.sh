#!/bin/bash
# validate-template.sh — Pre-release validation for agent-project-template
# Checks: version consistency, file existence, syntax, portability, trust hardening
# Usage: bash scripts/validate-template.sh

set -euo pipefail

ERRORS=0
WARNINGS=0

is_template_source_repo() {
  grep -Eq '^- Name: agent-project-template$' PROJECT_SPEC.md 2>/dev/null
}

echo "=== Template Validation ==="
echo ""

# 1. Version consistency
echo "[1/11] Checking version consistency..."
CLAUDE_VER=$(sed -n 's/.*Template Version: \([0-9.]*\).*/\1/p' CLAUDE.md 2>/dev/null || echo "MISSING")
AGENTS_VER=$(sed -n 's/.*Template Version: \([0-9.]*\).*/\1/p' AGENTS.md 2>/dev/null || echo "MISSING")
DRIFT_VER=$(sed -n 's/.*TEMPLATE_VERSION="\([^"]*\)".*/\1/p' scripts/check-drift.sh 2>/dev/null || echo "MISSING")
README_VER=$(sed -n 's/.*template-v\([0-9.]*\).*/\1/p' README.md 2>/dev/null || echo "MISSING")

if [ "$CLAUDE_VER" = "$AGENTS_VER" ]; then
  echo "  OK: CLAUDE.md ($CLAUDE_VER) = AGENTS.md ($AGENTS_VER)"
else
  echo "  ERROR: CLAUDE.md ($CLAUDE_VER) != AGENTS.md ($AGENTS_VER)"
  ERRORS=$((ERRORS + 1))
fi

if [ "$CLAUDE_VER" = "$DRIFT_VER" ]; then
  echo "  OK: CLAUDE.md ($CLAUDE_VER) = check-drift.sh ($DRIFT_VER)"
else
  echo "  ERROR: CLAUDE.md ($CLAUDE_VER) != check-drift.sh ($DRIFT_VER)"
  ERRORS=$((ERRORS + 1))
fi

if [ "$CLAUDE_VER" = "$README_VER" ]; then
  echo "  OK: CLAUDE.md ($CLAUDE_VER) = README.md ($README_VER)"
else
  echo "  ERROR: CLAUDE.md ($CLAUDE_VER) != README.md ($README_VER)"
  ERRORS=$((ERRORS + 1))
fi

if [ -n "${EXPECTED_RELEASE_TAG:-}" ]; then
  EXPECTED_VER="${EXPECTED_RELEASE_TAG#v}"
  if [ "$CLAUDE_VER" = "$EXPECTED_VER" ] && [ "$AGENTS_VER" = "$EXPECTED_VER" ] &&
     [ "$DRIFT_VER" = "$EXPECTED_VER" ] && [ "$README_VER" = "$EXPECTED_VER" ]; then
    echo "  OK: release tag $EXPECTED_RELEASE_TAG matches declared version $EXPECTED_VER"
  else
    echo "  ERROR: release tag $EXPECTED_RELEASE_TAG does not match all declared versions"
    ERRORS=$((ERRORS + 1))
  fi
fi

# 2. Agent frontmatter
echo ""
echo "[2/11] Checking agent frontmatter..."
for agent in .claude/agents/*.md; do
  [ -f "$agent" ] || continue
  if [ "$(basename "$agent")" = "PROTOCOL.md" ]; then
    continue
  fi
  first=$(head -1 "$agent")
  if [ "$first" != "---" ]; then
    echo "  ERROR: $agent missing frontmatter"
    ERRORS=$((ERRORS + 1))
    continue
  fi
  has_name=$(head -10 "$agent" | grep -c "^name:" 2>/dev/null) || has_name=0
  has_model=$(head -10 "$agent" | grep -c "^model:" 2>/dev/null) || has_model=0
  if [ "$has_name" -eq 0 ]; then
    echo "  ERROR: $agent missing name field"
    ERRORS=$((ERRORS + 1))
  fi
  if [ "$has_model" -eq 0 ]; then
    echo "  ERROR: $agent missing model field"
    ERRORS=$((ERRORS + 1))
  fi
done
echo "  Checked $(ls .claude/agents/*.md 2>/dev/null | wc -l | tr -d ' ') agents"

# 3. Skill files exist
echo ""
echo "[3/11] Checking skills..."
SKILL_COUNT=0
for skill_dir in .claude/skills/*/; do
  [ -d "$skill_dir" ] || continue
  if [ ! -f "${skill_dir}SKILL.md" ]; then
    echo "  ERROR: $skill_dir missing SKILL.md"
    ERRORS=$((ERRORS + 1))
  fi
  SKILL_COUNT=$((SKILL_COUNT + 1))
done
echo "  Found $SKILL_COUNT Claude skills"

CODEX_SKILL_COUNT=0
for skill_dir in .agents/skills/*/; do
  [ -d "$skill_dir" ] || continue
  if [ ! -f "${skill_dir}SKILL.md" ]; then
    echo "  ERROR: $skill_dir missing SKILL.md"
    ERRORS=$((ERRORS + 1))
  fi
  CODEX_SKILL_COUNT=$((CODEX_SKILL_COUNT + 1))
done
if [ "$CODEX_SKILL_COUNT" -lt 42 ]; then
  echo "  ERROR: Expected at least 42 Codex skills, found $CODEX_SKILL_COUNT"
  ERRORS=$((ERRORS + 1))
else
  echo "  Found $CODEX_SKILL_COUNT Codex skills"
fi
if ! node scripts/validate-codex-skills.js >/dev/null 2>&1; then
  echo "  ERROR: Codex skill validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Codex skills validate"
fi

CODEX_AGENT_COUNT=0
for agent_file in .codex/agents/*.toml; do
  [ -f "$agent_file" ] || continue
  CODEX_AGENT_COUNT=$((CODEX_AGENT_COUNT + 1))
done
if [ "$CODEX_AGENT_COUNT" -lt 12 ]; then
  echo "  ERROR: Expected at least 12 Codex agents, found $CODEX_AGENT_COUNT"
  ERRORS=$((ERRORS + 1))
else
  echo "  Found $CODEX_AGENT_COUNT Codex agents"
fi
if ! node scripts/validate-codex-agents.js >/dev/null 2>&1; then
  echo "  ERROR: Codex agent validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Codex agents validate"
fi
if ! node scripts/test-codex-routing.js >/dev/null 2>&1; then
  echo "  ERROR: Codex routing smoke failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Codex routing smoke passes"
fi
if ! node scripts/test-change-strategy.js >/dev/null 2>&1; then
  echo "  ERROR: Change strategy validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Change strategy validates"
fi
if ! node scripts/test-code-intelligence-tools.js >/dev/null 2>&1 || ! node scripts/code-intelligence-tools.js validate >/dev/null 2>&1; then
  echo "  ERROR: Code-intelligence catalog validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Code-intelligence catalog and selection validate"
fi
if ! node scripts/test-writing-references.js >/dev/null 2>&1 || ! node scripts/validate-writing-references.js >/dev/null 2>&1; then
  echo "  ERROR: Writing reference registry validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Writing references and project overlay validate"
fi
if ! node scripts/validate-production-standard.js >/dev/null 2>&1; then
  echo "  ERROR: Production standard validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Production standard validates"
fi
if ! node scripts/validate-agent-sot.js >/dev/null 2>&1; then
  echo "  ERROR: Agent SOT validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Agent SOT validates"
fi
if ! node scripts/validate-spec-kit.js >/dev/null 2>&1; then
  echo "  ERROR: Spec Kit snapshot validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Spec Kit snapshot validates"
fi
if ! node scripts/validate-text-policy.js >/dev/null 2>&1; then
  echo "  ERROR: Text policy validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Text policy validates"
fi
if ! node scripts/progressive-status.js --check >/dev/null 2>&1; then
  echo "  ERROR: Progressive status validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Progressive status validates"
fi
if ! node scripts/test-progressive-plan.js >/dev/null 2>&1; then
  echo "  ERROR: Progressive plan validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Progressive plan validates"
fi
if ! node scripts/test-subagent-trace.js >/dev/null 2>&1; then
  echo "  ERROR: Subagent trace validation failed"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: Subagent trace validates"
fi

# 4. Script syntax
echo ""
echo "[4/11] Checking script syntax..."
for script in scripts/*.sh scripts/lib/*.sh .claude/hooks/*.sh; do
  [ -f "$script" ] || continue
  if ! bash -n "$script" 2>/dev/null; then
    echo "  ERROR: $script has syntax errors"
    ERRORS=$((ERRORS + 1))
  fi
done
for script in scripts/*.js; do
  [ -f "$script" ] || continue
  if ! node --check "$script" >/dev/null 2>&1; then
    echo "  ERROR: $script has syntax errors"
    ERRORS=$((ERRORS + 1))
  fi
done
echo "  All scripts pass syntax check"

# 5. Required files exist
echo ""
echo "[5/11] Checking required files..."
REQUIRED_FILES=(
  "AGENTS.md"
  "CLAUDE.md"
  "README.md"
  "PROJECT_SPEC.md"
  "ecosystem.md"
  "docs/AGENT_PIPELINES.md"
  "docs/AGENT_CONTEXT_SOT.md"
  "docs/CODEX_FANOUT_PATTERNS.md"
  "docs/CODEX_SKILLS_AUDIT.md"
  "docs/CODEX_SUBAGENTS_AUDIT.md"
  "docs/MIGRATION_MATRIX.md"
  "docs/OPENAI_MODEL_GUIDANCE.md"
  "docs/CODE_INTELLIGENCE_TOOLCHAIN.md"
  "docs/PRODUCT_BOUNDARY.md"
  "docs/RELEASE_CHECKLIST.md"
  "docs/TEMPLATE_RELEASES.md"
  "docs/SAFE_DEFAULTS.md"
  "docs/SUPPORTED_ENVIRONMENTS.md"
  ".claude/library/product/production-product-standard.md"
  ".claude/library/process/product-goal-loop.md"
  ".claude/library/process/client-executor-contract.md"
  ".claude/library/process/change-strategy-gate.md"
  ".agents/skills/codex-change-strategy/SKILL.md"
  ".claude/library/domain/domain-design-system.md"
  "integrations/spec-kit/README.md"
  "scripts/validate-codex-skills.js"
  "scripts/validate-codex-agents.js"
  "scripts/validate-production-standard.js"
  "scripts/validate-change-strategy.js"
  "scripts/lib/change-strategy-policy.js"
  "scripts/lib/codex-discovery-reroute.js"
  "scripts/lib/codex-route-cli.js"
  "scripts/test-change-strategy.js"
  "scripts/code-intelligence-tools.js"
  "scripts/lib/code-intelligence-policy.js"
  "scripts/test-code-intelligence-tools.js"
  "scripts/configure-codex-mcp.js"
  "scripts/test-codex-mcp-config.js"
  "scripts/validate-writing-references.js"
  "scripts/validate-agent-sot.js"
  "scripts/validate-spec-kit.js"
  "scripts/validate-text-policy.js"
  "scripts/sync-spec-kit.sh"
  "scripts/init-spec-kit.sh"
  "scripts/codex-agent-policy.js"
  "scripts/codex-routing-cases-a.js"
  "scripts/codex-routing-cases-b.js"
  "scripts/codex-route-config.js"
  "scripts/codex-route-task.js"
  "scripts/lib/codex-route-summary.js"
  "tests/fixtures/change-strategy/discovery-architecture-mismatch.json"
  "scripts/test-codex-routing.js"
  "scripts/test-codex-agent-policy.js"
  "scripts/test-codex-subagents-live.sh"
  "_reference/README.md"
  "_reference/tool-registry.md"
  "_reference/code-intelligence-tools.json"
  "_reference/codex-mcp-config.toml"
  "_reference/agent-sot/README.md"
  "_reference/agent-sot/sources.json"
  "_reference/agent-sot/top-works.md"
  "_reference/agent-sot/originals/ai-agent-spec-v3-final.md"
  "_reference/spec-kit/README.md"
  "_reference/spec-kit/manifest.json"
  "_reference/spec-kit/upstream/templates/commands/specify.md"
  "_reference/spec-kit/upstream/templates/commands/plan.md"
  "_reference/spec-kit/upstream/templates/commands/tasks.md"
  "_reference/spec-kit/upstream/templates/commands/implement.md"
  ".claude/settings.json"
  "tasks/lessons.md"
  "tasks/current.md"
  "tasks/goal.md"
  "tasks/.research-cache.md"
  "tasks/post-mortems/TEMPLATE.md"
)
for f in "${REQUIRED_FILES[@]}"; do
  if [ ! -f "$f" ]; then
    echo "  ERROR: Missing required file: $f"
    ERRORS=$((ERRORS + 1))
  fi
done
if is_template_source_repo; then
  if [ ! -f ".github/workflows/release-template.yml" ]; then
    echo "  ERROR: Missing source-only release workflow: .github/workflows/release-template.yml"
    ERRORS=$((ERRORS + 1))
  else
    echo "  OK: source-only release workflow present"
  fi
else
  echo "  OK: source-only release workflow skipped for downstream"
fi
echo "  Checked ${#REQUIRED_FILES[@]} required files"
if ! node -e "const h=JSON.parse(require('fs').readFileSync('.codex/hooks.json','utf8')); for (const event of ['SessionStart','PreToolUse','PostToolUse','Stop']) { if (!Array.isArray(h.hooks?.[event]) || !h.hooks[event][0]?.hooks?.[0]?.command) process.exit(1); }" >/dev/null 2>&1; then
  echo "  ERROR: .codex/hooks.json must use Codex nested hook command schema"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: .codex/hooks.json uses Codex hook schema"
fi

# 6. Cross-platform safety: no bare python3, no <<<
echo ""
echo "[6/11] Checking cross-platform safety..."
# Direct python/python3 calls (should use _node or node)
BARE_PY=$(grep -rn "python3\? -[cm]" scripts/*.sh .claude/hooks/*.sh 2>/dev/null | grep -v "command -v" | grep -v "^#" | grep -v "platform.sh" || true)
if [ -n "$BARE_PY" ]; then
  echo "  ERROR: Python calls found (should use node/_node instead):"
  echo "$BARE_PY" | head -5
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: No Python calls (using Node.js)"
fi
# grep -P (Perl regex, not portable)
GREP_P=$(grep -rn "grep -[a-zA-Z]*P" scripts/*.sh .claude/hooks/*.sh setup.sh 2>/dev/null | grep -v "^scripts/validate-template.sh:" || true)
if [ -n "$GREP_P" ]; then
  echo "  ERROR: grep -P (Perl regex) found (not portable, use sed):"
  echo "$GREP_P" | head -5
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: No grep -P"
fi
# Here-strings (don't work on all Windows bash)
HERESTR=$(grep -rn "<<<" scripts/*.sh .claude/hooks/*.sh 2>/dev/null | grep -v "^scripts/validate-template.sh:" || true)
if [ -n "$HERESTR" ]; then
  echo "  ERROR: Here-strings (<<<) found (use pipe or process substitution):"
  echo "$HERESTR" | head -5
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: No here-strings"
fi

# 7. Library structure + MCP server
echo ""
echo "[7/11] Checking library structure..."
if [ -d ".claude/library" ]; then
  LIB_COUNT=$(find .claude/library -name "*.md" 2>/dev/null | wc -l | tr -d ' ')
  echo "  OK: .claude/library/ has $LIB_COUNT rule files"
  for subdir in process technical meta domain conflict; do
    if [ ! -d ".claude/library/$subdir" ]; then
      echo "  WARNING: .claude/library/$subdir/ missing"
      WARNINGS=$((WARNINGS + 1))
    fi
  done
  for writing_file in \
    .claude/library/technical/writing.md \
    .claude/library/technical/writing-mode-profiles.md \
    .claude/library/technical/russian-writing-profile.md \
    .claude/library/technical/russian-business-correspondence.md \
    .claude/library/technical/russian-explanation-and-persuasion.md \
    .claude/skills/writing-workflow/SKILL.md \
    .agents/skills/codex-writing-workflow/SKILL.md \
    scripts/lib/writing-intent.js; do
    if [ ! -f "$writing_file" ]; then
      echo "  ERROR: writing workflow file missing: $writing_file"
      ERRORS=$((ERRORS + 1))
    fi
  done
else
  echo "  ERROR: .claude/library/ not found"
  ERRORS=$((ERRORS + 1))
fi

echo ""
echo "[8/11] Checking MCP server..."
if [ -f "mcp-servers/context-router/package.json" ]; then
  echo "  OK: context-router package.json exists"
  if [ -f "mcp-servers/context-router/src/index.ts" ]; then
    echo "  OK: context-router src/index.ts exists"
  else
    echo "  ERROR: context-router src/index.ts missing"
    ERRORS=$((ERRORS + 1))
  fi
else
  echo "  WARNING: No MCP context-router (fallback to bash scripts)"
  WARNINGS=$((WARNINGS + 1))
fi

# 9. Counts
echo ""
echo "[9/11] Checking counts..."
CMD_COUNT=$(ls .claude/commands/*.md 2>/dev/null | wc -l | tr -d ' ')
AGENT_COUNT=$(ls .claude/agents/*.md 2>/dev/null | wc -l | tr -d ' ')
RULE_COUNT=$(ls .claude/rules/*.md 2>/dev/null | wc -l | tr -d ' ')
echo "  Commands: $CMD_COUNT"
echo "  Agents: $AGENT_COUNT"
echo "  Rules: $RULE_COUNT"
echo "  Claude Skills: $SKILL_COUNT"
echo "  Codex Skills: $CODEX_SKILL_COUNT"
echo "  Codex Agents: $CODEX_AGENT_COUNT"

# 10. Platform lib
echo ""
echo "[10/11] Checking platform.sh..."
if [ -f "scripts/lib/platform.sh" ]; then
  bash -n scripts/lib/platform.sh 2>/dev/null && echo "  OK: platform.sh valid" || { echo "  ERROR: platform.sh syntax error"; ERRORS=$((ERRORS + 1)); }
else
  echo "  ERROR: scripts/lib/platform.sh missing"
  ERRORS=$((ERRORS + 1))
fi

# 11. Trust hardening
echo ""
echo "[11/11] Checking trust hardening..."

if git ls-files --error-unmatch .claude/settings.local.json >/dev/null 2>&1; then
  echo "  ERROR: .claude/settings.local.json is tracked but must stay local-only"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: No tracked project-local Claude settings"
fi

if grep -Eq '^(model|model_reasoning_effort|approval_policy|sandbox_mode)\s*=' .codex/config.toml 2>/dev/null; then
  echo "  ERROR: .codex/config.toml contains user/IDE-owned defaults"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: .codex/config.toml keeps only project-specific Codex settings"
fi

AGENTS_BYTES=$(wc -c < AGENTS.md 2>/dev/null || echo 0)
if [ "$AGENTS_BYTES" -gt 32768 ]; then
  echo "  ERROR: AGENTS.md is $AGENTS_BYTES bytes, above Codex 32KB guidance"
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: AGENTS.md size $AGENTS_BYTES bytes"
fi

TRACKED_ARTIFACTS=$(git ls-files "mcp-servers/*/node_modules/*" "mcp-servers/*/dist/*" "docs/.ci-untracked-sentinel.txt" "docs/.setup-leak-sentinel*" 2>/dev/null || true)
if [ -n "$TRACKED_ARTIFACTS" ]; then
  echo "  ERROR: Generated or local-only artifacts are tracked:"
  echo "$TRACKED_ARTIFACTS" | head -5
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: No tracked dependency/cache artifacts in shipped roots"
fi

USER_NAME="$(basename "${HOME:-}")"
ABSOLUTE_PATH_MATCHES=""
for pattern in \
  "${HOME:-}" \
  "${PWD:-}" \
  "C:/Users/$USER_NAME" \
  "C:\\\\Users\\\\$USER_NAME" \
  "/Users/$USER_NAME" \
  "/home/$USER_NAME"
do
  [ -n "$pattern" ] || continue
  MATCHES=$(git grep -n -I -F "$pattern" -- . ':(exclude).git' 2>/dev/null || true)
  if [ -n "$MATCHES" ]; then
    ABSOLUTE_PATH_MATCHES="${ABSOLUTE_PATH_MATCHES}${MATCHES}"$'\n'
  fi
done

if [ -n "$ABSOLUTE_PATH_MATCHES" ]; then
  echo "  ERROR: Personal machine paths leaked into tracked files:"
  printf '%s' "$ABSOLUTE_PATH_MATCHES" | head -5
  ERRORS=$((ERRORS + 1))
else
  echo "  OK: No personal machine paths in tracked files"
fi

# Summary
echo ""
echo "=== Validation Summary ==="
echo "Errors: $ERRORS"
echo "Warnings: $WARNINGS"

if [ "$ERRORS" -gt 0 ]; then
  echo "FAIL: Fix $ERRORS error(s) before release."
  exit 1
else
  echo "PASS: Template validation subset passed. Run the aggregate release workflow before publishing."
  exit 0
fi
