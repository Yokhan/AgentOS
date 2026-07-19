#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

const SNAPSHOT_ROOT = "_reference/spec-kit/upstream";
const MANIFEST_PATH = "_reference/spec-kit/manifest.json";
const REQUIRED_FILES = [
  "_reference/spec-kit/README.md",
  MANIFEST_PATH,
  `${SNAPSHOT_ROOT}/README.md`,
  `${SNAPSHOT_ROOT}/LICENSE`,
  `${SNAPSHOT_ROOT}/spec-driven.md`,
  `${SNAPSHOT_ROOT}/docs/installation.md`,
  `${SNAPSHOT_ROOT}/docs/quickstart.md`,
  `${SNAPSHOT_ROOT}/docs/upgrade.md`,
  `${SNAPSHOT_ROOT}/docs/reference/core.md`,
  `${SNAPSHOT_ROOT}/docs/reference/integrations.md`,
  `${SNAPSHOT_ROOT}/docs/reference/extensions.md`,
  `${SNAPSHOT_ROOT}/docs/reference/presets.md`,
  `${SNAPSHOT_ROOT}/templates/spec-template.md`,
  `${SNAPSHOT_ROOT}/templates/plan-template.md`,
  `${SNAPSHOT_ROOT}/templates/tasks-template.md`,
  `${SNAPSHOT_ROOT}/templates/constitution-template.md`,
  `${SNAPSHOT_ROOT}/templates/commands/specify.md`,
  `${SNAPSHOT_ROOT}/templates/commands/plan.md`,
  `${SNAPSHOT_ROOT}/templates/commands/tasks.md`,
  `${SNAPSHOT_ROOT}/templates/commands/implement.md`,
  `${SNAPSHOT_ROOT}/scripts/bash/check-prerequisites.sh`,
  `${SNAPSHOT_ROOT}/scripts/powershell/check-prerequisites.ps1`,
  `${SNAPSHOT_ROOT}/integrations/catalog.json`,
];

const errors = [];

const readJson = (filePath) => JSON.parse(fs.readFileSync(filePath, "utf8"));

const listFiles = (dirPath) => {
  if (!fs.existsSync(dirPath)) {
    return [];
  }

  const entries = fs.readdirSync(dirPath, { withFileTypes: true });
  return entries.flatMap((entry) => {
    const entryPath = path.join(dirPath, entry.name);
    if (entry.isDirectory()) {
      return listFiles(entryPath);
    }
    return [entryPath];
  });
};

for (const filePath of REQUIRED_FILES) {
  if (!fs.existsSync(filePath)) {
    errors.push(`Missing Spec Kit snapshot file: ${filePath}`);
  }
}

if (fs.existsSync(path.join(SNAPSHOT_ROOT, ".git"))) {
  errors.push("Spec Kit snapshot must not include a .git directory");
}

if (fs.existsSync(MANIFEST_PATH)) {
  const manifest = readJson(MANIFEST_PATH);
  const requiredManifestFields = ["repo", "ref", "commit", "fetched_at", "snapshot_root"];

  for (const field of requiredManifestFields) {
    if (!manifest[field]) {
      errors.push(`Spec Kit manifest missing field: ${field}`);
    }
  }

  if (manifest.repo !== "https://github.com/github/spec-kit.git") {
    errors.push(`Unexpected Spec Kit repo: ${manifest.repo}`);
  }

  if (manifest.snapshot_root !== SNAPSHOT_ROOT) {
    errors.push(`Unexpected Spec Kit snapshot_root: ${manifest.snapshot_root}`);
  }
}

const templateFiles = listFiles(`${SNAPSHOT_ROOT}/templates`).filter((filePath) => filePath.endsWith(".md"));
const commandFiles = listFiles(`${SNAPSHOT_ROOT}/templates/commands`).filter((filePath) => filePath.endsWith(".md"));
const bashScripts = listFiles(`${SNAPSHOT_ROOT}/scripts/bash`).filter((filePath) => filePath.endsWith(".sh"));
const psScripts = listFiles(`${SNAPSHOT_ROOT}/scripts/powershell`).filter((filePath) => filePath.endsWith(".ps1"));

if (templateFiles.length < 10) {
  errors.push(`Expected at least 10 Spec Kit template files, found ${templateFiles.length}`);
}

if (commandFiles.length < 8) {
  errors.push(`Expected at least 8 Spec Kit command templates, found ${commandFiles.length}`);
}

if (bashScripts.length < 5) {
  errors.push(`Expected at least 5 Spec Kit bash scripts, found ${bashScripts.length}`);
}

if (psScripts.length < 5) {
  errors.push(`Expected at least 5 Spec Kit PowerShell scripts, found ${psScripts.length}`);
}

if (errors.length > 0) {
  for (const error of errors) {
    console.error(`ERROR: ${error}`);
  }
  process.exit(1);
}

console.log(
  `Spec Kit snapshot validates (${templateFiles.length} templates, ${commandFiles.length} commands)`
);
