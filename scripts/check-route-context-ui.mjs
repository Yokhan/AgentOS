import fs from "node:fs";

const chat = fs.readFileSync("src-ui/chat.js", "utf8");
const css = fs.readFileSync("src-ui/styles/chat.css", "utf8");

const checks = [
  {
    name: "route context helper exists",
    ok:
      chat.includes("function buildRouteContextChips") &&
      chat.includes("currentPlan?.next_step") &&
      chat.includes("activeRoute?.next_work_item"),
  },
  {
    name: "route carries plan/task context chips",
    ok:
      chat.includes("contextChips: buildRouteContextChips(") &&
      chat.includes("route.contextChips?.length") &&
      chat.includes("route-context-chip"),
  },
  {
    name: "route context opens plans/project",
    ok:
      chat.includes("function openRouteContextChip") &&
      chat.includes('activeWorkspaceTab.value = "plans"') &&
      chat.includes("showPlans.value = true") &&
      chat.includes("currentProject.value = chip.project"),
  },
  {
    name: "route context is exported for behavior smoke",
    ok:
      chat.includes("buildRouteContextChips,") &&
      chat.includes("isRenderableMapEvent,") &&
      chat.includes("isProviderStateEvent,"),
  },
  {
    name: "route context has compact visual style",
    ok:
      css.includes(".route-context-chips") &&
      css.includes(".route-context-chip.kind-task") &&
      css.includes(".route-context-chip:hover"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("route context UI checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`route context UI ok: ${checks.length} checks`);
