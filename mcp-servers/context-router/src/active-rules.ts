import { routeKeywords } from "./router.js";
import type { ServerState } from "./types.js";
import { appendWritingContract } from "./writing-contract.js";

export function buildActiveRulesOutput(
  state: ServerState,
  engramStatus: string,
): string {
  const restoredRoute = routeKeywords(state.taskDescription);
  const lines = [
    `MODE: ${state.currentModes.join("+")}`,
    `TASK: ${state.taskDescription}`,
    `ROUTED AT: ${state.lastRouteTime}`,
    engramStatus,
    `CODE_INTELLIGENCE: ${restoredRoute.codeIntelligence.id} | ${restoredRoute.codeIntelligence.tools.join(" -> ")}`,
    `RULES (${state.activeRules.length} files):`,
    ...state.activeRules.map((file) => `  .claude/library/${file}`),
  ];
  appendWritingContract(lines, restoredRoute);
  return lines.join("\n");
}
