import fs from "node:fs";

const views = fs.readFileSync("src-ui/views.js", "utf8");
const api = fs.readFileSync("src-ui/api.js", "utf8");
const css = fs.readFileSync("src-ui/styles/main.css", "utf8");
const delegation = fs.readFileSync(
  "src-tauri/src/commands/delegation.rs",
  "utf8",
);
const apiServer = fs.readFileSync("src-tauri/src/api_server.rs", "utf8");
const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));

const checks = [
  {
    name: "workspace has first-class delegation tab",
    ok:
      views.includes('["delegations", "Делегации"') &&
      views.includes("function DelegationsWorkspace") &&
      views.includes("function DelegationCard") &&
      views.includes('tab === "delegations"'),
  },
  {
    name: "delegation workspace exposes decisions and details",
    ok:
      views.includes("approve pending") &&
      views.includes("DELEGATE_RETRY") &&
      views.includes("DELEGATE_STATUS") &&
      views.includes("executor_provider") &&
      views.includes("gate_result") &&
      views.includes("review_verdict"),
  },
  {
    name: "frontend preserves full delegation payload",
    ok:
      (api.match(/\.\.\.item/g) || []).length >= 2 &&
      api.includes("mergeDelegationItems"),
  },
  {
    name: "backend snapshot returns active and recent terminal delegations",
    ok:
      delegation.includes("pub fn delegations_snapshot") &&
      delegation.includes('"terminal"') &&
      delegation.includes("terminal.into_iter().take(50)") &&
      apiServer.includes("delegations_snapshot(&state)"),
  },
  {
    name: "delegation workspace has dedicated styling",
    ok:
      css.includes(".delegations-workspace") &&
      css.includes(".delegation-work-card") &&
      css.includes(".delegation-summary-grid"),
  },
  {
    name: "gate is registered in check:ui",
    ok: pkg.scripts?.["check:ui"]?.includes(
      "check-delegation-workspace-ui.mjs",
    ),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("delegation workspace checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`delegation workspace ok: ${checks.length} checks`);
