#!/usr/bin/env node
const fs = require("fs");
const path = require("path");
const { getRoute } = require("./codex-route-task.js");

const REQUIRED_FILES = [
  "AGENTS.md",
  "CLAUDE.md",
  ".claude/agents/writer.md",
  ".claude/agents/technical-writer.md",
  ".claude/skills/writing-workflow/SKILL.md",
  ".claude/library/technical/writing.md",
  ".claude/library/technical/writing-mode-profiles.md",
  ".claude/library/technical/russian-writing-profile.md",
  ".claude/library/technical/russian-business-correspondence.md",
  ".claude/library/technical/russian-explanation-and-persuasion.md",
  ".claude/library/technical/technical-writing-profile.md",
  ".claude/library/technical/writing-editorial-board.md",
  ".claude/library/technical/writing-reference-registry.json",
  ".claude/library/product/production-product-standard.md",
  ".claude/library/process/product-goal-loop.md",
  ".claude/library/process/client-executor-contract.md",
  ".claude/library/process/change-strategy-gate.md",
  ".claude/skills/debug/SKILL.md",
  ".claude/library/domain/domain-design-system.md",
  "tasks/goal.md",
  ".agents/skills/codex-product-goal/SKILL.md",
  ".agents/skills/codex-change-strategy/SKILL.md",
  ".agents/skills/codex-progressive-jpeg-planner/SKILL.md",
  ".agents/skills/codex-progressive-jpeg-planner/references/domain-examples.md",
  ".agents/skills/codex-writing-workflow/SKILL.md",
  ".agents/skills/codex-technical-writing/SKILL.md",
  ".agents/skills/codex-technical-writing-review/SKILL.md",
  ".claude/library/technical/writing-mode-profiles.md",
  "docs/WRITING_WORKFLOW.md",
  "docs/WRITING_REFERENCE_PROVENANCE.md",
  ".agents/skills/codex-design-system-workflow/SKILL.md",
  ".agents/skills/codex-design-workflow/references/design-command-modes.md",
  ".agents/skills/codex-product-ux-audit/SKILL.md",
  ".agents/skills/codex-cross-project-lessons/SKILL.md",
  "scripts/lib/codex-route-intents.js",
  "scripts/lib/codex-discovery-reroute.js",
  "scripts/lib/codex-route-cli.js",
  "scripts/lib/codex-route-summary.js",
  "scripts/lib/writing-intent.js",
  "scripts/lib/writing-route-policy.js",
  "scripts/lib/writing-reference-policy.js",
  "scripts/lib/writing-external-tool-policy.js",
  "scripts/lib/writing-path-policy.js",
  "scripts/validate-writing-references.js",
  "tests/fixtures/writing-tools/external-tool-adapter.fixture.js",
  "scripts/codex-agent-policy.js",
  "scripts/codex-route-config.js",
  "scripts/progressive-status.js",
  "scripts/validate-progressive-plan.js",
  "scripts/validate-change-strategy.js",
  "scripts/lib/change-strategy-policy.js",
  "scripts/test-change-strategy.js",
  "scripts/code-intelligence-tools.js",
  "scripts/lib/code-intelligence-policy.js",
  "scripts/test-code-intelligence-tools.js",
  "_reference/code-intelligence-tools.json",
  ".codex/config.toml",
  "_reference/codex-mcp-config.toml",
  "scripts/configure-codex-mcp.js",
  "scripts/test-codex-mcp-config.js",
  "docs/CODE_INTELLIGENCE_TOOLCHAIN.md",
  "tests/fixtures/change-strategy/discovery-architecture-mismatch.json",
  "scripts/validate-subagent-trace.js",
];

const SOURCE_ONLY_REQUIRED_FILES = [
  "templates/project-starter/tasks/goal.md",
  "templates/project-starter/DESIGN.md",
  "templates/project-starter/design-policy.ignore",
];

const REQUIRED_TEXT = [
  { file: "AGENTS.md", text: "Production Product Standard" },
  { file: "AGENTS.md", text: "language of the user's request" },
  { file: "CLAUDE.md", text: "Production Product Standard" },
  { file: "CLAUDE.md", text: "language of the user's request" },
  { file: "AGENTS.md", text: "app-specific business outcomes first" },
  { file: "CLAUDE.md", text: "app-specific business outcomes first" },
  { file: "docs/SHARED_CONVENTIONS.md", text: "Client Executor Accountability" },
  { file: ".claude/library/product/production-product-standard.md", text: "Product Outcome Priority" },
  { file: ".claude/library/product/production-product-standard.md", text: "Client Executor Standard" },
  { file: ".claude/library/process/product-goal-loop.md", text: "Product/Business Priority" },
  { file: ".claude/library/process/product-goal-loop.md", text: "client-executor-contract.md" },
  { file: ".claude/library/process/client-executor-contract.md", text: "No Fake Completion" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "Objective Evidence Matrix" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "repair" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "replace" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "migrate" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "Lines of code" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "Performance Evidence" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "Decision Authority" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "Destination And Transition" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "Approved Change Envelope" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "response-only" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "bounded repair-path check" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "before the first patch" },
  { file: ".claude/library/process/change-strategy-gate.md", text: "--decision-file" },
  { file: ".claude/skills/debug/SKILL.md", text: "bounded negative check" },
  { file: ".claude/skills/debug/SKILL.md", text: "mandatory fallback circuit breaker" },
  { file: "AGENTS.md", text: "--discovery-file" },
  { file: ".claude/library/process/client-executor-contract.md", text: "Anti-Sycophancy Rules" },
  { file: ".claude/library/process/client-executor-contract.md", text: "evidence before claiming work is done" },
  { file: ".claude/library/process/client-executor-contract.md", text: "Progressive JPEG Delivery" },
  { file: "scripts/lib/codex-route-summary.js", text: "CODE_INTELLIGENCE:" },
  { file: "mcp-servers/context-router/src/index.ts", text: "CODE_INTELLIGENCE:" },
  { file: ".codex/config.toml", text: "[mcp_servers.context-router]" },
  { file: ".codex/config.toml", text: "[mcp_servers.engram]" },
  { file: ".codex/config.toml", text: "[mcp_servers.codebase-memory-mcp]" },
  { file: "scripts/sync-template.sh", text: "CODEX_MCP_MERGER" },
  { file: "scripts/sync-template.sh", text: "ls-files --error-unmatch" },
  { file: "docs/CODE_INTELLIGENCE_TOOLCHAIN.md", text: "Engram не надо удалять" },
  { file: ".claude/library/process/client-executor-contract.md", text: "Progressive JPEG Implementation Meaning" },
  { file: ".claude/library/process/client-executor-contract.md", text: "whole planned object at low detail" },
  { file: ".claude/library/process/client-executor-contract.md", text: "retire the old layer" },
  { file: ".claude/library/process/client-executor-contract.md", text: "node scripts/progressive-status.js --check" },
  { file: ".claude/library/process/client-executor-contract.md", text: "first useful view" },
  { file: ".claude/library/process/client-executor-contract.md", text: "next sharpened layer" },
  { file: ".claude/library/process/client-executor-contract.md", text: "replan trigger" },
  { file: ".claude/library/process/plan-first.md", text: "### Progressive JPEG" },
  { file: ".claude/library/process/plan-first.md", text: "1% callable" },
  { file: ".claude/library/process/plan-first.md", text: "Final object plan" },
  { file: ".claude/library/process/plan-first.md", text: "Replacement/cleanup" },
  { file: ".claude/library/process/product-goal-loop.md", text: "Progressive JPEG Checkpoint" },
  { file: ".claude/library/process/product-goal-loop.md", text: "PROGRESSIVE_STATUS" },
  { file: ".claude/library/process/product-goal-loop.md", text: "Progressive JPEG Implementation Gate" },
  { file: ".claude/library/process/product-goal-loop.md", text: "Verification order for object readiness" },
  { file: ".claude/library/process/product-goal-loop.md", text: "Progressive layer replacement gate" },
  { file: ".claude/library/product/production-product-standard.md", text: "Progressive JPEG Anti-Falsification Gate" },
  { file: ".claude/library/product/production-product-standard.md", text: "end-state skeleton" },
  { file: ".claude/library/product/production-product-standard.md", text: "Object Readiness Levels" },
  { file: ".claude/library/product/production-product-standard.md", text: "Progressive Layer Replacement Pipeline" },
  { file: ".claude/library/product/production-product-standard.md", text: "Progressive Status Headers And Project Slice" },
  { file: ".claude/library/product/production-product-standard.md", text: ".session-cache/progressive-status.json" },
  { file: ".claude/library/product/production-product-standard.md", text: "superseded-layer audit" },
  { file: ".claude/library/product/production-product-standard.md", text: "temporary migration" },
  { file: ".claude/library/product/production-product-standard.md", text: "Unreal/game actor" },
  { file: ".claude/library/process/product-goal-loop.md", text: "broken contract" },
  { file: ".claude/library/process/product-goal-loop.md", text: "smallest systemic fix" },
  { file: ".claude/library/meta/strategic-thinking.md", text: "TRIZ Contradiction Gate" },
  { file: ".claude/library/meta/strategic-thinking.md", text: "Sun Tzu / Stratagem Terrain Check" },
  { file: ".claude/library/meta/strategic-thinking.md", text: "Plan Reality Check" },
  { file: ".claude/library/technical/writing.md", text: "progressive JPEG shape" },
  { file: ".claude/library/technical/writing.md", text: "Four Semantic Modes" },
  { file: ".claude/library/technical/writing.md", text: "LitAI-Derived Workflow" },
  { file: ".claude/library/technical/writing.md", text: "Technical writing is a specialization" },
  { file: ".claude/library/technical/technical-writing-profile.md", text: "Technical Progressive JPEG" },
  { file: ".claude/library/technical/writing-editorial-board.md", text: "Independent Review Rule" },
  { file: ".claude/library/technical/russian-writing-profile.md", text: "Derived Russian Examples" },
  { file: ".claude/library/technical/russian-writing-profile.md", text: "English domain standard may change facts" },
  { file: ".claude/library/technical/russian-business-correspondence.md", text: "Correspondence Contract" },
  { file: ".claude/library/technical/russian-explanation-and-persuasion.md", text: "Explanation Contract" },
  { file: ".claude/library/technical/writing.md", text: "External Tool Truth Gate" },
  { file: "scripts/lib/writing-route-policy.js", text: "externalTools" },
  { file: ".claude/library/technical/writing-reference-registry.json", text: "glavred-api" },
  { file: ".claude/library/technical/writing-reference-registry.json", text: "requiresArtifactBinding" },
  { file: ".agents/skills/codex-technical-writing/SKILL.md", text: "declared environment" },
  { file: ".agents/skills/codex-technical-writing-review/SKILL.md", text: "Procedure" },
  { file: "scripts/lib/writing-route-policy.js", text: "TECHNICAL_EDITORS" },
  { file: "scripts/lib/writing-route-policy.js", text: "languageProfiles" },
  { file: "docs/WRITING_REFERENCE_PROVENANCE.md", text: "template baseline" },
  { file: ".claude/library/technical/writing.md", text: "deliberate typos" },
  { file: ".claude/library/technical/writing.md", text: "evasion of AI detectors" },
  { file: ".agents/skills/codex-writing-workflow/SKILL.md", text: "functional 1% whole" },
  { file: ".agents/skills/codex-writing-workflow/SKILL.md", text: "Never invent facts" },
  { file: ".agents/skills/codex-writing-workflow/SKILL.md", text: "genuine child trace" },
  { file: ".claude/library/technical/writing-mode-profiles.md", text: "Progressive Readiness" },
  { file: "docs/WRITING_WORKFLOW.md", text: "Writing Workflow Architecture And LitAI Adaptation" },
  { file: "AGENTS.md", text: "progressive JPEG delivery" },
  { file: "AGENTS.md", text: "end-state skeleton" },
  { file: "AGENTS.md", text: "1% callable" },
  { file: "AGENTS.md", text: "gate on a missing final plan" },
  { file: "AGENTS.md", text: "every implementation slice must fulfill the real product purpose end to end" },
  { file: "AGENTS.md", text: "evidence may not be fabricated" },
  { file: "AGENTS.md", text: "PROGRESSIVE_STATUS" },
  { file: "AGENTS.md", text: "SOT Conflict Protocol" },
  { file: "AGENTS.md", text: "Systemic Error Analysis" },
  { file: "AGENTS.md", text: "Change Strategy Gate" },
  { file: "AGENTS.md", text: "Thinking Tools Gate" },
  { file: "AGENTS.md", text: "semantic intent scoring" },
  { file: "AGENTS.md", text: "fanout" },
  { file: "AGENTS.md", text: "xhigh" },
  { file: "AGENTS.md", text: "TRIZ contradiction gate" },
  { file: "AGENTS.md", text: "Sun Tzu / stratagem terrain check" },
  { file: "AGENTS.md", text: "Marketing/GTM" },
  { file: "AGENTS.md", text: "Task Formulation Examples" },
  { file: "CLAUDE.md", text: "progressive JPEG delivery" },
  { file: "CLAUDE.md", text: "end-state skeleton" },
  { file: "CLAUDE.md", text: "SOT conflict protocol" },
  { file: "CLAUDE.md", text: "Thinking tools gate" },
  { file: "CLAUDE.md", text: "semantic intent scoring" },
  { file: "CLAUDE.md", text: "Sun Tzu/stratagem terrain check" },
  { file: "CLAUDE.md", text: "Systemic Error Analysis" },
  { file: "CLAUDE.md", text: "change-strategy-gate.md" },
  { file: "docs/AGENT_CONTEXT_SOT.md", text: "SOT Conflict Protocol" },
  { file: "docs/AGENT_CONTEXT_SOT.md", text: "ask the user with 2-3 options" },
  { file: "brain/03-knowledge/communication/ilyakhov-planning-principles.md", text: "Superseded integration decision" },
  { file: ".agents/skills/codex-product-goal/SKILL.md", text: "app-specific KPI" },
  { file: ".agents/skills/codex-change-strategy/SKILL.md", text: "objective evidence" },
  { file: ".agents/skills/codex-change-strategy/SKILL.md", text: "validate-change-strategy.js" },
  { file: ".agents/skills/codex-product-goal/SKILL.md", text: "fresh evidence" },
  { file: ".agents/skills/codex-product-goal/SKILL.md", text: "progressive JPEG delivery" },
  { file: ".agents/skills/codex-product-goal/SKILL.md", text: "1% callable" },
  { file: ".agents/skills/codex-product-goal/SKILL.md", text: "Object Readiness Check" },
  { file: ".agents/skills/codex-product-goal/SKILL.md", text: "superseded-layer audit" },
  { file: ".agents/skills/codex-product-goal/SKILL.md", text: "progressive-status.js --check" },
  { file: ".agents/skills/codex-strategic-review/SKILL.md", text: "product user" },
  { file: ".agents/skills/codex-strategic-review/SKILL.md", text: "sycophancy" },
  { file: ".agents/skills/codex-strategic-review/SKILL.md", text: "TRIZ contradiction gate" },
  { file: ".agents/skills/codex-strategic-review/SKILL.md", text: "Sun Tzu / stratagem terrain check" },
  { file: ".agents/skills/codex-strategic-review/SKILL.md", text: "marketing/GTM work" },
  { file: ".agents/skills/codex-strategic-review/SKILL.md", text: "Ilyakhov plan reality check" },
  { file: ".agents/skills/codex-strategic-review/SKILL.md", text: "progressive JPEG delivery" },
  { file: ".agents/skills/codex-strategic-review/SKILL.md", text: "legacy harness proof" },
  { file: ".agents/skills/codex-strategic-review/SKILL.md", text: "object readiness" },
  { file: ".agents/skills/codex-strategic-review/SKILL.md", text: "progressive layer replacement" },
  { file: ".agents/skills/codex-strategic-review/SKILL.md", text: "project-slice reporting" },
  { file: ".agents/skills/codex-agent-router/SKILL.md", text: "Marketing, GTM, positioning" },
  { file: ".agents/skills/codex-agent-router/SKILL.md", text: "semantic intent scoring" },
  { file: ".agents/skills/codex-agent-router/SKILL.md", text: "end-state skeleton" },
  { file: ".agents/skills/codex-agent-router/SKILL.md", text: "progressive layer replacement gate" },
  { file: ".agents/skills/codex-feature-workflow/SKILL.md", text: "Progressive JPEG Implementation" },
  { file: ".agents/skills/codex-feature-workflow/SKILL.md", text: "1% ready" },
  { file: ".agents/skills/codex-feature-workflow/SKILL.md", text: "final object plan" },
  { file: ".agents/skills/codex-feature-workflow/SKILL.md", text: "obsolete layers" },
  { file: ".agents/skills/codex-feature-workflow/SKILL.md", text: "PROGRESSIVE_STATUS" },
  { file: ".agents/skills/codex-design-workflow/SKILL.md", text: "1% callable component slots" },
  { file: ".agents/skills/codex-design-workflow/SKILL.md", text: "final plan" },
  { file: ".agents/skills/codex-design-workflow/SKILL.md", text: "Replace/cleanup" },
  { file: ".agents/skills/codex-design-system-workflow/SKILL.md", text: "End-state skeleton" },
  { file: ".agents/skills/codex-design-system-workflow/SKILL.md", text: "Plan gate" },
  { file: ".agents/skills/codex-design-system-workflow/SKILL.md", text: "Replacement gate" },
  { file: ".agents/skills/codex-decompose/SKILL.md", text: "progressive JPEG delivery" },
  { file: ".agents/skills/codex-decompose/SKILL.md", text: "progressive JPEG implementation" },
  { file: ".agents/skills/codex-decompose/SKILL.md", text: "object readiness level" },
  { file: ".agents/skills/codex-decompose/SKILL.md", text: "replacement/cleanup gate" },
  { file: ".agents/skills/codex-decompose/SKILL.md", text: "project-slice bar" },
  { file: "scripts/progressive-status.js", text: "Progressive JPEG Project Slice" },
  { file: "scripts/progressive-status.js", text: "content changed but PROGRESSIVE_STATUS header did not change" },
  { file: "scripts/validate-template.sh", text: "Progressive status validates" },
  { file: "tasks/current.md", text: "PROGRESSIVE_STATUS" },
  { file: ".claude/library/technical/testing.md", text: "Stale implementation paths" },
  { file: ".agents/skills/codex-domain-communication-review/SKILL.md", text: "independent review skill" },
  { file: ".agents/skills/codex-domain-communication-review/SKILL.md", text: "functional progressive whole" },
  { file: "scripts/lib/codex-route-intents.js", text: "INTENT_GROUPS" },
  { file: "scripts/lib/writing-intent.js", text: "classifyWritingIntent" },
  { file: "scripts/lib/codex-route-intents.js", text: "future capability" },
  { file: "scripts/lib/codex-route-intents.js", text: "remember this rule" },
  { file: "scripts/codex-route-task.js", text: "user-business-outcome-link" },
  { file: "scripts/codex-route-config.js", text: "client-executor" },
  { file: "scripts/codex-route-config.js", text: "single source of truth" },
  { file: "scripts/codex-route-config.js", text: "go-to-market" },
  { file: "scripts/codex-route-config.js", text: "write this into yourself" },
  { file: "scripts/codex-route-config.js", text: "superseded layer" },
  { file: "scripts/codex-route-task.js", text: "semanticMatches" },
  { file: "scripts/codex-route-task.js", text: "exact-patterns-plus-semantic-intent-scoring" },
  { file: "scripts/codex-agent-policy.js", text: "gpt-5.6-sol" },
  { file: "scripts/codex-agent-policy.js", text: "gpt-5.6-terra" },
  { file: "scripts/codex-agent-policy.js", text: "gpt-5.6-luna" },
  { file: "scripts/codex-agent-policy.js", text: "effortCeiling: \"xhigh\"" },
  { file: "scripts/codex-agent-policy.js", text: "maxAutomaticWaves: 1" },
  { file: ".agents/skills/codex-progressive-jpeg-planner/SKILL.md", text: "Anti-Falsification Gate" },
  { file: ".agents/skills/codex-progressive-jpeg-planner/SKILL.md", text: "Every implementation slice" },
  { file: ".agents/skills/codex-progressive-jpeg-planner/SKILL.md", text: "enabling checkpoint" },
  { file: ".agents/skills/codex-progressive-jpeg-planner/SKILL.md", text: "tasks/progressive-plan.json" },
  { file: ".agents/skills/codex-progressive-jpeg-planner/references/domain-examples.md", text: "Game actor" },
  { file: "scripts/validate-progressive-plan.js", text: "validateProgressivePlan" },
  { file: "scripts/validate-change-strategy.js", text: "validateChangeStrategy" },
  { file: "scripts/lib/change-strategy-policy.js", text: "MATERIAL_IMPACTS" },
  { file: "scripts/test-change-strategy.js", text: "Change strategy tests passed" },
  { file: "scripts/validate-subagent-trace.js", text: "validateSubagentTrace" },
  { file: ".claude/library/product/production-product-standard.md", text: "MVP/prototype" },
  { file: ".claude/library/process/product-goal-loop.md", text: "This is not a \"final slice\" model" },
  { file: ".claude/library/domain/domain-design-system.md", text: "Rendered Geometry Gate" },
  { file: ".claude/library/domain/domain-design-system.md", text: "end-state skeleton" },
  { file: ".claude/library/domain/domain-design-system.md", text: "retire superseded design-system layers" },
  { file: ".claude/library/domain/domain-design-pipeline.md", text: "Durable Design Context" },
  { file: ".claude/library/domain/domain-design-pipeline.md", text: "1% callable" },
  { file: ".claude/library/domain/domain-design-pipeline.md", text: "release-only harnesses" },
  { file: ".claude/library/domain/domain-design-pipeline.md", text: "Register Gate" },
  { file: ".claude/library/domain/domain-design-pipeline.md", text: "Human judgment before deterministic findings" },
  { file: ".claude/library/domain/domain-design-pipeline.md", text: "Browser And Visual Hardening Gate" },
  { file: ".agents/skills/codex-design-workflow/SKILL.md", text: "root `DESIGN.md`" },
  { file: ".agents/skills/codex-design-workflow/SKILL.md", text: "Hardening Evidence" },
  { file: ".agents/skills/codex-design-workflow/SKILL.md", text: "references/design-command-modes.md" },
  { file: ".agents/skills/codex-design-workflow/references/design-command-modes.md", text: "Register Gate" },
  { file: ".agents/skills/codex-design-workflow/references/design-command-modes.md", text: "Product register" },
  { file: ".agents/skills/codex-design-workflow/references/design-command-modes.md", text: "Human judgment first, validator output second" },
  { file: ".agents/skills/codex-domain-design-review/SKILL.md", text: "Automated findings are evidence, not the design verdict" },
  { file: "scripts/validate-design-policy.js", text: "Design policy notification" },
  { file: "tasks/goal.md", text: "Final Outcome" },
];

const SOURCE_ONLY_REQUIRED_TEXT = [
  { file: "templates/project-starter/DESIGN.md", text: "tasks/goal.md` owns product intent" },
  { file: "templates/project-starter/tasks/goal.md", text: "Quality Bar" },
  { file: "templates/project-starter/tasks/current.md", text: "PROGRESSIVE_STATUS" },
];

const ROUTE_CASES = [
  {
    task: "reading found duplicate state ownership and a source of truth conflict outside the accepted final path",
    skills: ["codex-change-strategy", "codex-product-goal", "codex-strategic-review"],
    gates: ["project-posture", "protected-contracts", "destination-transition"],
  },
  {
    task: "мы опять латаем тот же модуль: чинить или заменить архитектуру",
    skills: ["codex-change-strategy", "codex-product-goal", "codex-strategic-review"],
    gates: ["project-posture", "protected-contracts", "destination-transition", "objective-evidence-matrix"],
  },
  {
    task: "plan progressive JPEG iterations where every slice solves the product purpose end to end",
    skills: ["codex-progressive-jpeg-planner", "codex-product-goal", "codex-decompose"],
    gates: ["product-purpose", "end-to-end-user-victory", "anti-falsification", "final-path-evidence"],
  },
  {
    task: "agent template client-executor contract anti-sycophancy no fake completion",
    skills: ["codex-template-sync", "codex-product-goal", "codex-strategic-review"],
    gates: ["template-boundary", "verification-evidence"],
  },
  {
    task: "пропиши progressive JPEG правило: продуктовая сущность сразу имеет будущую форму на 1% callable, без legacy harness proof",
    skills: ["codex-template-sync", "codex-product-goal", "codex-strategic-review"],
    gates: ["template-boundary", "product-goal-artifact", "verification-evidence"],
  },
  {
    task: "компонент должен сразу содержать будущие функции на 1 процент и прокидывать debug что тут работает а не доказывать старый harness",
    skills: ["codex-feature-workflow", "codex-product-goal", "codex-strategic-review"],
    gates: ["user-business-outcome-link", "product-goal-artifact"],
  },
  {
    task: "пропиши себе что компонент сразу содержит будущие функции на 1 процент и debug а не доказывает harness",
    skills: ["codex-template-sync", "codex-product-goal", "codex-strategic-review", "codex-feature-workflow"],
    gates: ["template-boundary", "product-goal-artifact"],
  },
  {
    task: "сделай LLM агента для Unreal Engine actor персонажа: на 1% готовности создать классы компоненты анимации интерфейсы переменные функции по финальному плану",
    skills: ["codex-template-sync", "codex-product-goal", "codex-feature-workflow", "codex-strategic-review"],
    gates: ["template-boundary", "product-goal-artifact", "user-business-outcome-link"],
  },
  {
    task: "если финального плана нет, агент должен блокировать реализацию и создать план объекта, потом проверять полноту по плану и уровень детализации",
    skills: ["codex-template-sync", "codex-product-goal", "codex-strategic-review"],
    gates: ["template-boundary", "product-goal-artifact"],
  },
  {
    task: "сайт на 1% должен выполнять продакшн функцию показывать контакты и coming soon приложение",
    skills: ["codex-product-goal", "codex-strategic-review"],
    gates: ["product-goal-artifact", "user-business-outcome-link"],
  },
  {
    task: "прогрессивный JPEG старые неправильные итерации заглушки и косяки не сохранять выключенными а удалять заменять мигрировать",
    skills: ["codex-template-sync", "codex-product-goal", "codex-strategic-review"],
    gates: ["template-boundary", "product-goal-artifact", "verification-evidence"],
  },
  {
    task: "усилить AGENTS основной файл SOT conflict TRIZ образ мысли Ильяхов",
    skills: ["codex-template-sync", "codex-agent-router", "codex-product-goal", "codex-strategic-review"],
    gates: ["template-boundary", "sot-validation", "product-goal-artifact"],
  },
  {
    task: "маркетологи проверяют позиционирование оффер воронку кампанию и ICP",
    skills: ["codex-domain-communication-review", "codex-domain-business-review", "codex-product-goal", "codex-strategic-review"],
    gates: ["audience-icp", "positioning-offer-clarity", "measurement-and-ethics"],
  },
  {
    task: "напиши художественную сцену с конфликтом и поворотом",
    skills: ["codex-writing-workflow"],
    gates: ["writing-contract", "functional-whole", "mode-specific-review"],
    planRequired: false,
  },
  {
    task: "напиши руководство пользователю с проверяемым результатом",
    skills: ["codex-writing-workflow"],
    gates: ["writing-contract", "reader-task-completion"],
    planRequired: false,
  },
  {
    task: "Write generic API docs",
    skills: ["codex-writing-workflow", "codex-technical-writing", "codex-api-contract"],
    gates: ["reference-registry-valid", "technical-procedure-executed"],
    excludedSkills: ["codex-openai-model-guidance", "codex-feature-workflow"],
  },
  {
    task: "Write OpenAI Responses API docs",
    skills: ["codex-technical-writing", "codex-api-contract", "codex-openai-model-guidance"],
    gates: ["technical-procedure-executed"],
    needsFreshDocs: true,
  },
  {
    task: "напиши письмо клиенту с владельцем действия и сроком ответа",
    skills: ["codex-writing-workflow"],
    gates: ["writing-contract", "recipient-action-path"],
    planRequired: false,
  },
  {
    task: "пользователи не покупают повторно деньги теряются путь ломается",
    skills: ["codex-domain-communication-review", "codex-domain-business-review", "codex-product-goal"],
    gates: ["journey-or-funnel-fit", "user-business-outcome-link"],
  },
  {
    task: "кто-то может получить чужие данные из сессии",
    skills: ["codex-security-audit", "codex-strategic-review"],
    gates: ["rollback-or-plan-b"],
  },
  {
    task: "проверь стратагемы Сунь-цзы и конкурентную стратегию",
    skills: ["codex-strategic-review"],
    gates: ["verification-evidence"],
  },
  {
    task: "optimize plan for revenue, retention, loyalty, and business KPI",
    skills: ["codex-product-goal", "codex-strategic-review"],
    gates: ["product-goal-artifact", "user-business-outcome-link"],
  },
  {
    task: "запрети MVP мышление и веди как goal",
    skills: ["codex-product-goal"],
    gates: ["quality-bar", "current-step"],
  },
  {
    task: "доработай дизайн-систему, токены, Storybook и атомы",
    skills: ["codex-design-system-workflow"],
    gates: ["token-contract", "rendered-geometry"],
  },
  {
    task: "проверь UX входа, сервисы и dead ends",
    skills: ["codex-product-ux-audit"],
    gates: ["entry-to-value-flow", "no-dead-ends"],
  },
  {
    task: "изучи косяки недели и улучши шаблон",
    skills: ["codex-cross-project-lessons", "codex-template-sync"],
    gates: ["lesson-classification", "product-goal-artifact"],
  },
];

const state = {
  checks: 0,
  errors: [],
};

function addError(message) {
  state.errors.push(message);
}

function readText(filePath) {
  return fs.readFileSync(filePath, "utf8");
}

function isTemplateSourceRepo() {
  const specPath = path.join(process.cwd(), "PROJECT_SPEC.md");
  if (!fs.existsSync(specPath)) {
    return false;
  }
  const hasSourceOnlyStarter = fs.existsSync(path.join(process.cwd(), "templates/project-starter"));
  return hasSourceOnlyStarter && readText(specPath).split(/\r?\n/).some((line) => line.trim() === "- Name: agent-project-template");
}

function assertFile(relativePath) {
  state.checks += 1;
  if (!fs.existsSync(path.join(process.cwd(), relativePath))) {
    addError(`missing required v4 file: ${relativePath}`);
  }
}

function assertText(requirement) {
  state.checks += 1;
  const filePath = path.join(process.cwd(), requirement.file);
  if (!fs.existsSync(filePath)) {
    addError(`cannot check missing file: ${requirement.file}`);
    return;
  }
  if (!readText(filePath).includes(requirement.text)) {
    addError(`${requirement.file} missing text: ${requirement.text}`);
  }
}

function assertIncludes(values, expected, label) {
  state.checks += 1;
  if (!values.includes(expected)) {
    addError(`${label} missing ${expected}; got ${values.join(", ")}`);
  }
}

function assertRoute(routeCase) {
  const route = getRoute(routeCase.task);
  const expectedPlan = routeCase.planRequired ?? true;
  if (route.planContract?.required !== expectedPlan) {
    addError(`${routeCase.task}: planContract.required must be ${expectedPlan}`);
  }
  if (!route.productionBar?.noMvpByDefault) {
    addError(`${routeCase.task}: productionBar.noMvpByDefault must be true`);
  }
  if (route.productionBar?.outcomePriority !== "product-user-and-app-specific-business-kpis-first") {
    addError(`${routeCase.task}: productionBar.outcomePriority must prioritize product user and business KPIs`);
  }
  if (!route.qualityGates?.includes("user-business-outcome-link")) {
    addError(`${routeCase.task}: qualityGates must include user-business-outcome-link`);
  }
  if (!route.codeIntelligence?.id || !Array.isArray(route.codeIntelligence?.tools) ||
      (route.codeIntelligence.tools.length === 0 && route.codeIntelligence.id !== "no-code-intelligence")) {
    addError(`${routeCase.task}: code-intelligence workflow is missing`);
  }
  for (const skill of routeCase.skills) {
    assertIncludes(route.skills, skill, `${routeCase.task} skills`);
  }
  for (const skill of routeCase.excludedSkills || []) {
    state.checks += 1;
    if (route.skills.includes(skill)) addError(`${routeCase.task} skills unexpectedly include ${skill}`);
  }
  for (const gate of routeCase.gates) {
    assertIncludes(route.qualityGates || [], gate, `${routeCase.task} gates`);
  }
  if (typeof routeCase.needsFreshDocs === "boolean" && route.needsFreshDocs !== routeCase.needsFreshDocs) {
    addError(`${routeCase.task}: needsFreshDocs must be ${routeCase.needsFreshDocs}`);
  }
}

function assertCodeIntelligenceContract() {
  const catalog = JSON.parse(readText(path.join(process.cwd(), "_reference/code-intelligence-tools.json")));
  const ids = catalog.tools.map((tool) => tool.id);
  const expected = ["ripgrep", "engram", "codebase-memory", "probe", "serena", "ast-grep", "repomix", "dependency-cruiser", "semgrep", "gitleaks"];
  state.checks += 7;
  if (catalog.policy.default_profile !== "full") addError("code-intelligence default profile must install all ten tools");
  if (JSON.stringify(ids) !== JSON.stringify(expected)) addError(`code-intelligence ids drifted: ${ids.join(", ")}`);
  if (ids.includes("context-router")) addError("context-router is process infrastructure and must not count toward the ten tools");
  if (ids.includes("codesight")) addError("disabled Codesight must not remain in the active catalog");
  const mcp = JSON.parse(readText(path.join(process.cwd(), ".mcp.json"))).mcpServers || {};
  if (!mcp.engram || !mcp["codebase-memory-mcp"] || mcp.codesight) addError("MCP defaults must contain Engram and codebase-memory without Codesight");
  const { findBlock } = require("./configure-codex-mcp.js");
  const codexConfig = findBlock(readText(path.join(process.cwd(), ".codex/config.toml")), ".codex/config.toml");
  const codexReference = findBlock(readText(path.join(process.cwd(), "_reference/codex-mcp-config.toml")), "reference");
  if (!codexConfig || !codexReference) addError("Codex MCP managed block must exist in config and reference");
  else if (codexConfig.text.replace(/\r\n/g, "\n") !== codexReference.text.replace(/\r\n/g, "\n")) addError("Codex MCP managed block drifted from its reference");
}

function assertProactiveDelegationContract() {
  const files = [
    "AGENTS.md",
    ".agents/skills/codex-agent-router/SKILL.md",
    ".agents/skills/codex-subagent-orchestration/SKILL.md",
    "docs/AGENT_CONTEXT_SOT.md",
  ];
  const combined = files.map((file) => readText(path.join(process.cwd(), file))).join("\n");
  state.checks += 4;
  if (!combined.includes("without waiting for the user to request subagents")) {
    addError("agent policy must authorize useful proactive delegation without a separate user request");
  }
  if (!/a\s+separate user request is not required/iu.test(combined)) {
    addError("agent SOT must distinguish project authorization from direct user prompting");
  }
  if (/only (?:after|with|on) (?:an? )?explicit user request/iu.test(combined) ||
      /только[^\n]{0,80}явн\w*[^\n]{0,40}просьб/iu.test(combined)) {
    addError("agent policy must not restrict delegation to explicit user requests only");
  }
  if (!combined.includes("Explicit user opt-out")) {
    addError("proactive delegation must preserve explicit user opt-out");
  }
}

function assertContextRouterVersion() {
  const packageJson = JSON.parse(readText(path.join(process.cwd(), "mcp-servers/context-router/package.json")));
  const packageLock = JSON.parse(readText(path.join(process.cwd(), "mcp-servers/context-router/package-lock.json")));
  const serverSource = readText(path.join(process.cwd(), "mcp-servers/context-router/src/index.ts"));
  const serverVersion = serverSource.match(/version:\s*"([0-9.]+)"/)?.[1] || "missing";
  const versions = [packageJson.version, packageLock.version, packageLock.packages?.[""]?.version, serverVersion];
  state.checks += versions.length;
  if (!versions.every((version) => version === versions[0])) {
    addError(`context-router version mismatch: ${versions.join(", ")}`);
  }
}

function main() {
  for (const filePath of REQUIRED_FILES) {
    assertFile(filePath);
  }
  if (isTemplateSourceRepo()) {
    for (const filePath of SOURCE_ONLY_REQUIRED_FILES) {
      assertFile(filePath);
    }
  }
  for (const requirement of REQUIRED_TEXT) {
    assertText(requirement);
  }
  if (isTemplateSourceRepo()) {
    for (const requirement of SOURCE_ONLY_REQUIRED_TEXT) {
      assertText(requirement);
    }
  }
  for (const routeCase of ROUTE_CASES) {
    assertRoute(routeCase);
  }
  assertContextRouterVersion();
  assertCodeIntelligenceContract();
  assertProactiveDelegationContract();

  console.log(`Production standard checks: ${state.checks}`);
  for (const error of state.errors) {
    console.error(`ERROR: ${error}`);
  }
  if (state.errors.length > 0) {
    process.exit(1);
  }
}

main();
