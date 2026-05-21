import fs from "node:fs";

const pages = fs.readFileSync("src-ui/pages.js", "utf8");
const updater = fs.readFileSync(
  "src-tauri/src/commands/app_updates.rs",
  "utf8",
);
const lib = fs.readFileSync("src-tauri/src/lib.rs", "utf8");
const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));

const checks = [
  {
    name: "settings exposes manual update status",
    ok:
      pages.includes("application update") &&
      pages.includes("checkUpdatesNow") &&
      pages.includes("installUpdateNow") &&
      pages.includes("check_app_update") &&
      pages.includes("install_app_update"),
  },
  {
    name: "updater backend has manual check and install commands",
    ok:
      updater.includes("pub async fn check_app_update") &&
      updater.includes("pub async fn install_app_update") &&
      updater.includes("manual check requested") &&
      updater.includes("manual install requested"),
  },
  {
    name: "updater operations are bounded",
    ok:
      updater.includes("UPDATE_CHECK_TIMEOUT_SECS") &&
      updater.includes("UPDATE_INSTALL_TIMEOUT_SECS") &&
      updater.includes("tokio::time::timeout"),
  },
  {
    name: "tauri registers updater commands",
    ok:
      lib.includes("commands::app_updates::check_app_update") &&
      lib.includes("commands::app_updates::install_app_update"),
  },
  {
    name: "gate is registered in check:ui",
    ok: pkg.scripts?.["check:ui"]?.includes("check-updater-ui.mjs"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("updater UI checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`updater UI ok: ${checks.length} checks`);
