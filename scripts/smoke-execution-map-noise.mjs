import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

const root = process.cwd();
const srcDir = path.join(root, "src-ui");
const tempDir = fs.mkdtempSync(
  path.join(os.tmpdir(), "agentos-map-noise-smoke-"),
);
fs.writeFileSync(path.join(tempDir, "package.json"), '{"type":"module"}');

const files = [
  "api.js",
  "bridge.js",
  "chat.js",
  "provider-caps.js",
  "route-state.js",
  "run-state.js",
  "store.js",
  "utils.js",
  path.join("components", "chat-trace.js"),
  path.join("vendor", "preact-bundle.mjs"),
];

function rewriteImports(source, file) {
  const dir = path.dirname(file);
  const prefix = dir === "." ? "./" : "../".repeat(dir.split(path.sep).length);
  return source
    .replace(/from\s+["']\/([^"']+)["']/g, `from "${prefix}$1"`)
    .replace(/import\s+["']\/([^"']+)["']/g, `import "${prefix}$1"`);
}

for (const file of files) {
  const sourcePath = path.join(srcDir, file);
  const targetPath = path.join(tempDir, file);
  fs.mkdirSync(path.dirname(targetPath), { recursive: true });
  fs.writeFileSync(
    targetPath,
    rewriteImports(fs.readFileSync(sourcePath, "utf8"), file),
  );
}

const response = (data = {}) => ({
  ok: true,
  status: 200,
  json: async () => data,
  text: async () => JSON.stringify(data),
});

const fetchStub = async (url) => {
  if (String(url || "").startsWith("http://localhost:")) {
    return {
      ok: false,
      status: 404,
      json: async () => ({}),
      text: async () => "",
    };
  }
  return response({});
};

globalThis.fetch = fetchStub;
globalThis.window = {
  __TAURI_INTERNALS__: null,
  fetch: fetchStub,
  innerWidth: 1440,
  addEventListener: () => {},
  removeEventListener: () => {},
};
globalThis.document = {
  body: { classList: { add: () => {}, remove: () => {} } },
  createElement: () => ({ click: () => {} }),
  querySelector: () => null,
  hasFocus: () => true,
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
const {
  buildRouteContextChips,
  executionLaneOwnerLabel,
  isProviderStateEvent,
  isRenderableMapEvent,
} = await import(chatModuleUrl);

const providerHeartbeat = {
  kind: "provider_heartbeat",
  status: "running",
  title: "provider",
  detail: "Codex subprocess is still running",
};
const providerProgress = {
  kind: "progress",
  status: "running",
  title: "provider",
  detail: "waiting for provider output",
};
const semanticResult = {
  kind: "tool_result",
  status: "done",
  title: "tests passed",
};

if (!isProviderStateEvent(providerHeartbeat)) {
  throw new Error("provider heartbeat was not classified as provider state");
}
if (isRenderableMapEvent(providerHeartbeat)) {
  throw new Error("provider heartbeat leaked into execution map nodes");
}
if (isRenderableMapEvent(providerProgress)) {
  throw new Error("provider progress leaked into execution map nodes");
}
if (isRenderableMapEvent({ kind: "root", title: "run root" })) {
  throw new Error("root event leaked into execution map nodes");
}
if (isRenderableMapEvent({ kind: "progress", semantic: false })) {
  throw new Error("non-semantic event leaked into execution map nodes");
}
if (!isRenderableMapEvent(semanticResult)) {
  throw new Error("semantic result event was hidden");
}

const laneCases = [
  [{ kind: "orchestrator", label: "Orchestrator" }, "orchestrator"],
  [{ role: "reviewer", label: "Review lane" }, "reviewer"],
  [{ owner: "user", label: "Approval" }, "user"],
  [{ kind: "project-agent", label: "AgentOS delegation" }, "project-agent"],
  [{ provider: "codex", label: "Codex" }, "agent"],
];
for (const [lane, expected] of laneCases) {
  const actual = executionLaneOwnerLabel(lane);
  if (actual !== expected) {
    throw new Error(`lane owner mismatch: expected ${expected}, got ${actual}`);
  }
}

const chips = buildRouteContextChips(
  { kind: "plan", title: "Release plan", project: "AgentOS", plan_id: "p1" },
  {
    project: "AgentOS",
    big_plan: { stage_index: 9, stage_total: 9, label: "Route progress" },
    plans: [
      {
        id: "p1",
        title: "Release plan",
        next_step: {
          id: "s1",
          project: "AgentOS",
          task: "Verify execution map semantics",
          work_item_id: "wi1",
        },
      },
    ],
  },
  null,
);
for (const kind of ["project", "plan", "task", "stage"]) {
  if (!chips.some((chip) => chip.kind === kind)) {
    throw new Error(`route context chip missing: ${kind}`);
  }
}

console.log("execution map noise smoke ok");
