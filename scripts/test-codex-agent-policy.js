#!/usr/bin/env node
"use strict";

const assert = require("assert");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const {
  AGENT_POLICY,
  getAgentProfiles,
  getFanoutDecision,
  validateWriteAssignments,
} = require("./codex-agent-policy.js");

const EXPECTED_PROFILES = {
  scout: ["gpt-5.6-luna", "low", "read-only"],
  log_analyst: ["gpt-5.6-luna", "low", "read-only"],
  summarizer: ["gpt-5.6-luna", "low", "read-only"],
  pr_explorer: ["gpt-5.6-terra", "medium", "read-only"],
  docs_researcher: ["gpt-5.6-terra", "medium", "read-only"],
  tester: ["gpt-5.6-terra", "medium", "read-only"],
  implementer: ["gpt-5.6-terra", "high", "workspace-write"],
  reviewer: ["gpt-5.6-sol", "high", "read-only"],
  design_reviewer: ["gpt-5.6-sol", "high", "read-only"],
  product_reviewer: ["gpt-5.6-sol", "high", "read-only"],
  security_reviewer: ["gpt-5.6-sol", "xhigh", "read-only"],
  systems_reviewer: ["gpt-5.6-sol", "xhigh", "read-only"],
};

function getProfileSnapshot() {
  return Object.fromEntries(
    getAgentProfiles().map(({ name, model, effort, sandboxMode }) => [
      name,
      [model, effort, sandboxMode],
    ]),
  );
}

function assertFanoutDecision(decision, expected) {
  assert.strictEqual(decision.status, expected.status);
  assert.strictEqual(decision.reason, expected.reason);
  assert.strictEqual(decision.maxChildren, 3);
  assert.strictEqual(decision.maxDepth, 1);
  assert.strictEqual(decision.maxAutomaticWaves, 1);
  assert.strictEqual(decision.notifyUser, true);
  assert.strictEqual(decision.readOnlyFirst, true);
  assert.strictEqual(decision.requireRuntimeProfileEvidence, true);
}

function testFanoutDecisions() {
  assertFanoutDecision(
    getFanoutDecision({ task: "Что такое GPT-5.6?", risk: "MEDIUM", candidates: ["docs_researcher", "reviewer"] }),
    { status: "skip", reason: "xs-direct-task" },
  );
  assertFanoutDecision(
    getFanoutDecision({ task: "review docs", risk: "MEDIUM", candidates: ["docs_researcher"] }),
    { status: "conditional", reason: "parallel-value-not-yet-proven" },
  );
  assertFanoutDecision(
    getFanoutDecision({ task: "compare docs and implementation", risk: "MEDIUM", candidates: ["docs_researcher", "reviewer"] }),
    { status: "recommended", reason: "parallel-independent-lanes-available" },
  );
  const required = getFanoutDecision({
    task: "update and release the template",
    risk: "HIGH",
    candidates: ["pr_explorer", "systems_reviewer", "tester", "reviewer"],
  });
  assertFanoutDecision(required, {
    status: "required",
    reason: "high-risk-independent-verification",
  });
  assertFanoutDecision(
    getFanoutDecision({
      task: "Read-only template patch review; do not modify files",
      risk: "HIGH",
      modes: ["template"],
      candidates: ["systems_reviewer", "tester"],
    }),
    { status: "conditional", reason: "parallel-value-not-yet-proven" },
  );
  assert.strictEqual(required.candidates.length, 3);
}

function testXsAndReadOnlyBoundaries() {
  for (const task of ["Fix typo", "Check one line", "Update one comment"]) {
    assertFanoutDecision(
      getFanoutDecision({
        task,
        risk: "HIGH",
        candidates: ["scout", "tester", "reviewer"],
        modes: ["review"],
      }),
      { status: "skip", reason: "xs-direct-task" },
    );
  }

  assertFanoutDecision(
    getFanoutDecision({
      task: "Read-only review of AGENTS.md routing policy",
      risk: "HIGH",
      candidates: ["scout", "systems_reviewer", "tester"],
      modes: ["template", "review"],
    }),
    { status: "conditional", reason: "parallel-value-not-yet-proven" },
  );

  assertFanoutDecision(
    getFanoutDecision({
      task: "Comprehensive audit across modules without changes",
      risk: "HIGH",
      candidates: ["scout", "systems_reviewer", "tester"],
      modes: ["template", "review"],
    }),
    { status: "recommended", reason: "parallel-independent-lanes-available" },
  );
}

function testFanoutOptOuts() {
  for (const task of [
    "release without subagents",
    "do not delegate",
    "don't spawn subagents",
    "without any subagents",
    "no fan-out",
    "релиз без субагентов",
    "не запускай субагентов",
    "не делегируй",
  ]) {
    assertFanoutDecision(
      getFanoutDecision({ task, risk: "HIGH", candidates: ["tester", "reviewer"] }),
      { status: "skip", reason: "explicit-user-opt-out" },
    );
  }
}

function testFanoutRiskBoundaries() {
  assertFanoutDecision(
    getFanoutDecision({
      task: "Patch the authentication vulnerability",
      risk: "HIGH",
      candidates: ["security_reviewer", "tester"],
      modes: ["security"],
    }),
    { status: "required", reason: "high-risk-independent-verification" },
  );
  assertFanoutDecision(
    getFanoutDecision({
      task: "How do I fix a typo?",
      risk: "MEDIUM",
      candidates: ["reviewer", "tester"],
    }),
    { status: "skip", reason: "xs-direct-task" },
  );
}

function testWriteScopes() {
  const disjoint = validateWriteAssignments([
    { agent: "worker-a", files: ["src/a.js"] },
    { agent: "worker-b", files: ["src/b.js"] },
  ]);
  assert.strictEqual(disjoint.isValid, true);
}

function testWriteScopeConflicts() {
  const duplicate = validateWriteAssignments([
    { agent: "worker-a", files: ["src/shared.js"] },
    { agent: "worker-b", files: ["src/shared.js"] },
  ]);
  assert.strictEqual(duplicate.isValid, false);
  assert.strictEqual(duplicate.conflicts.length, 1);

  const nested = validateWriteAssignments([
    { agent: "worker-a", files: ["src/feature"] },
    { agent: "worker-b", files: ["src/feature/index.js"] },
  ]);
  assert.strictEqual(nested.isValid, false);

  const missing = validateWriteAssignments([{ agent: "worker-a", files: [] }]);
  assert.deepStrictEqual(missing.missingScopes, ["worker-a"]);
}

function testWriteScopeAliases() {
  const rootScope = validateWriteAssignments([
    { agent: "worker-a", files: ["."] },
    { agent: "worker-b", files: ["src/a.js"] },
  ]);
  assert.strictEqual(rootScope.isValid, false);
  assert.deepStrictEqual(rootScope.invalidScopes, [{ agent: "worker-a", scope: "." }]);

  const caseAlias = validateWriteAssignments([
    { agent: "worker-a", files: ["src/A.js"] },
    { agent: "worker-b", files: ["src/a.js"] },
  ]);
  assert.strictEqual(caseAlias.isValid, false);
}

function testCandidateRanking() {
  const decision = getFanoutDecision({
    task: "Update the OpenAI agent template and review release docs",
    risk: "HIGH",
    modes: ["template", "release", "openai"],
    candidates: [
      "pr_explorer",
      "systems_reviewer",
      "tester",
      "reviewer",
      "docs_researcher",
    ],
  });
  assert.deepStrictEqual(
    decision.candidates.map(({ name }) => name),
    ["docs_researcher", "systems_reviewer", "tester"],
  );
  assert.strictEqual(decision.candidateInventory.length, 5);
}

function createValidatorFixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "codex-agent-policy-"));
  fs.mkdirSync(path.join(root, ".codex"), { recursive: true });
  fs.cpSync(".codex/agents", path.join(root, ".codex/agents"), { recursive: true });
  fs.copyFileSync(".codex/config.toml", path.join(root, ".codex/config.toml"));
  return root;
}

function runValidator(root) {
  return spawnSync(
    process.execPath,
    [path.resolve("scripts/validate-codex-agents.js")],
    { cwd: root, encoding: "utf8" },
  );
}

function withValidatorFixture(mutate, expectedText) {
  const root = createValidatorFixture();
  try {
    mutate(root);
    const result = runValidator(root);
    assert.notStrictEqual(result.status, 0);
    assert.match(`${result.stdout}\n${result.stderr}`, expectedText);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

function testValidatorRejections() {
  withValidatorFixture((root) => {
    const file = path.join(root, ".codex/agents/scout.toml");
    const content = fs.readFileSync(file, "utf8").replace(
      'model = "gpt-5.6-luna"',
      'model = "gpt-5.6-sol"',
    );
    fs.writeFileSync(file, content, "utf8");
  }, /scout must use gpt-5\.6-luna/);

  withValidatorFixture((root) => {
    const file = path.join(root, ".codex/agents/pr-explorer.toml");
    const content = fs.readFileSync(file, "utf8").replace(
      'model_reasoning_effort = "medium"',
      'model_reasoning_effort = "max"',
    );
    fs.writeFileSync(file, content, "utf8");
  }, /must use medium effort/);

  withValidatorFixture((root) => {
    const file = path.join(root, ".codex/config.toml");
    const content = fs.readFileSync(file, "utf8").replace("max_depth = 1", "max_depth = 2");
    fs.writeFileSync(file, content, "utf8");
  }, /expected max_depth = 1/);

  withValidatorFixture((root) => {
    const file = path.join(root, ".codex/config.toml");
    fs.appendFileSync(file, '\nmodel = "gpt-5.6-sol"\n', "utf8");
  }, /contains user\/IDE-owned defaults/);
}

function main() {
  assert.deepStrictEqual(getProfileSnapshot(), EXPECTED_PROFILES);
  assert.strictEqual(AGENT_POLICY.parent.modelSource, "user-or-ide");
  assert.strictEqual(AGENT_POLICY.parent.effortCeiling, "xhigh");
  testFanoutDecisions();
  testXsAndReadOnlyBoundaries();
  testFanoutOptOuts();
  testFanoutRiskBoundaries();
  testWriteScopes();
  testWriteScopeConflicts();
  testWriteScopeAliases();
  testCandidateRanking();
  testValidatorRejections();
  console.log("Codex agent policy tests passed");
}

main();
