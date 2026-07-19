#!/usr/bin/env bash
# test-template.sh — Smoke test for agent-project-template
# Verifies all required files exist and JSON is valid

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

ERRORS=0
CHECKS=0
TEMPLATE_DIR="$(normalize_drive_path "$(cd "$(dirname "$0")/.." && pwd)")"
cd "$TEMPLATE_DIR"

[ -f "$TEMPLATE_DIR/scripts/lib/platform.sh" ] && source "$TEMPLATE_DIR/scripts/lib/platform.sh"

pass() { echo "  PASS: $1"; CHECKS=$((CHECKS+1)); }
fail() { echo "  FAIL: $1"; ERRORS=$((ERRORS+1)); CHECKS=$((CHECKS+1)); }
skip() { echo "  SKIP: $1"; CHECKS=$((CHECKS+1)); }
check() {
  local description="$1"
  local output
  local started=$SECONDS
  shift
  if output="$("$@" 2>&1)"; then
    pass "$description ($((SECONDS - started))s)"
    return
  fi
  [ -n "$output" ] && printf '%s\n' "$output"
  fail "$description ($((SECONDS - started))s)"
}
is_template_source_repo() { grep -Eq '^- Name: agent-project-template$' PROJECT_SPEC.md 2>/dev/null; }
source_only_check() {
  local description="$1"
  shift
  if is_template_source_repo; then
    check "$description" "$@"
  else
    skip "$description (template source repo only)"
  fi
}
validate_text_policy_rejects_mojibake() {
  local fixture
  fixture="$(_temp_file text-policy-fixture)"
  node -e "require('fs').writeFileSync(process.argv[1], 'Broken: \\u0432\\u0402\\u201D\\n', 'utf8')" "$fixture"
  if node scripts/validate-text-policy.js --path "$fixture" >/dev/null 2>&1; then
    rm -f "$fixture"
    return 1
  fi
  rm -f "$fixture"
  return 0
}
validate_progressive_status_rejects_stale_header() {
  local fixture_repo
  local script_path
  fixture_repo="$(_temp_dir progressive-status-fixture)"
  script_path="$TEMPLATE_DIR/scripts/progressive-status.js"

  (
    cd "$fixture_repo"
    git init -q
    git config user.email "template-smoke@example.invalid"
    git config user.name "Template Smoke"
    cat > work.md <<'EOF'
<!-- PROGRESSIVE_STATUS
id: fixture-work
status: active
updated: 2026-07-06
readiness: 10
plan: 10
inventory: 10
production: 10
cleanup: 10
tags: progressive-jpeg,fixture
next: first check
-->

# Work

Initial body.
EOF
    git add work.md
    git commit -q -m "fixture"
    printf '%s\n' "" "Changed body without status header update." >> work.md
    if node "$script_path" --check --path work.md >/dev/null 2>&1; then
      exit 1
    fi
  )
  local status=$?
  rm -rf "$fixture_repo"
  return "$status"
}
github_workflows_use_node24_actions() {
  ! grep -R -E 'actions/(checkout|setup-node)@v4' .github &&
    ! grep -R -E "node-version:[[:space:]]*['\"]?20" .github
}
validate_release_facing_doc_counts() {
  node <<'NODE'
const fs = require("fs");
const { execFileSync } = require("child_process");

const countDirs = (directory) =>
  fs.readdirSync(directory, { withFileTypes: true }).filter((entry) => entry.isDirectory()).length;
const countFiles = (directory, extension) =>
  fs.readdirSync(directory).filter((entry) => entry.endsWith(extension)).length;
const countFilesRecursive = (directory, extension) =>
  fs.readdirSync(directory, { withFileTypes: true }).reduce((total, entry) => {
    const fullPath = `${directory}/${entry.name}`;
    if (entry.isDirectory()) return total + countFilesRecursive(fullPath, extension);
    return total + (entry.name.endsWith(extension) ? 1 : 0);
  }, 0);
const countTrackedSkillDirs = (directory) => {
  try {
    const files = execFileSync("git", ["ls-files", `${directory}/*/SKILL.md`], { encoding: "utf8" })
      .trim()
      .split(/\r?\n/)
      .filter(Boolean);
    return new Set(files.map((file) => file.split("/").slice(0, 3).join("/"))).size;
  } catch {
    return countDirs(directory);
  }
};

const readme = fs.readFileSync("README.md", "utf8");
const claude = fs.readFileSync("CLAUDE.md", "utf8");
const counts = new Map([
  ["Rules", countFiles(".claude/rules", ".md") + countFilesRecursive(".claude/library", ".md")],
  ["Hooks", countFiles(".claude/hooks", ".sh")],
  ["Claude Skills", countTrackedSkillDirs(".claude/skills")],
  ["Codex Skills", countTrackedSkillDirs(".agents/skills")],
  ["Codex Subagents", countFiles(".codex/agents", ".toml")],
  ["Agents", countFiles(".claude/agents", ".md")],
  ["Commands", countFiles(".claude/commands", ".md")],
  ["Scripts", fs.readdirSync("scripts").filter((entry) => fs.statSync(`scripts/${entry}`).isFile()).length],
]);

if (!readme.startsWith("# Agent Project Template v4")) {
  throw new Error("README title must use the current major version");
}

for (const [label, count] of counts) {
  const expected = `| **${label}** | ${count} |`;
  if (!readme.includes(expected)) {
    throw new Error(`README count mismatch for ${label}; expected row prefix: ${expected}`);
  }
}

const commandNames = fs.readdirSync(".claude/commands")
  .filter((entry) => entry.endsWith(".md"))
  .map((entry) => entry.replace(/\.md$/, ""))
  .sort();
const commandSectionMatch = claude.match(/## Commands \((\d+)\)\n([\s\S]*?)\n\n## /);
if (!commandSectionMatch) {
  throw new Error("CLAUDE.md command section not found");
}
const headingCount = Number(commandSectionMatch[1]);
if (headingCount !== commandNames.length) {
  throw new Error(`CLAUDE.md command count ${headingCount} != ${commandNames.length}`);
}
const listed = Array.from(commandSectionMatch[2].matchAll(/\/[a-z0-9-]+/g))
  .map((match) => match[0].slice(1))
  .sort();
const missing = commandNames.filter((name) => !listed.includes(name));
const extra = listed.filter((name) => !commandNames.includes(name));
if (missing.length > 0 || extra.length > 0) {
  throw new Error(`CLAUDE.md commands mismatch; missing=${missing.join(",")} extra=${extra.join(",")}`);
}
NODE
}
validate_agent_safe_github_entrypoint() {
  node <<'NODE'
const fs = require("fs");

const read = (file) => fs.readFileSync(file, "utf8");
const readme = read("README.md");
const setupGuide = read("SETUP_GUIDE.md");
const releases = read("docs/TEMPLATE_RELEASES.md");
const agents = read("AGENTS.md");
const claude = read("CLAUDE.md");
const updateCommand = read(".claude/commands/update-template.md");
const syncSkill = read(".agents/skills/codex-template-sync/SKILL.md");
const allReleaseDocs = `${readme}\n${setupGuide}\n${releases}`;

const badgeMatch = readme.match(/template-v([0-9]+\.[0-9]+\.[0-9]+)-blue/);
if (!badgeMatch) {
  throw new Error("README template version badge not found");
}

const tag = `v${badgeMatch[1]}`;
const required = [
  ["README latest release link", readme, "https://github.com/Yokhan/agent-project-template/releases/latest"],
  ["README candidate tag", readme, `Release candidate tag: \`${tag}\``],
  ["README pinned clone", readme, `git clone --branch ${tag} --depth 1 https://github.com/Yokhan/agent-project-template.git agent-project-template`],
  ["README pinned sync dry-run", readme, `bash scripts/sync-template.sh --from-git --ref ${tag} --dry-run`],
  ["README pinned sync apply", readme, `bash scripts/sync-template.sh --from-git --ref ${tag}`],
  ["README main warning", readme, "`main` is for template development and explicit canary rollout only"],
  ["README explicit canary", readme, "bash scripts/sync-template.sh --from-git --canary --ref main --dry-run"],
  ["SETUP_GUIDE latest release link", setupGuide, "https://github.com/Yokhan/agent-project-template/releases/latest"],
  ["SETUP_GUIDE candidate tag", setupGuide, `Целевой release candidate: \`${tag}\``],
  ["SETUP_GUIDE pinned clone", setupGuide, `git clone --branch ${tag} --depth 1 https://github.com/Yokhan/agent-project-template.git agent-project-template`],
  ["SETUP_GUIDE pinned sync dry-run", setupGuide, `bash scripts/sync-template.sh --from-git --ref ${tag} --dry-run`],
  ["SETUP_GUIDE main warning", setupGuide, "`main` используйте только для разработки шаблона или явного canary-роллаута"],
  ["TEMPLATE_RELEASES latest release link", releases, "https://github.com/Yokhan/agent-project-template/releases/latest"],
  ["TEMPLATE_RELEASES candidate tag", releases, `Release candidate tag: \`${tag}\``],
  ["TEMPLATE_RELEASES pinned sync dry-run", releases, `bash scripts/sync-template.sh --from-git --ref ${tag} --dry-run`],
  ["TEMPLATE_RELEASES main warning", releases, "Use `main` only for template development, explicit canary rollout"],
  ["TEMPLATE_RELEASES explicit canary", releases, "bash scripts/sync-template.sh --from-git --canary --ref main --dry-run"],
  ["Release SOT workspace classification", releases, "### 1. Classify The Workspace"],
  ["Release SOT installed version", releases, ".template-manifest.json.template_version"],
  ["Release SOT remote verification", releases, "git remote get-url template"],
  ["Release SOT same-tag apply", releases, "apply command must use the exact tag"],
  ["Release SOT legacy fallback", releases, "target release checkout's script with `--project-dir`"],
  ["Release SOT honest status", releases, "authoritative GitHub Release/workflow state"],
  ["AGENTS hot update protocol", agents, "### Template Update Protocol"],
  ["CLAUDE hot update protocol", claude, "## Template Update Protocol"],
  ["Claude update command installed version", updateCommand, "Read the installed version from `.template-manifest.json`"],
  ["Claude update command same tag", updateCommand, "Use the same tag as the dry-run"],
  ["Codex sync skill release SOT", syncSkill, "#canonical-agent-update-protocol"],
  ["Codex sync skill post-sync verification", syncSkill, "before claiming success"],
  ["SETUP_GUIDE project-owned CLAUDE", setupGuide, "Сохраняет как project-owned: `CLAUDE.md`"],
];

const missing = required.filter(([, text, expected]) => !text.includes(expected));
if (missing.length > 0) {
  throw new Error(`Agent-safe GitHub entrypoint missing: ${missing.map(([label]) => label).join(", ")}`);
}

if (/v3\.8\.0/.test(allReleaseDocs)) {
  throw new Error("Release-facing docs still contain stale v3.8.0 examples");
}

if (/git clone https:\/\/github\.com\/Yokhan\/agent-project-template\.git agent-project-template/.test(readme)) {
  throw new Error("README contains branchless clone command; use a pinned release tag");
}

if (/git clone https:\/\/github\.com\/Yokhan\/agent-project-template\.git agent-project-template/.test(setupGuide)) {
  throw new Error("SETUP_GUIDE contains branchless clone command; use a pinned release tag");
}

if (/bash scripts\/sync-all\.sh/.test(setupGuide)) {
  throw new Error("SETUP_GUIDE contains batch apply without a preview gate");
}

if (/sync-template\.sh --from-git(?:\s+--dry-run)?\s*(?:#.*)?$/m.test(updateCommand)) {
  throw new Error("update-template command contains branchless normal-update instructions");
}

for (const [name, content] of [["README", readme], ["TEMPLATE_RELEASES", releases]]) {
  if (/sync-template\.sh --from-git(?:\s+--dry-run)?\s*(?:#.*)?$/m.test(content)) {
    throw new Error(`${name} contains an implicit branch update without --canary and --ref`);
  }
}
NODE
}
validate_screen_anatomy_contract() {
  node <<'NODE'
const fs = require("fs");

const required = new Map([
  [".claude/library/domain/domain-design-pipeline.md", [
    "### Screen Anatomy First",
    "Root frame:",
    "Base background:",
    "Background composition:",
    "Content frame:",
    "Overlay layer:",
    "Visible bounded surfaces",
    "Edge-to-edge"
  ]],
  [".claude/library/domain/domain-design-system.md", [
    "## Screen Anatomy Contract",
    "Root frame:",
    "Base background:",
    "Background composition:",
    "Content frame:",
    "Overlay layer:",
    "Atomic stories show the atom itself"
  ]],
  [".agents/skills/codex-design-workflow/SKILL.md", [
    "## Screen Anatomy Gate",
    "Every full screen starts from screen anatomy",
    "root frame",
    "background composition",
    "content frame",
    "overlay layer"
  ]],
  [".agents/skills/codex-design-system-workflow/SKILL.md", [
    "## Screen Anatomy Contract",
    "Atom isolation",
    "root frame",
    "base background",
    "background composition",
    "content frame",
    "overlay layer"
  ]],
  [".agents/skills/codex-design-workflow/references/design-command-modes.md", [
    "name screen anatomy layers",
    "root frame",
    "base background",
    "background composition",
    "content frame",
    "overlay policy"
  ]],
]);

for (const [file, terms] of required) {
  const text = fs.readFileSync(file, "utf8");
  const missing = terms.filter((term) => !text.includes(term));
  if (missing.length > 0) {
    throw new Error(`${file} missing design screen anatomy terms: ${missing.join(", ")}`);
  }
}
NODE
}

echo "=== Template Smoke Test: $TEMPLATE_DIR ==="
echo ""

echo "Required files:"
check "CLAUDE.md" test -f CLAUDE.md
check "AGENTS.md" test -f AGENTS.md
check "README.md" test -f README.md
check "SETUP_GUIDE.md" test -f SETUP_GUIDE.md
check "PROJECT_SPEC.md" test -f PROJECT_SPEC.md
check "ecosystem.md" test -f ecosystem.md
check "docs/PRODUCT_BOUNDARY.md" test -f docs/PRODUCT_BOUNDARY.md
check "docs/SAFE_DEFAULTS.md" test -f docs/SAFE_DEFAULTS.md
check "docs/SUPPORTED_ENVIRONMENTS.md" test -f docs/SUPPORTED_ENVIRONMENTS.md
check "docs/TEMPLATE_RELEASES.md" test -f docs/TEMPLATE_RELEASES.md
check ".claude/library/product/production-product-standard.md" test -f .claude/library/product/production-product-standard.md
check ".claude/library/process/product-goal-loop.md" test -f .claude/library/process/product-goal-loop.md
check ".claude/library/process/client-executor-contract.md" test -f .claude/library/process/client-executor-contract.md
check ".claude/library/domain/domain-design-system.md" test -f .claude/library/domain/domain-design-system.md
check "docs/AGENT_PIPELINES.md" test -f docs/AGENT_PIPELINES.md
check "docs/AGENT_CONTEXT_SOT.md" test -f docs/AGENT_CONTEXT_SOT.md
check "docs/CODEX_FANOUT_PATTERNS.md" test -f docs/CODEX_FANOUT_PATTERNS.md
check "docs/CODEX_SKILLS_AUDIT.md" test -f docs/CODEX_SKILLS_AUDIT.md
check "docs/CODEX_SUBAGENTS_AUDIT.md" test -f docs/CODEX_SUBAGENTS_AUDIT.md
check "docs/CODE_INTELLIGENCE_TOOLCHAIN.md" test -f docs/CODE_INTELLIGENCE_TOOLCHAIN.md
check "docs/OPENAI_MODEL_GUIDANCE.md" test -f docs/OPENAI_MODEL_GUIDANCE.md
check "_reference/code-intelligence-tools.json" test -f _reference/code-intelligence-tools.json
check "code-intelligence catalog" node scripts/code-intelligence-tools.js validate
check "code-intelligence selection tests" node scripts/test-code-intelligence-tools.js
check "_reference/codex-mcp-config.toml" test -f _reference/codex-mcp-config.toml
check "Codex MCP config merge tests" node scripts/test-codex-mcp-config.js
check "Codex MCP config current" node scripts/configure-codex-mcp.js --check
check "sync merges Codex MCP block" grep -q 'CODEX_MCP_MERGER' scripts/sync-template.sh
check "sync excludes untracked template payload" grep -q 'ls-files --error-unmatch' scripts/sync-template.sh
check "integrations/spec-kit/README.md" test -f integrations/spec-kit/README.md
source_only_check "setup.sh" test -f setup.sh
source_only_check "setup.bat" test -f setup.bat
check ".codex/config.toml" test -f .codex/config.toml
check ".codex/hooks.json" test -f .codex/hooks.json
check ".codex/hooks.json valid JSON" node -e "const h=JSON.parse(require('fs').readFileSync('.codex/hooks.json','utf8')); if(!h.hooks?.PreToolUse?.[0]?.hooks?.[0]?.command) process.exit(1)"
check ".gitignore" test -f .gitignore
check ".gitattributes" test -f .gitattributes
check ".env.example" test -f .env.example
source_only_check ".github/workflows/release-template.yml" test -f .github/workflows/release-template.yml
check "tasks/lessons.md" test -f tasks/lessons.md
check "tasks/current.md" test -f tasks/current.md
check "tasks/goal.md" test -f tasks/goal.md
source_only_check "starter tasks/current.md" test -f templates/project-starter/tasks/current.md
source_only_check "starter tasks/goal.md" test -f templates/project-starter/tasks/goal.md
source_only_check "starter DESIGN.md" test -f templates/project-starter/DESIGN.md
source_only_check "starter design-policy.ignore" test -f templates/project-starter/design-policy.ignore
source_only_check "starter .research-cache.md" test -f templates/project-starter/tasks/.research-cache.md
source_only_check "starter lessons.md" test -f templates/project-starter/tasks/lessons.md
source_only_check "starter tasks/audit/.gitkeep" test -f templates/project-starter/tasks/audit/.gitkeep
source_only_check "starter brain/01-daily/.gitkeep" test -f templates/project-starter/brain/01-daily/.gitkeep
check "scripts/check-drift.sh" test -f scripts/check-drift.sh
check "scripts/downstream-census.sh" test -f scripts/downstream-census.sh
check "scripts/generate-project-spec.sh" test -f scripts/generate-project-spec.sh
check "scripts/task-brief.sh" test -f scripts/task-brief.sh
check "scripts/validate-codex-agents.js" test -f scripts/validate-codex-agents.js
check "scripts/validate-codex-skills.js" test -f scripts/validate-codex-skills.js
check "scripts/validate-agent-sot.js" test -f scripts/validate-agent-sot.js
check "scripts/validate-production-standard.js" test -f scripts/validate-production-standard.js
check "scripts/validate-design-policy.js" test -f scripts/validate-design-policy.js
check "scripts/test-design-policy.js" test -f scripts/test-design-policy.js
check "scripts/validate-spec-kit.js" test -f scripts/validate-spec-kit.js
check "scripts/validate-text-policy.js" test -f scripts/validate-text-policy.js
check "scripts/progressive-status.js" test -f scripts/progressive-status.js
check "scripts/validate-progressive-plan.js" test -f scripts/validate-progressive-plan.js
check "scripts/test-progressive-plan.js" test -f scripts/test-progressive-plan.js
check "scripts/validate-change-strategy.js" test -f scripts/validate-change-strategy.js
check "scripts/lib/change-strategy-policy.js" test -f scripts/lib/change-strategy-policy.js
check "scripts/test-change-strategy.js" test -f scripts/test-change-strategy.js
check "scripts/validate-subagent-trace.js" test -f scripts/validate-subagent-trace.js
check "scripts/test-subagent-trace.js" test -f scripts/test-subagent-trace.js
check "scripts/sync-spec-kit.sh" test -f scripts/sync-spec-kit.sh
check "scripts/init-spec-kit.sh" test -f scripts/init-spec-kit.sh
check "scripts/codex-agent-policy.js" test -f scripts/codex-agent-policy.js
check "scripts/codex-routing-cases-a.js" test -f scripts/codex-routing-cases-a.js
check "scripts/codex-routing-cases-b.js" test -f scripts/codex-routing-cases-b.js
check "scripts/codex-route-config.js" test -f scripts/codex-route-config.js
check "scripts/codex-route-task.js" test -f scripts/codex-route-task.js
check "scripts/lib/codex-route-summary.js" test -f scripts/lib/codex-route-summary.js
check "scripts/lib/codex-route-cli.js" test -f scripts/lib/codex-route-cli.js
check "scripts/lib/codex-discovery-reroute.js" test -f scripts/lib/codex-discovery-reroute.js
check "change strategy discovery fixture" test -f tests/fixtures/change-strategy/discovery-architecture-mismatch.json
check "scripts/test-codex-routing.js" test -f scripts/test-codex-routing.js
check "scripts/test-codex-agent-policy.js" test -f scripts/test-codex-agent-policy.js
check "scripts/test-codex-subagents-live.sh" test -f scripts/test-codex-subagents-live.sh

echo ""
echo "Codex skills:"
check ">=42 Codex skill dirs" bash -c '[ $(ls -d .agents/skills/*/ 2>/dev/null | wc -l) -ge 42 ]'
check "progressive JPEG planner skill" test -f .agents/skills/codex-progressive-jpeg-planner/SKILL.md
check "change strategy skill" test -f .agents/skills/codex-change-strategy/SKILL.md
check "four-mode writing workflow skill" test -f .agents/skills/codex-writing-workflow/SKILL.md
check "technical writing skill" test -f .agents/skills/codex-technical-writing/SKILL.md
check "technical writing review skill" test -f .agents/skills/codex-technical-writing-review/SKILL.md
check "writing intent classifier" node scripts/test-writing-intent.js
check "writing reference policy" node scripts/test-writing-references.js
check "writing reference registry" node scripts/validate-writing-references.js
check "core Codex design skill" test -f .agents/skills/codex-design-workflow/SKILL.md
check "core Codex design command modes reference" test -f .agents/skills/codex-design-workflow/references/design-command-modes.md
check "core Codex design review skill" test -f .agents/skills/codex-domain-design-review/SKILL.md
check "core Codex Figma skill" test -f .agents/skills/codex-figma-workflow/SKILL.md
check "core Codex pipeline skill" test -f .agents/skills/codex-pipeline-workflow/SKILL.md
check "core Codex Mermaid board skill" test -f .agents/skills/codex-mermaid-board-workflow/SKILL.md
check "core Codex model guidance skill" test -f .agents/skills/codex-openai-model-guidance/SKILL.md
check "validate-codex-skills" node scripts/validate-codex-skills.js
check "test-codex-agent-policy" node scripts/test-codex-agent-policy.js
check "test-progressive-plan" node scripts/test-progressive-plan.js
check "test-change-strategy" node scripts/test-change-strategy.js
check "test-subagent-trace" node scripts/test-subagent-trace.js
check "test-codex-routing" node scripts/test-codex-routing.js
check "validate-production-standard" node scripts/validate-production-standard.js
check "test-design-policy" node scripts/test-design-policy.js
check "screen anatomy contract is enforced in design rules and skills" validate_screen_anatomy_contract
check "validate-agent-sot" node scripts/validate-agent-sot.js
check "validate-spec-kit" node scripts/validate-spec-kit.js
check "validate-text-policy" node scripts/validate-text-policy.js
check "validate-text-policy rejects mojibake" validate_text_policy_rejects_mojibake
check "progressive-status validates headers" node scripts/progressive-status.js --check
check "progressive-status rejects stale headers" validate_progressive_status_rejects_stale_header

echo ""
echo "Codex subagents:"
check ">=12 Codex agent files" bash -c '[ $(ls .codex/agents/*.toml 2>/dev/null | wc -l) -ge 12 ]'
check "Codex Luna scout agent" test -f .codex/agents/scout.toml
check "Codex Luna log analyst agent" test -f .codex/agents/log-analyst.toml
check "Codex Luna summarizer agent" test -f .codex/agents/summarizer.toml
check "Codex pr_explorer agent" test -f .codex/agents/pr-explorer.toml
check "Codex reviewer agent" test -f .codex/agents/reviewer.toml
check "Codex implementer agent" test -f .codex/agents/implementer.toml
check "Codex product reviewer agent" test -f .codex/agents/product-reviewer.toml
check "Codex systems reviewer agent" test -f .codex/agents/systems-reviewer.toml
check "validate-codex-agents" node scripts/validate-codex-agents.js

echo ""
echo "Claude config:"
check ".claude/settings.json" test -f .claude/settings.json
check "settings.json valid JSON" node -e "JSON.parse(require('fs').readFileSync('.claude/settings.json','utf8'))"
check "router.md exists" bash -c '[ -f .claude/rules/router.md ]'
check ">=15 library rule files" bash -c '[ $(find .claude/library -name "*.md" 2>/dev/null | wc -l) -ge 15 ]'
check ">=7 agent files" bash -c '[ $(ls .claude/agents/*.md 2>/dev/null | wc -l) -ge 7 ]'
check ">=21 skill dirs" bash -c '[ $(ls -d .claude/skills/*/ 2>/dev/null | wc -l) -ge 21 ]'
check ">=6 domain skill dirs" bash -c '[ $(ls -d .claude/skills/domain-*/ 2>/dev/null | wc -l) -ge 6 ]'
check ">=12 command files" bash -c '[ $(ls .claude/commands/*.md 2>/dev/null | wc -l) -ge 12 ]'
check ">=7 hook files" bash -c '[ $(ls .claude/hooks/*.sh 2>/dev/null | wc -l) -ge 7 ]'
check "scripts/test-hooks.sh" test -f scripts/test-hooks.sh

echo ""
echo "Agent SOT:"
check "_reference/agent-sot/README.md" test -f _reference/agent-sot/README.md
check "_reference/agent-sot/sources.json" test -f _reference/agent-sot/sources.json
check "_reference/agent-sot/top-works.md" test -f _reference/agent-sot/top-works.md
check "local ai-agent spec original" test -f _reference/agent-sot/originals/ai-agent-spec-v3-final.md
check "Agent SOT has >=20 top works" bash -c '[ $(grep -c "^## TW-" _reference/agent-sot/top-works.md) -ge 20 ]'
check "AGENTS links Agent SOT" bash -c "grep -q 'docs/AGENT_CONTEXT_SOT.md' AGENTS.md"
check "CLAUDE links Agent SOT" bash -c "grep -q 'docs/AGENT_CONTEXT_SOT.md' CLAUDE.md"

echo ""
echo "Spec Kit snapshot:"
check "_reference/spec-kit/README.md" test -f _reference/spec-kit/README.md
check "_reference/spec-kit/manifest.json" test -f _reference/spec-kit/manifest.json
check "Spec Kit command templates" test -f _reference/spec-kit/upstream/templates/commands/specify.md
check "Spec Kit plan template" test -f _reference/spec-kit/upstream/templates/plan-template.md
check "Spec Kit bash helper" test -f _reference/spec-kit/upstream/scripts/bash/check-prerequisites.sh
check "Spec Kit PowerShell helper" test -f _reference/spec-kit/upstream/scripts/powershell/check-prerequisites.ps1

echo ""
echo "Brain vault:"
check "brain/00-inbox" test -d brain/00-inbox
check "brain/01-daily" test -d brain/01-daily
check "brain/02-projects" test -d brain/02-projects
check "brain/03-knowledge" test -d brain/03-knowledge
check "brain/04-decisions" test -d brain/04-decisions
source_only_check "starter research/.gitkeep" test -f templates/project-starter/brain/03-knowledge/research/.gitkeep
source_only_check "starter audits/.gitkeep" test -f templates/project-starter/brain/03-knowledge/audits/.gitkeep
check ">=3 brain templates" bash -c '[ $(ls brain/templates/*.md 2>/dev/null | wc -l) -ge 3 ]'

echo ""
echo "File sizes:"
check "CLAUDE.md <=300 lines" bash -c '[ $(wc -l < CLAUDE.md) -le 300 ]'
check "AGENTS.md <=32KB" bash -c '[ $(wc -c < AGENTS.md) -le 32768 ]'
check "README and CLAUDE release-facing counts match filesystem" validate_release_facing_doc_counts

echo ""
echo "Entry points:"
check "README has no start.sh references" bash -c "! grep -q 'start\\.sh' README.md"
check "README has no start.bat references" bash -c "! grep -q 'start\\.bat' README.md"
check "README has no localhost:3333 references" bash -c "! grep -q 'localhost:3333' README.md"
check "README has no Command Center section" bash -c "! grep -q '## Command Center' README.md"
check "README/SETUP_GUIDE release entrypoint is agent-safe" validate_agent_safe_github_entrypoint
check "SETUP_GUIDE has no legacy --from sync syntax" bash -c "! grep -q 'sync-template\\.sh --from ' SETUP_GUIDE.md"
check "SETUP_GUIDE has no Python 3 bootstrap prerequisite" bash -c "! grep -q 'Python 3' SETUP_GUIDE.md"
check "GitHub workflows use Node24-compatible actions" github_workflows_use_node24_actions
check "Codex CI trust uses canonical user config" bash -c 'grep -q realpathSync .github/workflows/validate-template.yml && grep -q realpathSync .github/workflows/release-template.yml && ! grep -q project_key .github/workflows/validate-template.yml .github/workflows/release-template.yml'
check "Release workflow passes dispatch input through env" grep -q 'INPUT_TAG:.*inputs.tag' .github/workflows/release-template.yml
check "Release workflow validates semantic tag" grep -q 'Invalid release tag' .github/workflows/release-template.yml
check "Release workflow checks out resolved ref" grep -q "ref:.*workflow_dispatch.*inputs.tag" .github/workflows/release-template.yml
check "Release workflow binds HEAD to tag commit" grep -q 'head_commit.*tag_commit' .github/workflows/release-template.yml
check "Release workflow archives validated commit" grep -q 'git archive.*RELEASE_COMMIT' .github/workflows/release-template.yml
check "Release workflow rechecks tag before publish" grep -q 'current_tag_commit.*RELEASE_COMMIT' .github/workflows/release-template.yml
check "Release workflow does not clobber assets" bash -c '! grep -q -- "--clobber" .github/workflows/release-template.yml'
check "No tracked local Claude settings" bash -c '! git ls-files --error-unmatch .claude/settings.local.json >/dev/null 2>&1'
check "Codex config has no user-owned defaults" bash -c "! grep -Eq '^(model|model_reasoning_effort|approval_policy|sandbox_mode)\\s*=' .codex/config.toml"
check "downstream-census --json" bash -c 'bash scripts/downstream-census.sh --no-sync --json "$PWD" 2>/dev/null | node -e "const text=require(\"fs\").readFileSync(0,\"utf8\").trim(); JSON.parse(text || \"[]\")"'
check "generate-project-spec" bash -c 'bash scripts/generate-project-spec.sh | grep -q "^## Identity"'
check "scan-project --report" bash -c 'bash scripts/scan-project.sh --report >/dev/null'
check "task-brief --brief" bash scripts/task-brief.sh --brief
check "task-brief --json" bash -c 'bash scripts/task-brief.sh --json | node -e "JSON.parse(require(\"fs\").readFileSync(0, \"utf8\"))"'
check "codex-route-task template route" bash -c 'node scripts/codex-route-task.js "обнови агентский шаблон и release tag" | node -e "const r=JSON.parse(require(\"fs\").readFileSync(0,\"utf8\")); if(!r.skills.includes(\"codex-template-sync\") || !r.skills.includes(\"codex-health-check\")) process.exit(1)"'
check "route-task fallback includes strategic review" bash -c 'bash scripts/route-task.sh "template release" | grep -q "codex-strategic-review"'
check "route-task fallback requires high-risk fan-out" bash -c 'bash scripts/route-task.sh "template release" | grep -q "CODEX_FANOUT: required"'
check "route-task fallback honors fan-out opt-out" bash -c 'bash scripts/route-task.sh "template release without subagents" | grep -q "CODEX_FANOUT: skip"'
check "route-task fallback honors alternate opt-out" bash -c 'bash scripts/route-task.sh "template release, no fan-out" | grep -q "CODEX_FANOUT: skip"'
check "route-task fallback caps children at three" bash -c 'test "$(bash scripts/route-task.sh "template release" | sed -n "s/^CODEX_SUBAGENTS: //p" | wc -w | tr -d " ")" -le 3'
check "Unix setup manages JavaScript helpers" grep -q 'scripts/lib/.*\.js' setup.sh
check "Windows setup manages JavaScript helpers" grep -q "scripts/lib/\*.js" setup.bat
check "Windows setup manages change strategy discovery fixtures" grep -q "tests/fixtures/change-strategy/\*.json" setup.bat
check "Unix setup manages change strategy discovery fixtures" grep -q "tests/fixtures/change-strategy/" setup.sh

echo ""
echo "Bootstrap trust smoke:"
if is_template_source_repo; then
  SMOKE_SENTINEL="docs/.setup-leak-sentinel-$RANDOM-$$.txt"
  SMOKE_PROJECT="template-leak-smoke-$RANDOM-$$"
  SMOKE_INDEX=""
  cleanup_smoke() {
    rm -f "$SMOKE_SENTINEL"
    [ -n "$SMOKE_INDEX" ] && rm -f "$SMOKE_INDEX"
    rm -f "$SMOKE_PROJECT.setup.log"
    rm -rf "$SMOKE_PROJECT" 2>/dev/null || powershell.exe -NoProfile -Command "if (Test-Path '$SMOKE_PROJECT') { Remove-Item -Recurse -Force '$SMOKE_PROJECT' }" >/dev/null 2>&1 || true
  }
  run_setup_payload_smoke() {
    local project="$1"
    local sentinel="$2"

    SMOKE_INDEX="$(_temp_file setup-smoke-index)"
    GIT_INDEX_FILE="$SMOKE_INDEX" git read-tree HEAD
    GIT_INDEX_FILE="$SMOKE_INDEX" git add -A .agents .codex/agents .github/workflows/validate-template.yml _reference/agent-sot _reference/spec-kit integrations/spec-kit docs/AGENT_CONTEXT_SOT.md docs/AGENT_PIPELINES.md docs/CODEX_FANOUT_PATTERNS.md docs/CODEX_SKILLS_AUDIT.md docs/CODEX_SUBAGENTS_AUDIT.md docs/OPENAI_MODEL_GUIDANCE.md docs/TEMPLATE_RELEASES.md docs/WRITING_WORKFLOW.md docs/WRITING_REFERENCE_PROVENANCE.md .claude/agents/technical-writer.md .claude/skills/writing-workflow .claude/skills/technical-writing .claude/skills/technical-writing-review .claude/library/technical/writing.md .claude/library/technical/writing-mode-profiles.md .claude/library/technical/technical-writing-profile.md .claude/library/technical/writing-editorial-board.md .claude/library/technical/writing-reference-registry.json .claude/library/product/production-product-standard.md .claude/library/process/product-goal-loop.md .claude/library/process/client-executor-contract.md .claude/library/domain/domain-design-system.md .claude/library/domain/domain-design-pipeline.md templates/project-starter/DESIGN.md templates/project-starter/design-policy.ignore templates/project-starter/tasks/goal.md brain/03-knowledge/writing/reference-registry.json tests/fixtures/design-policy tests/fixtures/writing-tools scripts/lib/codex-route-intents.js scripts/lib/writing-intent.js scripts/lib/writing-route-policy.js scripts/lib/writing-reference-policy.js scripts/lib/writing-external-tool-policy.js scripts/lib/writing-path-policy.js scripts/lib/progressive-plan.js scripts/lib/subagent-trace.js scripts/codex-agent-policy.js scripts/codex-routing-cases-a.js scripts/codex-routing-cases-b.js scripts/codex-route-config.js scripts/codex-route-task.js scripts/test-writing-intent.js scripts/test-writing-references.js scripts/validate-writing-references.js scripts/test-codex-agent-policy.js scripts/test-codex-routing.js scripts/test-codex-subagents-live.sh scripts/test-progressive-plan.js scripts/test-subagent-trace.js scripts/init-spec-kit.sh scripts/sync-spec-kit.sh scripts/validate-agent-sot.js scripts/validate-spec-kit.js scripts/validate-text-policy.js scripts/progressive-status.js scripts/validate-progressive-plan.js scripts/validate-subagent-trace.js scripts/validate-codex-agents.js scripts/validate-codex-skills.js scripts/validate-production-standard.js scripts/validate-design-policy.js scripts/test-design-policy.js
    GIT_INDEX_FILE="$SMOKE_INDEX" git add -A .claude/library/technical/russian-writing-profile.md .claude/library/technical/russian-business-correspondence.md .claude/library/technical/russian-explanation-and-persuasion.md
    GIT_INDEX_FILE="$SMOKE_INDEX" git add -A AGENTS.md CLAUDE.md .codex/config.toml .mcp.json .gitignore README.md SETUP_GUIDE.md setup.sh setup.bat integrations/codesight.md docs/SHARED_CONVENTIONS.md docs/AGENT_PIPELINES.md docs/CODE_INTELLIGENCE_TOOLCHAIN.md docs/SAFE_DEFAULTS.md _reference/tool-registry.md _reference/code-intelligence-tools.json _reference/codex-mcp-config.toml .claude/library/process/change-strategy-gate.md .claude/library/process/plan-first.md .claude/library/process/product-goal-loop.md .claude/library/process/client-executor-contract.md .claude/library/product/production-product-standard.md .claude/library/technical/architecture.md .claude/library/meta/critical-thinking.md .agents/skills/codex-change-strategy .agents/skills/codex-debug/SKILL.md .agents/skills/codex-decompose/SKILL.md .agents/skills/codex-strategic-review/SKILL.md scripts/lib/change-strategy-policy.js scripts/lib/code-intelligence-policy.js scripts/lib/codex-route-intents.js scripts/lib/codex-route-summary.js scripts/lib/codex-route-cli.js scripts/lib/codex-discovery-reroute.js scripts/code-intelligence-tools.js scripts/test-code-intelligence-tools.js scripts/configure-codex-mcp.js scripts/test-codex-mcp-config.js scripts/bootstrap-mcp.sh scripts/import-graph.sh scripts/blast-radius.sh scripts/sync-template.sh scripts/validate-change-strategy.js scripts/test-change-strategy.js scripts/codex-route-task.js scripts/codex-agent-policy.js scripts/codex-routing-cases-b.js scripts/test-codex-routing.js scripts/test-codex-agent-policy.js scripts/validate-production-standard.js scripts/validate-template.sh tests/fixtures/change-strategy
    if ! GIT_INDEX_FILE="$SMOKE_INDEX" bash setup.sh "$project" >"$project.setup.log" 2>&1; then
      cat "$project.setup.log"
      return 1
    fi

    [ ! -f "$project/$sentinel" ] &&
      [ -f "$project/.agents/skills/codex-design-workflow/SKILL.md" ] &&
      [ -f "$project/.codex/agents/pr-explorer.toml" ] &&
      [ -f "$project/.codex/agents/product-reviewer.toml" ] &&
      [ -f "$project/.codex/agents/systems-reviewer.toml" ] &&
      [ -f "$project/docs/CODEX_FANOUT_PATTERNS.md" ] &&
      [ -f "$project/docs/AGENT_CONTEXT_SOT.md" ] &&
      [ -f "$project/.claude/library/product/production-product-standard.md" ] &&
      [ -f "$project/.claude/library/process/product-goal-loop.md" ] &&
      [ -f "$project/.claude/library/process/client-executor-contract.md" ] &&
      [ -f "$project/.claude/library/process/change-strategy-gate.md" ] &&
      [ -f "$project/.claude/library/domain/domain-design-system.md" ] &&
      [ -f "$project/.claude/library/domain/domain-design-pipeline.md" ] &&
      [ -f "$project/.agents/skills/codex-design-workflow/references/design-command-modes.md" ] &&
      [ -f "$project/.agents/skills/codex-writing-workflow/SKILL.md" ] &&
      [ -f "$project/.agents/skills/codex-change-strategy/SKILL.md" ] &&
      [ -f "$project/.agents/skills/codex-technical-writing/SKILL.md" ] &&
      [ -f "$project/.agents/skills/codex-technical-writing-review/SKILL.md" ] &&
      [ -f "$project/.claude/skills/writing-workflow/SKILL.md" ] &&
      [ -f "$project/.claude/skills/technical-writing/SKILL.md" ] &&
      [ -f "$project/.claude/skills/technical-writing-review/SKILL.md" ] &&
      [ -f "$project/.claude/agents/technical-writer.md" ] &&
      [ -f "$project/.claude/library/technical/writing-mode-profiles.md" ] &&
      [ -f "$project/.claude/library/technical/russian-writing-profile.md" ] &&
      [ -f "$project/.claude/library/technical/russian-business-correspondence.md" ] &&
      [ -f "$project/.claude/library/technical/russian-explanation-and-persuasion.md" ] &&
      [ -f "$project/.claude/library/technical/technical-writing-profile.md" ] &&
      [ -f "$project/.claude/library/technical/writing-editorial-board.md" ] &&
      [ -f "$project/.claude/library/technical/writing-reference-registry.json" ] &&
      [ -f "$project/docs/WRITING_WORKFLOW.md" ] &&
      [ -f "$project/docs/WRITING_REFERENCE_PROVENANCE.md" ] &&
      [ -f "$project/docs/CODE_INTELLIGENCE_TOOLCHAIN.md" ] &&
      [ -f "$project/_reference/code-intelligence-tools.json" ] &&
      [ -f "$project/_reference/codex-mcp-config.toml" ] &&
      [ -f "$project/scripts/code-intelligence-tools.js" ] &&
      [ -f "$project/scripts/lib/code-intelligence-policy.js" ] &&
      [ -f "$project/scripts/test-code-intelligence-tools.js" ] &&
      [ -f "$project/scripts/configure-codex-mcp.js" ] &&
      [ -f "$project/scripts/test-codex-mcp-config.js" ] &&
      [ ! -f "$project/tasks/toolchain-discovery.json" ] &&
      [ ! -f "$project/tasks/toolchain-change-strategy.json" ] &&
      (cd "$project" && node scripts/code-intelligence-tools.js validate >/dev/null) &&
      (cd "$project" && node scripts/test-code-intelligence-tools.js >/dev/null) &&
      (cd "$project" && node scripts/test-codex-mcp-config.js >/dev/null) &&
      (cd "$project" && node scripts/configure-codex-mcp.js --check >/dev/null) &&
      node -e "const m=JSON.parse(require('fs').readFileSync(process.argv[1]+'/.template-manifest.json','utf8')); if(m.files?.['.codex/config.toml']?.category!=='hybrid') process.exit(1)" "$project" &&
      node -e "const m=JSON.parse(require('fs').readFileSync(process.argv[1]+'/.mcp.json','utf8')); if(!m.mcpServers?.['codebase-memory-mcp']) process.exit(1)" "$project" &&
      node -e "const m=JSON.parse(require('fs').readFileSync(process.argv[1]+'/.template-manifest.json','utf8')); const paths=['docs/CODE_INTELLIGENCE_TOOLCHAIN.md','_reference/code-intelligence-tools.json','_reference/codex-mcp-config.toml','scripts/code-intelligence-tools.js','scripts/lib/code-intelligence-policy.js','scripts/test-code-intelligence-tools.js','scripts/configure-codex-mcp.js','scripts/test-codex-mcp-config.js']; if(paths.some((p)=>m.files?.[p]?.category!=='template')) process.exit(1)" "$project" &&
      [ -f "$project/tests/fixtures/writing-tools/external-tool-adapter.fixture.js" ] &&
      [ -f "$project/brain/03-knowledge/writing/reference-registry.json" ] &&
      [ -f "$project/DESIGN.md" ] &&
      [ -f "$project/design-policy.ignore" ] &&
      [ -f "$project/tests/fixtures/design-policy/pass/basic.css" ] &&
      [ -f "$project/tests/fixtures/design-policy/fail/gradient-text.css" ] &&
      (cd "$project" && node scripts/test-design-policy.js >/dev/null) &&
      (cd "$project" && node scripts/validate-production-standard.js >/dev/null) &&
      [ -f "$project/tasks/goal.md" ] &&
      node -e "const m=JSON.parse(require('fs').readFileSync(process.argv[1]+'/.template-manifest.json','utf8')); if(m.files?.['DESIGN.md']?.category!=='project') process.exit(1)" "$project" &&
      node -e "const m=JSON.parse(require('fs').readFileSync(process.argv[1]+'/.template-manifest.json','utf8')); if(m.files?.['design-policy.ignore']?.category!=='project') process.exit(1)" "$project" &&
      [ -f "$project/_reference/agent-sot/sources.json" ] &&
      [ -f "$project/_reference/spec-kit/manifest.json" ] &&
      [ -f "$project/integrations/spec-kit/README.md" ] &&
      [ -f "$project/docs/TEMPLATE_RELEASES.md" ] &&
      [ -f "$project/scripts/lib/codex-route-intents.js" ] &&
      [ -f "$project/scripts/lib/writing-intent.js" ] &&
      [ -f "$project/scripts/lib/writing-route-policy.js" ] &&
      [ -f "$project/scripts/lib/writing-reference-policy.js" ] &&
      [ -f "$project/scripts/lib/writing-external-tool-policy.js" ] &&
      (cd "$project" && node scripts/test-writing-intent.js >/dev/null) &&
      (cd "$project" && node scripts/test-writing-references.js >/dev/null) &&
      (cd "$project" && node scripts/validate-writing-references.js >/dev/null) &&
      node -e "const m=JSON.parse(require('fs').readFileSync(process.argv[1]+'/.template-manifest.json','utf8')); const paths=['.claude/library/technical/writing-reference-registry.json','docs/WRITING_WORKFLOW.md','docs/WRITING_REFERENCE_PROVENANCE.md','tests/fixtures/writing-tools/external-tool-adapter.fixture.js','scripts/lib/writing-external-tool-policy.js']; if(paths.some((p)=>m.files?.[p]?.category!=='template')) process.exit(1)" "$project" &&
      node -e "const m=JSON.parse(require('fs').readFileSync(process.argv[1]+'/.template-manifest.json','utf8')); if(m.files?.['scripts/lib/codex-route-intents.js']?.category!=='template') process.exit(1)" "$project" &&
      [ -f "$project/scripts/codex-agent-policy.js" ] &&
      [ -f "$project/scripts/codex-routing-cases-a.js" ] &&
      [ -f "$project/scripts/codex-routing-cases-b.js" ] &&
      [ -f "$project/scripts/codex-route-config.js" ] &&
      [ -f "$project/scripts/codex-route-task.js" ] &&
      [ -f "$project/scripts/test-codex-routing.js" ] &&
      [ -f "$project/scripts/test-codex-agent-policy.js" ] &&
      [ -f "$project/scripts/init-spec-kit.sh" ] &&
      [ -f "$project/scripts/sync-spec-kit.sh" ] &&
      [ -f "$project/scripts/validate-agent-sot.js" ] &&
      [ -f "$project/scripts/validate-spec-kit.js" ] &&
      [ -f "$project/scripts/validate-text-policy.js" ] &&
      [ -f "$project/scripts/lib/change-strategy-policy.js" ] &&
      [ -f "$project/scripts/lib/codex-route-summary.js" ] &&
      [ -f "$project/scripts/lib/codex-route-cli.js" ] &&
      [ -f "$project/scripts/lib/codex-discovery-reroute.js" ] &&
      [ -f "$project/scripts/validate-change-strategy.js" ] &&
      [ -f "$project/scripts/test-change-strategy.js" ] &&
      [ -f "$project/tests/fixtures/change-strategy/discovery-architecture-mismatch.json" ] &&
      [ -f "$project/.github/workflows/validate-template.yml" ] &&
      [ ! -f "$project/.github/workflows/release-template.yml" ] &&
      [ -f "$project/scripts/test-codex-subagents-live.sh" ] &&
      [ -f "$project/scripts/validate-codex-agents.js" ] &&
      [ -f "$project/scripts/validate-codex-skills.js" ] &&
      [ -f "$project/scripts/validate-production-standard.js" ] &&
      [ -f "$project/scripts/progressive-status.js" ] &&
      (cd "$project" && node scripts/validate-codex-agents.js >/dev/null) &&
      (cd "$project" && node scripts/test-codex-agent-policy.js >/dev/null) &&
      (cd "$project" && node scripts/test-codex-routing.js >/dev/null) &&
      (cd "$project" && node scripts/test-change-strategy.js >/dev/null)
  }
  trap cleanup_smoke EXIT
  printf 'sentinel\n' > "$SMOKE_SENTINEL"
  check "setup.sh excludes untracked payload sentinel and ships Codex skills" run_setup_payload_smoke "$SMOKE_PROJECT" "$SMOKE_SENTINEL"
  cleanup_smoke
  trap - EXIT
else
  skip "setup.sh bootstrap smoke (template source repo only)"
fi

echo ""
echo "Sync regression smoke:"
if is_template_source_repo; then
  SYNC_TEMPLATE_FIXTURE="$TEMPLATE_DIR/template-sync-fixture-$RANDOM-$$"
  SYNC_EMPTY_MANIFEST_PROJECT="$TEMPLATE_DIR/template-empty-manifest-smoke-$RANDOM-$$"
  SYNC_EMPTY_MANIFEST_OUTPUT="$SYNC_EMPTY_MANIFEST_PROJECT.out"
  SYNC_SOURCE_ONLY_PROJECT="$TEMPLATE_DIR/template-source-only-sync-smoke-$RANDOM-$$"
  SYNC_SOURCE_ONLY_OUTPUT="$SYNC_SOURCE_ONLY_PROJECT.out"
  SYNC_BOOTSTRAP_DRY_RUN_PROJECT="$TEMPLATE_DIR/template-bootstrap-dry-run-smoke-$RANDOM-$$"
  SYNC_BOOTSTRAP_DRY_RUN_OUTPUT="$SYNC_BOOTSTRAP_DRY_RUN_PROJECT.out"
  SYNC_GIT_TEMPLATE_FIXTURE="$TEMPLATE_DIR/template-sync-git-fixture-$RANDOM-$$"
  SYNC_GIT_DRY_RUN_PROJECT="$TEMPLATE_DIR/template-git-dry-run-smoke-$RANDOM-$$"
  SYNC_GIT_DRY_RUN_OUTPUT="$SYNC_GIT_DRY_RUN_PROJECT.out"
  cleanup_sync_smoke() {
    local path
    for path in \
      "$SYNC_TEMPLATE_FIXTURE" \
      "$SYNC_EMPTY_MANIFEST_PROJECT" "$SYNC_EMPTY_MANIFEST_OUTPUT" "$SYNC_EMPTY_MANIFEST_OUTPUT.apply" \
      "$SYNC_SOURCE_ONLY_PROJECT" "$SYNC_SOURCE_ONLY_OUTPUT" "$SYNC_SOURCE_ONLY_OUTPUT.apply" \
      "$SYNC_BOOTSTRAP_DRY_RUN_PROJECT" "$SYNC_BOOTSTRAP_DRY_RUN_OUTPUT" \
      "$SYNC_GIT_TEMPLATE_FIXTURE" "$SYNC_GIT_DRY_RUN_PROJECT" \
      "$SYNC_GIT_DRY_RUN_OUTPUT" "$SYNC_GIT_DRY_RUN_OUTPUT.apply" "$SYNC_GIT_DRY_RUN_OUTPUT.canary" \
      "$SYNC_GIT_DRY_RUN_OUTPUT.branch" "$SYNC_GIT_DRY_RUN_OUTPUT.conflict" "$SYNC_GIT_DRY_RUN_OUTPUT.missing"; do
      rm -rf "$path" 2>/dev/null || {
        sleep 1
        rm -rf "$path" 2>/dev/null || echo "  WARN: deferred cleanup required: $path"
      }
    done
  }
  create_sync_template_fixture() {
    local template="$1"

    mkdir -p \
      "$template/.agents/skills/codex-change-strategy/agents" \
      "$template/.codex/agents" \
      "$template/.claude/library/process" \
      "$template/.claude/library/technical" \
      "$template/brain/03-knowledge/communication" \
      "$template/scripts" \
      "$template/scripts/lib" \
      "$template/docs" \
      "$template/_reference/spec-kit" \
      "$template/templates/project-starter/tasks" \
      "$template/tests/fixtures/design-policy/pass" \
      "$template/tests/fixtures/design-policy/fail" \
      "$template/tests/fixtures/writing-tools" \
      "$template/tests/fixtures/change-strategy"

    printf '%s\n' '# Fixture Claude' '<!-- Template Version: 9.9.9 -->' > "$template/CLAUDE.md"
    printf '%s\n' '*.log' > "$template/.gitignore"
    printf '%s\n' '# Agent SOT fixture' > "$template/docs/AGENT_CONTEXT_SOT.md"
    printf '%s\n' '{"ref":"fixture"}' > "$template/_reference/spec-kit/manifest.json"
    cp scripts/sync-template.sh "$template/scripts/sync-template.sh"
    cp .codex/config.toml "$template/.codex/config.toml"
    cp .codex/agents/*.toml "$template/.codex/agents/"
    cp -R .agents/skills/. "$template/.agents/skills/"
    cp scripts/codex-agent-policy.js "$template/scripts/codex-agent-policy.js"
    cp scripts/codex-route-config.js "$template/scripts/codex-route-config.js"
    cp scripts/codex-route-task.js "$template/scripts/codex-route-task.js"
    cp scripts/codex-routing-cases-a.js "$template/scripts/codex-routing-cases-a.js"
    cp scripts/codex-routing-cases-b.js "$template/scripts/codex-routing-cases-b.js"
    cp scripts/test-codex-agent-policy.js "$template/scripts/test-codex-agent-policy.js"
    cp scripts/test-codex-routing.js "$template/scripts/test-codex-routing.js"
    cp scripts/test-progressive-plan.js "$template/scripts/test-progressive-plan.js"
    cp scripts/test-change-strategy.js "$template/scripts/test-change-strategy.js"
    cp scripts/test-subagent-trace.js "$template/scripts/test-subagent-trace.js"
    cp scripts/validate-progressive-plan.js "$template/scripts/validate-progressive-plan.js"
    cp scripts/validate-change-strategy.js "$template/scripts/validate-change-strategy.js"
    cp scripts/validate-subagent-trace.js "$template/scripts/validate-subagent-trace.js"
    cp scripts/validate-codex-agents.js "$template/scripts/validate-codex-agents.js"
    cp scripts/lib/codex-route-intents.js "$template/scripts/lib/codex-route-intents.js"
    cp scripts/lib/codex-route-summary.js "$template/scripts/lib/codex-route-summary.js"
    cp scripts/lib/codex-route-cli.js "$template/scripts/lib/codex-route-cli.js"
    cp scripts/lib/codex-discovery-reroute.js "$template/scripts/lib/codex-discovery-reroute.js"
    cp scripts/lib/writing-intent.js "$template/scripts/lib/writing-intent.js"
    cp scripts/lib/writing-route-policy.js "$template/scripts/lib/writing-route-policy.js"
    cp scripts/lib/writing-reference-policy.js "$template/scripts/lib/writing-reference-policy.js"
    cp scripts/lib/writing-external-tool-policy.js "$template/scripts/lib/writing-external-tool-policy.js"
    cp scripts/lib/writing-path-policy.js "$template/scripts/lib/writing-path-policy.js"
    cp scripts/lib/progressive-plan.js "$template/scripts/lib/progressive-plan.js"
    cp scripts/lib/change-strategy-policy.js "$template/scripts/lib/change-strategy-policy.js"
    cp scripts/lib/code-intelligence-policy.js "$template/scripts/lib/code-intelligence-policy.js"
    cp scripts/lib/subagent-trace.js "$template/scripts/lib/subagent-trace.js"
    cp .claude/library/process/change-strategy-gate.md "$template/.claude/library/process/change-strategy-gate.md"
    cp .claude/library/technical/writing-reference-registry.json "$template/.claude/library/technical/writing-reference-registry.json"
    cp docs/WRITING_REFERENCE_PROVENANCE.md "$template/docs/WRITING_REFERENCE_PROVENANCE.md"
    cp brain/03-knowledge/communication/ilyakhov-planning-principles.md "$template/brain/03-knowledge/communication/ilyakhov-planning-principles.md"
    cp .agents/skills/codex-change-strategy/SKILL.md "$template/.agents/skills/codex-change-strategy/SKILL.md"
    cp .agents/skills/codex-change-strategy/agents/openai.yaml "$template/.agents/skills/codex-change-strategy/agents/openai.yaml"
    cp tests/fixtures/change-strategy/discovery-architecture-mismatch.json "$template/tests/fixtures/change-strategy/discovery-architecture-mismatch.json"

    node -e 'const fs=require("fs"),path=require("path"),q=String.fromCharCode(39); const name="skill"+q+"]);require("+q+"fs"+q+").writeFileSync("+q+"SYNC_PATH_INJECTION"+q+","+q+"x"+q+");console.log(m.files["+q+"skill"; const dir=path.join(process.argv[1],".agents","skills",name); fs.mkdirSync(dir,{recursive:true}); fs.writeFileSync(path.join(dir,"SKILL.md"),"# inert path fixture\n","utf8");' "$template"

    printf '%s\n' '# source-only unix setup fixture' > "$template/setup.sh"
    printf '%s\r\n' '@echo off' 'rem source-only windows setup fixture' > "$template/setup.bat"
    printf '%s\n' '# source-only starter task fixture' > "$template/templates/project-starter/tasks/current.md"
    printf '%s\n' '.fixture-pass { color: var(--color-text); }' > "$template/tests/fixtures/design-policy/pass/basic.css"
    printf '%s\n' '.fixture-fail { background: linear-gradient(red, blue); background-clip: text; }' > "$template/tests/fixtures/design-policy/fail/gradient-text.css"
    printf '%s\n' 'module.exports = { provider: "fixture", configured: false };' > "$template/tests/fixtures/writing-tools/external-tool-adapter.fixture.js"
  }
  write_empty_trackable_manifest() {
    local project="$1"
    mkdir -p "$project"
    printf '%s\n' \
      '{' \
      '  "template_version": "unknown",' \
      '  "created": "2000-01-01",' \
      '  "updated": "2000-01-01",' \
      '  "template_remote": "",' \
      '  "files": {' \
      '    "CLAUDE.md": {"category": "project", "hash": "fixture"}' \
      '  }' \
      '}' > "$project/.template-manifest.json"
  }
  write_trackable_manifest() {
    local project="$1"
    local hash

    mkdir -p "$project"
    printf '%s\n' '# Local Claude' '<!-- Template Version: 4.1.1 -->' > "$project/CLAUDE.md"
    hash="$(_get_hash "$project/CLAUDE.md")"
    printf '%s\n' \
      '{' \
      '  "template_version": "4.1.1",' \
      '  "created": "2000-01-01",' \
      '  "updated": "2000-01-01",' \
      '  "template_remote": "",' \
      '  "files": {' \
      "    \"CLAUDE.md\": {\"category\": \"template\", \"hash\": \"$hash\"}" \
      '  }' \
      '}' > "$project/.template-manifest.json"
  }
  run_empty_manifest_sync_smoke() {
    local project="$1"
    local output="$2"

    write_empty_trackable_manifest "$project" || return 1

    bash scripts/sync-template.sh "$SYNC_TEMPLATE_FIXTURE" --project-dir "$project" --dry-run > "$output" 2>&1 || return 1
    grep -q "Manifest has no trackable files" "$output" || return 1
    grep -q "WOULD ADD: scripts/sync-template.sh" "$output" || return 1
    grep -q "WOULD ADD: CLAUDE.md" "$output" || return 1
    grep -q "WOULD ADD: .gitignore" "$output" || return 1
    grep -q "WOULD ADD: docs/AGENT_CONTEXT_SOT.md" "$output" || return 1
    grep -q "WOULD ADD: scripts/codex-agent-policy.js" "$output" || return 1
    grep -q "WOULD ADD: .codex/agents/product-reviewer.toml" "$output" || return 1
    grep -q "WOULD ADD: .codex/agents/systems-reviewer.toml" "$output" || return 1
    grep -q "WOULD ADD: .codex/agents/scout.toml" "$output" || return 1
    grep -q "WOULD ADD: scripts/validate-progressive-plan.js" "$output" || return 1
    grep -q "WOULD ADD: scripts/validate-change-strategy.js" "$output" || return 1
    grep -q "WOULD ADD: scripts/lib/codex-route-summary.js" "$output" || return 1
    grep -q "WOULD ADD: scripts/lib/codex-route-cli.js" "$output" || return 1
    grep -q "WOULD ADD: scripts/lib/codex-discovery-reroute.js" "$output" || return 1
    grep -q "WOULD ADD: brain/03-knowledge/communication/ilyakhov-planning-principles.md" "$output" || return 1
    grep -q "WOULD ADD: .agents/skills/codex-change-strategy/SKILL.md" "$output" || return 1
    grep -q "WOULD ADD: tests/fixtures/change-strategy/discovery-architecture-mismatch.json" "$output" || return 1
    grep -q "WOULD ADD: _reference/spec-kit/manifest.json" "$output" || return 1
    grep -q "WOULD ADD: tests/fixtures/design-policy/fail/gradient-text.css" "$output" || return 1
    grep -q "WOULD ADD: tests/fixtures/writing-tools/external-tool-adapter.fixture.js" "$output" || return 1
    [ ! -e "$project/SYNC_PATH_INJECTION" ] || return 1

    if ! bash scripts/sync-template.sh "$SYNC_TEMPLATE_FIXTURE" --project-dir "$project" > "$output.apply" 2>&1; then
      cat "$output.apply"
      return 1
    fi
    grep -q '"CLAUDE.md"' "$project/.template-manifest.json" || return 1
    grep -q '"docs/AGENT_CONTEXT_SOT.md"' "$project/.template-manifest.json" || return 1
    grep -q '"scripts/codex-agent-policy.js"' "$project/.template-manifest.json" || return 1
    grep -q '"scripts/lib/codex-route-intents.js"' "$project/.template-manifest.json" || return 1
    grep -q '".codex/agents/product-reviewer.toml"' "$project/.template-manifest.json" || return 1
    grep -q '".codex/agents/systems-reviewer.toml"' "$project/.template-manifest.json" || return 1
    grep -q '".codex/agents/scout.toml"' "$project/.template-manifest.json" || return 1
    grep -q '"scripts/validate-progressive-plan.js"' "$project/.template-manifest.json" || return 1
    grep -q '"scripts/validate-change-strategy.js"' "$project/.template-manifest.json" || return 1
    grep -q '"scripts/lib/codex-route-summary.js"' "$project/.template-manifest.json" || return 1
    grep -q '"scripts/lib/codex-route-cli.js"' "$project/.template-manifest.json" || return 1
    grep -q '"scripts/lib/codex-discovery-reroute.js"' "$project/.template-manifest.json" || return 1
    grep -q '"brain/03-knowledge/communication/ilyakhov-planning-principles.md"' "$project/.template-manifest.json" || return 1
    [ -f "$project/brain/03-knowledge/communication/ilyakhov-planning-principles.md" ] || return 1
    grep -q '".agents/skills/codex-change-strategy/SKILL.md"' "$project/.template-manifest.json" || return 1
    grep -q '"tests/fixtures/change-strategy/discovery-architecture-mismatch.json"' "$project/.template-manifest.json" || return 1
    grep -q '"_reference/spec-kit/manifest.json"' "$project/.template-manifest.json" || return 1
    grep -q '"tests/fixtures/design-policy/fail/gradient-text.css"' "$project/.template-manifest.json" || return 1
    grep -q '"tests/fixtures/writing-tools/external-tool-adapter.fixture.js"' "$project/.template-manifest.json" || return 1
    cmp "$SYNC_TEMPLATE_FIXTURE/tests/fixtures/writing-tools/external-tool-adapter.fixture.js" "$project/tests/fixtures/writing-tools/external-tool-adapter.fixture.js" || return 1
    ! grep -q '"templates/' "$project/.template-manifest.json" || return 1
    ! grep -q '"setup.sh"' "$project/.template-manifest.json" || return 1
    ! grep -q '"setup.bat"' "$project/.template-manifest.json" || return 1
    (cd "$project" && node scripts/validate-codex-agents.js >/dev/null) || return 1
    (cd "$project" && node scripts/test-codex-agent-policy.js >/dev/null) || return 1
    (cd "$project" && node scripts/test-codex-routing.js >/dev/null) || return 1
    (cd "$project" && node scripts/test-progressive-plan.js >/dev/null) || return 1
    (cd "$project" && node scripts/test-change-strategy.js >/dev/null) || return 1
    (cd "$project" && node scripts/test-subagent-trace.js >/dev/null) || return 1
    [ ! -e "$project/SYNC_PATH_INJECTION" ] || return 1
  }
  run_source_only_sync_smoke() {
    local project="$1"
    local output="$2"

    write_trackable_manifest "$project" || return 1
    mkdir -p "$project/scripts/lib"
    printf '%s\n' 'module.exports = { legacy: true };' > "$project/scripts/lib/codex-route-intents.js"

    bash scripts/sync-template.sh "$SYNC_TEMPLATE_FIXTURE" --project-dir "$project" --dry-run > "$output" 2>&1 || return 1
    grep -q "WOULD UPDATE: scripts/lib/codex-route-intents.js" "$output" || return 1
    ! grep -q "WOULD ADD: templates/" "$output" || return 1
    ! grep -q "WOULD ADD: setup.sh" "$output" || return 1
    ! grep -q "WOULD ADD: setup.bat" "$output" || return 1

    if ! bash scripts/sync-template.sh "$SYNC_TEMPLATE_FIXTURE" --project-dir "$project" > "$output.apply" 2>&1; then
      cat "$output.apply"
      return 1
    fi
    cmp -s "$project/scripts/lib/codex-route-intents.js" "$SYNC_TEMPLATE_FIXTURE/scripts/lib/codex-route-intents.js" || return 1
    grep -q '"scripts/lib/codex-route-intents.js"' "$project/.template-manifest.json" || return 1
    [ ! -e "$project/templates" ] &&
      [ ! -f "$project/setup.sh" ] &&
      [ ! -f "$project/setup.bat" ] &&
      ! grep -q '"templates/' "$project/.template-manifest.json" &&
      ! grep -q '"setup.sh"' "$project/.template-manifest.json" &&
      ! grep -q '"setup.bat"' "$project/.template-manifest.json"
  }
  run_bootstrap_dry_run_smoke() {
    local template="$1"
    local project="$2"
    local output="$3"

    mkdir -p "$project" || return 1
    bash scripts/sync-template.sh "$template" --project-dir "$project" --bootstrap --dry-run > "$output" 2>&1 || return 1
    grep -q "WOULD BOOTSTRAP" "$output" || return 1
    [ ! -e "$project/.template-manifest.json" ] || return 1
  }
  run_from_git_dry_run_smoke() {
    local template="$1"
    local project="$2"
    local output="$3"

    create_sync_template_fixture "$template" || return 1
    git -C "$template" init -q || return 1
    git -C "$template" add . || return 1
    git -C "$template" -c user.name="Template Smoke" -c user.email="template-smoke@example.invalid" commit -q -m "fixture" || return 1
    git -C "$template" branch -M main || return 1
    git -C "$template" tag v9.9.9 || return 1
    sed -i 's/9\.9\.9/10.0.0/g' "$template/CLAUDE.md" || return 1
    git -C "$template" add CLAUDE.md || return 1
    git -C "$template" -c user.name="Template Smoke" -c user.email="template-smoke@example.invalid" commit -q -m "main diverges from release" || return 1

    write_trackable_manifest "$project" || return 1
    git -C "$project" init -q || return 1
    local template_remote
    template_remote="$(cd "$template" && pwd)"
    node -e "const fs=require('fs'); const p=process.argv[1],m=JSON.parse(fs.readFileSync(p,'utf8')); m.template_remote=process.argv[2]; fs.writeFileSync(p,JSON.stringify(m,null,2));" "$project/.template-manifest.json" "$template_remote" || return 1
    git -C "$project" add CLAUDE.md .template-manifest.json || return 1
    git -C "$project" -c user.name="Project Smoke" -c user.email="project-smoke@example.invalid" commit -q -m "project fixture" || return 1

    local manifest_before
    local claude_before
    manifest_before="$(_get_hash "$project/.template-manifest.json")"
    claude_before="$(_get_hash "$project/CLAUDE.md")"
    bash scripts/sync-template.sh --from-git --ref v9.9.9 --project-dir "$project" --dry-run > "$output" 2>&1 || return 1
    grep -q "Fetching template preview from" "$output" || return 1
    grep -q "Current: 4.1.1.*New: 9.9.9" "$output" || return 1
    grep -q "WOULD UPDATE: CLAUDE.md" "$output" || return 1
    grep -q "(Dry run" "$output" || return 1
    [ "$manifest_before" = "$(_get_hash "$project/.template-manifest.json")" ] || return 1
    [ "$claude_before" = "$(_get_hash "$project/CLAUDE.md")" ] || return 1
    ! git -C "$project" remote get-url template >/dev/null 2>&1 || return 1
    [ ! -e "$project/.git/FETCH_HEAD" ] || return 1
    git -C "$project" remote add template "$template_remote-conflict" || return 1
    if bash scripts/sync-template.sh --from-git --ref v9.9.9 --project-dir "$project" --dry-run > "$output.conflict" 2>&1; then
      return 1
    fi
    grep -q "Template source conflict" "$output.conflict" || return 1
    git -C "$project" remote remove template || return 1
    if bash scripts/sync-template.sh --from-git --ref main --project-dir "$project" --dry-run > "$output.branch" 2>&1; then
      return 1
    fi
    grep -q "Non-release ref 'main' requires --canary" "$output.branch" || return 1
    bash scripts/sync-template.sh --from-git --canary --ref main --project-dir "$project" --dry-run > "$output.canary" 2>&1 || return 1
    grep -q "Current: 4.1.1.*New: 10.0.0" "$output.canary" || return 1
    [ "$manifest_before" = "$(_get_hash "$project/.template-manifest.json")" ] || return 1
    [ "$claude_before" = "$(_get_hash "$project/CLAUDE.md")" ] || return 1
    ! git -C "$project" remote get-url template >/dev/null 2>&1 || return 1
    [ ! -e "$project/.git/FETCH_HEAD" ] || return 1
    if bash scripts/sync-template.sh --from-git --ref v9.9.8 --project-dir "$project" --dry-run > "$output.missing" 2>&1; then
      return 1
    fi
    grep -q "Cannot fetch template ref: v9.9.8" "$output.missing" || return 1
    if ! bash scripts/sync-template.sh --from-git --ref v9.9.9 --project-dir "$project" > "$output.apply" 2>&1; then
      cat "$output.apply"
      return 1
    fi
    node -e "const m=require(process.argv[1]); if(m.template_version!=='9.9.9') process.exit(1)" "$project/.template-manifest.json" || return 1
    grep -q 'Template Version: 9.9.9' "$project/CLAUDE.md" || return 1
    ! grep -q 'Template Version: 10.0.0' "$project/CLAUDE.md" || return 1
    local applied_remote
    applied_remote="$(git -C "$project" remote get-url template)"
    [ "$(cd "$applied_remote" && pwd)" = "$(cd "$template_remote" && pwd)" ] || return 1
  }
  trap cleanup_sync_smoke EXIT
  create_sync_template_fixture "$SYNC_TEMPLATE_FIXTURE"
  check "sync-template dry-run handles empty trackable manifest" run_empty_manifest_sync_smoke "$SYNC_EMPTY_MANIFEST_PROJECT" "$SYNC_EMPTY_MANIFEST_OUTPUT"
  check "sync-template keeps source-only files out of generated projects" run_source_only_sync_smoke "$SYNC_SOURCE_ONLY_PROJECT" "$SYNC_SOURCE_ONLY_OUTPUT"
  check "sync-template bootstrap dry-run leaves legacy project unchanged" run_bootstrap_dry_run_smoke "$SYNC_TEMPLATE_FIXTURE" "$SYNC_BOOTSTRAP_DRY_RUN_PROJECT" "$SYNC_BOOTSTRAP_DRY_RUN_OUTPUT"
  check "sync-template pinned ref previews, rejects missing tags, and applies exact release" run_from_git_dry_run_smoke "$SYNC_GIT_TEMPLATE_FIXTURE" "$SYNC_GIT_DRY_RUN_PROJECT" "$SYNC_GIT_DRY_RUN_OUTPUT"
  cleanup_sync_smoke
  trap - EXIT
else
  skip "sync-template empty-manifest smoke (template source repo only)"
fi

echo ""
echo "Results: $((CHECKS-ERRORS))/$CHECKS passed"
if [ $ERRORS -eq 0 ]; then
    echo "Template is healthy!"
    exit 0
else
    echo "$ERRORS check(s) failed"
    exit 1
fi
