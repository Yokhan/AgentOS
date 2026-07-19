"use strict";

const DISCOVERY_KINDS = new Set([
  "local-leaf",
  "repeated-failure",
  "architecture-mismatch",
  "ownership-conflict",
  "sot-conflict",
  "duplicate-state",
  "duplicate-implementation",
  "obsolete-final-path",
  "compatibility-only-layer",
  "protected-boundary-unknown",
  "stale-path-test",
  "sunk-cost",
  "planned-breaking-change",
  "manual-review",
]);
const PHASES = new Set(["reading", "research", "implementation", "verification"]);
const FITNESS = new Set(["fit", "mismatch", "unknown"]);
const DISCOVERY_TRIGGER_MAP = {
  "repeated-failure": "repeated-failure",
  "architecture-mismatch": "architecture-mismatch",
  "ownership-conflict": "ownership-conflict",
  "sot-conflict": "sot-conflict",
  "duplicate-state": "duplicate-state",
  "duplicate-implementation": "duplicate-implementation",
  "obsolete-final-path": "obsolete-final-path",
  "compatibility-only-layer": "compatibility-shim",
  "protected-boundary-unknown": "manual-review",
  "stale-path-test": "stale-path-test",
  "sunk-cost": "sunk-cost",
  "planned-breaking-change": "planned-breaking-change",
  "manual-review": "manual-review",
};

function isText(value) {
  return typeof value === "string" && value.trim().length >= 8;
}

function evaluateDiscoveryReroute(discovery) {
  if (!discovery) {
    return { provided: false, required: false, blockEdits: false, reasons: [], issues: [] };
  }
  const issues = [];
  if (!DISCOVERY_KINDS.has(discovery.kind)) issues.push("discovery.kind is invalid");
  if (!PHASES.has(discovery.phase)) issues.push("discovery.phase must be reading or research");
  if (!FITNESS.has(discovery.architecture_fit)) issues.push("discovery.architecture_fit is invalid");
  for (const field of ["summary", "evidence_ref", "owner", "sot"]) {
    if (!isText(discovery[field])) issues.push(`discovery.${field} must be specific text`);
  }
  if (!Array.isArray(discovery.protected_boundaries)) {
    issues.push("discovery.protected_boundaries must be an array");
  }
  if (discovery.kind === "repeated-failure" && !isText(discovery.acceptance_id)) {
    issues.push("discovery.acceptance_id is required for repeated-failure");
  }
  const isLocalLeaf = discovery.kind === "local-leaf";
  const localLeafProven = isLocalLeaf && discovery.architecture_fit === "fit" &&
    discovery.protected_boundaries?.length === 0 && issues.length === 0;
  const required = !localLeafProven;
  return {
    provided: true,
    kind: discovery.kind,
    evidenceRef: discovery.evidence_ref,
    acceptanceId: discovery.acceptance_id,
    required,
    blockEdits: required,
    reasons: required ? [`discovery:${discovery.kind || "invalid"}`] : ["discovery:local-leaf-fit"],
    issues,
  };
}

function getDecisionBinding(discovery, decision, isChangeStrategyRequired = false) {
  if (!isChangeStrategyRequired && !discovery?.required) {
    return { isBound: true, issues: [] };
  }
  if (!discovery?.required || discovery.issues.length > 0) {
    return {
      isBound: false,
      issues: ["a valid structured discovery trigger is required before edits can resume"],
    };
  }
  const expectedKind = DISCOVERY_TRIGGER_MAP[discovery.kind];
  const issues = [];
  if (decision?.trigger?.kind !== expectedKind) {
    issues.push(`decision trigger must match discovery kind: ${expectedKind}`);
  }
  if (expectedKind === "repeated-failure" &&
      decision?.trigger?.acceptance_id !== discovery.acceptanceId) {
    issues.push("decision trigger acceptance_id must match discovery acceptance_id");
  }
  if (decision?.trigger?.evidence_ref !== discovery.evidenceRef) {
    issues.push("decision trigger evidence_ref must match discovery evidence_ref");
  }
  return { isBound: issues.length === 0, issues };
}

module.exports = { DISCOVERY_KINDS, evaluateDiscoveryReroute, getDecisionBinding };
