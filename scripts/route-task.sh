#!/bin/bash
# route-task.sh — Dynamic task router (zero AI tokens)
# Routes task to relevant rules, agent, and pipeline using the shared writing classifier plus grep fallbacks.
# Usage: bash scripts/route-task.sh "implement OAuth for auth module"
# Output: rule files to Read + suggested agent → saved to tasks/.active-rules
# Called: on EVERY new task, on task switch, on mode change

TASK="${1:-}"
LIB=".claude/library"
ACTIVE="tasks/.active-rules"

if [ -z "$TASK" ]; then
  echo "Usage: bash scripts/route-task.sh \"<task description>\""
  echo "Example: bash scripts/route-task.sh \"implement OAuth for auth module\""
  exit 1
fi

WRITING_TSV="$(node scripts/lib/writing-intent.js --tsv "$TASK" 2>/dev/null || printf '0\t\t\t')"
WRITING_IS_WRITING="$(printf '%s' "$WRITING_TSV" | cut -f1)"
WRITING_ACTION="$(printf '%s' "$WRITING_TSV" | cut -f2)"
WRITING_MODE="$(printf '%s' "$WRITING_TSV" | cut -f3)"
WRITING_OVERLAYS="$(printf '%s' "$WRITING_TSV" | cut -f4)"
WRITING_POLICY_TSV="$(node scripts/lib/writing-route-policy.js --tsv "$TASK" 2>/dev/null || printf '0\t\t\t\t\t\t\t\t\t\t\t')"
WRITING_POLICY_MODE="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f2)"
WRITING_POLICY_EXTRA_MODES="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f3)"
WRITING_POLICY_AGENT="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f4)"
WRITING_POLICY_SKILLS="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f5)"
WRITING_POLICY_SUBAGENTS="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f6)"
WRITING_POLICY_FILES="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f7)"
WRITING_POLICY_PIPELINE="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f8)"
WRITING_POLICY_RISK="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f9)"
WRITING_POLICY_PROFILES="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f10)"
WRITING_POLICY_EDITORS="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f11)"
WRITING_POLICY_GATES="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f12)"
WRITING_POLICY_FRESH_DOCS="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f13)"
WRITING_POLICY_TARGET_LANGUAGE="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f14)"
WRITING_POLICY_LANGUAGE_PROFILES="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f15)"
WRITING_POLICY_DOMAIN_PROFILES="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f16)"
WRITING_POLICY_TECHNICAL_PROFILES="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f17)"
WRITING_POLICY_REJECTED_PROFILES="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f18)"
WRITING_POLICY_LANGUAGE_RESOLUTION="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f19)"
WRITING_POLICY_PROCESS_PROFILES="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f20)"
WRITING_POLICY_EXTERNAL_TOOLS="$(printf '%s' "$WRITING_POLICY_TSV" | cut -f21)"

# Always needed
FILES="$LIB/process/context-first.md"
MODES=""

# PRODUCT GOAL / PRODUCTION STANDARD
if echo "$TASK" | grep -qiE "product.goal|final.outcome|quality.bar|production|prod|mvp|prototype|goal|roadmap|continue|finish|цель|финал|качество|доделай|продолжай|прототип"; then
  FILES="$FILES $LIB/product/production-product-standard.md $LIB/process/product-goal-loop.md $LIB/process/client-executor-contract.md $LIB/process/plan-first.md"
  MODES="$MODES product"
fi

# CODE
if [ "$WRITING_IS_WRITING" != "1" ] && echo "$TASK" | grep -qiE "implement|build|create|add|fix|bug|refactor|feature|module|function|class|api|endpoint|service|migrate|настрой|создай|добавь|исправь|починь|реализуй|напиши код|сделай|баг|не работает|падает|ошибка|сломал"; then
  FILES="$FILES $LIB/process/research-first.md $LIB/process/plan-first.md $LIB/process/self-verification.md $LIB/technical/architecture.md $LIB/technical/code-style.md $LIB/technical/error-handling.md $LIB/technical/atomic-reuse.md"
  MODES="$MODES code"
fi

# TEST
if echo "$TASK" | grep -qiE "test|coverage|tdd|spec|assert|mock|jest|pytest|vitest"; then
  FILES="$FILES $LIB/technical/testing.md $LIB/process/self-verification.md"
  MODES="$MODES test"
fi

# DESIGN
if [ "$WRITING_IS_WRITING" != "1" ] && echo "$TASK" | grep -qiE "design|figma|ui|ux|css|style|layout|component|token|color|font|responsive|tailwind|screen|дизайн|макет|фигма|экран|интерфейс|стиль"; then
  FILES="$FILES $LIB/product/production-product-standard.md $LIB/process/product-goal-loop.md $LIB/process/client-executor-contract.md $LIB/domain/domain-design-pipeline.md $LIB/meta/analysis.md $LIB/technical/atomic-reuse.md"
  MODES="$MODES design"
fi

# DESIGN SYSTEM
if echo "$TASK" | grep -qiE "design.system|storybook|tokens?|atomic|atoms?|molecules?|organisms?|component.library|spacing|radius|typography|rendered.geometry|bounding|дизайн-систем|сторибук|токен|атом|молекул|организм|отступ|скругл|типограф"; then
  FILES="$FILES $LIB/product/production-product-standard.md $LIB/process/product-goal-loop.md $LIB/process/client-executor-contract.md $LIB/domain/domain-design-system.md $LIB/domain/domain-design-pipeline.md"
  MODES="$MODES design-system"
fi

# REVIEW
if [ "$WRITING_IS_WRITING" != "1" ] && echo "$TASK" | grep -qiE "review|audit|check|analyze|report|status|health|inspect|evaluate|посмотри|проверь|оцени|разбери|покажи"; then
  FILES="$FILES $LIB/meta/analysis.md $LIB/meta/critical-thinking.md $LIB/process/self-verification.md"
  MODES="$MODES review"
fi

# WRITING
if [ "$WRITING_IS_WRITING" = "1" ]; then
  for writing_file in $(printf '%s' "$WRITING_POLICY_FILES" | tr ',' ' '); do
    FILES="$FILES $LIB/$writing_file"
  done
  FILES="$FILES $LIB/domain/domain-guards.md"
  MODES="$MODES $WRITING_POLICY_MODE $(printf '%s' "$WRITING_POLICY_EXTRA_MODES" | tr ',' ' ')"
fi

# GIT
if [ "$WRITING_IS_WRITING" != "1" ] && echo "$TASK" | grep -qiE "commit|push|(^|[^[:alpha:]])pr([^[:alpha:]]|$)|pull.request|merge|branch|release|deploy|tag|cherry"; then
  FILES="$FILES $LIB/technical/git-workflow.md"
  MODES="$MODES git"
fi

# TEMPLATE / CODEX ROUTING
if echo "$TASK" | grep -qiE "template|agents\.md|claude\.md|skill|subagent|router|route|sync-template|agent.project|client-executor|accountable.executor|anti-?sycophancy|sycophancy|fake.work|fake.completion|falsif|pretend.completed|no.fake|шаблон|агент|скилл|роут|маршрут|синхрон"; then
  FILES="$FILES $LIB/product/production-product-standard.md $LIB/process/product-goal-loop.md $LIB/process/client-executor-contract.md $LIB/meta/critical-thinking.md $LIB/technical/testing.md $LIB/technical/git-workflow.md"
  MODES="$MODES template"
fi

# RELEASE
if [ "$WRITING_IS_WRITING" != "1" ] && echo "$TASK" | grep -qiE "release|tag|version|changelog|publish|github release|deploy|релиз|верси|тег|опубликуй|выкат"; then
  FILES="$FILES $LIB/technical/git-workflow.md $LIB/technical/testing.md $LIB/meta/critical-thinking.md"
  MODES="$MODES release"
fi

# PLAN
if echo "$TASK" | grep -qiE "plan|strategy|architect|roadmap|estimate|decompose|breakdown|спланируй|декомпозируй|разбей|оцени сложность|архитектур"; then
  FILES="$FILES $LIB/meta/strategic-thinking.md $LIB/process/plan-first.md $LIB/process/client-executor-contract.md $LIB/conflict/conflict-resolution.md"
  MODES="$MODES plan"
fi

# SAFETY-CRITICAL
if echo "$TASK" | grep -qiE "health|fitness|medical|nutrition|exercise|science|evidence|study|research paper|здоровье|фитнес|тренировк|питание|наук"; then
  FILES="$FILES $LIB/domain/domain-guards.md $LIB/meta/critical-thinking.md"
  MODES="$MODES safety"
fi

# REFACTOR
if echo "$TASK" | grep -qiE "refactor|simplif|clean.up|extract|split|reorganize|рефактор|упрости|вынеси|раздели"; then
  FILES="$FILES $LIB/technical/architecture.md $LIB/technical/code-style.md $LIB/process/self-verification.md $LIB/technical/testing.md"
  MODES="$MODES refactor"
fi

# If nothing matched, load minimal code set
if [ -z "$MODES" ]; then
  FILES="$FILES $LIB/process/research-first.md $LIB/process/self-verification.md $LIB/technical/architecture.md"
  MODES="general"
fi

# Deduplicate
UNIQUE_FILES=$(echo "$FILES" | tr ' ' '\n' | sort -u)
FILE_COUNT=0
LINE_COUNT=0
VALID_FILES=""
for f in $UNIQUE_FILES; do
  if [ -f "$f" ]; then
    lines=$(wc -l < "$f" | tr -d ' ')
    LINE_COUNT=$((LINE_COUNT + lines))
    FILE_COUNT=$((FILE_COUNT + 1))
    VALID_FILES="$VALID_FILES $f"
  fi
done

# Determine agent
if [ "$WRITING_IS_WRITING" = "1" ]; then
  AGENT="$WRITING_POLICY_AGENT"
elif echo "$TASK" | grep -qiE "review|audit"; then AGENT="reviewer"
elif echo "$TASK" | grep -qiE "test|tdd|coverage"; then AGENT="test-engineer"
elif echo "$TASK" | grep -qiE "write|draft|rewrite|article|copy|story|novel|scene|email|message|guide|manual|напиш|перепиш|роман|рассказ|сцен|письм|сообщен|руководств|текст|статья"; then AGENT="writer"
elif echo "$TASK" | grep -qiE "security|vulnerability|owasp"; then AGENT="security-auditor"
elif echo "$TASK" | grep -qiE "performance|slow|optimize|profile|benchmark"; then AGENT="profiler"
elif echo "$TASK" | grep -qiE "document|readme|changelog|api.doc"; then AGENT="documenter"
elif echo "$TASK" | grep -qiE "simplif|reduce|clean"; then AGENT="simplifier"
else AGENT="implementer"; fi

CODEX_SKILLS="codex-audit"
CODEX_SUBAGENTS="reviewer"
PIPELINE="review"
RISK="MEDIUM"
if [ "$WRITING_IS_WRITING" = "1" ]; then
  CODEX_SKILLS="$(printf '%s' "$WRITING_POLICY_SKILLS" | tr ',' ' ')"
  CODEX_SUBAGENTS="$(printf '%s' "$WRITING_POLICY_SUBAGENTS" | tr ',' ' ')"
  PIPELINE="$WRITING_POLICY_PIPELINE"
  RISK="$WRITING_POLICY_RISK"
fi
if echo "$MODES" | grep -q "template"; then
  CODEX_SKILLS="codex-template-sync codex-skill-maintenance codex-test-rules codex-agent-router codex-product-goal codex-strategic-review"
  CODEX_SUBAGENTS="pr_explorer systems_reviewer tester"
  PIPELINE="template maintenance"
  RISK="HIGH"
elif echo "$MODES" | grep -q "release"; then
  CODEX_SKILLS="codex-template-sync codex-health-check codex-test-rules codex-strategic-review"
  CODEX_SUBAGENTS="tester reviewer security_reviewer"
  PIPELINE="release"
  RISK="HIGH"
elif echo "$TASK" | grep -qiE "security|vulnerability|secret|auth|permission|injection|xss|csrf|ssrf|cve|owasp"; then
  CODEX_SKILLS="codex-security-audit codex-strategic-review"
  CODEX_SUBAGENTS="security_reviewer pr_explorer tester"
  PIPELINE="security patch"
  RISK="HIGH"
elif echo "$MODES" | grep -q "plan"; then
  CODEX_SKILLS="codex-product-goal codex-strategic-review codex-decompose"
  CODEX_SUBAGENTS="systems_reviewer product_reviewer pr_explorer"
  PIPELINE="planning"
elif echo "$MODES" | grep -q "design-system"; then
  CODEX_SKILLS="codex-design-system-workflow codex-design-workflow codex-domain-design-review codex-product-goal"
  CODEX_SUBAGENTS="design_reviewer tester reviewer"
  PIPELINE="design system"
  RISK="HIGH"
elif echo "$MODES" | grep -q "design"; then
  CODEX_SKILLS="codex-design-workflow codex-domain-design-review codex-product-goal"
  CODEX_SUBAGENTS="design_reviewer tester reviewer"
  PIPELINE="design"
elif echo "$MODES" | grep -q "test"; then
  CODEX_SKILLS="codex-coverage"
  CODEX_SUBAGENTS="tester reviewer"
  PIPELINE="quality gate"
elif echo "$MODES" | grep -q "code"; then
  CODEX_SKILLS="codex-feature-workflow codex-pipeline-workflow"
  CODEX_SUBAGENTS="pr_explorer tester reviewer"
  PIPELINE="feature"
fi

# Match the machine-readable policy ceiling even when Node.js is unavailable.
CODEX_SUBAGENTS="$(printf '%s\n' "$CODEX_SUBAGENTS" | awk '{print $1, $2, $3}' | sed 's/[[:space:]]*$//')"

CODEX_FANOUT="conditional"
CODEX_FANOUT_REASON="spawn-only-if-independent-specialist-lane-exists"
if echo "$TASK" | grep -qiE "(do not|don.t|never) ((use|spawn|run|call) )?(any )?(sub-?agents?|delegation|fan-?out)|(do not|don.t|never) delegate|without ((using|any) )?(sub-?agents?|delegation|fan-?out)|no (sub-?agents?|delegation|fan-?out)|без (любых )?(субагент|сабагент|делегац|фан-?аут)|не (используй|запускай|вызывай) (любых )?(субагент|сабагент|делегац|фан-?аут)|не делегируй"; then
  CODEX_SUBAGENTS=""
  CODEX_FANOUT="skip"
  CODEX_FANOUT_REASON="explicit-user-opt-out"
elif [ "$RISK" = "HIGH" ]; then
  CODEX_FANOUT="required"
  CODEX_FANOUT_REASON="high-risk-independent-verification"
elif [ "$(printf '%s\n' "$CODEX_SUBAGENTS" | wc -w | tr -d ' ')" -ge 2 ]; then
  CODEX_FANOUT="recommended"
  CODEX_FANOUT_REASON="parallel-independent-lanes-available"
fi

if echo "$MODES" | grep -q "product" &&
   ! echo "$CODEX_SKILLS" | grep -q "codex-product-goal"; then
  CODEX_SKILLS="$CODEX_SKILLS codex-product-goal"
fi

if { [ "$RISK" = "HIGH" ] || echo "$MODES" | grep -q "plan"; } &&
   ! echo "$CODEX_SKILLS" | grep -q "codex-strategic-review"; then
  CODEX_SKILLS="$CODEX_SKILLS codex-strategic-review"
fi

# Save active rules
mkdir -p tasks
{
  echo "TASK=$TASK"
  echo "MODES=$MODES"
  echo "AGENT=$AGENT"
  echo "CODEX_SKILLS=$CODEX_SKILLS"
  echo "CODEX_SUBAGENTS=$CODEX_SUBAGENTS"
  echo "CODEX_FANOUT=$CODEX_FANOUT"
  echo "CODEX_FANOUT_REASON=$CODEX_FANOUT_REASON"
  echo "CODEX_AGENT_POLICY=scripts/codex-agent-policy.js"
  echo "WRITING_INTENT=$WRITING_ACTION:$WRITING_MODE:$WRITING_OVERLAYS"
  echo "WRITING_PROFILES=$WRITING_POLICY_PROFILES"
  echo "WRITING_EDITORS=$WRITING_POLICY_EDITORS"
  echo "WRITING_GATES=$WRITING_POLICY_GATES"
  echo "WRITING_FRESH_DOCS=$WRITING_POLICY_FRESH_DOCS"
  echo "WRITING_TARGET_LANGUAGE=$WRITING_POLICY_TARGET_LANGUAGE"
  echo "WRITING_LANGUAGE_PROFILES=$WRITING_POLICY_LANGUAGE_PROFILES"
  echo "WRITING_DOMAIN_PROFILES=$WRITING_POLICY_DOMAIN_PROFILES"
  echo "WRITING_TECHNICAL_PROFILES=$WRITING_POLICY_TECHNICAL_PROFILES"
  echo "WRITING_REJECTED_PROFILES=$WRITING_POLICY_REJECTED_PROFILES"
  echo "WRITING_LANGUAGE_RESOLUTION=$WRITING_POLICY_LANGUAGE_RESOLUTION"
  echo "WRITING_PROCESS_PROFILES=$WRITING_POLICY_PROCESS_PROFILES"
  echo "WRITING_EXTERNAL_TOOLS=$WRITING_POLICY_EXTERNAL_TOOLS"
  echo "PIPELINE=$PIPELINE"
  echo "RISK=$RISK"
  echo "STRATEGY_GATE=Goal -> Constraints -> Approach -> Verification -> Risk/Doubt"
  echo "PLAN_CONTRACT=required for M+/product/template/design/security work; language=match-user-request"
  echo "PRODUCT_BAR=final-product-quality; no-mvp-by-default=true"
  echo "COUNT=$FILE_COUNT files, $LINE_COUNT lines"
  echo "ROUTED_AT=$(date -u +%Y-%m-%dT%H:%M 2>/dev/null || date +%Y-%m-%dT%H:%M)"
  echo "---"
  for f in $VALID_FILES; do echo "$f"; done
} > "$ACTIVE"

# Output
echo "=== ROUTE ==="
echo "TASK: $TASK"
echo "MODES:$MODES"
echo "AGENT: $AGENT"
echo "CODEX_SKILLS: $CODEX_SKILLS"
echo "CODEX_SUBAGENTS: $CODEX_SUBAGENTS"
echo "CODEX_FANOUT: $CODEX_FANOUT | $CODEX_FANOUT_REASON"
echo "CODEX_AGENT_POLICY: scripts/codex-agent-policy.js"
echo "WRITING_INTENT: $WRITING_ACTION | $WRITING_MODE | $WRITING_OVERLAYS"
echo "WRITING_PROFILES: $WRITING_POLICY_PROFILES"
echo "WRITING_EDITORS: $WRITING_POLICY_EDITORS"
echo "WRITING_GATES: $WRITING_POLICY_GATES"
echo "WRITING_FRESH_DOCS: $WRITING_POLICY_FRESH_DOCS"
echo "WRITING_TARGET_LANGUAGE: $WRITING_POLICY_TARGET_LANGUAGE"
echo "WRITING_LANGUAGE_PROFILES: $WRITING_POLICY_LANGUAGE_PROFILES"
echo "WRITING_DOMAIN_PROFILES: $WRITING_POLICY_DOMAIN_PROFILES"
echo "WRITING_TECHNICAL_PROFILES: $WRITING_POLICY_TECHNICAL_PROFILES"
echo "WRITING_REJECTED_PROFILES: $WRITING_POLICY_REJECTED_PROFILES"
echo "WRITING_LANGUAGE_RESOLUTION: $WRITING_POLICY_LANGUAGE_RESOLUTION"
echo "WRITING_PROCESS_PROFILES: $WRITING_POLICY_PROCESS_PROFILES"
echo "WRITING_EXTERNAL_TOOLS: $WRITING_POLICY_EXTERNAL_TOOLS"
echo "PIPELINE: $PIPELINE"
echo "RISK: $RISK"
echo "STRATEGY_GATE: Goal -> Constraints -> Approach -> Verification -> Risk/Doubt"
echo "PLAN: required for M+/product/template/design/security work | match-user-request"
echo "PRODUCT_BAR: final-product-quality | no_mvp=true"
echo "RULES: $FILE_COUNT files, ~$LINE_COUNT lines"
echo "---"
echo "READ these files:"
for f in $VALID_FILES; do echo "  $f"; done
