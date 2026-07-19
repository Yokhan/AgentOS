"use strict";

const POSTURES = new Set(["greenfield", "evolving", "production", "unknown"]);
const DESTINATIONS = new Set(["repair", "bounded-replace", "retire-remove"]);
const TRANSITIONS = new Set([
  "direct-swap", "staged-swap", "versioned-coexistence", "expand-migrate-contract",
]);
const TRIGGERS = new Set([
  "repeated-failure", "compatibility-shim", "architecture-drift",
  "architecture-mismatch", "ownership-conflict", "sot-conflict",
  "duplicate-state", "duplicate-implementation", "obsolete-final-path",
  "stale-path-test", "sunk-cost", "planned-breaking-change", "manual-review",
]);
const DIRECTIONS = new Set(["better", "same", "worse", "unknown", "not-applicable"]);
const LEVELS = new Set(["measured", "observed", "authoritative", "estimated", "unknown"]);
const HARD_LEVELS = new Set(["measured", "observed", "authoritative"]);
const HARD_STATUSES = new Set(["pass", "fail", "unknown"]);
const COSTS = new Set(["none", "S", "M", "L", "unknown"]);
const REQUIRED_DIMENSIONS = [
  "product_outcome", "business_outcome", "correctness", "maintainability",
  "reliability", "performance", "security", "transition_cost", "reversibility",
];
const CRITICAL_DIMENSIONS = new Set([
  "product_outcome", "correctness", "reliability", "security",
]);
const HARD_CONSTRAINTS = [
  "product_function", "data_safety", "security", "protected_contracts",
  "verification", "rollback_recovery",
];
const COST_FIELDS = [
  "implementation", "transition", "verification", "maintenance", "operations",
  "change_amplification",
];
const MATERIAL_IMPACTS = new Set([
  "product-behavior", "business-kpi", "user-data", "public-contract", "security-boundary",
  "ownership", "release", "downtime", "scope", "cost", "timeline",
  "irreversible-state",
]);
const CHANGE_IMPACTS = new Set([...MATERIAL_IMPACTS, "external-dependency"]);
const CONTRACT_IMPACTS = {
  "public-api": "public-contract", "public-cli": "public-contract",
  "public-event": "public-contract", "public-config": "public-contract",
  "public-file-format": "public-contract", "user-data": "user-data",
  "security-boundary": "security-boundary",
  "external-dependency": "external-dependency",
};
const CONTRACT_KINDS = new Set(["internal-behavior", ...Object.keys(CONTRACT_IMPACTS)]);

function isText(value) {
  return typeof value === "string" && value.trim().length >= 8;
}

function requireText(object, fields, prefix, issues) {
  for (const field of fields) {
    if (!isText(object?.[field])) issues.push(`${prefix}.${field} must be specific text`);
  }
}

function validateEvidenceRef(record, prefix, issues, levels = LEVELS) {
  if (!levels.has(record?.level)) issues.push(`${prefix}.level is invalid`);
  requireText(record, ["summary", "ref"], prefix, issues);
}

function validateRepeatedAttempts(trigger, issues) {
  if (trigger?.kind !== "repeated-failure") {
    if (!isText(trigger?.evidence_ref)) {
      issues.push("trigger.evidence_ref is required for discovery or planned-change triggers");
    }
    return;
  }
  if (!isText(trigger?.acceptance_id)) issues.push("trigger.acceptance_id is required");
  if (!isText(trigger?.evidence_ref)) issues.push("trigger.evidence_ref is required");
  if (!Array.isArray(trigger?.attempts) || trigger.attempts.length < 2) {
    issues.push("repeated-failure requires at least two attempt records");
    return;
  }
  trigger.attempts.forEach((attempt, index) => {
    const prefix = `trigger.attempts[${index}]`;
    requireText(attempt, ["hypothesis", "change", "before", "after"], prefix, issues);
    if (!new Set(["failed", "recurred"]).has(attempt?.result)) {
      issues.push(`${prefix}.result must be failed or recurred`);
    }
  });
}

function validatePosture(project, issues) {
  if (!POSTURES.has(project?.posture)) issues.push("project.posture is invalid");
  if (project?.posture === "unknown") return;
  if (!Array.isArray(project?.posture_evidence) || project.posture_evidence.length === 0) {
    issues.push("project.posture_evidence is required");
    return;
  }
  project.posture_evidence.forEach((record, index) =>
    validateEvidenceRef(record, `project.posture_evidence[${index}]`, issues, HARD_LEVELS));
}

function validateContracts(contracts, issues) {
  if (!Array.isArray(contracts)) {
    issues.push("project.protected_contracts must be an array");
    return;
  }
  contracts.forEach((contract, index) => {
    const prefix = `project.protected_contracts[${index}]`;
    requireText(contract, ["id", "kind", "owner", "sot"], prefix, issues);
    if (!CONTRACT_KINDS.has(contract?.kind)) issues.push(`${prefix}.kind is invalid`);
    if (!new Set(["preserved", "changed", "removed", "unknown"]).has(contract?.impact)) {
      issues.push(`${prefix}.impact is invalid`);
    }
  });
}

function getContractImpacts(contracts) {
  const impacts = [];
  for (const contract of contracts || []) {
    if (contract?.impact === "preserved") continue;
    const impact = CONTRACT_IMPACTS[contract?.kind];
    if (impact) impacts.push({ impact, id: contract.id });
  }
  return impacts;
}

function validateMeasuredEvidence(evidence, prefix, issues) {
  requireText(evidence, ["baseline", "result", "ref"], prefix, issues);
  if (prefix.endsWith("performance")) {
    requireText(evidence, ["workload", "environment", "threshold"], prefix, issues);
  }
}

function validateMaintainabilityEvidence(evidence, prefix, issues) {
  if (evidence?.direction !== "better") return;
  if (!new Set(["measured", "observed", "authoritative"]).has(evidence.level)) {
    issues.push(`${prefix} needs non-estimated evidence for a better claim`);
  }
  if (!Array.isArray(evidence.metrics) || evidence.metrics.length === 0) {
    issues.push(`${prefix}.metrics must support a maintainability improvement`);
    return;
  }
  const useful = evidence.metrics.some((metric) =>
    !/^\s*(?:loc|lines?|line[- ]count)\s*$/i.test(String(metric)));
  if (!useful) issues.push(`${prefix}.metrics cannot rely only on line count`);
}

function validateEvidence(evidence, dimension, prefix, issues) {
  if (!DIRECTIONS.has(evidence?.direction)) issues.push(`${prefix}.direction is invalid`);
  if (!LEVELS.has(evidence?.level)) issues.push(`${prefix}.level is invalid`);
  if (!isText(evidence?.basis)) issues.push(`${prefix}.basis must explain the comparison`);
  if (["better", "worse"].includes(evidence?.direction) && evidence?.level === "unknown") {
    issues.push(`${prefix} cannot make a directional claim with unknown evidence`);
  }
  if (["measured", "observed", "authoritative"].includes(evidence?.level)) {
    validateMeasuredEvidence(evidence, prefix, issues);
  }
  if (dimension === "performance" && evidence?.direction === "better" && evidence?.level !== "measured") {
    issues.push(`${prefix} can claim better performance only with measured evidence`);
  }
  if (dimension === "maintainability") validateMaintainabilityEvidence(evidence, prefix, issues);
}

function validateHardConstraint(constraint, prefix, issues) {
  if (!HARD_STATUSES.has(constraint?.status)) issues.push(`${prefix}.status is invalid`);
  if (constraint?.status === "pass") {
    if (!Array.isArray(constraint?.evidence) || constraint.evidence.length === 0) {
      issues.push(`${prefix}.evidence is required for pass`);
      return;
    }
    constraint.evidence.forEach((record, index) =>
      validateEvidenceRef(record, `${prefix}.evidence[${index}]`, issues, HARD_LEVELS));
  } else if (!isText(constraint?.reason)) {
    issues.push(`${prefix}.reason is required for ${constraint?.status}`);
  }
}

function validateCosts(costs, prefix, issues) {
  requireText(costs, ["basis"], prefix, issues);
  for (const field of COST_FIELDS) {
    if (!COSTS.has(costs?.[field])) issues.push(`${prefix}.${field} is invalid`);
  }
}

function validateCleanup(cleanup, prefix, issues) {
  requireText(cleanup, ["owner", "removal_condition", "absence_check"], prefix, issues);
}

function validatePublicApiProfile(profile, prefix, issues) {
  requireText(profile, ["contract_diff", "consumer_tests", "versioning", "deprecation", "semantic_checks"], prefix, issues);
  for (const field of ["known_consumers", "unknown_consumers"]) {
    if (!Array.isArray(profile?.[field])) issues.push(`${prefix}.${field} must be an array`);
  }
}

function validateDataProfile(profile, prefix, issues) {
  requireText(profile, [
    "current_schema", "target_schema", "recovery", "dry_run", "idempotency",
    "reconciliation", "cutover", "contraction_condition",
  ], prefix, issues);
}

function validateDependencyProfile(profile, prefix, issues) {
  requireText(profile, ["pinned_version", "compatibility", "failure_behavior", "integration_evidence"], prefix, issues);
}

function validateCompatibility(option, impacts, prefix, issues) {
  const needsApi = impacts.includes("public-contract") || option?.transition === "versioned-coexistence";
  const needsData = impacts.includes("user-data") || option?.transition === "expand-migrate-contract";
  if (needsApi) validatePublicApiProfile(option?.compatibility?.public_api, `${prefix}.compatibility.public_api`, issues);
  if (needsData) validateDataProfile(option?.compatibility?.data, `${prefix}.compatibility.data`, issues);
  if (impacts.includes("external-dependency")) {
    validateDependencyProfile(option?.compatibility?.external_dependency, `${prefix}.compatibility.external_dependency`, issues);
  }
}

function validateOption(option, index, impacts, issues) {
  const prefix = `options[${index}]`;
  requireText(option, ["id", "summary"], prefix, issues);
  if (!DESTINATIONS.has(option?.destination)) issues.push(`${prefix}.destination is invalid`);
  if (!TRANSITIONS.has(option?.transition)) issues.push(`${prefix}.transition is invalid`);
  for (const field of HARD_CONSTRAINTS) {
    validateHardConstraint(option?.hard_constraints?.[field], `${prefix}.hard_constraints.${field}`, issues);
  }
  for (const dimension of REQUIRED_DIMENSIONS) {
    validateEvidence(option?.evidence?.[dimension], dimension, `${prefix}.evidence.${dimension}`, issues);
  }
  validateCosts(option?.total_cost, `${prefix}.total_cost`, issues);
  validateCompatibility(option, impacts, prefix, issues);
  if (option?.destination !== "repair" || option?.transition !== "direct-swap") {
    validateCleanup(option?.cleanup, `${prefix}.cleanup`, issues);
  }
}

function getApprovalReasons(plan, option) {
  const reasons = [];
  if (plan?.project?.posture === "unknown") reasons.push("project-posture-unknown");
  for (const contract of plan?.project?.protected_contracts || []) {
    if (contract.impact !== "preserved") reasons.push(`protected-contract-${contract.impact}:${contract.id}`);
  }
  for (const impact of plan?.change?.impacts || []) {
    if (MATERIAL_IMPACTS.has(impact)) reasons.push(`material-impact:${impact}`);
  }
  if (option?.evidence?.business_outcome?.direction === "worse") {
    reasons.push("evidence-regression:business-outcome");
  }
  if (plan?.change?.reversible === false) reasons.push("change-is-not-reversible");
  return reasons;
}

function validateEnvelope(envelope, option, plan, prefix, issues) {
  requireText(envelope, [
    "outcome", "scope", "risk_limit", "downtime_limit", "cost_limit",
    "release_limit", "environment_limit", "rollback_requirement", "ref",
  ], prefix, issues);
  if (envelope?.current !== true) issues.push(`${prefix}.current must be true`);
  if (envelope?.destination !== option?.destination) issues.push(`${prefix}.destination must match recommendation`);
  if (!Array.isArray(envelope?.transitions) || !envelope.transitions.includes(option?.transition)) {
    issues.push(`${prefix}.transitions must include the recommended transition`);
  }
  if (!Array.isArray(envelope?.protected_contract_ids)) {
    issues.push(`${prefix}.protected_contract_ids must be an array`);
  } else {
    const missing = (plan?.project?.protected_contracts || [])
      .map((contract) => contract.id)
      .filter((id) => !envelope.protected_contract_ids.includes(id));
    if (missing.length > 0) issues.push(`${prefix} omits protected contracts: ${missing.join(", ")}`);
  }
  if (!Array.isArray(envelope?.invalidation_conditions) || envelope.invalidation_conditions.length === 0) {
    issues.push(`${prefix}.invalidation_conditions must be non-empty`);
  }
}

function validateRecommendation(plan, option, approvalReasons, issues) {
  requireText(plan?.recommendation, ["reason", "rejected_alternative"], "recommendation", issues);
  if (!option) {
    issues.push("recommendation.option_id must select a compared option");
    return;
  }
  const failed = HARD_CONSTRAINTS.filter((field) =>
    option?.hard_constraints?.[field]?.status !== "pass");
  if (failed.length > 0) issues.push(`recommended option has non-passing hard constraints: ${failed.join(", ")}`);
  if (!Array.isArray(plan?.recommendation?.advantages) || plan.recommendation.advantages.length === 0) {
    issues.push("recommendation.advantages must cite evidence-backed dimensions");
  } else {
    for (const dimension of plan.recommendation.advantages) {
      const evidence = option?.evidence?.[dimension];
      if (evidence?.direction !== "better" || !HARD_LEVELS.has(evidence?.level)) {
        issues.push(`recommendation advantage is not evidence-backed: ${dimension}`);
      }
    }
  }
  const criticalRegression = [...CRITICAL_DIMENSIONS].find((dimension) =>
    option?.evidence?.[dimension]?.direction === "worse");
  if (criticalRegression && approvalReasons.length === 0) {
    issues.push(`critical regression requires client approval: ${criticalRegression}`);
  }
}

function validateApproval(plan, option, reasons, issues) {
  const status = plan?.approval?.status;
  if (!new Set(["covered", "pending", "approved"]).has(status)) {
    issues.push("approval.status is invalid");
    return;
  }
  if (reasons.length > 0 && status === "covered") {
    issues.push("material change cannot use a covered implicit envelope");
  }
  if (status !== "pending") {
    validateEnvelope(plan?.approval?.envelope, option, plan, "approval.envelope", issues);
  }
}

function validateChangeStrategy(plan) {
  const issues = [];
  if (plan?.schema_version !== 2) issues.push("schema_version must be 2");
  if (!TRIGGERS.has(plan?.trigger?.kind)) issues.push("trigger.kind is invalid");
  requireText(plan?.trigger, ["summary", "root_cause"], "trigger", issues);
  validateRepeatedAttempts(plan?.trigger, issues);
  validatePosture(plan?.project, issues);
  validateContracts(plan?.project?.protected_contracts, issues);
  requireText(plan?.product, ["user_victory", "final_path", "falsifier"], "product", issues);
  requireText(plan?.change, ["summary", "scope"], "change", issues);
  const impacts = Array.isArray(plan?.change?.impacts) ? plan.change.impacts : [];
  if (!Array.isArray(plan?.change?.impacts)) issues.push("change.impacts must be an array");
  for (const impact of impacts) {
    if (!CHANGE_IMPACTS.has(impact)) issues.push(`change.impacts contains an invalid value: ${impact}`);
  }
  const contractImpacts = getContractImpacts(plan?.project?.protected_contracts);
  for (const { impact, id } of contractImpacts) {
    if (!impacts.includes(impact)) {
      issues.push(`change.impacts must include ${impact} derived from protected contract ${id}`);
    }
  }
  const effectiveImpacts = [...new Set([
    ...impacts,
    ...contractImpacts.map(({ impact }) => impact),
  ])];
  if (typeof plan?.change?.reversible !== "boolean") {
    issues.push("change.reversible must be boolean");
  }
  if (!Array.isArray(plan?.options) || plan.options.length < 2 || plan.options.length > 3) {
    issues.push("options must contain two or three destination x transition choices");
  } else {
    plan.options.forEach((option, index) => validateOption(option, index, effectiveImpacts, issues));
    const ids = plan.options.map((option) => option.id);
    if (new Set(ids).size !== ids.length) issues.push("option ids must be unique");
    const choices = plan.options.map((option) => `${option.destination}:${option.transition}`);
    if (new Set(choices).size !== choices.length) {
      issues.push("options must compare unique destination x transition choices");
    }
  }
  const option = plan?.options?.find((candidate) => candidate.id === plan?.recommendation?.option_id);
  const approvalReasons = getApprovalReasons(plan, option);
  validateRecommendation(plan, option, approvalReasons, issues);
  validateApproval(plan, option, approvalReasons, issues);
  const approvalBlocked = approvalReasons.length > 0 && plan?.approval?.status !== "approved";
  return {
    isValid: issues.length === 0,
    blocked: issues.length > 0 || plan?.approval?.status === "pending" || approvalBlocked,
    approvalRequired: approvalReasons.length > 0,
    approvalReasons,
    issues,
  };
}

module.exports = { REQUIRED_DIMENSIONS, validateChangeStrategy };
