import assert from "node:assert/strict";
import { routeKeywords } from "./router.js";
import { buildActiveRulesOutput } from "./active-rules.js";

function assertRoute(
  task: string,
  expectedMode: string,
  expectedSkill: string,
  excludedModes: string[] = [],
): void {
  const route = routeKeywords(task);
  assert(route.modes.includes(expectedMode), `${task}: ${route.modes.join(", ")}`);
  assert(route.codexSkills.includes(expectedSkill), `${task}: ${route.codexSkills.join(", ")}`);
  for (const mode of excludedModes) {
    assert(!route.modes.includes(mode), `${task}: unexpected ${mode}`);
  }
}

function assertTechnicalRoute(
  task: string,
  expectedMode: string,
  excludedSkills: string[] = [],
): void {
  const route = routeKeywords(task);
  assert(route.modes.includes(expectedMode), `${task}: ${route.modes.join(", ")}`);
  assert(route.modes.includes("technical-writing"), `${task}: missing technical-writing`);
  assert(route.codexSkills.includes("codex-technical-writing"), `${task}: missing technical skill`);
  for (const skill of excludedSkills) {
    assert(!route.codexSkills.includes(skill), `${task}: unexpected ${skill}`);
  }
}

assertRoute(
  "Напиши сцену разговора героя с антагонистом",
  "writing-literary",
  "codex-writing-workflow",
  ["code", "docs"],
);
assertRoute(
  "Rewrite the landing page offer to improve conversion",
  "marketing",
  "codex-writing-workflow",
  ["code", "git"],
);
assert.deepEqual(routeKeywords("Rewrite the landing page offer to improve conversion").codeIntelligence.tools, []);
assertRoute(
  "Напиши руководство для нового пользователя",
  "writing-informational",
  "codex-writing-workflow",
  ["code", "docs"],
);
assertRoute(
  "Напиши письмо клиенту и попроси approve до пятницы",
  "writing-communication",
  "codex-writing-workflow",
  ["git", "review"],
);
assertRoute(
  "Review this marketing email",
  "marketing",
  "codex-domain-communication-review",
  ["write", "review"],
);
assertRoute(
  "Write an API integration guide",
  "writing-informational",
  "codex-api-contract",
  ["code", "design"],
);
assertTechnicalRoute("Write generic API docs", "api", ["codex-openai-model-guidance", "codex-feature-workflow"]);
assertTechnicalRoute("Document a Stripe API endpoint", "api", ["codex-openai-model-guidance", "codex-feature-workflow"]);
assertTechnicalRoute("Write a PostgreSQL recovery runbook", "writing-informational", ["codex-openai-model-guidance"]);
assertTechnicalRoute("Write an API outage incident update", "writing-communication", ["codex-openai-model-guidance"]);
assertTechnicalRoute("Write ORM data-model documentation", "writing-informational", ["codex-openai-model-guidance"]);

const openAiDocs = routeKeywords("Write OpenAI Responses API docs");
assert(openAiDocs.codexSkills.includes("codex-openai-model-guidance"));
assert(openAiDocs.needsFreshDocs);

const implementation = routeKeywords("Implement an API endpoint");
assert(implementation.modes.includes("code"));
assert(!implementation.modes.includes("technical-writing"));
assert(!implementation.codexSkills.includes("codex-technical-writing"));
assert.deepEqual(implementation.codeIntelligence.tools, ["codebase-memory", "ripgrep"]);

const symbolRefactor = routeKeywords("Rename the authentication symbol and update references");
assert.equal(symbolRefactor.codeIntelligence.id, "symbol-refactor");
assert.deepEqual(symbolRefactor.codeIntelligence.tools, ["codebase-memory", "serena", "ripgrep"]);

const securityRelease = routeKeywords("Run a security release audit for leaked secrets");
assert.equal(securityRelease.codeIntelligence.id, "security-and-release");
assert.deepEqual(securityRelease.codeIntelligence.tools, ["codebase-memory", "semgrep", "gitleaks", "ripgrep"]);

const russianTechnical = routeKeywords("Write this API integration guide in Russian");
assert.equal(russianTechnical.targetLanguage, "ru");
assert.deepEqual(russianTechnical.writingLanguageProfiles, ["russian-infostyle-core", "ilyakhov-russian-voice-decisions"]);
assert.deepEqual(russianTechnical.writingProcessProfiles, ["ilyakhov-plan-and-client-method"]);
assert(russianTechnical.writingDomainProfiles.includes("russian-explanation-and-persuasion"));
assert(russianTechnical.writingDomainProfiles.includes("reader-task-architecture"));
assert(russianTechnical.writingTechnicalProfiles.includes("technical-developer-conventions"));
assert.deepEqual(russianTechnical.writingExternalTools, [{ id: "glavred-api", access: "not-configured", execution: "not-run", paid: true }]);
assert(russianTechnical.writingGates.includes("external-tool-unavailable"));

const russianLetter = routeKeywords("Напиши на русском письмо клиенту с запросом решения");
assert(russianLetter.writingLanguageProfiles.includes("russian-business-correspondence-language"));
assert(russianLetter.writingProcessProfiles.includes("russian-business-correspondence-process"));
assert(russianLetter.writingDomainProfiles.includes("russian-business-correspondence"));
assert(russianLetter.writingDomainProfiles.includes("russian-explanation-and-persuasion"));

for (const task of ["Проверь в Главреде", "Дай оценку Главреда", "Отредактируй по Главреду", "Проверь этот текст в Главреде"]) {
  const route = routeKeywords(task);
  assert(route.modes.includes("writing-informational"), `${task}: ${route.modes.join(", ")}`);
  assert(!route.modes.includes("writing-literary"), `${task}: unexpected literary route`);
  assert.deepEqual(route.writingExternalTools, [{ id: "glavred-api", access: "not-configured", execution: "not-run", paid: true }]);
}

const englishTechnical = routeKeywords("Напиши на английском руководство по API");
assert.equal(englishTechnical.targetLanguage, "en");
assert.deepEqual(englishTechnical.writingLanguageProfiles, []);
assert.deepEqual(englishTechnical.writingProcessProfiles, ["ilyakhov-plan-and-client-method"]);
assert(englishTechnical.writingTechnicalProfiles.includes("technical-developer-conventions"));
assert(englishTechnical.writingRejectedProfiles.some(({ id, reason }) => id === "russian-infostyle-core" && reason === "target-language-mismatch"));

const mixedTechnical = routeKeywords("Напиши руководство на русском и английском по API");
assert.equal(mixedTechnical.targetLanguage, "mixed");
assert.deepEqual(mixedTechnical.writingLanguageProfiles, []);
assert(mixedTechnical.writingGates.includes("per-section-language-resolution"));

const restoredOutput = buildActiveRulesOutput(
  {
    currentModes: ["writing-informational"],
    activeRules: ["technical/writing.md"],
    lastRouteTime: "2026-07-17T00:00:00.000Z",
    taskDescription: "Check this Russian text in Glavred",
  },
  "ENGRAM: disabled",
);
assert(restoredOutput.includes("TARGET_LANGUAGE: ru"));
assert(restoredOutput.includes("WRITING_LANGUAGE_PROFILES: russian-infostyle-core"));
assert(restoredOutput.includes("WRITING_EXTERNAL_TOOLS: glavred-api:not-configured:not-run:paid"));
assert(restoredOutput.includes("external-tool-evidence-required"));
assert(restoredOutput.includes("CODE_INTELLIGENCE:"));

console.log("Context router writing and code-intelligence parity passed");
