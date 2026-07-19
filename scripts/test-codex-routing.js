#!/usr/bin/env node
const assert = require("assert");
const fs = require("fs");
const os = require("os");
const path = require("path");

const { formatSummary, getRoute } = require("./codex-route-task.js");
const { runRouteCli } = require("./lib/codex-route-cli.js");
const { evaluateDiscoveryReroute, getDecisionBinding } =
  require("./lib/codex-discovery-reroute.js");
const { AGENT_POLICY, getAgentProfiles } = require("./codex-agent-policy.js");
const { runRouteCasesA } = require("./codex-routing-cases-a.js");
const { runRouteCasesB } = require("./codex-routing-cases-b.js");
const { makeArchitectureDecision, makeDecision } = require("./test-change-strategy.js");

function assertIncludes(values, expected, message) {
  assert(
    values.includes(expected),
    `${message}: expected ${expected}, got ${values.join(", ")}`,
  );
}

function testRoute(task, expectations) {
  const route = getRoute(task, expectations.options || {});
  if (expectations.exactModes) {
    assert.deepStrictEqual(
      new Set(route.modes),
      new Set(expectations.exactModes),
      `${task} exact modes`,
    );
  }
  if (expectations.exactSubagents) {
    assert.deepStrictEqual(
      route.subagents,
      expectations.exactSubagents,
      `${task} exact subagents`,
    );
  }
  for (const mode of expectations.modes || []) {
    assertIncludes(route.modes, mode, `${task} modes`);
  }
  for (const mode of expectations.notModes || []) {
    assert(
      !route.modes.includes(mode),
      `${task} modes: expected no ${mode}, got ${route.modes.join(", ")}`,
    );
  }
  for (const skill of expectations.skills || []) {
    assertIncludes(route.skills, skill, `${task} skills`);
  }
  for (const skill of expectations.notSkills || []) {
    assert(
      !route.skills.includes(skill),
      `${task} skills: expected no ${skill}, got ${route.skills.join(", ")}`,
    );
  }
  for (const rule of expectations.sharedRules || []) {
    assertIncludes(route.sharedRules, rule, `${task} shared rules`);
  }
  for (const subagent of expectations.subagents || []) {
    assertIncludes(route.subagents, subagent, `${task} subagents`);
  }
  for (const subagent of expectations.notSubagents || []) {
    assert(
      !route.subagents.includes(subagent),
      `${task} subagents: expected no ${subagent}, got ${route.subagents.join(", ")}`,
    );
  }
  for (const gate of expectations.qualityGates || []) {
    assertIncludes(route.qualityGates || [], gate, `${task} quality gates`);
  }
  if (expectations.risk) {
    assert.strictEqual(route.risk, expectations.risk, `${task} risk`);
  }
  if (expectations.pipeline) {
    assert.strictEqual(route.pipeline, expectations.pipeline, `${task} pipeline`);
  }
  if (expectations.orchestrator) {
    assert.strictEqual(
      route.orchestrator.owner,
      expectations.orchestrator,
      `${task} orchestrator`,
    );
  }
  if (typeof expectations.needsFreshDocs === "boolean") {
    assert.strictEqual(
      route.needsFreshDocs,
      expectations.needsFreshDocs,
      `${task} fresh docs`,
    );
  }
  if (typeof expectations.planRequired === "boolean") {
    assert.strictEqual(
      route.planContract.required,
      expectations.planRequired,
      `${task} plan required`,
    );
  }
  if (expectations.fanoutStatus) {
    assert.strictEqual(
      route.fanout.status,
      expectations.fanoutStatus,
      `${task} fanout status`,
    );
  }
  if (typeof expectations.changeStrategyRequired === "boolean") {
    assert.strictEqual(
      route.changeStrategy.required,
      expectations.changeStrategyRequired,
      `${task} change strategy activation`,
    );
  }
  if (expectations.changeStrategyRecordMode) {
    assert.strictEqual(
      route.changeStrategy.recordMode,
      expectations.changeStrategyRecordMode,
      `${task} change strategy record mode`,
    );
  }
  if (typeof expectations.blockEdits === "boolean") {
    assert.strictEqual(route.blockEdits, expectations.blockEdits, `${task} block edits`);
  }
  if (expectations.discoveryKind) {
    assert.strictEqual(route.discovery.kind, expectations.discoveryKind, `${task} discovery kind`);
  }
  for (const mode of expectations.semanticMatches || []) {
    assertIncludes(route.semanticMatches || [], mode, `${task} semantic matches`);
  }
  for (const mode of expectations.exactMatches || []) {
    assertIncludes(route.exactMatches || [], mode, `${task} exact matches`);
  }
}

function withTempProject(setup, callback) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "codex-route-"));
  try {
    setup(root);
    callback(root);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
}

function main() {
  assert.strictEqual(AGENT_POLICY.parent.modelSource, "user-or-ide");
  assert.strictEqual(AGENT_POLICY.parent.effortCeiling, "xhigh");
  assert.deepStrictEqual(
    new Set(getAgentProfiles().map(({ model }) => model)),
    new Set(["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna"]),
  );
  const symbolRoute = getRoute("rename the authentication symbol");
  assert.deepStrictEqual(symbolRoute.codeIntelligence.tools, ["codebase-memory", "serena", "ripgrep"]);
  assert(formatSummary(symbolRoute).includes("CODE_INTELLIGENCE: symbol-refactor | codebase-memory -> serena -> ripgrep"));
  const securityRoute = getRoute("security release audit for leaked secrets");
  assert.deepStrictEqual(securityRoute.codeIntelligence.tools, ["codebase-memory", "semgrep", "gitleaks", "ripgrep"]);

  const russianWriting = getRoute("Напиши на русском руководство по интеграции API");
  assert.deepStrictEqual(russianWriting.writingPolicy.languageProfiles, ["russian-infostyle-core", "ilyakhov-russian-voice-decisions"]);
  assert(russianWriting.writingPolicy.domainProfiles.includes("russian-explanation-and-persuasion"));
  assert(russianWriting.writingPolicy.domainProfiles.includes("reader-task-architecture"));
  assert(russianWriting.writingPolicy.technicalProfiles.includes("technical-developer-conventions"));
  assert.deepStrictEqual(russianWriting.writingPolicy.externalTools, [{ id: "glavred-api", access: "not-configured", execution: "not-run", paid: true }]);
  assert(russianWriting.writingPolicy.gates.includes("external-tool-unavailable"));

  const russianLetter = getRoute("Напиши на русском деловое письмо клиенту");
  assert(russianLetter.writingPolicy.languageProfiles.includes("russian-business-correspondence-language"));
  assert(russianLetter.writingPolicy.processProfiles.includes("russian-business-correspondence-process"));
  assert(russianLetter.writingPolicy.domainProfiles.includes("russian-business-correspondence"));

  for (const task of ["Проверь в Главреде", "Дай оценку Главреда", "Отредактируй по Главреду", "Проверь этот текст в Главреде"]) {
    const glavredRoute = getRoute(task);
    assert(glavredRoute.modes.includes("writing-informational"), `${task}: ${glavredRoute.modes.join(", ")}`);
    assert(!glavredRoute.modes.includes("writing-literary"), `${task}: unexpected literary route`);
    assert.deepStrictEqual(glavredRoute.writingPolicy.externalTools, [{ id: "glavred-api", access: "not-configured", execution: "not-run", paid: true }]);
  }

  const mixedWriting = getRoute("Напиши руководство на русском и английском по API");
  assert.strictEqual(mixedWriting.writingPolicy.targetLanguage, "mixed");
  assert.deepStrictEqual(mixedWriting.writingPolicy.languageProfiles, []);
  assert(mixedWriting.writingPolicy.gates.includes("per-section-language-resolution"));

  testRoute("план итераций по прогрессивному джипегу, где каждый срез решает цель продукта", {
    modes: ["progressive-planning"],
    notModes: ["design-system"],
    skills: ["codex-progressive-jpeg-planner", "codex-product-goal", "codex-decompose"],
    qualityGates: ["product-purpose", "end-to-end-user-victory", "anti-falsification"],
    planRequired: true,
  });

  testRoute("Check subagent token usage and report whether the configured models work", {
    notModes: ["design-system"],
    notSubagents: ["product_reviewer"],
    fanoutStatus: "conditional",
  });

  runRouteCasesA(testRoute);
  runRouteCasesB(testRoute);

  const discoveryFixture = path.join(
    __dirname,
    "..",
    "tests",
    "fixtures",
    "change-strategy",
    "discovery-architecture-mismatch.json",
  );
  const discoveryOutput = runRouteCli(
    ["fix the display bug", "--discovery-file", discoveryFixture],
    { formatSummary, getRoute },
  );
  const discoveryRoute = JSON.parse(discoveryOutput);
  assert.strictEqual(discoveryRoute.blockEdits, true);
  assert(discoveryRoute.modes.includes("bugfix"));
  assert(discoveryRoute.skills.includes("codex-change-strategy"));

  const resolvedRoute = getRoute("fix the display bug", {
    discovery: JSON.parse(fs.readFileSync(discoveryFixture, "utf8")),
    changeStrategyDecision: makeArchitectureDecision(),
  });
  assert.strictEqual(resolvedRoute.blockEdits, false);
  assert.strictEqual(
    resolvedRoute.changeStrategy.lifecycle,
    "resolved-resume-base-pipeline",
  );
  assert(resolvedRoute.modes.includes("bugfix"));
  assert.strictEqual(resolvedRoute.pipeline, discoveryRoute.pipeline);
  assert.strictEqual(
    new Set(resolvedRoute.skills).size,
    resolvedRoute.skills.length,
  );
  const decisionRoot = fs.mkdtempSync(path.join(os.tmpdir(), "route-decision-"));
  try {
    const decisionFile = path.join(decisionRoot, "decision.json");
    fs.writeFileSync(decisionFile, JSON.stringify(makeArchitectureDecision()));
    const resolvedOutput = runRouteCli(
      [
        "fix the display bug",
        "--discovery-file", discoveryFixture,
        "--decision-file", decisionFile,
      ],
      { formatSummary, getRoute },
    );
    assert.strictEqual(JSON.parse(resolvedOutput).blockEdits, false);
  } finally {
    fs.rmSync(decisionRoot, { recursive: true, force: true });
  }
  const unrelatedDecision = getRoute("fix the display bug", {
    discovery: JSON.parse(fs.readFileSync(discoveryFixture, "utf8")),
    changeStrategyDecision: makeDecision(),
  });
  assert.strictEqual(unrelatedDecision.blockEdits, true);
  assert.strictEqual(unrelatedDecision.decisionBinding.isBound, false);

  const bindingCases = [
    ["repeated-failure", "repeated-failure", { acceptance_id: "fixture-acceptance" }],
    ["architecture-mismatch", "architecture-mismatch"],
    ["ownership-conflict", "ownership-conflict"],
    ["sot-conflict", "sot-conflict"],
    ["duplicate-state", "duplicate-state"],
    ["duplicate-implementation", "duplicate-implementation"],
    ["obsolete-final-path", "obsolete-final-path"],
    ["compatibility-only-layer", "compatibility-shim"],
    ["protected-boundary-unknown", "manual-review"],
    ["stale-path-test", "stale-path-test"],
    ["sunk-cost", "sunk-cost"],
    ["planned-breaking-change", "planned-breaking-change"],
    ["manual-review", "manual-review"],
  ];
  for (const [discoveryKind, triggerKind, extra = {}] of bindingCases) {
    const discovery = evaluateDiscoveryReroute({
      phase: "reading",
      kind: discoveryKind,
      architecture_fit: discoveryKind === "protected-boundary-unknown"
        ? "unknown" : "mismatch",
      summary: `Reading found a qualifying ${discoveryKind} condition.`,
      evidence_ref: `fixture:${discoveryKind}`,
      owner: "Affected subsystem owner",
      sot: "Accepted architecture contract",
      protected_boundaries: ["public result contract"],
      ...extra,
    });
    const binding = getDecisionBinding(discovery, {
      trigger: {
        kind: triggerKind,
        evidence_ref: `fixture:${discoveryKind}`,
        ...extra,
      },
    }, true);
    assert.strictEqual(binding.isBound, true, discoveryKind);
  }

  const repeatedDiscovery = evaluateDiscoveryReroute({
    phase: "implementation",
    kind: "repeated-failure",
    architecture_fit: "mismatch",
    summary: "Two interventions failed against the same acceptance criterion.",
    evidence_ref: "fixture:repeated-failure",
    acceptance_id: "fixture-acceptance",
    owner: "Affected subsystem owner",
    sot: "Accepted architecture contract",
    protected_boundaries: ["public result contract"],
  });
  const wrongEvidenceDecision = makeDecision();
  wrongEvidenceDecision.trigger.acceptance_id = "fixture-acceptance";
  const wrongEvidenceBinding = getDecisionBinding(
    repeatedDiscovery, wrongEvidenceDecision, true);
  assert.strictEqual(wrongEvidenceBinding.isBound, false);
  assert(wrongEvidenceBinding.issues.some((issue) => issue.includes("evidence_ref")));

  const pendingDecision = makeArchitectureDecision();
  pendingDecision.approval = { status: "pending" };
  const pendingRoute = getRoute("fix the display bug", {
    discovery: JSON.parse(fs.readFileSync(discoveryFixture, "utf8")),
    changeStrategyDecision: pendingDecision,
  });
  assert.strictEqual(pendingRoute.blockEdits, true);

  const malformedRoot = fs.mkdtempSync(path.join(os.tmpdir(), "route-malformed-"));
  try {
    const malformedFile = path.join(malformedRoot, "malformed.json");
    fs.writeFileSync(malformedFile, "{not-json\n");
    assert.throws(() => runRouteCli(
      ["fix the display bug", "--discovery-file", malformedFile, "--write-state"],
      { formatSummary, getRoute },
    ));
    assert.throws(() => runRouteCli(
      ["fix the display bug", "--decision-file", path.join(malformedRoot, "missing.json")],
      { formatSummary, getRoute },
    ));
    assert.throws(
      () => runRouteCli(["fix the display bug", "--discovery-file"], { formatSummary, getRoute }),
      /requires a file path/,
    );
    assert.throws(
      () => runRouteCli(["fix the display bug", "--decision-file"], { formatSummary, getRoute }),
      /requires a file path/,
    );
    assert.throws(
      () => runRouteCli(["fix the display bug", "--unknown"], { formatSummary, getRoute }),
      /Unknown option/,
    );
  } finally {
    fs.rmSync(malformedRoot, { recursive: true, force: true });
  }

  const incompleteDiscovery = getRoute("fix the display bug", {
    discovery: { kind: "architecture-mismatch" },
  });
  assert.strictEqual(incompleteDiscovery.blockEdits, true);
  assert(incompleteDiscovery.discovery.issues.length >= 5);

  const architecturePhrase = getRoute(
    "Architecture mismatch discovered while reading before the first patch.",
    { changeStrategyDecision: makeArchitectureDecision() },
  );
  assert.strictEqual(architecturePhrase.changeStrategy.required, true);
  assert.strictEqual(architecturePhrase.blockEdits, true);
  assert.strictEqual(architecturePhrase.decisionBinding.isBound, false);
  const fallbackPhrase = getRoute(
    "The second failed repair hit the same acceptance criterion.",
    { changeStrategyDecision: makeDecision() },
  );
  assert.strictEqual(fallbackPhrase.changeStrategy.required, true);
  assert.strictEqual(fallbackPhrase.blockEdits, true);
  assert.strictEqual(fallbackPhrase.decisionBinding.isBound, false);

  withTempProject(
    (root) => fs.writeFileSync(path.join(root, "DESIGN.md"), "# Product design\n"),
    (root) => {
      const route = getRoute("fix the display bug", { cwd: root });
      assert(!route.artifacts.some(({ name }) => name === "kiro"));
    },
  );

  withTempProject(
    (root) => {
      fs.mkdirSync(path.join(root, ".agent-os"), { recursive: true });
      fs.mkdirSync(path.join(root, "tasks"), { recursive: true });
      fs.writeFileSync(path.join(root, "tasks", "current.md"), "fixture\n");
    },
    (root) => {
      testRoute("implement feature from AgentOS plan", {
        modes: ["feature"],
        skills: ["codex-feature-workflow"],
        orchestrator: "agentos",
        options: { cwd: root },
      });
    },
  );

  console.log("Codex routing smoke passed");
}

main();
