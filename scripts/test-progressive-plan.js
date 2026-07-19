#!/usr/bin/env node
"use strict";

const assert = require("assert");
const { validateProgressivePlan } = require("./lib/progressive-plan.js");

const DOMAINS = [
  ["software", "user-path"],
  ["game", "gameplay-loop"],
  ["site", "conversion-path"],
  ["book", "reader-outcome"],
  ["internal-tool", "operator-task"],
  ["module", "public-integration"],
];

function makeSlice(id, readiness, evidenceKind) {
  return {
    kind: "product-slice",
    id,
    readiness,
    user_victory: "The named user completes one meaningful job in the real context.",
    journey: {
      entry: "The user enters through the intended production entry point.",
      action: "The user performs the narrow primary action.",
      feedback: "The product reports an understandable real result.",
      outcome: "The useful product outcome is delivered without a mock path.",
      return: "The user can continue, exit, or return without a dead end.",
    },
    purpose_mechanism: "This behavior directly creates the product purpose at current depth.",
    positioning: "It is preferable to the current manual or fragmented alternative.",
    kpi: {
      leading: "Successful completion of the narrow user journey.",
      lagging: "Improved application-specific adoption or retention KPI.",
      guardrail: "No increase in user errors, safety incidents, or support load.",
    },
    final_path: "The slice uses the accepted production route and contracts.",
    acceptance_evidence: [{ kind: evidenceKind, ref: `e2e:${id}-purpose-path` }],
    falsifier: "A target user cannot complete the claimed outcome through the final path.",
    truth_boundary: {
      stubs: [],
      excluded_claims: ["Architecture and debug output are not acceptance evidence."],
      outcome_depends_on_stub: false,
    },
    replacement: "Delete superseded paths in this slice or record a dated removal condition.",
    rough_edges: readiness === 100 ? [] : ["Lower detail remains outside the completed narrow path."],
    next_sharpening: readiness === 100 ? "Production monitoring and feedback." : "Sharpen the next user-visible capability.",
  };
}

function makePlan(domain, evidenceKind) {
  return {
    schema_version: 1,
    product: {
      purpose: `Deliver the real ${domain} outcome to its target user.`,
      user: `The target ${domain} user in the actual usage context.`,
      final_path: "The accepted production entry, action, result, and return path.",
      current_alternative: "The current manual, fragmented, or unavailable workflow.",
      kpi: {
        leading: "Purpose-path completion rate.",
        lagging: "Application-specific adoption, retention, revenue, or efficiency KPI.",
        guardrail: "Safety, quality, accessibility, and support-load protection.",
      },
    },
    enabling_checkpoints: [],
    slices: [makeSlice(`${domain}-one-percent`, 1, evidenceKind), makeSlice(`${domain}-production`, 100, evidenceKind)],
  };
}

function assertInvalid(plan, issueText) {
  const result = validateProgressivePlan(plan);
  assert.strictEqual(result.isValid, false);
  assert(result.issues.some((issue) => issue.includes(issueText)), result.issues);
}

function main() {
  for (const [domain, evidenceKind] of DOMAINS) {
    assert.strictEqual(validateProgressivePlan(makePlan(domain, evidenceKind)).isValid, true);
  }
  const architectureOnly = makePlan("software", "user-path");
  architectureOnly.slices[0].acceptance_evidence = [{ kind: "internal", ref: "unit:compiles" }];
  assertInvalid(architectureOnly, "no product-outcome evidence");
  const fakeStub = makePlan("game", "gameplay-loop");
  fakeStub.slices[0].truth_boundary.outcome_depends_on_stub = true;
  assertInvalid(fakeStub, "outcome_depends_on_stub must be false");
  const checkpointAsSlice = makePlan("book", "reader-outcome");
  checkpointAsSlice.slices[0].kind = "enabling-checkpoint";
  assertInvalid(checkpointAsSlice, "kind must be product-slice");
  console.log("Progressive plan tests passed");
}

main();
