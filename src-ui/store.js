// AgentOS state management — all reactive signals
import { signal, effect } from "/vendor/preact-bundle.mjs";

// ===== STATE (signals — reactive, global) =====
const agents = signal([]);
const currentProject = signal(null);
const sideMessages = signal([]);
const sideTitle = signal("orchestrator");
const streamText = signal("");
const isStreaming = signal(false);
const streamChain = signal([]);
const thinkStart = signal(0);
const lastUserMsg = signal("");
const legacyModel = localStorage.getItem("agentos_model") || "";
const legacyEffort = localStorage.getItem("agentos_effort") || "";
const selectedClaudeModel = signal(
  localStorage.getItem("agentos_claude_model") ||
    (["opus", "sonnet", "haiku"].includes(legacyModel) ? legacyModel : ""),
);
const selectedClaudeEffort = signal(
  localStorage.getItem("agentos_claude_effort") ||
    (["low", "medium", "high", "max"].includes(legacyEffort)
      ? legacyEffort
      : ""),
);
const selectedCodexModel = signal(
  localStorage.getItem("agentos_codex_model") ||
    (legacyModel.startsWith("gpt-5") ? legacyModel : ""),
);
const selectedCodexEffort = signal(
  localStorage.getItem("agentos_codex_effort") || "",
);
effect(() =>
  localStorage.setItem("agentos_claude_model", selectedClaudeModel.value),
);
effect(() =>
  localStorage.setItem("agentos_claude_effort", selectedClaudeEffort.value),
);
effect(() =>
  localStorage.setItem("agentos_codex_model", selectedCodexModel.value),
);
effect(() =>
  localStorage.setItem("agentos_codex_effort", selectedCodexEffort.value),
);
const subModel = signal("sonnet"); // model for orchestrator's sub-project calls
const isRec = signal(false);
const attFiles = signal([]);
const isOn = signal(true);
const isDrag = signal(false);
const inpHist = [];
let hIdx = -1;
const hasDraft = signal(false);
const curActivity = signal("");
const lastStats = signal(null);
const showSettings = signal(false);
const showNewProject = signal(false);
const showStrategy = signal(false);
const showPlans = signal(false);
const showGraph = signal(false);
const graphData = signal(null);
const graphLevel = signal("overview");
const graphSelected = signal(null);
const plansData = signal([]);
const goals = signal([]);
const strategies = signal([]);
const activeStrategy = signal(null);
const strategyLoading = signal(false);
const permData = signal(null);
const inboxData = signal({ items: [], count: 0, needs_user: false });
const slashOpen = signal(false);
const slashItems = signal([]);
const pastedImg = signal(null);
// SLASH_CMDS imported from utils.js
const delegations = signal({});
const delegStreams = signal({}); // {id: {stage:'L1', label:'balanced', events:[]}}
const theme = signal(localStorage.getItem("theme") || "dark");
const feedItems = signal([]);
const signalsData = signal({
  signals: [],
  counts: { critical: 0, warn: 0, info: 0 },
});
const chatMode = signal("");

const toasts = signal([]);
const isLoading = signal(true);
let toastId = 0;
function showToast(msg, type = "info", ttl = 4000) {
  const id = ++toastId;
  toasts.value = [...toasts.value, { id, msg, type }];
  setTimeout(() => {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }, ttl);
}

const clock = signal(new Date().toLocaleTimeString());
const segments = signal({});
const activeFilter = signal("");
const sortBy = signal("");
const viewMode = signal(localStorage.getItem("agentos_viewmode") || "grid"); // 'grid' or 'list'
effect(() => localStorage.setItem("agentos_viewmode", viewMode.value));
const modules = signal([]);
const searchQuery = signal("");
const actionPlan = signal(null);
const queueTasks = signal([]);
const showKbHelp = signal(false);
const showPlan = signal(true);
const orchOk = signal(false);
const projectPlan = signal(null);
const activities = signal({}); // project → {action, detail, started}

const showDualAgents = signal(false);
const chatCollabMode = signal(
  localStorage.getItem("agentos_chat_collab_mode") || "solo",
);
effect(() =>
  localStorage.setItem("agentos_chat_collab_mode", chatCollabMode.value),
);
const activeRoomTab = signal(
  localStorage.getItem("agentos_active_room_tab") || "chat",
);
effect(() =>
  localStorage.setItem("agentos_active_room_tab", activeRoomTab.value),
);
const activeDualSession = signal(null);
const dualSessionData = signal(null);
const dualHistories = signal({});
const dualBusy = signal("");

// Clock interval (global, no cleanup needed for desktop app)
setInterval(() => (clock.value = new Date().toLocaleTimeString()), 1000);
// NOTE: theme + project change effects moved to app.js (they need api.js imports)

export {
  agents,
  currentProject,
  sideMessages,
  sideTitle,
  streamText,
  isStreaming,
  streamChain,
  thinkStart,
  lastUserMsg,
  selectedClaudeModel,
  selectedClaudeEffort,
  selectedCodexModel,
  selectedCodexEffort,
  subModel,
  isRec,
  attFiles,
  isOn,
  isDrag,
  hasDraft,
  curActivity,
  lastStats,
  showSettings,
  showNewProject,
  showStrategy,
  showPlans,
  showGraph,
  graphData,
  graphLevel,
  graphSelected,
  plansData,
  goals,
  strategies,
  activeStrategy,
  strategyLoading,
  permData,
  inboxData,
  slashOpen,
  slashItems,
  pastedImg,
  delegations,
  delegStreams,
  theme,
  feedItems,
  signalsData,
  toasts,
  isLoading,
  clock,
  segments,
  activeFilter,
  sortBy,
  viewMode,
  modules,
  searchQuery,
  actionPlan,
  queueTasks,
  showKbHelp,
  showPlan,
  orchOk,
  projectPlan,
  activities,
  chatMode,
  showDualAgents,
  chatCollabMode,
  activeRoomTab,
  activeDualSession,
  dualSessionData,
  dualHistories,
  dualBusy,
  showToast,
};
