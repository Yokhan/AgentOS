"use strict";

const { resolveRepoFile } = require("./writing-path-policy.js");

const ACCESS_STATES = new Set(["not-configured", "project-configured"]);
const KINDS = new Set(["paid-api"]);
const MODES = new Set(["literary", "marketing", "informational", "communication"]);

function isObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function requireText(value, label, errors) {
  if (typeof value !== "string" || value.trim() === "") errors.push(`${label} must be non-empty text`);
}

function requireTextList(value, label, errors) {
  if (!Array.isArray(value) || !value.length || value.some((item) => typeof item !== "string" || !item.trim())) {
    errors.push(`${label} must contain at least 1 non-empty text item(s)`);
  }
}

function validateDate(value, label, errors) {
  if (typeof value !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(value) || Number.isNaN(Date.parse(`${value}T00:00:00Z`))) {
    errors.push(`${label} must be a valid YYYY-MM-DD date`);
  }
}

function validateScope(tool, errors) {
  if (!Array.isArray(tool.modes) || !tool.modes.length || tool.modes.some((mode) => !MODES.has(mode))) {
    errors.push(`external tool ${tool.id}.modes contains an unsupported value`);
  }
  if (!Array.isArray(tool.outputLanguages) || !tool.outputLanguages.length) {
    errors.push(`external tool ${tool.id}.outputLanguages contains an unsupported value`);
  }
}

function validateConfiguration(tool, root, errors) {
  if (tool.access !== "project-configured") return;
  const evidence = tool.configurationEvidence;
  if (!isObject(evidence)) {
    errors.push(`external tool ${tool.id}.configurationEvidence must be structured`);
    return;
  }
  requireText(evidence.owner, `external tool ${tool.id}.configurationEvidence.owner`, errors);
  requireText(evidence.secretReference, `external tool ${tool.id}.configurationEvidence.secretReference`, errors);
  if (typeof evidence.secretReference === "string") {
    const envMatch = evidence.secretReference.match(/^env:([A-Z][A-Z0-9_]*)$/);
    if (!envMatch) {
      errors.push(`external tool ${tool.id}.configurationEvidence.secretReference must be a verifiable env reference, not contain a secret`);
    } else if (!process.env[envMatch[1]]) {
      errors.push(`external tool ${tool.id}.configurationEvidence.secretReference env variable is not available`);
    }
  }
  validateDate(evidence.checkedOn, `external tool ${tool.id}.configurationEvidence.checkedOn`, errors);
  if (!isObject(evidence.adapter) || evidence.adapter.type !== "repo") {
    errors.push(`external tool ${tool.id}.configurationEvidence.adapter must be a repo locator`);
    return;
  }
  requireText(evidence.adapter.value, `external tool ${tool.id}.configurationEvidence.adapter.value`, errors);
  const resolved = resolveRepoFile(root, evidence.adapter.value, { forbidPrefixes: ["tests/fixtures/"] });
  if (resolved.error) errors.push(`external tool ${tool.id} configured adapter ${resolved.error}`);
}

function validateExternalTool(tool, root, errors) {
  requireText(tool.title, `external tool ${tool.id}.title`, errors);
  requireText(tool.provider, `external tool ${tool.id}.provider`, errors);
  if (!KINDS.has(tool.kind)) errors.push(`external tool ${tool.id} has unsupported kind`);
  if (!ACCESS_STATES.has(tool.access)) errors.push(`external tool ${tool.id} has unsupported access`);
  if (typeof tool.paid !== "boolean") errors.push(`external tool ${tool.id}.paid must be boolean`);
  validateScope(tool, errors);
  if (!isObject(tool.locator) || tool.locator.type !== "url" || !/^https:\/\//i.test(tool.locator.value || "")) {
    errors.push(`external tool ${tool.id} requires an HTTPS URL locator`);
  }
  requireTextList(tool.capabilities, `external tool ${tool.id}.capabilities`, errors);
  requireTextList(tool.evidenceRequired, `external tool ${tool.id}.evidenceRequired`, errors);
  requireTextList(tool.forbiddenClaims, `external tool ${tool.id}.forbiddenClaims`, errors);
  const claims = tool.claimEvidence;
  if (!isObject(claims) || claims.requiresArtifactBinding !== true || claims.requiresProviderResponse !== true || claims.requiresSuccessfulStatus !== true) {
    errors.push(`external tool ${tool.id}.claimEvidence must require artifact binding, provider response, and successful status`);
  }
  validateConfiguration(tool, root, errors);
}

module.exports = { validateExternalTool };
