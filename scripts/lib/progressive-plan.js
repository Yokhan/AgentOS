"use strict";

const PRODUCT_EVIDENCE_KINDS = new Set([
  "user-path",
  "gameplay-loop",
  "reader-outcome",
  "operator-task",
  "public-integration",
  "conversion-path",
]);
const JOURNEY_FIELDS = ["entry", "action", "feedback", "outcome", "return"];
const PRODUCT_FIELDS = ["purpose", "user", "final_path", "current_alternative"];
const SLICE_FIELDS = [
  "id", "user_victory", "purpose_mechanism", "positioning", "final_path",
  "falsifier", "replacement", "next_sharpening",
];

function isText(value) {
  return typeof value === "string" && value.trim().length >= 8;
}

function requireText(object, fields, prefix, issues) {
  for (const field of fields) {
    if (!isText(object?.[field])) issues.push(`${prefix}.${field} must be specific text`);
  }
}

function validateKpi(kpi, prefix, issues) {
  requireText(kpi, ["leading", "lagging", "guardrail"], prefix, issues);
}

function validateJourney(journey, prefix, issues) {
  requireText(journey, JOURNEY_FIELDS, prefix, issues);
}

function validateEvidence(evidence, prefix, issues) {
  if (!Array.isArray(evidence) || evidence.length === 0) {
    issues.push(`${prefix} must include product evidence`);
    return;
  }
  const hasProductEvidence = evidence.some((item) =>
    PRODUCT_EVIDENCE_KINDS.has(item?.kind) && isText(item?.ref),
  );
  if (!hasProductEvidence) issues.push(`${prefix} has no product-outcome evidence kind`);
}

function validateTruthBoundary(boundary, prefix, issues) {
  if (!boundary || typeof boundary !== "object") {
    issues.push(`${prefix} is required`);
    return;
  }
  if (boundary.outcome_depends_on_stub !== false) {
    issues.push(`${prefix}.outcome_depends_on_stub must be false`);
  }
  for (const field of ["stubs", "excluded_claims"]) {
    if (!Array.isArray(boundary[field])) issues.push(`${prefix}.${field} must be an array`);
  }
}

function validateSlice(slice, index, issues) {
  const prefix = `slices[${index}]`;
  if (slice?.kind !== "product-slice") issues.push(`${prefix}.kind must be product-slice`);
  requireText(slice, SLICE_FIELDS, prefix, issues);
  if (!Number.isInteger(slice?.readiness) || slice.readiness < 1 || slice.readiness > 100) {
    issues.push(`${prefix}.readiness must be an integer from 1 to 100`);
  }
  validateJourney(slice?.journey, `${prefix}.journey`, issues);
  validateKpi(slice?.kpi, `${prefix}.kpi`, issues);
  validateEvidence(slice?.acceptance_evidence, `${prefix}.acceptance_evidence`, issues);
  validateTruthBoundary(slice?.truth_boundary, `${prefix}.truth_boundary`, issues);
  if (!Array.isArray(slice?.rough_edges)) issues.push(`${prefix}.rough_edges must be an array`);
}

function validateSequence(slices, issues) {
  const ids = new Set();
  let previousReadiness = 0;
  for (const slice of slices) {
    if (ids.has(slice.id)) issues.push(`duplicate slice id: ${slice.id}`);
    ids.add(slice.id);
    if (Number.isInteger(slice.readiness) && slice.readiness <= previousReadiness) {
      issues.push("slice readiness must increase strictly");
    }
    previousReadiness = slice.readiness;
  }
  if (slices.at(-1)?.readiness !== 100) issues.push("final planned slice must reach readiness 100");
}

function validateProgressivePlan(plan) {
  const issues = [];
  if (plan?.schema_version !== 1) issues.push("schema_version must be 1");
  requireText(plan?.product, PRODUCT_FIELDS, "product", issues);
  validateKpi(plan?.product?.kpi, "product.kpi", issues);
  if (!Array.isArray(plan?.enabling_checkpoints)) {
    issues.push("enabling_checkpoints must be an array");
  }
  if (!Array.isArray(plan?.slices) || plan.slices.length === 0) {
    issues.push("slices must contain at least one product-slice");
    return { isValid: false, issues };
  }
  plan.slices.forEach((slice, index) => validateSlice(slice, index, issues));
  validateSequence(plan.slices, issues);
  return { isValid: issues.length === 0, issues };
}

module.exports = { PRODUCT_EVIDENCE_KINDS, validateProgressivePlan };
