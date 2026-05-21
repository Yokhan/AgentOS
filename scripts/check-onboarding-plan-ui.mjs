import fs from "node:fs";

const views = fs.readFileSync("src-ui/views.js", "utf8");
const chatParse = fs.readFileSync(
  "src-tauri/src/commands/chat_parse.rs",
  "utf8",
);
const paCommands = fs.readFileSync(
  "src-tauri/src/commands/pa_commands.rs",
  "utf8",
);
const paOps = fs.readFileSync(
  "src-tauri/src/commands/pa_commands_ops.rs",
  "utf8",
);
const onboarding = fs.readFileSync(
  "src-tauri/src/commands/project_onboarding.rs",
  "utf8",
);
const lib = fs.readFileSync("src-tauri/src/lib.rs", "utf8");
const doc = fs.readFileSync("docs/PROJECT_ONBOARDING.md", "utf8");
const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));

const checks = [
  {
    name: "focus screen exposes natural-language onboarding wave shortcut",
    ok:
      views.includes("prepare onboarding wave") &&
      views.includes("Подключи проекты к AgentOS безопасной волной") &&
      views.includes("Не проси меня писать PA-теги руками") &&
      views.includes('activeFilter.value = "unmanaged"'),
  },
  {
    name: "natural language routing recommends safe onboarding plan",
    ok:
      chatParse.includes("[PROJECT_ONBOARD_PLAN:Other:balanced:5]") &&
      chatParse.includes("safe wave plan") &&
      chatParse.includes("do not ask them to type PA command tags manually"),
  },
  {
    name: "PA command parser understands onboarding plan",
    ok:
      paOps.includes("ProjectOnboardPlan") &&
      paOps.includes("RE_PROJECT_ONBOARD_PLAN") &&
      paCommands.includes("ProjectOnboardPlan { .. } => true"),
  },
  {
    name: "backend formats read-only onboarding wave plan",
    ok:
      onboarding.includes("format_onboarding_plan") &&
      onboarding.includes("git_dirty_count") &&
      onboarding.includes("Canary template sync") &&
      lib.includes("project_onboarding_plan"),
  },
  {
    name: "operator doc explains safe onboarding flow",
    ok:
      doc.includes("[PROJECT_ONBOARD_PLAN") &&
      doc.includes("metadata repair") &&
      doc.includes("canary"),
  },
  {
    name: "gate is registered in check:ui",
    ok: pkg.scripts?.["check:ui"]?.includes("check-onboarding-plan-ui.mjs"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("onboarding plan UI checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`onboarding plan UI ok: ${checks.length} checks`);
