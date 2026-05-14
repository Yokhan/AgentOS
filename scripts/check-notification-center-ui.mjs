import fs from "node:fs";

const views = fs.readFileSync("src-ui/views.js", "utf8");
const css = fs.readFileSync("src-ui/styles/main.css", "utf8");

const checks = [
  {
    name: "notification center has source/severity/project filters",
    ok:
      views.includes("severityFilter") &&
      views.includes("sourceFilter") &&
      views.includes("projectFilter") &&
      views.includes("notification-filters") &&
      css.includes(".notification-filters"),
  },
  {
    name: "notification rows expose routing context",
    ok:
      views.includes("notificationContextLabel") &&
      views.includes("route_id") &&
      views.includes("delegation_id") &&
      views.includes("run_id") &&
      views.includes("project:"),
  },
  {
    name: "notification center filters visible data before grouping",
    ok:
      views.includes("const visibleItems = items.filter") &&
      views.includes("visibleItems.filter") &&
      views.includes("reset filters"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("notification center UI checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`notification center UI ok: ${checks.length} checks`);
