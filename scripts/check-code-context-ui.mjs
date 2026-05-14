import fs from "node:fs";

const read = (file) => fs.readFileSync(file, "utf8");
const chat = read("src-ui/chat.js");
const api = read("src-ui/api.js");
const store = read("src-ui/store.js");
const css = read("src-ui/styles/chat.css");

const checks = [
  {
    name: "frontend can load backend code context bundles",
    ok:
      api.includes("async function loadCodeContextBundle") &&
      api.includes('get_code_context_bundle"') &&
      api.includes("/api/code-context"),
  },
  {
    name: "chat exposes code context attachment state",
    ok:
      chat.includes("function CodeContextInspector") &&
      chat.includes("attachCodeContext") &&
      chat.includes("codeContextBusy") &&
      chat.includes("codeContextPreview"),
  },
  {
    name: "attached context is wrapped before send",
    ok:
      api.includes("--- ATTACHED CONTEXT") &&
      api.includes("--- END ATTACHED CONTEXT ---") &&
      api.includes("[USER_TASK]") &&
      api.includes("contextAttachments.value = []") &&
      api.includes('codeContextError.value = ""') &&
      api.includes("codeContextPreview.value = null"),
  },
  {
    name: "code context UI has visible styling",
    ok:
      css.includes(".code-context-inspector") &&
      css.includes(".context-chip.kind-code") &&
      css.includes(".route-lite-action"),
  },
  {
    name: "code context signals are exported",
    ok:
      store.includes("const codeContextBusy") &&
      store.includes("codeContextError") &&
      store.includes("codeContextPreview") &&
      store.includes("const codeContextBudget"),
  },
  {
    name: "code context has budget and multi-project scope",
    ok:
      chat.includes("CODE_CONTEXT_BUDGET_OPTIONS") &&
      chat.includes("parseCodeContextProjects") &&
      chat.includes("contextProjectDraft") &&
      css.includes(".route-lite-input") &&
      css.includes(".route-lite-select"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("code context UI checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`code context UI ok: ${checks.length} checks`);
