import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

const root = process.cwd();
const srcDir = path.join(root, "src-ui");
const tempDir = fs.mkdtempSync(
  path.join(os.tmpdir(), "agentos-code-context-smoke-"),
);
fs.writeFileSync(path.join(tempDir, "package.json"), '{"type":"module"}');

const files = [
  "api.js",
  "bridge.js",
  "provider-caps.js",
  "route-state.js",
  "run-state.js",
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
  fs.writeFileSync(
    targetPath,
    rewriteImports(fs.readFileSync(sourcePath, "utf8")),
  );
}

let capturedChatStreamBody = null;

const response = (data) => ({
  ok: true,
  status: 200,
  json: async () => data,
  text: async () => JSON.stringify(data),
});

const fetchStub = async (url, opts = {}) => {
  const pathName = String(url || "");
  if (pathName.startsWith("http://localhost:")) {
    return {
      ok: false,
      status: 404,
      json: async () => ({}),
      text: async () => "",
    };
  }
  if (pathName.includes("/api/chat-stream")) {
    capturedChatStreamBody = JSON.parse(String(opts.body || "{}"));
    return {
      ok: true,
      body: {
        getReader: () => ({
          read: async () => ({ done: true, value: undefined }),
        }),
      },
    };
  }
  if (pathName.includes("/api/chat/")) {
    return response({
      project: "AgentOS",
      total: 1,
      messages: [{ role: "assistant", msg: "history ok" }],
    });
  }
  return response({});
};

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
const apiModuleUrl = pathToFileURL(path.join(tempDir, "api.js")).href;
const {
  currentProject,
  contextAttachments,
  codeContextError,
  codeContextPreview,
  isStreaming,
} = await import(storeModuleUrl);
const { sendMessage } = await import(apiModuleUrl);

currentProject.value = "AgentOS";
codeContextError.value = "old warning";
codeContextPreview.value = { kind: "code", label: "old preview" };
contextAttachments.value = [
  {
    kind: "code",
    key: "smoke-context",
    label: "AgentOS, PersonalAssistant code context",
    schema: "agentos.code_context.v1",
    projects: ["AgentOS", "PersonalAssistant"],
    budget: "deep",
    maxChars: 24000,
    truncated: true,
    prompt: "CODE_CONTEXT_BODY",
  },
];

await sendMessage("Implement shared auth safely.");

if (!capturedChatStreamBody?.message) {
  throw new Error("sendMessage did not call /api/chat-stream with a message");
}

const sent = String(capturedChatStreamBody.message || "");
for (const fragment of [
  "--- ATTACHED CONTEXT",
  "kind=code",
  "projects=AgentOS, PersonalAssistant",
  "schema=agentos.code_context.v1",
  "truncated=true",
  "CODE_CONTEXT_BODY",
  "--- END ATTACHED CONTEXT ---",
  "[USER_TASK]",
  "Implement shared auth safely.",
]) {
  if (!sent.includes(fragment)) {
    throw new Error(`attached context payload missing: ${fragment}`);
  }
}

if (contextAttachments.value.length !== 0) {
  throw new Error("context attachments were not cleared after send");
}
if (codeContextError.value || codeContextPreview.value) {
  throw new Error(
    "code context preview/error state was not cleared after send",
  );
}
if (isStreaming.value) {
  throw new Error("sendMessage left chat in streaming state");
}

console.log("code context send smoke ok");
