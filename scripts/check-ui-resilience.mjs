import fs from "node:fs";

const read = (file) => fs.readFileSync(file, "utf8");
const app = read("src-ui/app.js");
const api = read("src-ui/api.js");
const store = read("src-ui/store.js");
const views = read("src-ui/views.js");
const resilience = read("src-ui/resilience.js");
const config = read("src-tauri/src/commands/config.rs");
const lib = read("src-tauri/src/lib.rs");
const docs = read("docs/UI_RESILIENCE.md");
const pkg = JSON.parse(read("package.json"));

const checks = [
  {
    name: "runtime diagnostics installs long-task and event-loop watchdogs",
    ok:
      resilience.includes("PerformanceObserver") &&
      resilience.includes("event_loop_lag") &&
      resilience.includes("long_task"),
  },
  {
    name: "startup installs diagnostics before polling",
    ok:
      app.includes("installUiDiagnostics({ recordRemote: recordUiDiagnostic })") &&
      app.indexOf("installUiDiagnostics") < app.indexOf("runStartupLoad"),
  },
  {
    name: "safe mode state is persisted centrally",
    ok:
      store.includes('const safeMode = signal(localStorage.getItem("agentos_safe_mode") === "1")') &&
      store.includes("safeMode,"),
  },
  {
    name: "heavy loaders are safe-mode aware",
    ok:
      api.includes("safeModeBlocksHeavyLoad") &&
      api.includes('safeModeBlocksHeavyLoad("executionMap")') &&
      api.includes('safeModeBlocksHeavyLoad("orchestrationMap")') &&
      api.includes('safeModeBlocksHeavyLoad("graph")'),
  },
  {
    name: "polling skips heavy work in safe mode",
    ok:
      app.includes("shouldDeferHeavyPolling() || safeMode.value") &&
      app.includes("Graph is disabled in safe mode"),
  },
  {
    name: "safe mode is visible and user-toggleable",
    ok:
      views.includes("safe on") &&
      views.includes("Live map is paused") &&
      views.includes("disable safe mode"),
  },
  {
    name: "ui diagnostics are persisted in backend",
    ok:
      config.includes("record_ui_diagnostic") &&
      config.includes(".ui-diagnostics.jsonl") &&
      lib.includes("commands::config::record_ui_diagnostic"),
  },
  {
    name: "architecture doc explains why freezes were possible",
    ok:
      docs.includes("Why this was architecturally possible") &&
      docs.includes("Only `app.js` owns automatic polling") &&
      docs.includes("Components may request manual refresh"),
  },
  {
    name: "gate is registered in check:ui",
    ok: pkg.scripts?.["check:ui"]?.includes("check-ui-resilience.mjs"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("UI resilience checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`UI resilience checks ok: ${checks.length} checks`);
