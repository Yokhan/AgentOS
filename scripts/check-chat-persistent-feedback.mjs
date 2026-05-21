import fs from "node:fs";

const api = fs.readFileSync("src-ui/api.js", "utf8");
const chat = fs.readFileSync("src-ui/chat.js", "utf8");
const stream = fs.readFileSync("src-tauri/src/commands/chat_stream.rs", "utf8");
const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));

const checks = [
  {
    name: "backend persists PA feedback into chat history",
    ok:
      stream.includes('"kind": "pa_feedback"') &&
      stream.includes('"pa_type": event_type') &&
      stream.includes("append_jsonl_logged("),
  },
  {
    name: "frontend renders orphan PA feedback instead of dropping it",
    ok:
      api.includes("function appendOrShowPaFeedback") &&
      api.includes('kind: "pa_feedback_notice"') &&
      !api.includes("if (m.kind === \"pa_feedback\") {\n        const prev"),
  },
  {
    name: "routine system filter keeps warnings and confirmations visible",
    ok:
      chat.includes("m.kind !== \"pa_feedback_notice\"") &&
      chat.includes("needs user") &&
      chat.includes("approve") &&
      chat.includes("confirm") &&
      chat.includes("return false;"),
  },
  {
    name: "gate is registered in check:ui",
    ok: pkg.scripts?.["check:ui"]?.includes("check-chat-persistent-feedback.mjs"),
  },
];

const failed = checks.filter((check) => !check.ok);
if (failed.length) {
  console.error("chat persistent feedback checks failed:");
  failed.forEach((check) => console.error("  - " + check.name));
  process.exit(1);
}

console.log(`chat persistent feedback ok: ${checks.length} checks`);
