import fs from "node:fs";

const views = fs.readFileSync("src-ui/views.js", "utf8");
const component = fs.readFileSync("src-ui/components/notifications.js", "utf8");
const css = fs.readFileSync("src-ui/styles/main.css", "utf8");

const checks = [
  {
    name: "notification center has source/severity/project filters",
    ok:
      component.includes("severityFilter") &&
      component.includes("sourceFilter") &&
      component.includes("projectFilter") &&
      component.includes("notification-filters") &&
      css.includes(".notification-filters"),
  },
  {
    name: "notification rows expose routing context",
    ok:
      component.includes("contextLabels") &&
      component.includes("route_id") &&
      component.includes("delegation_id") &&
      component.includes("run_id") &&
      component.includes("project:"),
  },
  {
    name: "notification center filters visible data before grouping",
    ok:
      component.includes("const visibleItems = items.filter") &&
      component.includes("visibleItems.filter") &&
      component.includes("reset filters"),
  },
  {
    name: "views delegates notification implementation to component module",
    ok:
      views.includes("NotificationsWorkspaceView") &&
      !views.includes("function NotificationsWorkspace") &&
      !views.includes("notificationContextLabel"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("notification center UI checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`notification center UI ok: ${checks.length} checks`);
