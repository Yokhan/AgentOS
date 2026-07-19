#!/usr/bin/env node
const fs = require("fs");
const path = require("path");

const SKILLS_ROOT = ".agents/skills";
const MAX_DESCRIPTION_CHARS = 420;
const MAX_SKILL_LINES = 500;
const REQUIRED_CORE_SKILLS = [
  "codex-pipeline-workflow",
  "codex-design-workflow",
  "codex-figma-workflow",
  "codex-audit",
  "codex-debug",
  "codex-security-audit",
  "codex-setup-project",
  "codex-feature-workflow",
  "codex-domain-design-review",
  "codex-openai-model-guidance",
  "codex-agent-router",
  "codex-subagent-orchestration",
  "codex-mermaid-board-workflow",
  "codex-product-goal",
  "codex-design-system-workflow",
  "codex-product-ux-audit",
  "codex-cross-project-lessons",
  "codex-writing-workflow",
  "codex-technical-writing",
  "codex-technical-writing-review",
];

const state = {
  checks: 0,
  errors: [],
  warnings: [],
};

function addError(message) {
  state.errors.push(message);
}

function addWarning(message) {
  state.warnings.push(message);
}

function readText(filePath) {
  return fs.readFileSync(filePath, "utf8");
}

function getSkillDirs(root) {
  if (!fs.existsSync(root)) {
    return [];
  }

  return fs
    .readdirSync(root, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(root, entry.name));
}

function parseFrontmatter(content) {
  const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n/);
  if (!match) {
    return null;
  }

  const fields = {};
  for (const line of match[1].split(/\r?\n/)) {
    const field = line.match(/^([A-Za-z_-]+):\s*(.*)$/);
    if (!field) {
      continue;
    }

    fields[field[1]] = field[2].replace(/^["']|["']$/g, "").trim();
  }
  return fields;
}

function validateOpenAiYaml(skillName, skillDir) {
  const metadataPath = path.join(skillDir, "agents", "openai.yaml");
  if (!fs.existsSync(metadataPath)) {
    addWarning(`${skillName}: agents/openai.yaml missing`);
    return;
  }

  const content = readText(metadataPath);
  if (!content.includes("interface:")) {
    addError(`${skillName}: openai.yaml missing interface`);
  }
  if (!content.includes(`$${skillName}`)) {
    addError(
      `${skillName}: openai.yaml default_prompt must mention $${skillName}`,
    );
  }
}

function validateSkill(skillDir) {
  const skillName = path.basename(skillDir);
  const skillPath = path.join(skillDir, "SKILL.md");
  state.checks += 1;

  if (skillName.startsWith("project-")) {
    return;
  }
  if (!fs.existsSync(skillPath)) {
    addError(`${skillName}: missing SKILL.md`);
    return;
  }

  const content = readText(skillPath);
  const frontmatter = parseFrontmatter(content);
  const lineCount = content.split(/\r?\n/).length;

  if (!frontmatter) {
    addError(`${skillName}: missing YAML frontmatter`);
    return;
  }
  if (frontmatter.name !== skillName) {
    addError(`${skillName}: frontmatter name must match directory name`);
  }
  if (!frontmatter.description) {
    addError(`${skillName}: missing description`);
  }
  if ((frontmatter.description || "").length > MAX_DESCRIPTION_CHARS) {
    addWarning(
      `${skillName}: description exceeds ${MAX_DESCRIPTION_CHARS} chars`,
    );
  }
  if (lineCount > MAX_SKILL_LINES) {
    addWarning(`${skillName}: SKILL.md exceeds ${MAX_SKILL_LINES} lines`);
  }

  validateOpenAiYaml(skillName, skillDir);
}

function validateRequiredSkills(skillDirs) {
  const available = new Set(
    skillDirs.map((skillDir) => path.basename(skillDir)),
  );
  for (const skillName of REQUIRED_CORE_SKILLS) {
    state.checks += 1;
    if (!available.has(skillName)) {
      addError(`missing required core skill: ${skillName}`);
    }
  }
}

function main() {
  const skillDirs = getSkillDirs(SKILLS_ROOT);
  if (skillDirs.length === 0) {
    addError(`no skills found under ${SKILLS_ROOT}`);
  }

  for (const skillDir of skillDirs) {
    validateSkill(skillDir);
  }
  validateRequiredSkills(skillDirs);

  console.log(`Codex skills checked: ${state.checks}`);
  for (const warning of state.warnings) {
    console.log(`WARNING: ${warning}`);
  }
  for (const error of state.errors) {
    console.error(`ERROR: ${error}`);
  }

  if (state.errors.length > 0) {
    process.exit(1);
  }
}

main();
