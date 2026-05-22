import fs from "node:fs";

const files = {
  views: "src-ui/views.js",
  chat: "src-ui/chat.js",
  delegations: "src-ui/components/delegations.js",
  notifications: "src-ui/components/notifications.js",
  routes: "src-ui/components/routes.js",
  audit: "docs/UX_ARCHITECTURE_AUDIT.md",
};

const text = Object.fromEntries(
  Object.entries(files).map(([key, path]) => [
    key,
    fs.readFileSync(path, "utf8"),
  ]),
);

const size = (path) => fs.statSync(path).size;

const checks = [
  {
    name: "workspace composer stays below current size budget",
    ok: size(files.views) < 70_000,
  },
  {
    name: "delegation implementation is outside views.js",
    ok:
      text.views.includes("/components/delegations.js") &&
      !text.views.includes("function DelegationCard") &&
      !text.views.includes("DELEGATION_RUNNING_STATUSES"),
  },
  {
    name: "delegation component does not import chat monolith",
    ok: !text.delegations.includes("/chat.js"),
  },
  {
    name: "route decision implementation is outside views.js",
    ok:
      text.views.includes("/components/routes.js") &&
      !text.views.includes("function RouteDecisionPanel") &&
      !text.views.includes("function routeNeedsDecision"),
  },
  {
    name: "route component owns route actions",
    ok:
      text.routes.includes("RouteDecisionPanelCompact") &&
      text.routes.includes("DELEGATE_STATUS") &&
      text.routes.includes("DELEGATE_RETRY") &&
      text.routes.includes("HEALTH_CHECK"),
  },
  {
    name: "route component does not import chat monolith",
    ok: !text.routes.includes("/chat.js"),
  },
  {
    name: "notification implementation is outside views.js",
    ok:
      text.views.includes("/components/notifications.js") &&
      !text.views.includes("function NotificationsWorkspace") &&
      !text.views.includes("notificationContextLabel"),
  },
  {
    name: "notification component does not import chat monolith",
    ok: !text.notifications.includes("/chat.js"),
  },
  {
    name: "chat does not own delegation workspace",
    ok:
      !text.chat.includes("DelegationsWorkspace") &&
      !text.chat.includes("delegation-summary-grid"),
  },
  {
    name: "architecture audit records remaining split debt",
    ok:
      text.audit.includes("Split `chat.js`") &&
      text.audit.includes("src-ui/components/delegations.js"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("UI architecture boundary checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`UI architecture boundaries ok: ${checks.length} checks`);
