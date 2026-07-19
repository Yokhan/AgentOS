#!/usr/bin/env node
"use strict";

const assert = require("assert");
const { validateChangeStrategy } = require("./lib/change-strategy-policy.js");

const DIMENSIONS = [
  "product_outcome", "business_outcome", "correctness", "maintainability",
  "reliability", "performance", "security", "transition_cost", "reversibility",
];
const CONSTRAINTS = [
  "product_function", "data_safety", "security", "protected_contracts",
  "verification", "rollback_recovery",
];

function makeRef(summary, ref = "test:change-strategy-contract") {
  return { level: "observed", summary, ref };
}

function makeEvidence(overrides = {}) {
  return {
    direction: "same",
    level: "estimated",
    basis: "The option is forecast to preserve this dimension under the same contract.",
    ...overrides,
  };
}

function makeEvidenceMatrix(destination) {
  const matrix = Object.fromEntries(DIMENSIONS.map((name) => [name, makeEvidence()]));
  if (destination === "bounded-replace") {
    matrix.maintainability = makeEvidence({
      direction: "better",
      level: "observed",
      basis: "A common change crosses fewer owned boundaries and removes duplicate state.",
      baseline: "The same rule is currently changed in three modules.",
      result: "The proposed boundary owns the rule in one module.",
      ref: "analysis:change-amplification-map",
      metrics: ["modules-touched", "duplicate-rule-count"],
    });
  }
  return matrix;
}

function makeHardConstraints() {
  return Object.fromEntries(CONSTRAINTS.map((name) => [name, {
    status: "pass",
    evidence: [makeRef(`Observed evidence supports the ${name} constraint.`)],
  }]));
}

function makeCosts(destination) {
  return {
    implementation: destination === "repair" ? "S" : "M",
    transition: destination === "repair" ? "none" : "S",
    verification: "M",
    maintenance: destination === "repair" ? "L" : "S",
    operations: "S",
    change_amplification: destination === "repair" ? "L" : "S",
    basis: "Categorical costs reflect the current dependency and verification map.",
  };
}

function makeCleanup() {
  return {
    owner: "context router module owner",
    removal_condition: "Remove the old path after contract and rollback checks pass.",
    absence_check: "Search obsolete route names and run the final contract suite.",
  };
}

function makeOption(id, destination, transition = "direct-swap") {
  return {
    id,
    destination,
    transition,
    summary: destination === "repair"
      ? "Repair the current implementation after identifying the root cause."
      : "Replace bounded internal implementation behind every named contract.",
    hard_constraints: makeHardConstraints(),
    evidence: makeEvidenceMatrix(destination),
    total_cost: makeCosts(destination),
    ...(destination !== "repair" ? { cleanup: makeCleanup() } : {}),
  };
}

function makeEnvelope(option, contractIds) {
  return {
    outcome: "Stop repeated routing failure while preserving the public result contract.",
    scope: "Internal context router ownership boundary only.",
    protected_contract_ids: contractIds,
    destination: option.destination,
    transitions: [option.transition],
    risk_limit: "No increase beyond the accepted medium internal-change risk.",
    downtime_limit: "No user-visible downtime is permitted for this internal change.",
    cost_limit: "Remain inside the approved bounded implementation task.",
    release_limit: "No release or version publication is included in this envelope.",
    environment_limit: "Apply only to the repository and verified downstream fixture.",
    rollback_requirement: "Revert the bounded commit after a failed contract check.",
    ref: "user-request:implement-approved-change-strategy-gate",
    current: true,
    invalidation_conditions: ["new consumer", "contract change", "risk escalation"],
  };
}

function makeDecision() {
  const repair = makeOption("repair-current", "repair");
  const replace = makeOption("replace-bounded", "bounded-replace");
  const contractIds = ["public route result"];
  return {
    schema_version: 2,
    trigger: {
      kind: "repeated-failure",
      acceptance_id: "route-contract-same-output",
      evidence_ref: "test:route-contract-repeated-failure",
      summary: "Two repairs failed against the same route result acceptance criterion.",
      root_cause: "The old ownership boundary duplicates one decision across modules.",
      attempts: [
        {
          hypothesis: "The exact route pattern alone caused the repeated mismatch.",
          change: "Adjusted the exact pattern without changing semantic ownership.",
          before: "The representative request selected the wrong route.",
          after: "A semantic variant still selected the wrong route.",
          result: "failed",
        },
        {
          hypothesis: "Adding another fallback would preserve the expected output.",
          change: "Added a compatibility fallback around the duplicate decision.",
          before: "One variant failed the route contract.",
          after: "The fallback fixed one variant but another recurrence remained.",
          result: "recurred",
        },
      ],
    },
    project: {
      posture: "evolving",
      posture_evidence: [makeRef("The router has downstream consumers but no live user data.")],
      protected_contracts: [{
        id: contractIds[0],
        kind: "public-api",
        owner: "context router module",
        sot: "scripts/codex-route-task.js exported route result",
        impact: "preserved",
      }],
    },
    product: {
      user_victory: "A downstream agent selects the correct workflow without repeated patch loops.",
      final_path: "User request through semantic router to one stable route contract.",
      falsifier: "A representative semantic request still enters the wrong workflow.",
    },
    change: {
      summary: "Replace duplicate internal routing ownership behind a stable contract.",
      scope: "Internal context router ownership boundary only.",
      impacts: [],
      reversible: true,
    },
    options: [repair, replace],
    recommendation: {
      option_id: replace.id,
      advantages: ["maintainability"],
      reason: "Bounded replacement removes duplicate ownership with observed change-amplification evidence.",
      rejected_alternative: "Repair retains the root cause and the same repeated-change surface.",
    },
    approval: {
      status: "covered",
      envelope: makeEnvelope(replace, contractIds),
    },
  };
}

function assertIssue(decision, text) {
  const result = validateChangeStrategy(decision);
  assert.strictEqual(result.isValid, false);
  assert(result.issues.some((issue) => issue.includes(text)), result.issues);
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function addPublicApiProfiles(decision) {
  decision.options.forEach((option) => {
    option.compatibility = {
      public_api: {
        contract_diff: "Machine-readable contract comparison is attached to the decision.",
        known_consumers: ["context-router MCP adapter"],
        unknown_consumers: ["third-party downstream overlays"],
        consumer_tests: "Run route-result contract fixtures for known consumers.",
        versioning: "Keep the public result schema unchanged in the bounded change.",
        deprecation: "No field is deprecated by this internal replacement.",
        semantic_checks: "Auth, error, and routing semantics remain contract-tested.",
      },
    };
  });
}

function main() {
  const valid = makeDecision();
  assert.strictEqual(validateChangeStrategy(valid).blocked, false);

  const architectureTrigger = clone(valid);
  architectureTrigger.trigger = {
    kind: "architecture-mismatch",
    summary: "Initial reading found implementation outside the accepted final architecture.",
    root_cause: "Two modules own the same route decision despite one accepted owner.",
    evidence_ref: "src/game/legacy-state.ts and plan final-path section",
  };
  assert.strictEqual(validateChangeStrategy(architectureTrigger).blocked, false);

  const oneAttempt = clone(valid);
  oneAttempt.trigger.attempts.pop();
  assertIssue(oneAttempt, "at least two attempt records");

  const selfCertified = clone(valid);
  selfCertified.options[1].hard_constraints.data_safety = { status: "pass", evidence: [] };
  assertIssue(selfCertified, "evidence is required for pass");

  const publicChange = clone(valid);
  publicChange.project.protected_contracts[0].impact = "changed";
  publicChange.change.impacts = ["public-contract"];
  publicChange.options[1].transition = "versioned-coexistence";
  publicChange.options[1].cleanup = makeCleanup();
  addPublicApiProfiles(publicChange);
  publicChange.approval = { status: "pending" };
  const pending = validateChangeStrategy(publicChange);
  assert.strictEqual(pending.isValid, true);
  assert.strictEqual(pending.blocked, true);
  assert.strictEqual(pending.approvalRequired, true);

  const staleCover = clone(publicChange);
  staleCover.approval = {
    status: "covered",
    envelope: makeEnvelope(staleCover.options[1], ["public route result"]),
  };
  assertIssue(staleCover, "material change cannot use a covered implicit envelope");

  const approved = clone(publicChange);
  approved.approval = {
    status: "approved",
    envelope: makeEnvelope(approved.options[1], ["public route result"]),
  };
  assert.strictEqual(validateChangeStrategy(approved).blocked, false);

  const undeclaredContractImpact = clone(valid);
  undeclaredContractImpact.project.protected_contracts[0].impact = "changed";
  undeclaredContractImpact.approval = {
    status: "approved",
    envelope: makeEnvelope(
      undeclaredContractImpact.options[1],
      ["public route result"],
    ),
  };
  assertIssue(undeclaredContractImpact, "must include public-contract");
  assertIssue(undeclaredContractImpact, "compatibility.public_api.contract_diff");

  const freeTextContractKind = clone(valid);
  freeTextContractKind.project.protected_contracts[0].kind = "public CLI command surface";
  freeTextContractKind.project.protected_contracts[0].impact = "changed";
  assertIssue(freeTextContractKind, "protected_contracts[0].kind is invalid");

  const duplicateChoice = clone(valid);
  duplicateChoice.options[0] = clone(duplicateChoice.options[1]);
  duplicateChoice.options[0].id = "same-choice-different-id";
  assertIssue(duplicateChoice, "unique destination x transition choices");

  const fakeSpeed = clone(valid);
  fakeSpeed.options[1].evidence.performance = makeEvidence({
    direction: "better",
    level: "estimated",
    basis: "The replacement looks faster from code inspection alone.",
  });
  assertIssue(fakeSpeed, "only with measured evidence");

  const measuredSpeed = clone(valid);
  measuredSpeed.options[1].evidence.performance = makeEvidence({
    direction: "better",
    level: "measured",
    basis: "The same routing workload meets the declared latency budget.",
    baseline: "p95 latency was 42 ms over the fixed route corpus.",
    result: "p95 latency is 28 ms over the same route corpus.",
    ref: "benchmark:route-policy-2026-07-18",
    workload: "Ten thousand fixed route decisions after warm-up.",
    environment: "Pinned Node version and dedicated Windows runner.",
    threshold: "No more than 35 ms p95 latency on the fixed workload.",
  });
  assert.strictEqual(validateChangeStrategy(measuredSpeed).isValid, true);

  const locOnly = clone(valid);
  locOnly.options[1].evidence.maintainability.metrics = ["LOC"];
  assertIssue(locOnly, "cannot rely only on line count");

  const missingDataProfile = clone(valid);
  missingDataProfile.change.impacts = ["user-data"];
  missingDataProfile.approval = { status: "pending" };
  assertIssue(missingDataProfile, "compatibility.data.current_schema");

  const incomplete = clone(valid);
  delete incomplete.options[0].evidence.maintainability;
  assertIssue(incomplete, "options[0].evidence.maintainability.direction is invalid");

  const unknownImpact = clone(valid);
  unknownImpact.change.impacts = ["public-api-ish"];
  assertIssue(unknownImpact, "change.impacts contains an invalid value");

  const missingReversibility = clone(valid);
  delete missingReversibility.change.reversible;
  assertIssue(missingReversibility, "change.reversible must be boolean");

  const worseBusinessOutcome = clone(valid);
  worseBusinessOutcome.options[1].evidence.business_outcome.direction = "worse";
  worseBusinessOutcome.approval = { status: "pending" };
  const businessDecision = validateChangeStrategy(worseBusinessOutcome);
  assert.strictEqual(businessDecision.isValid, true);
  assert.strictEqual(businessDecision.approvalRequired, true);
  assert(businessDecision.approvalReasons.includes("evidence-regression:business-outcome"));

  console.log("Change strategy tests passed");
}

if (require.main === module) main();

function makeArchitectureDecision() {
  const decision = makeDecision();
  return {
    ...decision,
    trigger: {
      kind: "architecture-mismatch",
      summary: "Initial reading found implementation outside the accepted final architecture.",
      root_cause: "Two modules own the same route decision despite one accepted owner.",
      evidence_ref: "src/game/legacy-state.ts and plan final-path section",
    },
  };
}

module.exports = { makeArchitectureDecision, makeDecision };
