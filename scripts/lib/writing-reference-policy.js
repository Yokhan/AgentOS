"use strict";

const fs = require("fs");
const path = require("path");
const crypto = require("crypto");
const { validateExternalTool } = require("./writing-external-tool-policy.js");
const { resolveRepoFile } = require("./writing-path-policy.js");

const MODES = new Set(["literary", "marketing", "informational", "communication"]);
const OVERLAYS = new Set(["technical"]);
const DEFAULT_USES = new Set(["default", "conditional", "opt-in", "required-when-present"]);
const LOCATOR_TYPES = new Set(["url", "repo", "project-slot"]);
const CONTENT_POLICIES = new Set(["link-and-properties", "properties-only", "project-owned-only"]);
const SOURCE_KINDS = new Set(["project-slot", "derived-lens", "local-user-source", "author-guidance", "regulator-guidance", "public-domain-work", "official-guide", "standard"]);
const TRUST_LEVELS = new Set(["project-authority", "diagnostic", "primary", "secondary"]);
const PROVENANCE_STATUSES = new Set(["project-required", "unverified-derived", "user-declared", "verified", "verified-public-domain"]);
const FORBIDDEN_CONTENT_KEYS = new Set(["content", "excerpt", "quote", "quotes", "example", "examples", "sample", "samples", "properties"]);
const SOURCE_LANGUAGE_SPECIALS = new Set(["multilingual", "language-neutral"]);
const OUTPUT_LANGUAGE_SPECIALS = new Set(["mixed", "all"]);
const USAGE_CLASSES = new Set(["project-reference", "derived-lens", "language-method", "editorial-method", "domain-standard", "regulatory-standard", "historical-method", "tool-documentation"]);
const AUTHORITY_GROUPS = new Set(["language", "process", "domain", "technical"]);
const LANGUAGE_AUTHORITY_EFFECTS = new Set(["lexical-choice", "line-editing", "syntax", "voice"]);
const PROCESS_AUTHORITY_EFFECTS = new Set(["client-communication", "planning"]);
const EFFECTS = new Set([
  "accessibility", "character", "claims", "client-communication", "context-framing", "correspondence",
  "domain-correctness", "evidence", "example-method", "information-architecture",
  "explanation", "genre-structure", "line-editing", "localization", "measurement", "narrative",
  "normative-semantics", "offer-structure", "planning", "reader-action",
  "persuasion", "reader-purpose", "recipient-care", "reuse", "risk-communication", "security", "structure",
  "syntax", "technical-identifiers", "lexical-choice", "visual-structure", "voice", "working-system", "worldbuilding",
  "tool-capability-boundary",
]);
const LANGUAGE_SENSITIVE_EFFECTS = new Set(["example-method", "lexical-choice", "line-editing", "syntax", "voice"]);
const DOMAIN_CLASSES = new Set(["domain-standard", "regulatory-standard"]);

function isObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function requireText(value, label, errors) {
  if (typeof value !== "string" || value.trim() === "") errors.push(`${label} must be non-empty text`);
}

function requireTextList(value, label, errors, minimum = 1) {
  if (!Array.isArray(value) || value.length < minimum || value.some((item) => typeof item !== "string" || !item.trim())) {
    errors.push(`${label} must contain at least ${minimum} non-empty text item(s)`);
  }
}

function isLanguageTag(value) {
  return typeof value === "string" && /^[a-z]{2,3}(?:-[A-Za-z0-9]{2,8})*$/.test(value);
}

function validateOutputLanguages(values, label, errors) {
  if (!Array.isArray(values) || values.length === 0 || values.some((value) => !OUTPUT_LANGUAGE_SPECIALS.has(value) && !isLanguageTag(value))) {
    errors.push(`${label} contains an unsupported value`);
  }
}

function validateUniqueIds(items, label, errors) {
  const seen = new Set();
  for (const item of items) {
    requireText(item?.id, `${label}.id`, errors);
    if (seen.has(item?.id)) errors.push(`${label} has duplicate id: ${item.id}`);
    seen.add(item?.id);
  }
  return seen;
}

function validateDate(value, label, errors) {
  if (typeof value !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(value) || Number.isNaN(Date.parse(`${value}T00:00:00Z`))) {
    errors.push(`${label} must be a valid YYYY-MM-DD date`);
    return null;
  }
  return new Date(`${value}T00:00:00Z`);
}

function findForbiddenKeys(value, trail = "registry", found = []) {
  if (Array.isArray(value)) {
    value.forEach((item, index) => findForbiddenKeys(item, `${trail}[${index}]`, found));
    return found;
  }
  if (!isObject(value)) return found;
  for (const [key, child] of Object.entries(value)) {
    if (FORBIDDEN_CONTENT_KEYS.has(key)) found.push(`${trail}.${key}`);
    findForbiddenKeys(child, `${trail}.${key}`, found);
  }
  return found;
}

function validateEditor(editor, root, errors) {
  requireText(editor.purpose, `editor ${editor.id}.purpose`, errors);
  requireTextList(editor.codexRoles, `editor ${editor.id}.codexRoles`, errors);
  requireTextList(editor.skills, `editor ${editor.id}.skills`, errors);
  const needsEffectScope = editor.id?.startsWith("technical-") || editor.id === "russian-language";
  if (needsEffectScope || editor.allowedEffects !== undefined) {
    validateEnumList(editor.allowedEffects, EFFECTS, `editor ${editor.id}.allowedEffects`, errors);
  }
  if (editor.id?.startsWith("technical-") && editor.allowedEffects?.some((effect) => LANGUAGE_SENSITIVE_EFFECTS.has(effect))) {
    errors.push(`editor ${editor.id} technical editors cannot affect voice, syntax, line editing, or example method`);
  }
  for (const role of editor.codexRoles || []) {
    const file = `${String(role).replaceAll("_", "-")}.toml`;
    if (!fs.existsSync(path.join(root, ".codex", "agents", file))) errors.push(`editor ${editor.id} references missing Codex role: ${role}`);
  }
  for (const skill of editor.skills || []) {
    if (!fs.existsSync(path.join(root, ".agents", "skills", skill, "SKILL.md"))) errors.push(`editor ${editor.id} references missing skill: ${skill}`);
  }
}

function validateIntegrity(source, root, errors) {
  if (source.locator?.type !== "repo") return;
  if (source.integrity?.algorithm !== "sha256" || !/^[a-f0-9]{64}$/.test(source.integrity?.value || "")) {
    errors.push(`source ${source.id} requires a sha256 integrity value`);
    return;
  }
  const resolved = resolveRepoFile(root, source.locator.value);
  if (resolved.error) return;
  const actual = crypto.createHash("sha256").update(fs.readFileSync(resolved.file)).digest("hex");
  if (actual !== source.integrity.value) errors.push(`source ${source.id} integrity hash does not match`);
}

function validateLocator(source, root, errors) {
  const locator = source.locator;
  if (!isObject(locator) || !LOCATOR_TYPES.has(locator.type)) {
    errors.push(`source ${source.id}.locator has unsupported type`);
    return;
  }
  requireText(locator.value, `source ${source.id}.locator.value`, errors);
  if (locator.type === "url" && !/^https:\/\//i.test(locator.value || "")) {
    errors.push(`source ${source.id} URL must use HTTPS`);
  }
  if (locator.type === "repo") {
    const resolved = resolveRepoFile(root, locator.value);
    if (resolved.error) errors.push(`source ${source.id} repo locator ${resolved.error}`);
  }
}

function validateProvenance(source, errors) {
  const provenance = source.provenance;
  if (!isObject(provenance)) {
    errors.push(`source ${source.id}.provenance is required`);
    return;
  }
  requireText(provenance.status, `source ${source.id}.provenance.status`, errors);
  if (!PROVENANCE_STATUSES.has(provenance.status)) errors.push(`source ${source.id} has unsupported provenance status`);
  validateDate(provenance.checkedOn, `source ${source.id}.provenance.checkedOn`, errors);
  requireText(provenance.rights, `source ${source.id}.provenance.rights`, errors);
  if (!CONTENT_POLICIES.has(provenance.contentPolicy)) {
    errors.push(`source ${source.id} has unsupported content policy`);
  }
}

function validateFreshness(source, today, errors) {
  const freshness = source.freshness;
  if (!isObject(freshness) || typeof freshness.required !== "boolean") {
    errors.push(`source ${source.id}.freshness requires a boolean required field`);
    return;
  }
  if (!freshness.required) return;
  if (!Number.isInteger(freshness.maxAgeDays) || freshness.maxAgeDays <= 0) {
    errors.push(`source ${source.id}.freshness.maxAgeDays must be a positive integer`);
    return;
  }
  const checked = validateDate(source.provenance?.checkedOn, `source ${source.id}.provenance.checkedOn`, []);
  if (checked && (today - checked) / 86400000 > freshness.maxAgeDays) {
    errors.push(`source ${source.id} is stale; refresh provenance before use`);
  }
}

function validateSource(source, context, errors) {
  requireText(source.title, `source ${source.id}.title`, errors);
  requireText(source.creator, `source ${source.id}.creator`, errors);
  requireText(source.kind, `source ${source.id}.kind`, errors);
  requireText(source.trust, `source ${source.id}.trust`, errors);
  if (!SOURCE_KINDS.has(source.kind)) errors.push(`source ${source.id} has unsupported kind`);
  if (!TRUST_LEVELS.has(source.trust)) errors.push(`source ${source.id} has unsupported trust level`);
  if (!SOURCE_LANGUAGE_SPECIALS.has(source.language) && !isLanguageTag(source.language)) errors.push(`source ${source.id} has unsupported language`);
  if (!USAGE_CLASSES.has(source.usageClass)) errors.push(`source ${source.id} has unsupported usageClass`);
  validateEnumList(source.allowedEffects, EFFECTS, `source ${source.id}.allowedEffects`, errors);
  if (DOMAIN_CLASSES.has(source.usageClass) && source.allowedEffects?.some((effect) => LANGUAGE_SENSITIVE_EFFECTS.has(effect))) {
    errors.push(`source ${source.id} domain standards cannot affect voice, syntax, line editing, or example method`);
  }
  validateLocator(source, context.root, errors);
  validateIntegrity(source, context.root, errors);
  validateProvenance(source, errors);
  validateFreshness(source, context.today, errors);
}

function validateEnumList(values, allowed, label, errors, minimum = 1) {
  if (!Array.isArray(values) || values.length < minimum || values.some((value) => !allowed.has(value))) {
    errors.push(`${label} contains an unsupported value`);
  }
}

function validatePropertiesByEffect(profile, errors) {
  if (!isObject(profile.propertiesByEffect) || Object.keys(profile.propertiesByEffect).length === 0) {
    errors.push(`profile ${profile.id}.propertiesByEffect must map declared effects to properties`);
    return;
  }
  for (const [effect, properties] of Object.entries(profile.propertiesByEffect)) {
    if (!profile.effects?.includes(effect)) errors.push(`profile ${profile.id} properties use undeclared effect: ${effect}`);
    requireTextList(properties, `profile ${profile.id}.propertiesByEffect.${effect}`, errors);
  }
  for (const effect of profile.effects || []) {
    if (!Object.hasOwn(profile.propertiesByEffect, effect)) {
      errors.push(`profile ${profile.id}.propertiesByEffect is missing declared effect: ${effect}`);
    }
  }
}

function validateAuthorityGroup(profile, errors) {
  if (!AUTHORITY_GROUPS.has(profile.authorityGroup)) {
    errors.push(`profile ${profile.id} has unsupported authorityGroup`);
    return;
  }
  if (profile.authorityGroup !== "language" && profile.effects?.some((effect) => LANGUAGE_AUTHORITY_EFFECTS.has(effect))) {
    errors.push(`profile ${profile.id} leaks language authority outside the language group`);
  }
  if (profile.authorityGroup !== "process" && profile.effects?.some((effect) => PROCESS_AUTHORITY_EFFECTS.has(effect))) {
    errors.push(`profile ${profile.id} leaks process authority outside the process group`);
  }
}

function validateLanguageSourceMatch(profile, sources, errors) {
  if (profile.outputLanguages?.includes("mixed")) {
    errors.push(`profile ${profile.id} needs per-section language resolution for language-sensitive effects`);
    return;
  }
  for (const effect of profile.effects || []) {
    if (!LANGUAGE_SENSITIVE_EFFECTS.has(effect)) continue;
    const effectSources = sources.filter((source) => source.allowedEffects?.includes(effect));
    for (const language of profile.outputLanguages || []) {
      const isCovered = effectSources.some((source) => source.language === language || source.language === "multilingual");
      if (!isCovered) errors.push(`profile ${profile.id} has no ${language} source allowed to affect ${effect}`);
    }
  }
}

function validateProfile(profile, sourceMap, editorIds, errors) {
  requireTextList(profile.sourceIds, `profile ${profile.id}.sourceIds`, errors);
  validateEnumList(profile.modes, MODES, `profile ${profile.id}.modes`, errors);
  validateEnumList(profile.overlays, OVERLAYS, `profile ${profile.id}.overlays`, errors, 0);
  validateOutputLanguages(profile.outputLanguages, `profile ${profile.id}.outputLanguages`, errors);
  validateEnumList(profile.effects, EFFECTS, `profile ${profile.id}.effects`, errors);
  validateAuthorityGroup(profile, errors);
  if (!DEFAULT_USES.has(profile.defaultUse)) errors.push(`profile ${profile.id} has unsupported defaultUse`);
  validatePropertiesByEffect(profile, errors);
  requireTextList(profile.editors, `profile ${profile.id}.editors`, errors);
  requireTextList(profile.constraints, `profile ${profile.id}.constraints`, errors);
  for (const id of profile.sourceIds || []) if (!sourceMap.has(id)) errors.push(`profile ${profile.id} references missing source: ${id}`);
  for (const id of profile.editors || []) if (!editorIds.has(id)) errors.push(`profile ${profile.id} references missing editor: ${id}`);
  const sources = (profile.sourceIds || []).map((id) => sourceMap.get(id)).filter(Boolean);
  const allowedEffects = new Set(sources.flatMap((source) => source.allowedEffects || []));
  for (const effect of profile.effects || []) {
    if (!allowedEffects.has(effect)) errors.push(`profile ${profile.id} effect is not allowed by its sources: ${effect}`);
  }
  const hasLanguageSensitiveEffect = (profile.effects || []).some((effect) => LANGUAGE_SENSITIVE_EFFECTS.has(effect));
  if (hasLanguageSensitiveEffect && profile.outputLanguages?.includes("all")) {
    errors.push(`profile ${profile.id} cannot apply language-sensitive effects to all output languages`);
  }
  if (hasLanguageSensitiveEffect) validateLanguageSourceMatch(profile, sources, errors);
  const isUnverified = (profile.sourceIds || []).some((id) => sourceMap.get(id)?.provenance?.status === "unverified-derived");
  if (isUnverified && profile.defaultUse !== "opt-in") errors.push(`profile ${profile.id} uses unverified sources and must be opt-in`);
}

function validateWritingReferenceRegistry(registry, options = {}) {
  const errors = [];
  const root = options.root || process.cwd();
  const today = validateDate(options.today || new Date().toISOString().slice(0, 10), "today", errors) || new Date(0);
  if (!isObject(registry) || registry.schemaVersion !== 2) errors.push("registry.schemaVersion must equal 2");
  validateDate(registry?.reviewedOn, "registry.reviewedOn", errors);
  const editors = Array.isArray(registry?.editorRoles) ? registry.editorRoles : [];
  const externalTools = Array.isArray(registry?.externalTools) ? registry.externalTools : [];
  const sources = Array.isArray(registry?.sources) ? registry.sources : [];
  const profiles = Array.isArray(registry?.profiles) ? registry.profiles : [];
  if (!editors.length || !sources.length || !profiles.length) errors.push("registry requires editors, sources, and profiles");
  const editorIds = validateUniqueIds(editors, "editorRoles", errors);
  validateUniqueIds(externalTools, "externalTools", errors);
  const sourceIds = validateUniqueIds(sources, "sources", errors);
  validateUniqueIds(profiles, "profiles", errors);
  editors.forEach((editor) => validateEditor(editor, root, errors));
  externalTools.forEach((tool) => validateExternalTool(tool, root, errors));
  sources.forEach((source) => validateSource(source, { root, today }, errors));
  const sourceMap = new Map(sources.map((source) => [source.id, source]));
  profiles.forEach((profile) => validateProfile(profile, sourceMap, editorIds, errors));
  for (const location of findForbiddenKeys(registry)) errors.push(`${location} embeds reference content; store provenance and properties only`);
  if (sourceIds.size !== sources.length) errors.push("source IDs must be unique");
  return errors;
}

function withoutSuperseded(items, ids) {
  const removed = new Set(Array.isArray(ids) ? ids : []);
  return (Array.isArray(items) ? items : []).filter((item) => !removed.has(item.id));
}

function validateSupersedes(projectRegistry, templateRegistry, errors) {
  const editorIds = new Set(templateRegistry.editorRoles.map((item) => item.id));
  const sourceIds = new Set(templateRegistry.sources.map((item) => item.id));
  const profileIds = new Set(templateRegistry.profiles.map((item) => item.id));
  const toolIds = new Set((templateRegistry.externalTools || []).map((item) => item.id));
  const editorOverrides = projectRegistry.supersedes?.editorIds;
  const sourceOverrides = projectRegistry.supersedes?.sourceIds;
  const profileOverrides = projectRegistry.supersedes?.profileIds;
  const toolOverrides = projectRegistry.supersedes?.toolIds;
  const projectToolIds = new Set((projectRegistry.externalTools || []).map((item) => item.id));
  for (const id of Array.isArray(editorOverrides) ? editorOverrides : []) if (!editorIds.has(id)) errors.push(`project registry supersedes missing template editor: ${id}`);
  for (const id of Array.isArray(sourceOverrides) ? sourceOverrides : []) if (!sourceIds.has(id)) errors.push(`project registry supersedes missing template source: ${id}`);
  for (const id of Array.isArray(profileOverrides) ? profileOverrides : []) if (!profileIds.has(id)) errors.push(`project registry supersedes missing template profile: ${id}`);
  for (const id of Array.isArray(toolOverrides) ? toolOverrides : []) {
    if (!toolIds.has(id)) errors.push(`project registry supersedes missing template tool: ${id}`);
    if (!projectToolIds.has(id)) errors.push(`project registry must replace superseded template tool with the same id: ${id}`);
  }
}

function mergeWritingReferenceRegistries(projectRegistry, templateRegistry) {
  if (!projectRegistry) return templateRegistry;
  const projectEditors = Array.isArray(projectRegistry.editorRoles) ? projectRegistry.editorRoles : [];
  const projectTools = Array.isArray(projectRegistry.externalTools) ? projectRegistry.externalTools : [];
  const projectSources = Array.isArray(projectRegistry.sources) ? projectRegistry.sources : [];
  const projectProfiles = Array.isArray(projectRegistry.profiles) ? projectRegistry.profiles : [];
  return {
    ...templateRegistry,
    editorRoles: [
      ...withoutSuperseded(templateRegistry.editorRoles, projectRegistry.supersedes?.editorIds),
      ...projectEditors,
    ],
    externalTools: [
      ...withoutSuperseded(templateRegistry.externalTools, projectRegistry.supersedes?.toolIds),
      ...projectTools,
    ],
    sources: [
      ...withoutSuperseded(templateRegistry.sources, projectRegistry.supersedes?.sourceIds),
      ...projectSources,
    ],
    profiles: [
      ...withoutSuperseded(templateRegistry.profiles, projectRegistry.supersedes?.profileIds),
      ...projectProfiles,
    ],
  };
}

function validateProjectWritingRegistry(projectRegistry, templateRegistry, options = {}) {
  const errors = [];
  if (!isObject(projectRegistry) || projectRegistry.schemaVersion !== 2 || projectRegistry.owner !== "project") errors.push("project registry requires schemaVersion 2 and owner project");
  validateDate(projectRegistry?.updatedOn, "project registry.updatedOn", errors);
  validateSupersedes(projectRegistry, templateRegistry, errors);
  const combined = mergeWritingReferenceRegistries(projectRegistry, templateRegistry);
  return [...errors, ...validateWritingReferenceRegistry(combined, options)];
}

module.exports = { mergeWritingReferenceRegistries, validateProjectWritingRegistry, validateWritingReferenceRegistry };
