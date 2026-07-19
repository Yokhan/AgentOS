import type { RouteResult } from "./types.js";

export function appendWritingContract(sections: string[], route: RouteResult): void {
  if (!route.targetLanguage) return;
  sections.push(`TARGET_LANGUAGE: ${route.targetLanguage}`);
  if (route.languageResolution) sections.push(`LANGUAGE_RESOLUTION: ${route.languageResolution}`);
  if (route.writingLanguageProfiles.length) sections.push(`WRITING_LANGUAGE_PROFILES: ${route.writingLanguageProfiles.join(", ")}`);
  if (route.writingProcessProfiles.length) sections.push(`WRITING_PROCESS_PROFILES: ${route.writingProcessProfiles.join(", ")}`);
  if (route.writingDomainProfiles.length) sections.push(`WRITING_DOMAIN_PROFILES: ${route.writingDomainProfiles.join(", ")}`);
  if (route.writingTechnicalProfiles.length) sections.push(`WRITING_TECHNICAL_PROFILES: ${route.writingTechnicalProfiles.join(", ")}`);
  if (route.writingRejectedProfiles.length) sections.push(`WRITING_REJECTED: ${route.writingRejectedProfiles.map(({ id, reason }) => `${id}:${reason}`).join(", ")}`);
  if (route.writingEditors.length) sections.push(`WRITING_EDITORS: ${route.writingEditors.join(", ")}`);
  if (route.writingExternalTools.length) sections.push(`WRITING_EXTERNAL_TOOLS: ${route.writingExternalTools.map(({ id, access, execution, paid }) => `${id}:${access}:${execution}:${paid ? "paid" : "free"}`).join(", ")}`);
  if (route.writingGates.length) sections.push(`WRITING_GATES: ${route.writingGates.join(", ")}`);
}
