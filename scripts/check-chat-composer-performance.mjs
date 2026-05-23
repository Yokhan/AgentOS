import fs from "node:fs";

const chat = fs.readFileSync("src-ui/chat.js", "utf8");
const api = fs.readFileSync("src-ui/api.js", "utf8");
const app = fs.readFileSync("src-ui/app.js", "utf8");

const checks = [
  {
    name: "composer text is not ChatPanel state",
    ok: !chat.includes("const [composerText, setComposerText]"),
  },
  {
    name: "textarea input does not rerender ChatPanel on every keystroke",
    ok:
      !chat.includes("setComposerText(e.target.value)") &&
      chat.includes("composerTextRef.current = e.target.value") &&
      chat.includes("scheduleComposerPreviewRefresh()"),
  },
  {
    name: "composer preview refresh is debounced",
    ok:
      chat.includes("composerPreviewTimer") &&
      chat.includes("setTimeout(refreshComposerPreview, 180)"),
  },
  {
    name: "composer focus marks input-critical section",
    ok:
      chat.includes("markComposerInteraction") &&
      chat.includes("onFocus=${markComposerInteraction}") &&
      chat.includes("onPointerDown=${markComposerInteraction}"),
  },
  {
    name: "textarea disables browser assist that can stall WebView focus",
    ok:
      chat.includes('spellcheck="false"') &&
      chat.includes('autocomplete="off"') &&
      chat.includes('autocorrect="off"') &&
      chat.includes('autocapitalize="off"'),
  },
  {
    name: "draft loading is capped before assigning textarea value",
    ok:
      api.includes("MAX_DRAFT_CHARS") &&
      api.includes("trimDraftText") &&
      api.includes("ta.value = text"),
  },
  {
    name: "heavy UI loaders are coalesced",
    ok:
      api.includes("coalesceLoad") &&
      api.includes("executionMap:${project") &&
      api.includes("orchestrationMap:${project") &&
      api.includes("executionTimeline:${project"),
  },
  {
    name: "heavy polling is deferred while composer is active",
    ok:
      app.includes("shouldDeferHeavyPolling") &&
      app.includes("__AGENTOS_COMPOSER_ACTIVE_UNTIL") &&
      app.includes("deferHeavy ? []"),
  },
  {
    name: "delegation polling does not replace equal snapshots",
    ok:
      api.includes("compactDelegationsSignature") &&
      api.includes("if (signature === _delegationsSignature) return;"),
  },
  {
    name: "activity polling does not replace equal snapshots",
    ok:
      api.includes("compactActivitySignature") &&
      api.includes("if (signature !== _activitySignature)"),
  },
  {
    name: "delegation stream polling writes only on new events",
    ok:
      api.includes("let changed = false") &&
      api.includes("if (changed) delegStreams.value = updated;"),
  },
  {
    name: "live polling cannot overlap when backend is slow",
    ok:
      app.includes("_livePollInFlight") &&
      app.includes("if (_livePollInFlight) return;"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("chat composer performance checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`chat composer performance checks ok: ${checks.length} checks`);
