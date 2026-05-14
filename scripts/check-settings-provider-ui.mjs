import fs from "node:fs";

const pages = fs.readFileSync("src-ui/pages.js", "utf8");
const css = fs.readFileSync("src-ui/styles/main.css", "utf8");
const providerRunner = fs.readFileSync(
  "src-tauri/src/commands/provider_runner.rs",
  "utf8",
);
const doc = fs.readFileSync("docs/PROVIDER_ROUTING.md", "utf8");
const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));

const checks = [
  {
    name: "settings renders one effective provider route table",
    ok:
      pages.includes("Effective Provider Routes") &&
      pages.includes("settings-provider-table") &&
      pages.includes("effectiveRoutes.map") &&
      pages.includes("technical_reviewer"),
  },
  {
    name: "settings shows active GPT account snapshot",
    ok:
      pages.includes("active GPT account") &&
      pages.includes("codex-account-card") &&
      pages.includes("codexModelsSource"),
  },
  {
    name: "settings explains disabled Claude fallback",
    ok:
      pages.includes("Claude is disabled globally") &&
      pages.includes("Any Claude route resolves to Codex before execution"),
  },
  {
    name: "provider status includes technical reviewer effective route",
    ok:
      providerRunner.includes('"technical_reviewer"') &&
      providerRunner.includes("technical_reviewer_model") &&
      providerRunner.includes("technical_reviewer_effort") &&
      providerRunner.includes('status["role_settings"]["technical_reviewer"]'),
  },
  {
    name: "settings route table and account card are styled",
    ok:
      css.includes(".settings-provider-table") &&
      css.includes(".settings-provider-health") &&
      css.includes(".codex-account-card") &&
      css.includes(".settings-diagnostics"),
  },
  {
    name: "provider routing operator doc exists",
    ok:
      doc.includes("claude_enabled=false") &&
      doc.includes("orchestrator") &&
      doc.includes("technical_reviewer") &&
      doc.includes("delegation"),
  },
  {
    name: "gate is registered in check:ui",
    ok: pkg.scripts?.["check:ui"]?.includes("check-settings-provider-ui.mjs"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("settings/provider UI checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`settings/provider UI ok: ${checks.length} checks`);
