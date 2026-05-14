import fs from "node:fs";

const chat = fs.readFileSync("src-ui/chat.js", "utf8");
const css = fs.readFileSync("src-ui/styles/chat.css", "utf8");
const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));

const checks = [
  {
    name: "approve/reject refreshes execution map",
    ok:
      chat.includes("const refreshAfter") &&
      chat.includes("refreshAfter(approveDel(item.id))") &&
      chat.includes("refreshAfter(rejectDel(item.id))"),
  },
  {
    name: "incomplete map is actionable",
    ok:
      chat.includes("const askMapRepair") &&
      chat.includes("Drafted execution-map repair prompt") &&
      chat.includes("exec-map-warning-actions"),
  },
  {
    name: "waiting cards expose details",
    ok:
      chat.includes("exec-map-waiting-details") &&
      chat.includes("id: ${item.id") &&
      css.includes(".exec-map-waiting-details"),
  },
  {
    name: "selected waiting item has a readable detail panel",
    ok:
      chat.includes("selectedWaitingId") &&
      chat.includes("openWaitingDetails") &&
      chat.includes("exec-map-waiting-detail-panel") &&
      chat.includes("exec-map-waiting-detail-actions") &&
      css.includes(".exec-map-waiting-detail-panel") &&
      css.includes(".exec-map-waiting-detail-grid"),
  },
  {
    name: "execution map lanes expose owner role",
    ok:
      chat.includes("function executionLaneOwnerLabel") &&
      chat.includes("executionLaneOwnerLabel(lane)") &&
      chat.includes("exec-map-lane-owner") &&
      css.includes(".exec-map-lane-owner"),
  },
  {
    name: "retry/status draft refreshes execution map",
    ok:
      chat.includes("const draftMapCommand") &&
      chat.includes("draftMapCommand(") &&
      chat.includes("DELEGATE_RETRY") &&
      chat.includes("DELEGATE_STATUS"),
  },
  {
    name: "heartbeat/provider state stays out of map event nodes",
    ok:
      chat.includes("function isProviderStateEvent") &&
      chat.includes("provider_heartbeat") &&
      chat.includes("event?.semantic === false") &&
      chat.includes(
        "const visibleRawEvents = rawEvents.filter(isRenderableMapEvent)",
      ) &&
      chat.includes("смысловых событий нет"),
  },
  {
    name: "code context state clears after send",
    ok:
      chat.includes("codeContextPreview.value = null") &&
      fs
        .readFileSync("src-ui/api.js", "utf8")
        .includes("codeContextPreview.value = null"),
  },
  {
    name: "execution map heartbeat behavior smoke is registered",
    ok:
      fs.existsSync("scripts/smoke-execution-map-noise.mjs") &&
      pkg.scripts?.["check:ui"]?.includes("smoke-execution-map-noise.mjs"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("execution map UX checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`execution map UX ok: ${checks.length} checks`);
