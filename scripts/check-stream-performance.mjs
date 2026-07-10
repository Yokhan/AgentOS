import fs from "node:fs";

const read = (file) => fs.readFileSync(file, "utf8");
const api = read("src-ui/api.js");
const app = read("src-ui/app.js");
const chat = read("src-ui/chat.js");
const stream = read("src-tauri/src/commands/chat_stream.rs");
const poll = read("src-tauri/src/commands/chat_stream_poll.rs");
const ops = read("src-tauri/src/commands/operation_state.rs");
const views = read("src-ui/views.js");

function functionSource(source, name) {
  const start = source.indexOf(`function ${name}(`);
  if (start < 0) return "";
  const next = source.indexOf("\nfunction ", start + 1);
  return source.slice(start, next < 0 ? source.length : next);
}

const executionFlowStage = functionSource(views, "ExecutionFlowStage");

const checks = [
  {
    name: "poll_stream reads from byte offset",
    ok:
      poll.includes("SeekFrom::Start(safe_offset as u64)") &&
      poll.includes("byte_offset"),
  },
  {
    name: "text_delta stream does not persist full accumulated text",
    ok:
      !stream.includes('"full": full_text') &&
      stream.includes('"text_len": full_text.len()'),
  },
  {
    name: "frontend appends delta text when full is absent",
    ok: api.includes('full + (evt.text || "")'),
  },
  {
    name: "stream UI commits are batched",
    ok:
      api.includes("flushStreamView") &&
      api.includes("markChainDirty") &&
      api.includes("persistDraftMaybe"),
  },
  {
    name: "chat map refresh is not tied to every stream block",
    ok: !/loadExecutionMap[\s\S]{0,700}streamChain\.value\.length/.test(chat),
  },
  {
    name: "live polling avoids heavy dashboard reloads while streaming",
    ok:
      app.includes("LIVE_HEAVY_REFRESH_MS = 30000") &&
      app.includes("now - _lastLiveProjectRefresh > LIVE_HEAVY_REFRESH_MS") &&
      !/now - _lastLiveProjectRefresh > LIVE_HEAVY_REFRESH_MS[\s\S]{0,260}loadAgents/.test(
        app,
      ),
  },
  {
    name: "main execution map has no component-owned polling interval",
    ok:
      executionFlowStage.length > 0 &&
      !/(setInterval|setTimeout)/.test(executionFlowStage),
  },
  {
    name: "operation snapshots are compact",
    ok:
      ops.includes("MAX_EVENTS_IN_SNAPSHOT_OPERATION") &&
      ops.includes("MAX_OPERATIONS_IN_SNAPSHOT"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("stream performance checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`stream performance checks ok: ${checks.length} checks`);
