"use strict";

function runRouteCasesA(testRoute) {
  testRoute("почини падающий тест логина", {
    modes: ["bugfix", "testing"],
    skills: ["codex-debug", "codex-coverage"],
    subagents: ["tester", "reviewer"],
    risk: "MEDIUM",
  });

  testRoute("после обновления форма зависает и раньше это работало", {
    modes: ["bugfix"],
    skills: ["codex-debug"],
    semanticMatches: ["bugfix"],
    risk: "MEDIUM",
  });

  testRoute("кто-то может получить чужие данные из сессии", {
    modes: ["security", "product-ux"],
    skills: ["codex-security-audit", "codex-strategic-review"],
    semanticMatches: ["security"],
    risk: "HIGH",
  });

  testRoute("сделай дизайн экрана и figma mockup", {
    modes: ["design", "figma"],
    skills: ["codex-design-workflow", "codex-figma-workflow"],
    subagents: ["design_reviewer", "tester"],
    risk: "MEDIUM",
  });

  testRoute("polish harden typeset product surface", {
    modes: ["design"],
    skills: ["codex-design-workflow", "codex-domain-design-review"],
    qualityGates: ["token-contract", "state-coverage", "responsive-check"],
    risk: "MEDIUM",
  });

  testRoute("страница выглядит кустарно и пользователи не доверяют", {
    modes: ["design"],
    skills: ["codex-design-workflow"],
    semanticMatches: ["design"],
    risk: "MEDIUM",
  });

  testRoute("люди начинают путь, бросают его и не могут вернуться к ценности", {
    modes: ["product-ux"],
    skills: ["codex-product-ux-audit"],
    qualityGates: ["entry-to-value-flow", "return-path"],
    semanticMatches: ["product-ux"],
    planRequired: true,
  });

  testRoute("critique distill harden brand/product register UI pipeline for conversion KPI", {
    modes: ["design", "product-goal"],
    skills: [
      "codex-design-workflow",
      "codex-domain-design-review",
      "codex-product-goal",
      "codex-strategic-review",
    ],
    qualityGates: [
      "token-contract",
      "state-coverage",
      "responsive-check",
      "user-business-outcome-link",
    ],
    planRequired: true,
    risk: "MEDIUM",
  });

  testRoute("доработай дизайн-систему: токены, Storybook, атомы, молекулы и формы", {
    modes: ["design-system"],
    skills: ["codex-design-system-workflow", "codex-design-workflow"],
    qualityGates: ["token-contract", "composition-trace", "rendered-geometry"],
    planRequired: true,
    risk: "HIGH",
  });

  testRoute("проверь UX личного кабинета: вход, сервисы, dead ends и возврат на главную", {
    modes: ["product-ux"],
    skills: ["codex-product-ux-audit"],
    qualityGates: ["entry-to-value-flow", "no-dead-ends", "return-path"],
    planRequired: true,
  });

  testRoute("проверь безопасность auth secrets injection", {
    modes: ["security", "review"],
    skills: ["codex-security-audit", "codex-audit", "codex-strategic-review"],
    subagents: ["security_reviewer", "tester"],
    risk: "HIGH",
  });

  testRoute("обнови агентский шаблон, AGENTS.md, skills и router", {
    modes: ["template"],
    skills: [
      "codex-template-sync",
      "codex-skill-maintenance",
      "codex-test-rules",
      "codex-product-goal",
      "codex-strategic-review",
    ],
    subagents: ["scout", "tester"],
    qualityGates: ["template-boundary", "product-goal-artifact"],
    planRequired: true,
    risk: "HIGH",
  });

  testRoute("усилить AGENTS основной файл: единый SOT conflict protocol, больше примеров формулировки задач из references, системный анализ ошибок вместо локальных фиксов", {
    modes: ["template"],
    skills: [
      "codex-template-sync",
      "codex-agent-router",
      "codex-product-goal",
      "codex-strategic-review",
    ],
    qualityGates: ["template-boundary", "sot-validation", "product-goal-artifact"],
    planRequired: true,
    risk: "HIGH",
  });

  testRoute("проверь куда делся образ мысли ТРИЗ в основном агентском файле", {
    modes: ["template", "strategy"],
    skills: [
      "codex-template-sync",
      "codex-agent-router",
      "codex-product-goal",
      "codex-strategic-review",
    ],
    qualityGates: ["template-boundary", "product-goal-artifact"],
    planRequired: true,
    risk: "HIGH",
  });

  testRoute("проверь стратагемы и Сунь-цзы для конкурентной стратегии", {
    modes: ["review", "strategy"],
    notModes: ["release"],
    skills: ["codex-audit", "codex-strategic-review"],
    planRequired: true,
    risk: "MEDIUM",
  });

  testRoute("стратегический обзор продукта и рынка", {
    modes: ["strategy"],
    notModes: ["release"],
    skills: ["codex-strategic-review"],
    planRequired: true,
    risk: "MEDIUM",
  });

  testRoute("маркетологи должны проверить позиционирование, оффер, воронку и кампанию", {
    modes: ["marketing", "review"],
    skills: [
      "codex-domain-communication-review",
      "codex-domain-business-review",
      "codex-product-goal",
      "codex-strategic-review",
    ],
    notSkills: ["codex-writing-workflow"],
    qualityGates: [
      "audience-icp",
      "positioning-offer-clarity",
      "journey-or-funnel-fit",
      "measurement-and-ethics",
      "product-goal-artifact",
    ],
    planRequired: true,
    risk: "MEDIUM",
  });

  testRoute("marketing positioning campaign funnel offer ICP", {
    modes: ["marketing"],
    skills: ["codex-domain-communication-review", "codex-domain-business-review"],
    qualityGates: ["audience-icp", "channel-distribution-plan"],
    planRequired: true,
    risk: "MEDIUM",
  });

  testRoute("пользователи не покупают повторно, деньги теряются, нужно понять где ломается путь", {
    modes: ["marketing", "product-goal"],
    skills: ["codex-domain-communication-review", "codex-domain-business-review"],
    qualityGates: ["audience-icp", "journey-or-funnel-fit", "user-business-outcome-link"],
    semanticMatches: ["marketing"],
    planRequired: true,
  });

  testRoute("Напиши сцену разговора героя с антагонистом", {
    exactModes: ["writing-literary"],
    notModes: ["api", "writing-informational"],
    skills: ["codex-writing-workflow"],
    notSkills: ["codex-api-contract", "codex-feature-workflow"],
    sharedRules: [".claude/library/technical/writing.md"],
    qualityGates: ["writing-contract", "functional-whole", "mode-specific-review", "independent-review-or-self-check-label"],
    semanticMatches: ["writing-literary"],
    risk: "LOW",
  });

  testRoute("Перепиши оффер лендинга для целевой аудитории, чтобы повысить конверсию", {
    exactModes: ["marketing"],
    notModes: ["release", "review"],
    skills: [
      "codex-writing-workflow",
      "codex-domain-communication-review",
      "codex-domain-business-review",
      "codex-product-goal",
    ],
    sharedRules: [".claude/library/technical/writing.md"],
    qualityGates: ["audience-icp", "positioning-offer-clarity", "measurement-and-ethics"],
    risk: "MEDIUM",
  });

  testRoute("Напиши руководство для нового пользователя по настройке аккаунта", {
    exactModes: ["writing-informational"],
    notModes: ["api", "feature"],
    skills: ["codex-writing-workflow"],
    notSkills: ["codex-api-contract", "codex-feature-workflow"],
    sharedRules: [".claude/library/technical/writing.md"],
    qualityGates: ["reader-task-completion", "source-truth-boundary"],
    risk: "LOW",
  });

  testRoute("Напиши письмо клиенту о задержке и следующем обновлении", {
    exactModes: ["writing-communication"],
    notModes: ["marketing", "writing-informational"],
    skills: ["codex-writing-workflow"],
    notSkills: ["codex-domain-business-review"],
    sharedRules: [".claude/library/technical/writing.md"],
    qualityGates: ["recipient-action-path", "functional-whole"],
    risk: "LOW",
  });

  testRoute("Review this marketing email", {
    exactModes: ["marketing"],
    notModes: ["writing-communication", "review"],
    skills: ["codex-domain-communication-review", "codex-domain-business-review"],
    notSkills: ["codex-writing-workflow"],
    sharedRules: [".claude/library/technical/writing.md"],
    risk: "MEDIUM",
  });

  testRoute("Write an API integration guide", {
    exactModes: ["api", "technical-writing", "writing-informational"],
    notModes: ["design", "marketing"],
    skills: ["codex-api-contract", "codex-writing-workflow", "codex-technical-writing"],
    sharedRules: [".claude/library/technical/writing.md"],
    risk: "MEDIUM",
  });

  testRoute("Write generic API docs", {
    exactModes: ["api", "technical-writing", "writing-informational"],
    notModes: ["openai", "feature"],
    skills: ["codex-writing-workflow", "codex-technical-writing", "codex-api-contract"],
    notSkills: ["codex-openai-model-guidance", "codex-feature-workflow"],
    qualityGates: ["reference-registry-valid", "technical-procedure-executed"],
    needsFreshDocs: false,
    risk: "MEDIUM",
  });

  testRoute("Document a Stripe API endpoint", {
    exactModes: ["api", "technical-writing", "writing-informational"],
    notModes: ["openai", "feature"],
    notSkills: ["codex-openai-model-guidance", "codex-feature-workflow"],
    skills: ["codex-technical-writing", "codex-api-contract"],
  });

  testRoute("Write a PostgreSQL recovery runbook", {
    exactModes: ["technical-writing", "writing-informational"],
    notModes: ["openai", "feature"],
    skills: ["codex-technical-writing"],
    notSkills: ["codex-openai-model-guidance"],
  });

  testRoute("Write an API outage incident update", {
    exactModes: ["api", "technical-writing", "writing-communication"],
    skills: ["codex-technical-writing", "codex-api-contract"],
    notSkills: ["codex-openai-model-guidance"],
  });

  testRoute("Write OpenAI Responses API docs", {
    exactModes: ["api", "openai", "technical-writing", "writing-informational"],
    skills: ["codex-openai-model-guidance", "codex-technical-writing", "codex-api-contract"],
    needsFreshDocs: true,
  });

  testRoute("Write ORM data-model documentation", {
    exactModes: ["technical-writing", "writing-informational"],
    notModes: ["openai"],
    notSkills: ["codex-openai-model-guidance"],
    skills: ["codex-technical-writing"],
  });

  testRoute("Implement an API endpoint", {
    modes: ["api", "feature"],
    notModes: ["technical-writing", "writing-informational"],
    skills: ["codex-api-contract", "codex-feature-workflow"],
    notSkills: ["codex-technical-writing"],
  });

  testRoute("Review API docs", {
    exactModes: ["api", "technical-writing", "writing-informational"],
    skills: ["codex-domain-communication-review", "codex-technical-writing-review", "codex-api-contract"],
    notSkills: ["codex-writing-workflow", "codex-feature-workflow", "codex-openai-model-guidance"],
  });

  testRoute("Напиши клиентский отчёт: что было, что стало, что это даёт и что дальше", {
    exactModes: ["writing-communication"],
    notModes: ["api", "review"],
    skills: ["codex-writing-workflow"],
    sharedRules: [".claude/library/technical/writing.md"],
    risk: "LOW",
  });

  testRoute("протокол между клиентом и сервисом разошелся, поля не совместимы", {
    modes: ["api"],
    skills: ["codex-api-contract"],
    semanticMatches: ["api"],
    risk: "MEDIUM",
  });

}

module.exports = { runRouteCasesA };
