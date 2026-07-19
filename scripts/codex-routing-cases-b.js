"use strict";

function runRouteCasesB(testRoute) {
  testRoute("мы опять латаем тот же модуль: чинить или заменить архитектуру", {
    modes: ["strategy"],
    skills: ["codex-change-strategy", "codex-strategic-review"],
    subagents: ["systems_reviewer", "tester"],
    fanoutStatus: "recommended",
    qualityGates: ["project-posture", "destination-transition", "objective-evidence-matrix"],
    changeStrategyRequired: true,
    changeStrategyRecordMode: "orchestrator-artifact",
    planRequired: true,
  });

  testRoute("текущая реализация держится на обходах; сравни новую архитектуру с сохранением публичных границ и эксплуатационными доказательствами", {
    skills: ["codex-change-strategy", "codex-product-goal", "codex-strategic-review"],
    semanticMatches: ["change-strategy"],
    qualityGates: ["protected-contracts", "objective-evidence-matrix", "approved-change-envelope"],
    changeStrategyRequired: true,
    planRequired: true,
  });

  testRoute("refactor an internal helper in a fresh toy app", {
    notModes: ["migration"],
    notSkills: ["codex-change-strategy", "codex-migrate"],
    changeStrategyRequired: false,
  });

  testRoute("review architecture documentation", {
    notSkills: ["codex-change-strategy"],
    changeStrategyRequired: false,
  });

  testRoute("the first repair failed; diagnose again before changing code", {
    notSkills: ["codex-change-strategy"],
    changeStrategyRequired: false,
  });

  testRoute("compatibility-only layer found while reading the affected path", {
    skills: ["codex-change-strategy"],
    changeStrategyRequired: true,
  });

  testRoute("fix typo in a private UI label", {
    modes: ["bugfix"],
    notSkills: ["codex-change-strategy"],
    qualityGates: ["bounded-repair-path-check"],
    changeStrategyRequired: false,
    discoveryKind: "local-leaf",
    blockEdits: false,
    options: {
      discovery: {
        phase: "reading",
        kind: "local-leaf",
        architecture_fit: "fit",
        summary: "The defect is confined to one private label formatter.",
        evidence_ref: "src/ui/label-formatter.ts direct consumer check",
        owner: "UI label module",
        sot: "Current component specification",
        protected_boundaries: [],
      },
    },
  });

  testRoute("fix the display bug", {
    modes: ["bugfix"],
    skills: ["codex-change-strategy"],
    notSkills: ["codex-product-goal", "codex-strategic-review"],
    subagents: ["systems_reviewer", "tester"],
    semanticMatches: ["change-strategy"],
    qualityGates: ["bounded-repair-path-check", "discovery-evidence-before-edit"],
    changeStrategyRequired: true,
    discoveryKind: "architecture-mismatch",
    blockEdits: true,
    options: {
      discovery: {
        phase: "reading",
        kind: "architecture-mismatch",
        architecture_fit: "mismatch",
        summary: "Reading found duplicate state ownership outside the accepted final path.",
        evidence_ref: "src/game/legacy-state.ts and plan final-path section",
        owner: "Gameplay state subsystem",
        sot: "Accepted product architecture plan",
        protected_boundaries: ["player-state contract"],
      },
    },
  });

  testRoute("read-only review: compare repair and replacement, do not modify files", {
    exactModes: ["review"],
    pipeline: "review",
    skills: ["codex-change-strategy"],
    notSkills: [
      "codex-debug", "codex-pipeline-workflow",
      "codex-product-goal", "codex-strategic-review",
    ],
    changeStrategyRequired: true,
    changeStrategyRecordMode: "response-only",
  });

  testRoute("read-only diagnose why the worker crashes; do not modify files", {
    modes: ["bugfix"],
    pipeline: "bugfix",
    skills: ["codex-debug"],
    changeStrategyRequired: false,
  });

  testRoute("переносим данные в новое хранилище без простоя и с откатом", {
    modes: ["migration"],
    skills: ["codex-migrate", "codex-strategic-review"],
    semanticMatches: ["migration"],
    risk: "HIGH",
  });

  testRoute("agent template client-executor contract anti-sycophancy no fake completion", {
    modes: ["template"],
    skills: [
      "codex-template-sync",
      "codex-skill-maintenance",
      "codex-test-rules",
      "codex-product-goal",
      "codex-strategic-review",
    ],
    qualityGates: ["template-boundary", "verification-evidence"],
    planRequired: true,
    risk: "HIGH",
  });

  testRoute("пропиши progressive JPEG правило: продуктовая сущность сразу имеет будущую форму на 1% callable, без legacy harness proof", {
    modes: ["template", "product-goal"],
    skills: [
      "codex-template-sync",
      "codex-product-goal",
      "codex-strategic-review",
    ],
    qualityGates: ["template-boundary", "product-goal-artifact", "verification-evidence"],
    planRequired: true,
    risk: "HIGH",
  });

  testRoute("компонент должен сразу содержать будущие функции на 1 процент и прокидывать debug что тут работает а не доказывать старый harness", {
    modes: ["feature", "product-goal"],
    skills: [
      "codex-feature-workflow",
      "codex-product-goal",
      "codex-strategic-review",
    ],
    qualityGates: ["user-business-outcome-link", "product-goal-artifact"],
    semanticMatches: ["product-goal"],
    planRequired: true,
  });

  testRoute("пропиши себе что компонент сразу содержит будущие функции на 1 процент и debug а не доказывает harness", {
    modes: ["template", "product-goal", "feature"],
    skills: [
      "codex-template-sync",
      "codex-product-goal",
      "codex-strategic-review",
      "codex-feature-workflow",
    ],
    qualityGates: ["template-boundary", "product-goal-artifact"],
    planRequired: true,
    risk: "HIGH",
  });

  testRoute("сделай LLM агента для Unreal Engine actor персонажа: на 1% готовности создать классы компоненты анимации интерфейсы переменные функции по финальному плану", {
    modes: ["template", "product-goal", "feature"],
    skills: [
      "codex-template-sync",
      "codex-product-goal",
      "codex-feature-workflow",
      "codex-strategic-review",
    ],
    qualityGates: ["template-boundary", "product-goal-artifact", "user-business-outcome-link"],
    planRequired: true,
    risk: "HIGH",
  });

  testRoute("если финального плана нет, агент должен блокировать реализацию и создать план объекта, потом проверять полноту по плану и уровень детализации", {
    modes: ["template", "product-goal", "strategy"],
    skills: [
      "codex-template-sync",
      "codex-product-goal",
      "codex-strategic-review",
    ],
    qualityGates: ["template-boundary", "product-goal-artifact"],
    planRequired: true,
    risk: "HIGH",
  });

  testRoute("сайт на 1% должен выполнять продакшн функцию показывать контакты и coming soon приложение", {
    modes: ["product-goal"],
    skills: ["codex-product-goal", "codex-strategic-review"],
    qualityGates: ["product-goal-artifact", "user-business-outcome-link"],
    planRequired: true,
  });

  testRoute("прогрессивный JPEG старые неправильные итерации заглушки и косяки не сохранять выключенными а удалять заменять мигрировать", {
    modes: ["template", "lessons"],
    skills: [
      "codex-template-sync",
      "codex-product-goal",
      "codex-strategic-review",
    ],
    qualityGates: ["template-boundary", "product-goal-artifact", "verification-evidence"],
    planRequired: true,
    risk: "HIGH",
  });

  testRoute("update API contract request/response pagination", {
    modes: ["api"],
    skills: ["codex-api-contract", "codex-feature-workflow"],
    risk: "MEDIUM",
  });

  testRoute("запрети MVP мышление и веди задачу как /goal с финальным качеством продукта", {
    modes: ["product-goal"],
    skills: ["codex-product-goal", "codex-strategic-review"],
    qualityGates: ["quality-bar", "current-step", "language-match"],
    planRequired: true,
  });

  testRoute("optimize roadmap for revenue, loyalty, retention, conversion, and app KPI", {
    modes: ["product-goal"],
    skills: ["codex-product-goal", "codex-strategic-review"],
    qualityGates: ["user-business-outcome-link", "quality-bar"],
    planRequired: true,
  });

  testRoute("изучи косяки последней недели и преврати уроки в правила шаблона", {
    modes: ["lessons", "template"],
    skills: ["codex-cross-project-lessons", "codex-template-sync"],
    qualityGates: ["lesson-classification", "validator-or-route-check"],
    planRequired: true,
    risk: "HIGH",
  });

  testRoute("нужно сохранить вывод из повторяющегося провала и поставить защиту на следующий раз", {
    modes: ["lessons"],
    skills: ["codex-cross-project-lessons", "codex-strategic-review"],
    semanticMatches: ["lessons"],
    planRequired: true,
  });

  testRoute("выпусти release v3.8.0 и tag чтобы проекты качали релиз", {
    modes: ["release"],
    skills: [
      "codex-template-sync",
      "codex-health-check",
      "codex-strategic-review",
    ],
    subagents: ["security_reviewer", "tester"],
    risk: "HIGH",
  });

  testRoute("проверь current OpenAI GPT-5.5 model docs", {
    modes: ["openai"],
    skills: ["codex-openai-model-guidance"],
    subagents: ["docs_researcher"],
    needsFreshDocs: true,
  });

  testRoute("нарисуй mermaid control board for release flow", {
    modes: ["mermaid", "release"],
    skills: ["codex-mermaid-board-workflow"],
  });

  testRoute("изучи сайт релиза GPT-5.6 и диаграммы на нём, подготовь таблицу", {
    modes: ["openai"],
    notModes: ["bugfix", "mermaid", "release"],
    skills: ["codex-openai-model-guidance"],
    subagents: ["docs_researcher", "reviewer"],
    fanoutStatus: "recommended",
    needsFreshDocs: true,
  });

  testRoute("Research the GPT-5.6 release announcement and diagrams", {
    modes: ["openai"],
    notModes: ["bugfix", "mermaid", "release"],
    skills: ["codex-openai-model-guidance"],
    subagents: ["docs_researcher", "reviewer"],
    fanoutStatus: "recommended",
    needsFreshDocs: true,
  });

  testRoute("изучи сайт релиза GPT-5.6 и диаграммы, без субагентов", {
    modes: ["openai"],
    notModes: ["bugfix", "mermaid", "release"],
    notSubagents: ["docs_researcher", "reviewer"],
    fanoutStatus: "skip",
  });

  testRoute("Что такое GPT-5.6?", {
    modes: ["openai"],
    fanoutStatus: "skip",
  });

  testRoute("обнови шаблон агентов, проверь и выпусти версию 4.6", {
    modes: ["template", "release"],
    subagents: ["systems_reviewer", "tester"],
    fanoutStatus: "required",
    risk: "HIGH",
  });

  testRoute("update the GPT-5.6 release site diagram and publish it", {
    modes: ["openai", "release", "mermaid"],
    notModes: ["bugfix"],
  });

  testRoute("draw a mermaid sequence", {
    modes: ["mermaid"],
    fanoutStatus: "conditional",
  });

}

module.exports = { runRouteCasesB };
