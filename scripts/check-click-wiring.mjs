import fs from "node:fs";

const read = (file) => fs.readFileSync(file, "utf8");
const chat = read("src-ui/chat.js");
const api = read("src-ui/api.js");
const views = read("src-ui/views.js");
const pages = read("src-ui/pages.js");
const pkg = JSON.parse(read("package.json"));
const checklist = read("tasks/UX_RELEASE_CHECKLIST.md");

const checks = [
  {
    name: "unified live status is rendered",
    ok: chat.includes("<${LiveStatusStrip} />"),
  },
  {
    name: "old live banner is not rendered",
    ok:
      !chat.includes("<${RunningBanner} />") &&
      !chat.includes("<${LiveRunHud} />"),
  },
  {
    name: "context chips are visible and consumed",
    ok:
      chat.includes("function ContextChips") &&
      api.includes("contextAttachments.value = []") &&
      api.includes("[USER_TASK]"),
  },
  {
    name: "Duo does not replace center canvas",
    ok:
      !views.includes("duoWorkspaceActive") &&
      !views.includes("duoWorkspace ="),
  },
  {
    name: "plan opens unified Duo mode",
    ok: pages.includes('chatCollabMode.value = "duo"'),
  },
  {
    name: "rail has full quick filters",
    ok:
      views.includes('"dirty"') &&
      views.includes('"delegation"') &&
      views.includes('"plan"') &&
      views.includes("projectHasDelegation") &&
      views.includes("projectHasPlan"),
  },
  {
    name: "route-state is covered by check:ui",
    ok: pkg.scripts?.["check:ui"]?.includes("src-ui/route-state.js"),
  },
  {
    name: "live status exposes stop/copy/details",
    ok:
      chat.includes("LiveStatusStrip") &&
      chat.includes("copyLiveOutput") &&
      chat.includes("Stop current agent run") &&
      chat.includes("Show run events"),
  },
  {
    name: "rail keyboard navigation is wired",
    ok:
      views.includes('e.key === "ArrowDown"') &&
      views.includes('e.key === "ArrowUp"') &&
      views.includes('e.key === "Escape"'),
  },
  {
    name: "release checklist covers manual UX smoke",
    ok:
      checklist.includes("Route persistence") &&
      checklist.includes("Duo toggle") &&
      checklist.includes("Live run recovery"),
  },
  {
    name: "mojibake check is covered by check:ui",
    ok: pkg.scripts?.["check:ui"]?.includes("scripts/check-mojibake.mjs"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("critical UI wiring checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`critical UI wiring ok: ${checks.length} checks`);
