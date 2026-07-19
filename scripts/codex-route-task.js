#!/usr/bin/env node
const fs = require("fs");
const path = require("path");
const { getChangeStrategyActivation, getIntentMatch, shouldSuppressRoute } =
  require("./lib/codex-route-intents.js");
const { classifyWritingIntent } = require("./lib/writing-intent.js");
const { getWritingRoutePolicy } = require("./lib/writing-route-policy.js");
const { validateChangeStrategy } = require("./lib/change-strategy-policy.js");
const { evaluateDiscoveryReroute, getDecisionBinding } = require("./lib/codex-discovery-reroute.js");
const { runRouteCli, writeState } = require("./lib/codex-route-cli.js");
const { formatSummary } = require("./lib/codex-route-summary.js");
const { getFanoutDecision } = require("./codex-agent-policy.js");
const { getToolWorkflow } = require("./lib/code-intelligence-policy.js");
const { ROUTES, SHARED_RULES } = require("./codex-route-config.js");
function unique(values) {
  return Array.from(new Set(values.filter(Boolean)));
}
function pathExists(root, relativePath) {
  return fs.existsSync(path.join(root, relativePath));
}
function detectCodeStacks(root) {
  const checks = [
    ["typescript", ["tsconfig.json"]],
    ["javascript", ["package.json"]],
    ["python", ["pyproject.toml", "requirements.txt"]],
    ["go", ["go.mod"]],
    ["rust", ["Cargo.toml"]],
  ];
  return checks.filter(([, files]) => files.some((file) => pathExists(root, file))).map(([stack]) => stack);
}
function hasGlobMatch(root, parts) {
  const [first, second, third] = parts;
  const firstPath = path.join(root, first);
  if (!fs.existsSync(firstPath)) return false;
  return fs.readdirSync(firstPath, { withFileTypes: true }).some((entry) => {
    const secondPath = path.join(firstPath, entry.name, second);
    return entry.isDirectory() && fs.existsSync(path.join(secondPath, third));
  });
}
function detectArtifacts(root) {
  const artifactChecks = [
    {
      name: "agentos",
      present:
        pathExists(root, ".agent-os") ||
        pathExists(root, ".agentos") ||
        pathExists(root, "agentos") ||
        pathExists(root, "AgentOS.md"),
      role: "external orchestrator owns Strategy/Tactic/Plan/Todo/Gate state",
    },
    {
      name: "spec-kit",
      present:
        pathExists(root, ".specify") ||
        hasGlobMatch(root, ["specs", "spec.md", ""]) ||
        hasGlobMatch(root, ["specs", "plan.md", ""]),
      role: "spec/plan/tasks artifacts are the input contract",
    },
    {
      name: "kiro",
      present:
        pathExists(root, ".kiro") ||
        fs.readdirSync(root).includes("requirements.md") ||
        fs.readdirSync(root).includes("design.md"),
      role: "requirements/design/tasks artifacts are the input contract",
    },
    {
      name: "litkit",
      present:
        pathExists(root, "litkit") || pathExists(root, "core/config.yaml"),
      role: "domain pipeline and vocabulary own sequencing",
    },
    {
      name: "project-overlays",
      present:
        pathExists(root, ".agents/skills") &&
        fs.readdirSync(path.join(root, ".agents/skills"), { withFileTypes: true })
          .some((entry) => entry.isDirectory() && entry.name.startsWith("project-")),
      role: "project-specific skills extend the route",
    },
    {
      name: "template-native",
      present:
        pathExists(root, "PROJECT_SPEC.md") ||
        pathExists(root, "tasks/current.md") ||
        pathExists(root, "docs/AGENT_PIPELINES.md"),
      role: "template context and pipelines are available",
    },
  ];
  return artifactChecks.filter((item) => item.present);
}
function getOrchestrator(artifacts) {
  if (artifacts.some((artifact) => artifact.name === "agentos")) {
    return {
      owner: "agentos",
      codexRole: "worker",
      instruction:
        "AgentOS is the orchestrator. Use AgentOS artifacts as the source task graph; Codex only executes the assigned route.",
    };
  }
  if (artifacts.some((artifact) => artifact.name !== "template-native")) {
    return {
      owner: "project-artifacts",
      codexRole: "parent",
      instruction:
        "Existing project artifacts own the task graph. Codex sequences work around them instead of replacing them.",
    };
  }
  return {
    owner: "codex-parent",
    codexRole: "orchestrator",
    instruction:
      "No external orchestrator detected. Parent Codex thread owns sequencing, consolidation, edits, and verification.",
  };
}
function needsStrategicReview(selected, risk, artifacts) {
  const strategicModes = new Set(["strategy", "template", "release", "security", "migration", "product-goal", "lessons"]);
  const externalOrchestrators = new Set(["agentos", "spec-kit", "kiro", "litkit"]);
  return risk === "HIGH" ||
    selected.some((route) => strategicModes.has(route.mode)) ||
    artifacts.some((artifact) => externalOrchestrators.has(artifact.name));
}
function needsProductGoal(selected, risk) {
  const productModes = new Set(["product-goal", "product-ux", "design-system", "marketing", "template", "release", "strategy", "lessons"]);
  return risk !== "LOW" && selected.some((route) => productModes.has(route.mode));
}
function getPlanContract(selected, risk, changeStrategy) {
  const modes = new Set(selected.map((route) => route.mode));
  return {
    required: risk !== "LOW" || modes.has("product-goal") || modes.has("template"),
    language: "match-user-request",
    writeTo: "tasks/current.md",
    goalArtifact: modes.has("product-goal") || modes.has("template") || modes.has("design-system")
      ? "read-or-create tasks/goal.md for M+ product work"
      : "read tasks/goal.md when present",
    approval: changeStrategy.required
      ? "change-strategy-gate-decides-auto-internal-vs-client-tradeoff"
      : risk === "CRITICAL" ? "ask-user-before-state-change" : "state-strategy-before-state-change",
    outcomePriority:
      "name product-user experience and app-specific business KPI before technical work",
  };
}
function getProductionBar(selected) {
  const modes = selected.map((route) => route.mode);
  return {
    default: "final-product-quality",
    noMvpByDefault: true,
    currentStepAllowed: true,
    preserveFinalOutcome: true,
    outcomePriority: "product-user-and-app-specific-business-kpis-first",
    technicalWorkCondition:
      "must-unblock-protect-or-measurably-improve-user-business-outcome",
    businessOutcomes: [
      "revenue",
      "loyalty",
      "retention",
      "conversion",
      "activation",
      "support-load",
      "app-specific-kpi",
    ],
    appliesToModes: modes,
  };
}
function getQualityGates(selected, risk, shouldUseProductGoal = false, changeStrategy = { required: false }) {
  const base = ["success-criteria", "user-business-outcome-link", "verification-evidence", "confidence-and-doubt"];
  const riskGates = risk === "HIGH" || risk === "CRITICAL"
    ? ["rollback-or-plan-b", "route-state-written"]
    : [];
  const productGoalGates = shouldUseProductGoal
    ? ["product-goal-artifact", "quality-bar", "current-step", "language-match"]
    : [];
  const changeStrategyGates = changeStrategy.required
    ? ["project-posture", "protected-contracts", "destination-transition", "objective-evidence-matrix", "approved-change-envelope", "superseded-path-removal"]
    : [];
  const discoveryGates = changeStrategy.discoveryRequired ? ["discovery-evidence-before-edit"] : [];
  return unique([
    ...base,
    ...riskGates,
    ...productGoalGates,
    ...changeStrategyGates,
    ...discoveryGates,
    ...selected.flatMap((route) => route.gates || []),
  ]);
}
const WRITING_NOISE_MODES = new Set([
  "api",
  "design",
  "design-system",
  "feature",
  "marketing",
  "mermaid",
  "openai",
  "release",
  "review",
  "product-ux",
  "writing-communication",
  "writing-informational",
  "writing-literary",
  "technical-writing",
]);
function getRouteByMode(mode) {
  return ROUTES.find((route) => route.mode === mode);
}
function createWritingMatch(mode, policy, rawMatches, isPrimary) {
  const base = getRouteByMode(mode);
  const original = rawMatches.find((match) => match.route.mode === mode);
  return {
    exact: Boolean(original?.exact),
    intent: original?.intent || { isMatch: true, score: 1, threshold: 1 },
    route: {
      ...base,
      pipeline: policy.pipeline,
      risk: policy.risk,
      skills: isPrimary ? policy.skills : [],
      subagents: isPrimary ? policy.subagents : [],
      rules: isPrimary ? unique([...(base.rules || []), ...policy.rules]) : [],
      gates: isPrimary ? unique([...(base.gates || []), ...policy.gates]) : [],
      writingProfiles: isPrimary ? policy.profiles : [],
      writingLanguageProfiles: isPrimary ? policy.languageProfiles : [],
      writingProcessProfiles: isPrimary ? policy.processProfiles : [],
      writingDomainProfiles: isPrimary ? policy.domainProfiles : [],
      writingTechnicalProfiles: isPrimary ? policy.technicalProfiles : [],
      writingEditors: isPrimary ? policy.editors : [],
      writingExternalTools: isPrimary ? policy.externalTools : [],
      writingRejectedProfiles: isPrimary ? policy.rejectedProfiles : [],
    },
  };
}
function applyWritingIntent(task, rawMatches) {
  const intent = classifyWritingIntent(task);
  if (!intent.isWriting) return rawMatches;
  const policy = getWritingRoutePolicy(intent);
  const preserved = rawMatches.filter(
    (match) => !WRITING_NOISE_MODES.has(match.route.mode),
  );
  const modes = [policy.mode, ...policy.extraModes];
  const writingMatches = modes.map((mode, index) =>
    createWritingMatch(mode, policy, rawMatches, index === 0));
  return [...writingMatches, ...preserved];
}
function getMatchedRoutes(task) {
  const rawMatches = ROUTES.map((route) => {
    const exact = route.pattern.test(task);
    const intent = getIntentMatch(route.mode, task);
    return { exact, intent, route };
  }).filter((match) =>
    (match.exact || match.intent.isMatch) &&
    !shouldSuppressRoute(match.route.mode, task),
  );
  return applyWritingIntent(task, rawMatches);
}
function getRoute(task, options = {}) {
  const cwd = options.cwd || process.cwd();
  const writingIntent = classifyWritingIntent(task);
  const writingPolicy = getWritingRoutePolicy(writingIntent);
  const requestActivation = getChangeStrategyActivation(task);
  const matches = getMatchedRoutes(task).filter((match) => !requestActivation.required ||
    requestActivation.recordMode !== "response-only" || match.route.mode !== "bugfix");
  const discovery = evaluateDiscoveryReroute(options.discovery);
  const activation = discovery.required ? {
    ...requestActivation,
    required: true,
    semantic: !requestActivation.exact,
    discoveryRequired: true,
    reasons: unique([...requestActivation.reasons, ...discovery.reasons]),
  } : { ...requestActivation, discoveryRequired: false };
  const strategyDecision = options.changeStrategyDecision
    ? validateChangeStrategy(options.changeStrategyDecision)
    : null;
  const decisionBinding = getDecisionBinding(discovery, options.changeStrategyDecision, activation.required);
  const blockEdits = activation.required &&
    (!strategyDecision || strategyDecision.blocked || !decisionBinding.isBound);
  const changeStrategy = {
    ...activation,
    lifecycle: !activation.required
      ? "not-required"
      : blockEdits ? "pending-decision" : "resolved-resume-base-pipeline",
  };
  const defaultMode = /сделай|сделать|do it|make it/i.test(task)
    ? "feature"
    : changeStrategy.required ? "strategy" : "review";
  const selectedMatches = matches.length > 0
    ? matches
    : [{ exact: false, intent: { isMatch: false, score: 0, threshold: 0 }, route: ROUTES.find((route) => route.mode === defaultMode) }];
  const selected = selectedMatches.map((match) => match.route);
  const semanticMatches = unique([
    ...selectedMatches
      .filter((match) => !match.exact && match.intent.isMatch)
      .map((match) => match.route.mode),
    changeStrategy.semantic ? "change-strategy" : "",
  ]);
  const exactMatches = unique([
    ...selectedMatches
      .filter((match) => match.exact)
      .map((match) => match.route.mode),
    changeStrategy.exact ? "change-strategy" : "",
  ]);
  const artifacts = detectArtifacts(cwd);
  const riskOrder = { LOW: 1, MEDIUM: 2, HIGH: 3, CRITICAL: 4 };
  const routeRisk = selected.reduce(
    (current, route) =>
      riskOrder[route.risk] > riskOrder[current] ? route.risk : current,
    "LOW",
  );
  const risk = changeStrategy.required && riskOrder[routeRisk] < riskOrder.MEDIUM
    ? "MEDIUM"
    : routeRisk;
  const shouldUseStrategicReview = needsStrategicReview(
    selected,
    risk,
    artifacts,
  );
  const shouldUseProductGoal = needsProductGoal(selected, risk);
  const candidateSubagents = unique([
    ...(changeStrategy.required ? ["systems_reviewer", "tester"] : []),
    ...selected.flatMap((route) => route.subagents || []),
  ]);
  const baseFanout = getFanoutDecision({
    task, risk, modes: selected.map((route) => route.mode), candidates: candidateSubagents,
    priorityCandidates: changeStrategy.required ? ["systems_reviewer", "tester"] : [],
  });
  const fanout = changeStrategy.required && baseFanout.status === "conditional"
    ? { ...baseFanout, status: "recommended", reason: "change-strategy-independent-system-and-test-review" }
    : baseFanout;
  const ruleGroups = unique([
    "base",
    ...selected.flatMap((route) => route.rules || []),
    changeStrategy.required ? "changeStrategy" : "",
  ]);
  return {
    task,
    routedAt: new Date().toISOString(),
    modes: unique(selected.map((route) => route.mode)),
    pipeline: selected[0].pipeline,
    codeIntelligence: getToolWorkflow(task, detectCodeStacks(cwd), selected.map((route) => route.mode)),
    risk,
    skills: unique([
      ...selected.flatMap((route) => route.skills || []),
      changeStrategy.required ? "codex-change-strategy" : "",
      shouldUseProductGoal ? "codex-product-goal" : "",
      shouldUseStrategicReview ? "codex-strategic-review" : "",
    ]),
    subagents: fanout.candidates.map(({ name }) => name),
    fanout,
    sharedRules: unique(
      ruleGroups.flatMap((group) => SHARED_RULES[group] || []),
    ),
    planContract: getPlanContract(selected, risk, changeStrategy),
    productionBar: getProductionBar(selected),
    languagePolicy: "plans-audits-status-and-final-reports-match-user-request-language",
    matchPolicy: "exact-patterns-plus-semantic-intent-scoring",
    writingIntent,
    writingPolicy,
    exactMatches,
    semanticMatches,
    changeStrategy,
    strategyDecision,
    decisionBinding,
    discovery,
    blockEdits,
    qualityGates: getQualityGates(
      selected,
      risk,
      shouldUseProductGoal,
      changeStrategy,
    ),
    needsFreshDocs: selected.some((route) => route.needsFreshDocs),
    artifacts,
    orchestrator: getOrchestrator(artifacts),
  };
}
if (require.main === module) {
  try {
    console.log(runRouteCli(process.argv.slice(2), { getRoute, formatSummary }));
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
}
module.exports = {
  ROUTES,
  detectArtifacts,
  formatSummary,
  getRoute,
  writeState,
};
