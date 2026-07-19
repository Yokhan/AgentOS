"use strict";

const os = require("os");
const { formatAgentProfiles } = require("../codex-agent-policy.js");

function getWritingLines(policy) {
  if (!policy) return [];
  return [
    `WRITING_LANGUAGE: ${policy.targetLanguage}`,
    `WRITING_LANGUAGE_RESOLUTION: ${policy.languageResolution}`,
    `WRITING_LANGUAGE_PROFILES: ${policy.languageProfiles.join(", ") || "none"}`,
    `WRITING_PROCESS_PROFILES: ${policy.processProfiles.join(", ") || "none"}`,
    `WRITING_DOMAIN_PROFILES: ${policy.domainProfiles.join(", ") || "none"}`,
    `WRITING_TECHNICAL_PROFILES: ${policy.technicalProfiles.join(", ") || "none"}`,
    `WRITING_REJECTED: ${policy.rejectedProfiles.map(({ id, reason }) => `${id}:${reason}`).join(", ") || "none"}`,
    `WRITING_EXTERNAL_TOOLS: ${policy.externalTools.map(({ id, access, execution, paid }) => `${id}:${access}:${execution}:${paid ? "paid" : "free"}`).join(", ") || "none"}`,
    `WRITING_EDITORS: ${policy.editors.join(", ") || "none"}`,
    `WRITING_GATES: ${policy.gates.join(", ") || "none"}`,
  ];
}

function formatSummary(route) {
  return [
    `ROUTE: ${route.modes.join("+")}`,
    `PIPELINE: ${route.pipeline}`,
    `CODE_INTELLIGENCE: ${route.codeIntelligence.id} | ${route.codeIntelligence.tools.join(" -> ")}`,
    `RISK: ${route.risk}`,
    `MATCHES: exact=${route.exactMatches.join("+") || "none"} | semantic=${route.semanticMatches.join("+") || "none"}`,
    `CHANGE_STRATEGY: ${route.changeStrategy.required ? "required" : "not-required"} | lifecycle=${route.changeStrategy.lifecycle} | record=${route.changeStrategy.recordMode} | reasons=${route.changeStrategy.reasons.join("+") || "none"}`,
    `DISCOVERY: ${route.discovery.provided ? route.discovery.kind : "none"} | block_edits=${route.blockEdits} | binding=${route.decisionBinding.isBound ? "matched" : "mismatch"} | issues=${[...route.discovery.issues, ...route.decisionBinding.issues].join("+") || "none"}`,
    `SKILLS: ${route.skills.join(", ")}`,
    `SUBAGENTS: ${route.subagents.join(", ") || "none"}`,
    `FANOUT: ${route.fanout.status} | ${route.fanout.reason} | max_children=${route.fanout.maxChildren}`,
    `PROFILES: ${formatAgentProfiles(route.fanout.candidates).join(", ") || "none"}`,
    `ORCHESTRATOR: ${route.orchestrator.owner} (${route.orchestrator.codexRole})`,
    `PLAN: ${route.planContract.required ? "required" : "optional"} | ${route.planContract.language}`,
    `PRODUCT_BAR: ${route.productionBar.default} | outcome=${route.productionBar.outcomePriority} | no_mvp=${route.productionBar.noMvpByDefault}`,
    `GATES: ${route.qualityGates.join(", ")}`,
    `RULES: ${route.sharedRules.join(", ")}`,
    ...getWritingLines(route.writingPolicy),
    route.needsFreshDocs ? "FRESH_DOCS: required" : "FRESH_DOCS: not required by route",
  ].join(os.EOL);
}

module.exports = { formatSummary };
