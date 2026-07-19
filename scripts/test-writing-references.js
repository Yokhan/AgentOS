#!/usr/bin/env node
"use strict";

const assert = require("assert");
const fs = require("fs");
const os = require("os");
const path = require("path");
const registry = require("../.claude/library/technical/writing-reference-registry.json");
const { validateExternalTool } = require("./lib/writing-external-tool-policy.js");
const { validateProjectWritingRegistry, validateWritingReferenceRegistry } = require("./lib/writing-reference-policy.js");
const projectRegistryPath = path.join(__dirname, "..", "brain", "03-knowledge", "writing", "reference-registry.json");
const projectRegistry = fs.existsSync(projectRegistryPath)
  ? JSON.parse(fs.readFileSync(projectRegistryPath, "utf8"))
  : { schemaVersion: 2, updatedOn: "2026-07-17", owner: "project", supersedes: { editorIds: [], toolIds: [], sourceIds: [], profileIds: [] }, editorRoles: [], externalTools: [], sources: [], profiles: [] };

const OPTIONS = { root: process.cwd(), today: "2026-07-17" };

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function expectError(change, pattern) {
  const candidate = clone(registry);
  change(candidate);
  const errors = validateWritingReferenceRegistry(candidate, OPTIONS);
  assert(errors.some((error) => pattern.test(error)), errors.join("\n"));
}

function main() {
  assert.deepStrictEqual(validateWritingReferenceRegistry(registry, OPTIONS), []);
  assert.deepStrictEqual(validateProjectWritingRegistry(projectRegistry, registry, OPTIONS), []);
  assert(!registry.profiles.some((profile) => profile.sourceIds.includes("glavred-infostyle-method")), "Glavred documentation must not be a default writing profile source");
  expectError((copy) => copy.sources[1].id = copy.sources[0].id, /duplicate id/);
  expectError((copy) => copy.sources[0].provenance.checkedOn = "17-07-2026", /valid YYYY-MM-DD/);
  expectError((copy) => copy.sources[1].locator.value = "missing/reference.md", /does not exist/);
  expectError((copy) => copy.sources[1].locator.value = "../outside-repository.md", /inside the repository/);
  expectError((copy) => copy.sources[1].locator.value = ".", /inside the repository|is not a file/);
  expectError((copy) => copy.sources[1].integrity.value = "0".repeat(64), /integrity hash does not match/);
  expectError((copy) => copy.sources[3].examples = ["copied prose"], /embeds reference content/);
  expectError((copy) => copy.profiles[0].properties = ["unscoped behavior"], /embeds reference content/);
  expectError((copy) => copy.profiles.find((profile) => profile.id === "litai-bunin-lens").defaultUse = "default", /unverified sources and must be opt-in/);
  expectError((copy) => copy.profiles[0].editors = ["missing-editor"], /missing editor/);
  expectError((copy) => copy.sources.find((source) => source.id === "google-developer-style").provenance.checkedOn = "2025-01-01", /is stale/);
  expectError((copy) => copy.sources[0].trust = "popular", /unsupported trust level/);
  expectError((copy) => copy.profiles[0].effects = [], /contains an unsupported value/);
  expectError((copy) => copy.profiles[0].outputLanguages = [], /contains an unsupported value/);
  expectError((copy) => copy.profiles[0].propertiesByEffect = {}, /must map declared effects/);
  expectError((copy) => delete copy.profiles[0].propertiesByEffect.voice, /missing declared effect: voice/);
  expectError((copy) => {
    const profile = copy.profiles.find((item) => item.id === "technical-developer-conventions");
    profile.propertiesByEffect.syntax = ["natural prose"];
  }, /properties use undeclared effect: syntax/);
  expectError((copy) => copy.sources[0].language = "unknown", /unsupported language/);
  expectError((copy) => copy.sources.find((source) => source.id === "google-developer-style").allowedEffects.push("syntax"), /domain standards cannot affect/);
  expectError((copy) => copy.profiles.find((profile) => profile.id === "technical-developer-conventions").effects.push("syntax"), /effect is not allowed by its sources/);
  expectError((copy) => {
    const profile = copy.profiles.find((item) => item.id === "russian-infostyle-core");
    profile.outputLanguages = ["all"];
  }, /cannot apply language-sensitive effects to all output languages/);
  expectError((copy) => {
    const profile = copy.profiles.find((item) => item.id === "russian-infostyle-core");
    profile.outputLanguages = ["en"];
  }, /has no en source allowed to affect/);
  expectError((copy) => copy.editorRoles[0].skills = ["missing-skill"], /missing skill/);
  expectError((copy) => copy.editorRoles.find((editor) => editor.id === "technical-language").allowedEffects.push("syntax"), /technical editors cannot affect/);
  expectError((copy) => {
    const profile = copy.profiles.find((item) => item.id === "russian-business-correspondence");
    profile.effects.push("voice");
    profile.propertiesByEffect.voice = ["leaked voice"];
  }, /leaks language authority outside the language group/);
  expectError((copy) => copy.externalTools[0].access = "configured", /unsupported access/);
  expectError((copy) => copy.externalTools[0].evidenceRequired = [], /must contain at least/);
  expectError((copy) => copy.externalTools[0].claimEvidence.requiresProviderResponse = false, /claimEvidence must require/);
  const missingToolReplacement = clone(projectRegistry);
  missingToolReplacement.supersedes.toolIds = ["glavred-api"];
  assert(validateProjectWritingRegistry(missingToolReplacement, registry, OPTIONS).some((error) => /replace superseded template tool with the same id/.test(error)));
  missingToolReplacement.externalTools = [{ ...clone(registry.externalTools[0]), id: "other-tool" }];
  assert(validateProjectWritingRegistry(missingToolReplacement, registry, OPTIONS).some((error) => /replace superseded template tool with the same id/.test(error)));
  const collision = clone(projectRegistry);
  collision.sources = [clone(registry.sources[0])];
  assert(validateProjectWritingRegistry(collision, registry, OPTIONS).some((error) => /duplicate id/.test(error)));
  const editorOverride = clone(projectRegistry);
  editorOverride.supersedes.editorIds = [registry.editorRoles[0].id];
  editorOverride.editorRoles = [clone(registry.editorRoles[0])];
  assert.deepStrictEqual(validateProjectWritingRegistry(editorOverride, registry, OPTIONS), []);
  const toolOverride = clone(projectRegistry);
  toolOverride.supersedes.toolIds = ["glavred-api"];
  toolOverride.externalTools = [{
    ...clone(registry.externalTools[0]),
    access: "project-configured",
    configurationEvidence: {
      adapter: { type: "repo", value: "tests/fixtures/writing-tools/external-tool-adapter.fixture.js" },
      secretReference: "env:GLAVRED_API_TOKEN",
      owner: "project-writing-platform",
      checkedOn: "2026-07-17",
    },
  }];
  const unavailableConfiguration = validateProjectWritingRegistry(toolOverride, registry, OPTIONS);
  assert(unavailableConfiguration.some((error) => /env variable is not available/.test(error)));
  assert(unavailableConfiguration.some((error) => /cannot use forbidden path/.test(error)));
  toolOverride.externalTools[0].configurationEvidence.secretReference = "actual-token-value";
  assert(validateProjectWritingRegistry(toolOverride, registry, OPTIONS).some((error) => /must be a verifiable env reference/.test(error)));
  toolOverride.externalTools[0].configurationEvidence.secretReference = "env:GLAVRED_API_TOKEN";
  toolOverride.externalTools[0].configurationEvidence = "configured manually";
  assert(validateProjectWritingRegistry(toolOverride, registry, OPTIONS).some((error) => /configurationEvidence must be structured/.test(error)));
  validateConfiguredToolDirectly();
  console.log("Writing reference policy tests passed");
}

function validateConfiguredToolDirectly() {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "writing-adapter-"));
  const envName = "WRITING_TEST_PROVIDER_TOKEN";
  try {
    fs.writeFileSync(path.join(tempRoot, "adapter.js"), "module.exports = {};\n", "utf8");
    process.env[envName] = "present-for-validation";
    const tool = {
      ...clone(registry.externalTools[0]),
      access: "project-configured",
      configurationEvidence: {
        adapter: { type: "repo", value: "adapter.js" },
        secretReference: `env:${envName}`,
        owner: "test-owner",
        checkedOn: "2026-07-17",
      },
    };
    const errors = [];
    validateExternalTool(tool, tempRoot, errors);
    assert.deepStrictEqual(errors, []);
    tool.configurationEvidence.adapter.value = "../outside.js";
    const traversalErrors = [];
    validateExternalTool(tool, tempRoot, traversalErrors);
    assert(traversalErrors.some((error) => /inside the repository/.test(error)));
  } finally {
    delete process.env[envName];
    fs.rmSync(tempRoot, { recursive: true, force: true });
  }
}

main();
