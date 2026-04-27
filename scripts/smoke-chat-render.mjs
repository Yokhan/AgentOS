import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

const root = process.cwd();
const srcDir = path.join(root, "src-ui");
const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "agentos-ui-smoke-"));
fs.writeFileSync(path.join(tempDir, "package.json"), '{"type":"module"}');

const files = [
  "api.js",
  "bridge.js",
  "chat.js",
  "pages.js",
  "provider-caps.js",
  "store.js",
  "utils.js",
  "views.js",
  path.join("vendor", "preact-bundle.mjs"),
];

function rewriteImports(source) {
  return source
    .replace(/from\s+["']\/([^"']+)["']/g, 'from "./$1"')
    .replace(/import\s+["']\/([^"']+)["']/g, 'import "./$1"');
}

for (const file of files) {
  const sourcePath = path.join(srcDir, file);
  const targetPath = path.join(tempDir, file);
  fs.mkdirSync(path.dirname(targetPath), { recursive: true });
  const source = fs.readFileSync(sourcePath, "utf8");
  fs.writeFileSync(targetPath, rewriteImports(source));
}

const fetchStub = async () => ({
  ok: false,
  json: async () => ({}),
  text: async () => "",
});

globalThis.fetch = fetchStub;
globalThis.window = {
  __TAURI_INTERNALS__: null,
  fetch: fetchStub,
};
Object.defineProperty(globalThis, "navigator", {
  configurable: true,
  value: {
    clipboard: {
      writeText: async () => {},
    },
  },
});
globalThis.localStorage = {
  getItem: () => null,
  setItem: () => {},
  removeItem: () => {},
};

const realSetInterval = globalThis.setInterval.bind(globalThis);
globalThis.setInterval = (...args) => {
  const timer = realSetInterval(...args);
  timer.unref?.();
  return timer;
};

const storeModuleUrl = pathToFileURL(path.join(tempDir, "store.js")).href;
const chatModuleUrl = pathToFileURL(path.join(tempDir, "chat.js")).href;
const viewsModuleUrl = pathToFileURL(path.join(tempDir, "views.js")).href;
const pagesSource = fs.readFileSync(path.join(srcDir, "pages.js"), "utf8");
if (
  pagesSource.includes("useRef(") &&
  !/import\s*\{[^}]*\buseRef\b[^}]*\}\s*from\s+["']\/vendor\/preact-bundle\.mjs["']/.test(
    pagesSource,
  )
) {
  throw new Error("pages.js uses useRef but does not import it");
}
const {
  agents,
  isLoading,
  segments,
  delegations,
  queueTasks,
  searchQuery,
  activeFilter,
  sortBy,
} = await import(storeModuleUrl);
const { DetailView, ExecutionTimelineCard } = await import(chatModuleUrl);
const { DashboardWorkbenchView } = await import(viewsModuleUrl);

isLoading.value = false;
searchQuery.value = "";
activeFilter.value = "";
sortBy.value = "";
agents.value = [
  {
    name: "AgentOS",
    status: "working",
    task: "Workbench smoke",
    uncommitted: 4,
    days: 0,
    segment: "Infrastructure",
  },
  {
    name: "BlockedProject",
    status: "blocked",
    task: "Needs unblock",
    blockers: true,
    uncommitted: 42,
    days: 16,
    segment: "Other",
  },
];
segments.value = {
  Infrastructure: ["AgentOS"],
  Other: ["BlockedProject"],
};
delegations.value = {
  smoke: { status: "running" },
};
queueTasks.value = [{ done: false, text: "smoke task" }];

DetailView();
DashboardWorkbenchView();

ExecutionTimelineCard({
  timeline: {
    status: "ok",
    schema_version: "agentos.event.v1",
    big_plan: {
      label: "Smoke timeline",
      stage_index: 9,
      stage_total: 9,
    },
    counts: {
      items: 1,
      warnings: 1,
    },
    items: [
      {
        status: "done",
        source: "chat",
        kind: "run",
        project: "AgentOS",
        title: "Rendered",
        detail: "smoke event",
        ts: new Date().toISOString(),
      },
    ],
  },
  contract: {
    schema_version: "agentos.event.v1",
    sources: [
      {
        id: "chat",
        coverage: ["run_done"],
      },
    ],
  },
  onRefresh: () => {},
});

console.log("chat/dashboard render smoke ok");
