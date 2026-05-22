import fs from "node:fs";

const chat = fs.readFileSync("src-ui/chat.js", "utf8");
const chatTrace = fs.readFileSync("src-ui/components/chat-trace.js", "utf8");
const views = fs.readFileSync("src-ui/views.js", "utf8");
const css = fs.readFileSync("src-ui/styles/main.css", "utf8");
const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));

const checks = [
  {
    name: "routine system trace is hidden from chat transcript",
    ok:
      chatTrace.includes("function isRoutineSystemTraceMessage") &&
      chatTrace.includes("auto-continuing after") &&
      chatTrace.includes("waiting coordinator:") &&
      chat.includes("isRoutineSystemTraceMessage(m.msg)") &&
      chat.includes('m.kind !== "pa_feedback_notice"'),
  },
  {
    name: "routine trace helper is exported for tests",
    ok:
      chat.includes("isRoutineSystemTraceMessage,") &&
      chatTrace.includes("isRoutineSystemTraceMessage,"),
  },
  {
    name: "project rail derives active plan/task context",
    ok:
      views.includes("function projectRailWorkBadge") &&
      views.includes("orchestrationMap.value") &&
      views.includes("next_work_item") &&
      views.includes("firstOpenPlanStep"),
  },
  {
    name: "project rail renders work badge and state",
    ok:
      views.includes("const workBadge = projectRailWorkBadge(ag.name)") &&
      views.includes("project-rail-work") &&
      views.includes("project-rail-state ${workBadge ?"),
  },
  {
    name: "project rail work badge is styled",
    ok:
      css.includes(".project-rail-state.has-work") &&
      css.includes(".project-rail-work.task") &&
      css.includes(".project-rail-work.plan"),
  },
  {
    name: "gate is registered in check:ui",
    ok: pkg.scripts?.["check:ui"]?.includes("check-chat-noise-rail-ui.mjs"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("chat noise/project rail checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`chat noise/project rail UI ok: ${checks.length} checks`);
