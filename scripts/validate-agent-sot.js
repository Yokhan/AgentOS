#!/usr/bin/env node

const fs = require("fs");
const crypto = require("crypto");

const MAX_STALENESS_DAYS = 30;
const REQUIRED_FILES = [
  "docs/AGENT_CONTEXT_SOT.md",
  "_reference/agent-sot/README.md",
  "_reference/agent-sot/sources.json",
  "_reference/agent-sot/top-works.md",
  "_reference/agent-sot/originals/ai-agent-spec-v3-final.md",
];

const REQUIRED_SOURCE_IDS = [
  "local-ai-agent-spec-v3",
  "openai-codex-agents-md",
  "openai-codex-skills",
  "openai-codex-hooks",
  "openai-codex-subagents",
  "zed-external-agents",
  "local-spec-kit-snapshot",
  "claude-code-extend",
  "claude-code-skills",
  "claude-code-hooks",
  "claude-code-subagents",
  "github-spec-kit",
  "trailofbits-claude-code-config",
  "humanlayer-claude-md",
  "nx-ai-agent-skills",
  "anthropic-building-effective-agents",
  "anthropic-effective-context-engineering",
  "anthropic-multi-agent-research-system",
  "martinfowler-context-engineering-coding-agents",
  "martinfowler-harness-engineering",
  "thoughtworks-context-engineering-radar",
  "arxiv-codified-context",
  "openreview-agentic-context-engineering",
  "steve-yegge-beads",
  "boris-tane-claude-code-workflow",
  "ian-bull-sinks-not-pipes",
  "ian-bull-working-memory-cliff",
  "ian-bull-planning-bottleneck",
  "ian-bull-change-reviews",
  "humanlayer-12-factor-agents",
  "vuong-ngo-scaffolding-monorepo",
  "boris-cherny-customization-tips",
  "arxiv-spec-kit-agents",
  "arxiv-dive-into-claude-code",
  "arxiv-agentic-context-description-language",
  "arxiv-efficient-agents",
];

const requiredMentions = [
  ["AGENTS.md", "docs/AGENT_CONTEXT_SOT.md"],
  ["AGENTS.md", "SOT Conflict Protocol"],
  ["AGENTS.md", "Thinking Tools Gate"],
  ["CLAUDE.md", "docs/AGENT_CONTEXT_SOT.md"],
  ["CLAUDE.md", "SOT conflict protocol"],
  ["CLAUDE.md", "Thinking tools gate"],
  ["docs/AGENT_CONTEXT_SOT.md", "SOT Conflict Protocol"],
  ["docs/AGENT_CONTEXT_SOT.md", "ask the user with 2-3 options"],
  ["scripts/validate-template.sh", "validate-agent-sot.js"],
  ["scripts/test-template.sh", "validate-agent-sot"],
  ["scripts/check-drift.sh", "validate-agent-sot"],
];

const errors = [];
const warnings = [];

const readText = (path) => fs.readFileSync(path, "utf8");

const hasFile = (path) => {
  if (!fs.existsSync(path)) {
    errors.push(`Missing required Agent SOT file: ${path}`);
    return false;
  }
  return true;
};

const parseDate = (value) => {
  const date = new Date(`${value}T00:00:00Z`);
  return Number.isNaN(date.getTime()) ? null : date;
};

const daysBetween = (a, b) =>
  Math.floor((a.getTime() - b.getTime()) / 86400000);

const hashFile = (path) => {
  const bytes = fs.readFileSync(path);
  return crypto.createHash("sha256").update(bytes).digest("hex");
};

for (const path of REQUIRED_FILES) {
  hasFile(path);
}

let registry = null;
let sourceIds = new Set();
if (fs.existsSync("_reference/agent-sot/sources.json")) {
  try {
    registry = JSON.parse(readText("_reference/agent-sot/sources.json"));
  } catch (error) {
    errors.push(`sources.json is invalid JSON: ${error.message}`);
  }
}

if (registry) {
  const sources = Array.isArray(registry.sources) ? registry.sources : [];
  sourceIds = new Set(sources.map((source) => source.id));

  for (const id of REQUIRED_SOURCE_IDS) {
    if (!sourceIds.has(id)) {
      errors.push(`Missing required source id: ${id}`);
    }
  }

  const today = new Date();
  for (const source of sources) {
    if (
      !source.id ||
      !source.title ||
      !source.last_checked ||
      !source.local_conclusion
    ) {
      errors.push(
        `Source is missing required fields: ${source.id || "<unknown>"}`,
      );
      continue;
    }

    if (source.kind !== "local-original" && !source.url) {
      errors.push(`Non-local source is missing url: ${source.id}`);
    }

    const checkedAt = parseDate(source.last_checked);
    if (!checkedAt) {
      errors.push(`Source has invalid last_checked date: ${source.id}`);
      continue;
    }

    if (
      source.requires_fresh_check &&
      daysBetween(today, checkedAt) > MAX_STALENESS_DAYS
    ) {
      warnings.push(
        `Source should be rechecked before behavior changes: ${source.id}`,
      );
    }

    if (
      source.local_path &&
      fs.existsSync(source.local_path) &&
      source.sha256
    ) {
      const actualHash = hashFile(source.local_path);
      if (actualHash !== source.sha256) {
        errors.push(`Source hash mismatch for ${source.id}: ${actualHash}`);
      }
    }
  }
}

let topWorkCount = 0;
if (fs.existsSync("_reference/agent-sot/top-works.md")) {
  const topWorks = readText("_reference/agent-sot/top-works.md");
  topWorkCount = (topWorks.match(/^## TW-\d+/gm) || []).length;
  const referencedIds = [...topWorks.matchAll(/source_id:\s+`([^`]+)`/g)].map(
    (match) => match[1],
  );

  if (topWorkCount < 20) {
    errors.push(
      `top-works.md must include at least 20 source cards; found ${topWorkCount}`,
    );
  }

  if (referencedIds.length !== topWorkCount) {
    errors.push("Each top work must include a source_id line");
  }

  for (const id of referencedIds) {
    if (!sourceIds.has(id)) {
      errors.push(`top-works.md references missing source id: ${id}`);
    }
  }
}

for (const [path, text] of requiredMentions) {
  if (!fs.existsSync(path)) {
    errors.push(`Cannot check mention; missing ${path}`);
    continue;
  }

  if (!readText(path).includes(text)) {
    errors.push(`${path} must mention ${text}`);
  }
}

if (warnings.length > 0) {
  for (const warning of warnings) {
    console.warn(`WARNING: ${warning}`);
  }
}

if (errors.length > 0) {
  for (const error of errors) {
    console.error(`ERROR: ${error}`);
  }
  process.exit(1);
}

console.log(
  `Agent SOT validation passed (${REQUIRED_SOURCE_IDS.length} required sources, ${topWorkCount} top works)`,
);
