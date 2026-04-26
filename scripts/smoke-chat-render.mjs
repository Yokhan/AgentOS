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
  "provider-caps.js",
  "store.js",
  "utils.js",
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

const chatModuleUrl = pathToFileURL(path.join(tempDir, "chat.js")).href;
const { DetailView, ExecutionTimelineCard } = await import(chatModuleUrl);

DetailView();

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

console.log("chat render smoke ok");
