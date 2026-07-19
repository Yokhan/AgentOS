"use strict";

const fs = require("fs");
const path = require("path");
const templateRegistry = require("../../.claude/library/technical/writing-reference-registry.json");
const projectRegistryPath = path.join(__dirname, "..", "..", "brain", "03-knowledge", "writing", "reference-registry.json");
const { mergeWritingReferenceRegistries, validateProjectWritingRegistry, validateWritingReferenceRegistry } = require("./writing-reference-policy.js");
const REPO_ROOT = path.join(__dirname, "..", "..");

function loadProjectRegistry(filePath) {
  return fs.existsSync(filePath) ? JSON.parse(fs.readFileSync(filePath, "utf8")) : null;
}

const projectRegistry = loadProjectRegistry(projectRegistryPath);

const MODE_NAMES = {
  literary: "writing-literary",
  marketing: "marketing",
  informational: "writing-informational",
  communication: "writing-communication",
};

const ENGLISH_LANGUAGE_EDITORS = {
  literary: ["literary-prose"],
  marketing: ["marketing-language"],
  informational: ["informational-language"],
  communication: ["communication-language"],
};

const MODE_DOMAIN_EDITORS = {
  literary: ["literary-structure", "literary-continuity"],
  marketing: ["marketing-claims", "marketing-business", "marketing-ethics"],
  informational: ["informational-subject", "informational-task"],
  communication: ["communication-recipient", "communication-risk"],
};

const TECHNICAL_EDITORS = ["technical-accuracy", "technical-procedure", "technical-architecture", "technical-language"];
const WRITING_FILES = [
  "technical/writing.md",
  "technical/writing-mode-profiles.md",
  "technical/writing-reference-registry.json",
  "technical/writing-editorial-board.md",
];
function unique(values) {
  return Array.from(new Set(values.filter(Boolean)));
}

function canApplyLanguageProfiles(intent) {
  return intent.languageResolution === "explicit" || ["create", "plan"].includes(intent.action);
}

function hasRealProjectSource(profile, sourceMap) {
  if (!Array.isArray(profile.sourceIds)) return false;
  const sources = profile.sourceIds.map((id) => sourceMap.get(id)).filter(Boolean);
  return !sources.some((source) => source.provenance?.status === "project-required");
}

function resolveProfiles(profileIds, outputLanguage, registry, mode = null) {
  const profiles = Array.isArray(registry?.profiles) ? registry.profiles : [];
  const sources = Array.isArray(registry?.sources) ? registry.sources : [];
  const profileMap = new Map(profiles.map((profile) => [profile.id, profile]));
  const sourceMap = new Map(sources.map((source) => [source.id, source]));
  return profileIds.reduce((result, id) => {
    const profile = profileMap.get(id);
    if (!profile) return { ...result, rejected: [...result.rejected, { id, reason: "missing-profile" }] };
    if (!Array.isArray(profile.outputLanguages) || !Array.isArray(profile.sourceIds) || !Array.isArray(profile.modes) || !Array.isArray(profile.effects)) {
      return { ...result, rejected: [...result.rejected, { id, reason: "invalid-profile-shape" }] };
    }
    if (mode && !profile.modes.includes(mode)) {
      return { ...result, rejected: [...result.rejected, { id, reason: "mode-mismatch" }] };
    }
    if (["conditional", "opt-in"].includes(profile.defaultUse)) {
      return { ...result, rejected: [...result.rejected, { id, reason: "explicit-activation-required" }] };
    }
    if (!profile.outputLanguages.includes("all") && !profile.outputLanguages.includes(outputLanguage)) {
      return { ...result, rejected: [...result.rejected, { id, reason: "target-language-mismatch" }] };
    }
    if (profile.defaultUse === "required-when-present" && !hasRealProjectSource(profile, sourceMap)) {
      return { ...result, rejected: [...result.rejected, { id, reason: "project-source-absent" }] };
    }
    return { ...result, selected: [...result.selected, id] };
  }, { selected: [], rejected: [] });
}

function getProfileGroup(profile) {
  return profile.authorityGroup;
}

function getRegistryProfileIds(registry, activeProjectRegistry, group, mode) {
  const projectProfiles = Array.isArray(activeProjectRegistry?.profiles) ? activeProjectRegistry.profiles : [];
  const profiles = Array.isArray(registry?.profiles) ? registry.profiles : [];
  const projectIds = new Set(projectProfiles.map((profile) => profile.id));
  return profiles
    .filter((profile) => ["default", "required-when-present"].includes(profile.defaultUse))
    .filter((profile) => profile.modes?.includes(mode) && getProfileGroup(profile) === group)
    .sort((left, right) => Number(projectIds.has(right.id)) - Number(projectIds.has(left.id)))
    .map((profile) => profile.id);
}

function getSkills(intent) {
  const isReview = intent.action === "review";
  const skills = [isReview ? "codex-domain-communication-review" : "codex-writing-workflow"];
  if (intent.primaryMode === "marketing") skills.push("codex-domain-communication-review", "codex-domain-business-review", "codex-product-goal", "codex-strategic-review");
  if (intent.specializations.includes("technical")) skills.push(isReview ? "codex-technical-writing-review" : "codex-technical-writing");
  if (intent.domains.includes("api")) skills.push("codex-api-contract");
  if (intent.vendors.includes("openai")) skills.push("codex-openai-model-guidance");
  return unique(skills);
}

function getSubagents(intent) {
  const subagents = ["reviewer"];
  if (intent.specializations.includes("technical")) subagents.push("docs_researcher", "tester");
  if (intent.vendors.includes("openai")) subagents.push("docs_researcher");
  return unique(subagents);
}

function getFiles(intent) {
  const files = [...WRITING_FILES];
  if (intent.outputLanguage === "ru" && canApplyLanguageProfiles(intent)) {
    files.push("technical/russian-writing-profile.md");
    if (["marketing", "informational", "communication"].includes(intent.primaryMode)) {
      files.push("technical/russian-explanation-and-persuasion.md");
    }
    if (intent.primaryMode === "communication") files.push("technical/russian-business-correspondence.md");
  }
  if (intent.specializations.includes("technical")) files.push("technical/technical-writing-profile.md", "technical/testing.md");
  if (intent.primaryMode === "marketing") files.push("product/production-product-standard.md", "process/product-goal-loop.md");
  return unique(files);
}

function getProfileEditors(profileIds, registry) {
  const profileMap = new Map(registry.profiles.map((profile) => [profile.id, profile]));
  return unique(profileIds.flatMap((id) => profileMap.get(id)?.editors || []));
}

function getModeLanguageEditors(intent) {
  if (intent.primaryMode === "literary") return ["literary-prose"];
  if (intent.outputLanguage === "en") return ENGLISH_LANGUAGE_EDITORS[intent.primaryMode] || [];
  return [];
}

function getWritingSelection(intent, registry, activeProjectRegistry, isTechnical) {
  const mode = intent.primaryMode;
  const languageIds = getRegistryProfileIds(registry, activeProjectRegistry, "language", mode);
  const processIds = getRegistryProfileIds(registry, activeProjectRegistry, "process", mode);
  const domainIds = getRegistryProfileIds(registry, activeProjectRegistry, "domain", mode);
  const technicalIds = isTechnical ? getRegistryProfileIds(registry, activeProjectRegistry, "technical", mode) : [];
  const language = canApplyLanguageProfiles(intent)
    ? resolveProfiles(languageIds, intent.outputLanguage, registry, mode)
    : { selected: [], rejected: languageIds.map((id) => ({ id, reason: "target-language-unconfirmed" })) };
  const domain = resolveProfiles(domainIds, intent.outputLanguage, registry, mode);
  const process = resolveProfiles(processIds, intent.outputLanguage, registry, mode);
  const technical = resolveProfiles(technicalIds, intent.outputLanguage, registry, mode);
  return { language, process, domain, technical };
}

function getExternalTools(intent, registry) {
  const requestedTools = new Set(intent.externalTools || []);
  return (registry.externalTools || [])
    .filter((tool) => requestedTools.has(tool.id) || (
      canApplyLanguageProfiles(intent)
      && tool.modes?.includes(intent.primaryMode)
      && (tool.outputLanguages?.includes("all") || tool.outputLanguages?.includes(intent.outputLanguage))
    ))
    .map(({ id, access, paid }) => ({ id, access, execution: "not-run", paid }));
}

function validateRuntimeRegistry(activeProjectRegistry, activeTemplateRegistry, root) {
  const errors = activeProjectRegistry
    ? validateProjectWritingRegistry(activeProjectRegistry, activeTemplateRegistry, { root })
    : validateWritingReferenceRegistry(activeTemplateRegistry, { root });
  if (errors.length) throw new Error(`Writing registry validation failed:\n${errors.join("\n")}`);
}

function getWritingRoutePolicy(intent, options = {}) {
  if (!intent.isWriting || !intent.primaryMode) return null;
  const activeProjectRegistry = Object.prototype.hasOwnProperty.call(options, "projectRegistry")
    ? options.projectRegistry
    : projectRegistry;
  const activeTemplateRegistry = options.templateRegistry || templateRegistry;
  validateRuntimeRegistry(activeProjectRegistry, activeTemplateRegistry, options.root || REPO_ROOT);
  const registry = mergeWritingReferenceRegistries(activeProjectRegistry, activeTemplateRegistry);
  const isTechnical = intent.specializations.includes("technical");
  const { language, process, domain, technical } = getWritingSelection(intent, registry, activeProjectRegistry, isTechnical);
  const languageEditors = unique([...getModeLanguageEditors(intent), ...getProfileEditors(language.selected, registry)]);
  const domainEditors = unique([...(MODE_DOMAIN_EDITORS[intent.primaryMode] || []), ...getProfileEditors(domain.selected, registry)]);
  const processEditors = getProfileEditors(process.selected, registry);
  const technicalEditors = isTechnical ? unique([...TECHNICAL_EDITORS, ...getProfileEditors(technical.selected, registry)]) : [];
  const externalTools = getExternalTools(intent, registry);
  const hasUnavailableTools = externalTools.some(({ access }) => access !== "project-configured");
  return {
    mode: MODE_NAMES[intent.primaryMode],
    extraModes: unique([isTechnical ? "technical-writing" : "", intent.domains.includes("api") ? "api" : "", intent.vendors.includes("openai") ? "openai" : ""]),
    pipeline: `${intent.primaryMode} writing${intent.action === "review" ? " review" : ""}`,
    risk: intent.primaryMode === "marketing" || isTechnical ? "MEDIUM" : "LOW",
    agent: intent.action === "review" ? "reviewer" : isTechnical ? "technical-writer" : "writer",
    skills: getSkills(intent),
    subagents: getSubagents(intent),
    rules: unique(["writing", intent.action === "review" ? "review" : "", isTechnical ? "testing" : ""]),
    files: getFiles(intent),
    targetLanguage: intent.outputLanguage,
    languageResolution: intent.languageResolution,
    languageProfiles: language.selected,
    processProfiles: process.selected,
    domainProfiles: domain.selected,
    technicalProfiles: technical.selected,
    profiles: unique([...language.selected, ...process.selected, ...domain.selected, ...technical.selected]),
    rejectedProfiles: [...language.rejected, ...process.rejected, ...domain.rejected, ...technical.rejected],
    languageEditors,
    domainEditors,
    processEditors,
    technicalEditors,
    editors: unique([...languageEditors, ...processEditors, ...domainEditors, ...technicalEditors]),
    externalTools,
    gates: unique(["writing-contract", intent.languageResolution === "explicit" ? "target-language-explicit" : "target-language-confirm-before-language-edit", "source-truth-boundary", "functional-whole", "reference-registry-valid", "reference-effects-isolated", externalTools.length ? "external-tool-evidence-required" : "", externalTools.length ? "external-tool-not-run" : "", hasUnavailableTools ? "external-tool-unavailable" : "", languageEditors.length ? "editorial-board-covered" : "language-editor-missing", intent.outputLanguage === "mixed" ? "per-section-language-resolution" : "", isTechnical ? "technical-procedure-executed" : ""]),
    needsFreshDocs: intent.vendors.includes("openai"),
  };
}

module.exports = { getWritingRoutePolicy, loadProjectRegistry, resolveProfiles };

function main(argv) {
  const { classifyWritingIntent } = require("./writing-intent.js");
  const isTsv = argv[0] === "--tsv";
  const task = (isTsv ? argv.slice(1) : argv).join(" ");
  const policy = getWritingRoutePolicy(classifyWritingIntent(task));
  if (!isTsv) {
    process.stdout.write(JSON.stringify(policy));
    return;
  }
  if (!policy) {
    process.stdout.write(["0", ...Array(20).fill("")].join("\t"));
    return;
  }
  process.stdout.write([
    "1", policy.mode, policy.extraModes.join(","), policy.agent,
    policy.skills.join(","), policy.subagents.join(","), policy.files.join(","),
    policy.pipeline, policy.risk, policy.profiles.join(","),
    policy.editors.join(","), policy.gates.join(","), policy.needsFreshDocs ? "1" : "0",
    policy.targetLanguage, policy.languageProfiles.join(","),
    policy.domainProfiles.join(","), policy.technicalProfiles.join(","),
    policy.rejectedProfiles.map(({ id, reason }) => `${id}:${reason}`).join(","),
    policy.languageResolution,
    policy.processProfiles.join(","),
    policy.externalTools.map(({ id, access, execution, paid }) => `${id}:${access}:${execution}:${paid ? "paid" : "free"}`).join(","),
  ].join("\t"));
}

if (require.main === module) main(process.argv.slice(2));
