#!/usr/bin/env node
"use strict";

const assert = require("assert");
const fs = require("fs");
const path = require("path");
const { classifyWritingIntent } = require("./lib/writing-intent.js");
const { getWritingRoutePolicy, loadProjectRegistry, resolveProfiles } = require("./lib/writing-route-policy.js");
const registry = require("../.claude/library/technical/writing-reference-registry.json");

function expectIntent(task, expected) {
  const actual = classifyWritingIntent(task);
  for (const [key, value] of Object.entries(expected)) {
    assert.deepStrictEqual(actual[key], value, `${task}: ${key}`);
  }
}

function main() {
  expectIntent("Напиши сцену разговора героя с антагонистом", {
    isWriting: true,
    action: "create",
    primaryMode: "literary",
    overlays: [],
    outputLanguage: "ru",
  });
  expectIntent("Rewrite the landing page offer to improve conversion", {
    isWriting: true,
    action: "edit",
    primaryMode: "marketing",
    overlays: [],
    outputLanguage: "en",
  });
  expectIntent("Напиши руководство для нового пользователя", {
    isWriting: true,
    action: "create",
    primaryMode: "informational",
    overlays: [],
  });
  expectIntent("Напиши письмо клиенту о задержке", {
    isWriting: true,
    action: "create",
    primaryMode: "communication",
    overlays: [],
  });
  expectIntent("Review this marketing email", {
    isWriting: true,
    action: "review",
    primaryMode: "marketing",
    overlays: [],
  });
  expectIntent("Write an API integration guide", {
    isWriting: true,
    action: "create",
    primaryMode: "informational",
    overlays: ["api"],
    specializations: ["technical"],
    domains: ["api"],
    vendors: [],
    outputLanguage: "en",
  });
  expectIntent("Напиши руководство на русском и английском по API", {
    isWriting: true,
    outputLanguage: "mixed",
    languageResolution: "explicit",
  });
  expectIntent("Подготовь русскую и английскую версии письма", {
    isWriting: true,
    outputLanguage: "mixed",
    languageResolution: "explicit",
  });
  expectIntent("Проверь английскую документацию API", { isWriting: true, outputLanguage: "en", languageResolution: "explicit" });
  expectIntent("Review Russian API documentation", { isWriting: true, outputLanguage: "ru", languageResolution: "explicit" });
  expectIntent("Write API documentation in French", { isWriting: true, outputLanguage: "fr", languageResolution: "explicit" });
  expectIntent("Write a PostgreSQL recovery runbook", {
    isWriting: true,
    action: "create",
    primaryMode: "informational",
    specializations: ["technical"],
    domains: [],
  });
  expectIntent("Review API outage incident update", {
    isWriting: true,
    action: "review",
    primaryMode: "communication",
    specializations: ["technical"],
    domains: ["api"],
  });
  expectIntent("Write OpenAI Responses API docs", {
    isWriting: true,
    primaryMode: "informational",
    specializations: ["technical"],
    domains: ["api"],
    vendors: ["openai"],
  });
  expectIntent("Implement an API endpoint", {
    isWriting: false,
    action: null,
    primaryMode: null,
    specializations: [],
    domains: [],
    outputLanguage: null,
  });
  expectIntent("Проверь позиционирование, ICP и воронку", {
    isWriting: false,
    action: null,
    primaryMode: null,
    overlays: [],
  });
  for (const task of ["Проверь в Главреде", "Дай оценку Главреда", "Отредактируй по Главреду", "Проверь этот текст в Главреде"]) {
    expectIntent(task, {
      isWriting: true,
      primaryMode: "informational",
      externalTools: ["glavred-api"],
    });
    const policy = getWritingRoutePolicy(classifyWritingIntent(task));
    assert.deepStrictEqual(policy.externalTools, [{ id: "glavred-api", access: "not-configured", execution: "not-run", paid: true }]);
    assert(policy.gates.includes("external-tool-not-run"));
    assert(policy.gates.includes("external-tool-unavailable"));
  }

  const technicalPolicy = getWritingRoutePolicy(classifyWritingIntent("Review API outage incident update"));
  assert.deepStrictEqual(new Set(technicalPolicy.extraModes), new Set(["technical-writing", "api"]));
  assert(technicalPolicy.skills.includes("codex-technical-writing-review"));
  for (const editor of ["communication-recipient", "technical-accuracy", "technical-procedure", "technical-architecture", "technical-language"]) {
    assert(technicalPolicy.editors.includes(editor), `missing editor ${editor}`);
  }
  for (const profile of ["technical-document-architecture", "technical-developer-conventions"]) {
    assert(technicalPolicy.profiles.includes(profile), `missing profile ${profile}`);
  }

  const russianPolicy = getWritingRoutePolicy(classifyWritingIntent("Write this API integration guide in Russian"));
  assert.strictEqual(russianPolicy.targetLanguage, "ru");
  assert.deepStrictEqual(russianPolicy.languageProfiles, ["russian-infostyle-core", "ilyakhov-russian-voice-decisions"]);
  assert.deepStrictEqual(russianPolicy.processProfiles, ["ilyakhov-plan-and-client-method"]);
  assert(russianPolicy.domainProfiles.includes("russian-explanation-and-persuasion"));
  assert(russianPolicy.domainProfiles.includes("ilyakhov-purpose-and-structure"));
  assert(russianPolicy.files.includes("technical/russian-writing-profile.md"));
  assert(russianPolicy.technicalProfiles.includes("technical-developer-conventions"));
  assert(!russianPolicy.technicalProfiles.includes("russian-infostyle-core"));
  assert(russianPolicy.languageEditors.includes("russian-language"));
  assert.deepStrictEqual(russianPolicy.externalTools, [{ id: "glavred-api", access: "not-configured", execution: "not-run", paid: true }]);
  assert(russianPolicy.gates.includes("external-tool-evidence-required"));
  assert(russianPolicy.gates.includes("external-tool-unavailable"));

  const russianLetterPolicy = getWritingRoutePolicy(classifyWritingIntent("Напиши на русском деловое письмо клиенту"));
  assert(russianLetterPolicy.languageProfiles.includes("russian-business-correspondence-language"));
  assert(russianLetterPolicy.processProfiles.includes("russian-business-correspondence-process"));
  assert(russianLetterPolicy.domainProfiles.includes("russian-business-correspondence"));
  assert(russianLetterPolicy.domainProfiles.includes("russian-explanation-and-persuasion"));
  assert(russianLetterPolicy.files.includes("technical/russian-business-correspondence.md"));
  assert(russianLetterPolicy.files.includes("technical/russian-explanation-and-persuasion.md"));

  const englishPolicy = getWritingRoutePolicy(classifyWritingIntent("Напиши на английском руководство по API"));
  assert.strictEqual(englishPolicy.targetLanguage, "en");
  assert.deepStrictEqual(englishPolicy.languageProfiles, []);
  assert.deepStrictEqual(englishPolicy.processProfiles, ["ilyakhov-plan-and-client-method"]);
  assert(englishPolicy.languageEditors.includes("informational-language"));
  assert(englishPolicy.rejectedProfiles.some(({ id, reason }) => id === "russian-infostyle-core" && reason === "target-language-mismatch"));
  assert(englishPolicy.technicalProfiles.includes("technical-developer-conventions"));
  assert(!englishPolicy.files.includes("technical/russian-writing-profile.md"));
  assert.deepStrictEqual(englishPolicy.externalTools, []);

  const russianLiterary = getWritingRoutePolicy(classifyWritingIntent("Напиши на русском сцену разговора"));
  assert.deepStrictEqual(russianLiterary.externalTools, []);

  const inferredReview = getWritingRoutePolicy(classifyWritingIntent("Проверь эту документацию API"));
  assert.strictEqual(inferredReview.languageResolution, "inferred");
  assert.deepStrictEqual(inferredReview.languageProfiles, []);
  assert(inferredReview.rejectedProfiles.some(({ reason }) => reason === "target-language-unconfirmed"));
  assert(!inferredReview.files.includes("technical/russian-writing-profile.md"));

  const frenchPolicy = getWritingRoutePolicy(classifyWritingIntent("Write API documentation in French"));
  assert.strictEqual(frenchPolicy.targetLanguage, "fr");
  assert.deepStrictEqual(frenchPolicy.languageProfiles, []);
  assert(frenchPolicy.technicalProfiles.includes("technical-developer-conventions"));

  const mixedPolicy = getWritingRoutePolicy(classifyWritingIntent("Напиши руководство на русском и английском по API"));
  assert.strictEqual(mixedPolicy.targetLanguage, "mixed");
  assert.deepStrictEqual(mixedPolicy.languageProfiles, []);
  assert(mixedPolicy.domainProfiles.includes("reader-task-architecture"));
  assert(mixedPolicy.technicalProfiles.includes("technical-developer-conventions"));
  assert(mixedPolicy.gates.includes("per-section-language-resolution"));
  assert(mixedPolicy.gates.includes("language-editor-missing"));

  const mixedLetterPolicy = getWritingRoutePolicy(classifyWritingIntent("Подготовь русскую и английскую версии письма"));
  assert.strictEqual(mixedLetterPolicy.targetLanguage, "mixed");
  assert.strictEqual(mixedLetterPolicy.languageResolution, "explicit");
  assert(mixedLetterPolicy.gates.includes("per-section-language-resolution"));

  const malformed = resolveProfiles(["broken"], "ru", {
    sources: [],
    profiles: [{ id: "broken" }],
  });
  assert.deepStrictEqual(malformed.rejected, [{ id: "broken", reason: "invalid-profile-shape" }]);
  assert.strictEqual(loadProjectRegistry("Z:\\definitely-missing-writing-registry.json"), null);
  const noOverlayPolicy = getWritingRoutePolicy(classifyWritingIntent("Напиши на русском справочную статью"), { projectRegistry: null });
  assert(noOverlayPolicy.languageProfiles.includes("russian-infostyle-core"));
  const optInRegistry = JSON.parse(JSON.stringify(registry));
  optInRegistry.profiles.find((profile) => profile.id === "reader-task-architecture").defaultUse = "opt-in";
  assert.deepStrictEqual(
    resolveProfiles(["reader-task-architecture"], "ru", optInRegistry, "informational").rejected,
    [{ id: "reader-task-architecture", reason: "explicit-activation-required" }],
  );

  const projectOverride = {
    schemaVersion: 2,
    updatedOn: "2026-07-17",
    owner: "project",
    supersedes: { editorIds: [], sourceIds: [], profileIds: ["russian-infostyle-core"], toolIds: [] },
    editorRoles: [],
    sources: [{
      id: "project-russian-style",
      title: "Approved Russian legal style",
      creator: "Project owner",
      kind: "project-slot",
      trust: "project-authority",
      language: "ru",
      usageClass: "project-reference",
      allowedEffects: ["voice", "syntax"],
      locator: { type: "project-slot", value: "approved Russian legal writing" },
      provenance: { status: "user-declared", checkedOn: "2026-07-17", rights: "Project-owned.", contentPolicy: "project-owned-only" },
      freshness: { required: false, maxAgeDays: null },
    }],
    profiles: [{
      id: "project-legal-russian-voice",
      authorityGroup: "language",
      sourceIds: ["project-russian-style"],
      modes: ["informational"],
      overlays: ["technical"],
      outputLanguages: ["ru"],
      effects: ["voice", "syntax"],
      defaultUse: "default",
      propertiesByEffect: {
        voice: ["approved legal terminology"],
        syntax: ["natural Russian syntax"],
      },
      editors: ["russian-language"],
      constraints: ["Project authority overrides the superseded generic profile."],
    }],
  };
  const projectPolicy = getWritingRoutePolicy(classifyWritingIntent("Напиши на русском справочную статью"), { projectRegistry: projectOverride });
  assert.deepStrictEqual(projectPolicy.languageProfiles, ["project-legal-russian-voice", "ilyakhov-russian-voice-decisions"]);
  assert(projectPolicy.domainProfiles.includes("russian-explanation-and-persuasion"));
  assert(projectPolicy.domainProfiles.includes("ilyakhov-purpose-and-structure"));
  assert.deepStrictEqual(projectPolicy.processProfiles, ["ilyakhov-plan-and-client-method"]);
  assert(!projectPolicy.languageProfiles.includes("russian-infostyle-core"));

  testConfiguredToolRouting(projectOverride);

  const editorMap = new Map(registry.editorRoles.map((editor) => [editor.id, editor]));
  for (const id of russianPolicy.technicalEditors) {
    const effects = editorMap.get(id)?.allowedEffects || [];
    assert(!effects.some((effect) => ["voice", "syntax", "line-editing", "example-method"].includes(effect)), `${id} leaks language effects`);
  }

  console.log("Writing intent classifier passed");
}

function testConfiguredToolRouting(projectOverride) {
  const adapterDir = fs.mkdtempSync(path.join(process.cwd(), ".writing-adapter-test-"));
  const adapterPath = path.join(adapterDir, "adapter.js");
  const relativeAdapter = path.relative(process.cwd(), adapterPath).split(path.sep).join("/");
  try {
    fs.writeFileSync(adapterPath, "module.exports = {};\n", "utf8");
    process.env.WRITING_TEST_PROVIDER_TOKEN = "present-for-validation";
    const configuredToolRegistry = JSON.parse(JSON.stringify(projectOverride));
    configuredToolRegistry.supersedes.toolIds = ["glavred-api"];
    configuredToolRegistry.externalTools = [{
      ...JSON.parse(JSON.stringify(registry.externalTools[0])),
      access: "project-configured",
      configurationEvidence: {
        adapter: { type: "repo", value: relativeAdapter },
        secretReference: "env:WRITING_TEST_PROVIDER_TOKEN",
        owner: "project-writing-platform",
        checkedOn: "2026-07-17",
      },
    }];
    const intent = classifyWritingIntent("Проверь этот русский текст в Главреде");
    const policy = getWritingRoutePolicy(intent, { projectRegistry: configuredToolRegistry });
    assert.deepStrictEqual(policy.externalTools, [{ id: "glavred-api", access: "project-configured", execution: "not-run", paid: true }]);
    assert(policy.gates.includes("external-tool-evidence-required"));
    assert(policy.gates.includes("external-tool-not-run"));
    assert(!policy.gates.includes("external-tool-unavailable"));
    delete configuredToolRegistry.externalTools[0].configurationEvidence;
    assert.throws(() => getWritingRoutePolicy(intent, { projectRegistry: configuredToolRegistry }), /Writing registry validation failed/);
  } finally {
    delete process.env.WRITING_TEST_PROVIDER_TOKEN;
    fs.rmSync(adapterDir, { recursive: true, force: true });
  }
}

main();
