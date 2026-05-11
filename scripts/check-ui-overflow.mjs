import fs from "node:fs";

const chat = fs.readFileSync("src-ui/chat.js", "utf8");
const chatCss = fs.readFileSync("src-ui/styles/chat.css", "utf8");
const toolCss = fs.readFileSync("src-ui/styles/toolcards.css", "utf8");

function assertHas(source, needle, label) {
  if (!source.includes(needle)) {
    console.error(`UI overflow gate failed: missing ${label}`);
    process.exit(1);
  }
}

assertHas(chat, "CHAT_RENDER_RECENT_LIMIT", "transcript render cap");
assertHas(chat, "PA_TRACE_COMPACT_ROW_LIMIT", "compact PA trace row cap");
assertHas(chat, "groupWaitingItems", "execution-map waiting grouping");
assertHas(chat, "visibleWaitingGroups", "bounded waiting group rendering");
assertHas(chat, "compactPaTrace=", "compact PA trace propagation");
assertHas(chatCss, ".chat-render-window", "render-window overflow notice");
assertHas(chatCss, ".exec-map-waiting-grid", "grouped waiting grid styles");
assertHas(chatCss, "height: clamp(560px, calc(100vh - 260px), 880px)", "bounded stage map viewport");
assertHas(toolCss, ".run-card.compact", "compact run-card styles");
assertHas(toolCss, ".run-show-more", "compact trace expansion control");

console.log("UI overflow gate passed");
