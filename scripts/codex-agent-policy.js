"use strict";

const path = require("path");

const EFFORT_LEVELS = Object.freeze(["low", "medium", "high", "xhigh"]);

const AGENT_POLICY = Object.freeze({
  version: "4.9.0",
  parent: Object.freeze({
    modelSource: "user-or-ide",
    recommendedModel: "gpt-5.6-sol",
    baselineEffort: "medium",
    effortCeiling: "xhigh",
  }),
  fanout: Object.freeze({
    maxChildren: 3,
    maxDepth: 1,
    maxAutomaticWaves: 1,
    notifyUser: true,
    readOnlyFirst: true,
    requireRuntimeProfileEvidence: true,
    writePolicy: "exact-non-overlapping-files-only",
  }),
  profiles: Object.freeze({
    scout: Object.freeze({
      file: "scout.toml",
      model: "gpt-5.6-luna",
      effort: "low",
      sandboxMode: "read-only",
    }),
    log_analyst: Object.freeze({
      file: "log-analyst.toml",
      model: "gpt-5.6-luna",
      effort: "low",
      sandboxMode: "read-only",
    }),
    summarizer: Object.freeze({
      file: "summarizer.toml",
      model: "gpt-5.6-luna",
      effort: "low",
      sandboxMode: "read-only",
    }),
    pr_explorer: Object.freeze({
      file: "pr-explorer.toml",
      model: "gpt-5.6-terra",
      effort: "medium",
      sandboxMode: "read-only",
    }),
    docs_researcher: Object.freeze({
      file: "docs-researcher.toml",
      model: "gpt-5.6-terra",
      effort: "medium",
      sandboxMode: "read-only",
    }),
    tester: Object.freeze({
      file: "tester.toml",
      model: "gpt-5.6-terra",
      effort: "medium",
      sandboxMode: "read-only",
    }),
    implementer: Object.freeze({
      file: "implementer.toml",
      model: "gpt-5.6-terra",
      effort: "high",
      sandboxMode: "workspace-write",
    }),
    reviewer: Object.freeze({
      file: "reviewer.toml",
      model: "gpt-5.6-sol",
      effort: "high",
      sandboxMode: "read-only",
    }),
    design_reviewer: Object.freeze({
      file: "design-reviewer.toml",
      model: "gpt-5.6-sol",
      effort: "high",
      sandboxMode: "read-only",
    }),
    product_reviewer: Object.freeze({
      file: "product-reviewer.toml",
      model: "gpt-5.6-sol",
      effort: "high",
      sandboxMode: "read-only",
    }),
    security_reviewer: Object.freeze({
      file: "security-reviewer.toml",
      model: "gpt-5.6-sol",
      effort: "xhigh",
      sandboxMode: "read-only",
    }),
    systems_reviewer: Object.freeze({
      file: "systems-reviewer.toml",
      model: "gpt-5.6-sol",
      effort: "xhigh",
      sandboxMode: "read-only",
    }),
  }),
});

const OPT_OUT_PATTERN =
  /(?:\b(?:do not|don't|dont|never)\s+(?:(?:use|spawn|run|call)\s+)?(?:any\s+)?(?:sub-?agents?|delegation|fan-?out)\b|\b(?:do not|don't|dont|never)\s+delegate\b|\bwithout\s+(?:(?:using|any)\s+)?(?:sub-?agents?|delegation|fan-?out)\b|\bno\s+(?:sub-?agents?|delegation|fan-?out)\b|(?:без|не\s+(?:используй|запускай|вызывай|делегируй))\s+(?:любых\s+)?(?:субагент|сабагент|делегац|фан-?аут)|не\s+делегируй)/iu;
const MUTATION_PATTERN =
  /\b(?:build|change|create|deploy|fix|harden|implement|migrate|patch|publish|release|remediate|tag|update|write)\b|выпусти|исправ|измен|мигрир|обнов|опубликуй|реализ|релизь|созда|тегир|выкат|запиши/iu;
const READ_ONLY_PATTERN =
  /\b(?:read[ -]?only|inspect|review|audit|analy[sz]e|evaluate|explain|report|research|look up)\b|без\s+изменений|только\s+чтение|проверь|аудит|разбери|оцени|посмотри|изучи|объясни|отч[её]т/iu;
const NEGATED_MUTATION_PATTERN =
  /\b(?:do not|don't|dont|never)\s+(?:edit|modify|change|patch|write)(?:\s+(?:files?|code))?\b|не\s+(?:редактир|изменя|патч|прав|трогай)|без\s+(?:правок|изменений)/iu;
const XS_TASK_PATTERN =
  /\b(?:fix|check|inspect|update|change|rename|format)\b.{0,24}\b(?:a\s+)?(?:typo|one\s+line|single\s+line|one\s+comment|single\s+comment|spelling|label)\b|(?:исправ|проверь|обнов|измени|переименуй|формат).{0,24}(?:опечат|одну\s+строк|один\s+коммент|подпис)/iu;
const PARALLEL_VALUE_PATTERN =
  /\b(?:parallel|independent lanes?|compare|cross-check|deep audit|comprehensive audit|across (?:modules|systems|sources|projects)|multiple (?:modules|sources|projects)|release announcement.{0,40}diagrams?)\b|параллел|независим\w*\s+(?:поток|провер)|сравни|глубок\w*\s+аудит|комплексн\w*\s+аудит|нескольк\w*\s+(?:модул|источник|проект)|сайт\w*\s+релиз.{0,40}диаграм/iu;
const SIMPLE_QUESTION_PATTERN =
  /^(?:what|when|where|who|why|how|is|are|can|does|do|что|когда|где|кто|почему|как|можно|есть|выпущен)(?:\s|$)/iu;

function getAgentProfile(name) {
  return AGENT_POLICY.profiles[name] || null;
}

function getAgentProfiles() {
  return Object.entries(AGENT_POLICY.profiles).map(([name, profile]) => ({
    name,
    ...profile,
  }));
}

function isLikelySmallTask(task) {
  const trimmed = task.trim();
  return XS_TASK_PATTERN.test(trimmed) ||
    (trimmed.length < 120 && SIMPLE_QUESTION_PATTERN.test(trimmed));
}

function hasParallelValue(task, candidates, modes) {
  if (PARALLEL_VALUE_PATTERN.test(task)) return true;
  return task.trim().length >= 220 && candidates.length >= 2 && modes.length >= 2;
}

function getFanoutDecision(options) {
  const task = options.task || "";
  const risk = options.risk || "LOW";
  const candidates = Array.from(new Set(options.candidates || []));
  const modes = Array.from(new Set(options.modes || []));
  const isExplicitReadOnly = NEGATED_MUTATION_PATTERN.test(task) ||
    (READ_ONLY_PATTERN.test(task) && !MUTATION_PATTERN.test(task));
  const isStateChanging = options.isStateChanging ??
    (MUTATION_PATTERN.test(task) && !isExplicitReadOnly);

  if (OPT_OUT_PATTERN.test(task)) {
    return createDecision("skip", "explicit-user-opt-out", []);
  }
  if (candidates.length === 0) {
    return createDecision("skip", "no-specialist-candidates", []);
  }
  if (isLikelySmallTask(task)) {
    return createDecision("skip", "xs-direct-task", []);
  }

  const ranked = rankCandidates(candidates, modes, options.priorityCandidates || []);
  const selected = ranked.slice(0, AGENT_POLICY.fanout.maxChildren);
  if ((risk === "HIGH" || risk === "CRITICAL") && isStateChanging) {
    return createDecision("required", "high-risk-independent-verification", selected, ranked);
  }
  if (selected.length >= 2 && hasParallelValue(task, selected, modes)) {
    return createDecision("recommended", "parallel-independent-lanes-available", selected, ranked);
  }
  return createDecision("conditional", "parallel-value-not-yet-proven", selected, ranked);
}

function rankCandidates(candidates, modes, priorityCandidates = []) {
  const priorityByMode = {
    security: ["security_reviewer", "tester"],
    openai: ["docs_researcher"],
    template: ["systems_reviewer", "tester"],
    migration: ["systems_reviewer", "tester"],
    bugfix: ["scout", "tester", "reviewer", "log_analyst"],
    testing: ["log_analyst", "tester", "reviewer"],
    docs: ["summarizer", "reviewer"],
    review: ["scout", "reviewer", "tester", "summarizer"],
    release: ["security_reviewer", "tester", "reviewer"],
    strategy: ["systems_reviewer", "product_reviewer"],
    product: ["product_reviewer"],
    "product-ux": ["product_reviewer", "design_reviewer"],
    design: ["design_reviewer"],
    "design-system": ["design_reviewer"],
  };
  const orderedModes = Object.keys(priorityByMode).filter((mode) => modes.includes(mode));
  const preferred = orderedModes.flatMap((mode) => priorityByMode[mode]);
  return Array.from(new Set([...priorityCandidates, ...preferred, ...candidates])).filter((name) =>
    candidates.includes(name),
  );
}

function createDecision(status, reason, candidates, inventory = candidates) {
  return {
    status,
    reason,
    candidates: candidates.map((name) => ({
      name,
      profile: getAgentProfile(name),
    })),
    candidateInventory: inventory.map((name) => name),
    maxChildren: AGENT_POLICY.fanout.maxChildren,
    maxDepth: AGENT_POLICY.fanout.maxDepth,
    maxAutomaticWaves: AGENT_POLICY.fanout.maxAutomaticWaves,
    notifyUser: AGENT_POLICY.fanout.notifyUser,
    readOnlyFirst: AGENT_POLICY.fanout.readOnlyFirst,
    requireRuntimeProfileEvidence: AGENT_POLICY.fanout.requireRuntimeProfileEvidence,
    writePolicy: AGENT_POLICY.fanout.writePolicy,
    independenceGate:
      "spawn only when the lane is independent, useful, non-duplicative, and faster or safer in parallel",
  };
}

function formatAgentProfiles(candidates) {
  return candidates.map(({ name, profile }) => {
    if (!profile) return `${name}:project-defined`;
    return `${name}:${profile.model}@${profile.effort}`;
  });
}

function normalizeWriteScope(scope) {
  const normalized = String(scope || "")
    .trim()
    .replaceAll("\\", "/")
    .replace(/^\.\//, "");
  return normalized
    ? path.posix.normalize(normalized).replace(/\/$/, "").toLocaleLowerCase("en-US")
    : "";
}

function doScopesOverlap(left, right) {
  if (!left || !right) return false;
  return left === right || left.startsWith(`${right}/`) || right.startsWith(`${left}/`);
}

function isUnsafeWriteScope(scope) {
  return scope === "." || scope === "/" || /^[a-z]:\/?$/iu.test(scope);
}

function validateWriteAssignments(assignments = []) {
  const normalized = assignments.map(({ agent, files = [] }) => ({
    agent,
    files: files.map(normalizeWriteScope).filter(Boolean),
  }));
  const missingScopes = normalized
    .filter(({ files }) => files.length === 0)
    .map(({ agent }) => agent);
  const invalidScopes = normalized.flatMap(({ agent, files }) =>
    files.filter(isUnsafeWriteScope).map((scope) => ({ agent, scope })),
  );
  const conflicts = [];
  for (let left = 0; left < normalized.length; left += 1) {
    for (let right = left + 1; right < normalized.length; right += 1) {
      collectWriteConflicts(normalized[left], normalized[right], conflicts);
    }
  }
  return {
    isValid:
      missingScopes.length === 0 && invalidScopes.length === 0 && conflicts.length === 0,
    missingScopes,
    invalidScopes,
    conflicts,
  };
}

function collectWriteConflicts(left, right, conflicts) {
  for (const leftScope of left.files) {
    for (const rightScope of right.files) {
      if (doScopesOverlap(leftScope, rightScope)) {
        conflicts.push({
          agents: [left.agent, right.agent],
          scopes: [leftScope, rightScope],
        });
      }
    }
  }
}

module.exports = {
  AGENT_POLICY,
  EFFORT_LEVELS,
  formatAgentProfiles,
  getAgentProfile,
  getAgentProfiles,
  getFanoutDecision,
  validateWriteAssignments,
};
