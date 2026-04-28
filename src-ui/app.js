// AgentOS main entry — init + keyboard + render
import { render, html, effect } from "/vendor/preact-bundle.mjs";
import { __IS_TAURI, __invoke } from "/bridge.js";
import {
  currentProject,
  sideTitle,
  showSettings,
  showStrategy,
  showPlans,
  showDualAgents,
  chatCollabMode,
  activeRoomTab,
  activeDualSession,
  showGraph,
  graphSelected,
  showKbHelp,
  theme,
  isLoading,
  isStreaming,
  activeRun,
  activities,
  delegations,
  sideMessages,
  projectPlan,
  permData,
  inboxData,
  showToast,
} from "/store.js";
import {
  loadAgents,
  loadSegments,
  loadFeed,
  loadActivity,
  loadPlan,
  loadQueue,
  checkOrch,
  chkConn,
  loadInbox,
  loadPlansData,
  loadChat,
  loadDelegations,
  loadModules,
  loadProjectPlan,
  loadGraph,
  loadSignals,
  loadPerms,
  ensureDualSession,
  loadDualSession,
  loadActiveScope,
  loadExecutionMap,
  loadAppInfo,
} from "/api.js";
import { App } from "/views.js";
import { normalizeProjectKey, projectParam } from "/route-state.js";

function syncRecoveredActiveRun() {
  const projectKey = normalizeProjectKey(currentProject.value || "");
  const act = activities.value?.[projectKey];
  if (!act) return;
  const detail = [act.action, act.detail].filter(Boolean).join(": ");
  const startedAt = act.started
    ? Number(act.started) * 1000
    : activeRun.value?.startedAt || Date.now();
  const current = activeRun.value;
  if (
    current?.project === projectKey &&
    !["done", "failed", "cancelled"].includes(current.status || "")
  ) {
    if (detail && detail !== current.detail) {
      activeRun.value = {
        ...current,
        status: "running",
        phase: act.action || current.phase || "backend",
        detail,
        updatedAt: Date.now(),
        events: [
          ...(current.events || []),
          {
            type: "activity",
            phase: act.action || "backend",
            detail,
            receivedAt: Date.now(),
          },
        ].slice(-40),
      };
    }
    return;
  }
  activeRun.value = {
    id: "recovered-" + projectKey + "-" + Date.now(),
    project: projectKey,
    provider: "agent",
    model: "",
    effort: "",
    mode: "act",
    access: "write",
    status: "running",
    phase: act.action || "backend",
    detail: detail || "recovered running task",
    outcome: "",
    startedAt,
    updatedAt: Date.now(),
    events: [
      {
        type: "activity",
        phase: act.action || "backend",
        detail: detail || "recovered running task",
        receivedAt: Date.now(),
      },
    ],
  };
}

async function refreshCurrentRoute() {
  const p = projectParam(currentProject.value || "");
  const chatKey = normalizeProjectKey(currentProject.value || "");
  await Promise.allSettled([
    loadAgents(),
    loadActivity(),
    loadPlan(),
    loadQueue(),
    loadFeed(),
    loadSignals(),
    loadInbox(),
    loadChat(chatKey),
    p ? loadModules(p) : Promise.resolve(),
    p ? loadProjectPlan(p) : Promise.resolve(),
    loadActiveScope(p, activeDualSession.value || null),
  ]);
  showToast(p ? `refreshed ${p}` : "refreshed orchestrator", "success", 1200);
}

// Project change effect
effect(() => {
  const p = projectParam(currentProject.value || "");
  const chatKey = normalizeProjectKey(currentProject.value || "");
  sideTitle.value = p ? p + " agent" : "orchestrator";
  loadChat(chatKey);
  if (p) {
    loadModules(p);
    loadProjectPlan(p);
  } else {
    projectPlan.value = null;
  }
  loadActiveScope(p || "", activeDualSession.value || null).catch((e) =>
    console.warn("scope load failed:", e),
  );
});

// Theme effect
effect(() => {
  document.documentElement.setAttribute(
    "data-theme",
    theme.value === "light" ? "light" : "",
  );
  localStorage.setItem("theme", theme.value);
});

// Keyboard shortcuts
document.addEventListener("keydown", (e) => {
  if ((e.ctrlKey || e.metaKey) && (e.key === "r" || e.key === "R")) {
    e.preventDefault();
    refreshCurrentRoute().catch((err) =>
      console.warn("route refresh failed:", err),
    );
    return;
  }
  if (
    e.target.tagName === "INPUT" ||
    e.target.tagName === "TEXTAREA" ||
    e.target.isContentEditable
  )
    return;
  if (e.key === "Escape") {
    if (showGraph.value && graphSelected.value) {
      graphSelected.value = null;
    } else if (showGraph.value) {
      showGraph.value = false;
    } else if (showSettings.value) {
      showSettings.value = false;
    } else if (showPlans.value) {
      showPlans.value = false;
    } else if (
      chatCollabMode.value === "duo" &&
      activeRoomTab.value !== "chat"
    ) {
      activeRoomTab.value = "chat";
    } else if (showDualAgents.value) {
      showDualAgents.value = false;
    } else if (showStrategy.value) {
      showStrategy.value = false;
    } else {
      currentProject.value = null;
    }
    return;
  }
  if (e.key === "/") {
    e.preventDefault();
    document.querySelector(".ch-inp textarea")?.focus();
    return;
  }
  if (e.key === "d" || e.key === "D") {
    theme.value = theme.value === "dark" ? "light" : "dark";
    return;
  }
  if (e.key === "?") {
    showKbHelp.value = !showKbHelp.value;
    return;
  }
  if (e.key === "s" || e.key === "S") {
    e.preventDefault();
    document.querySelector(".search input")?.focus();
    return;
  }
  if (e.key === "r" || e.key === "R") {
    refreshCurrentRoute().catch((err) =>
      console.warn("route refresh failed:", err),
    );
    return;
  }
  if (e.key === "p" || e.key === "P") {
    showPlans.value = !showPlans.value;
    return;
  }
  if (e.key === "g" || e.key === "G") {
    showGraph.value = !showGraph.value;
    if (showGraph.value) loadGraph("overview");
    return;
  }
});

// Init
try {
  await Promise.all([
    loadAgents(),
    loadSegments(),
    loadFeed(),
    loadActivity(),
    loadPlan(),
    loadQueue(),
    checkOrch(),
    chkConn(),
    loadInbox(),
    loadPlansData(),
    loadSignals(),
    loadPerms(),
    loadAppInfo(),
    loadDelegations(),
    loadExecutionMap(),
  ]);
  syncRecoveredActiveRun();
  await loadChat(normalizeProjectKey(currentProject.value || ""));
} catch (e) {
  console.error("AgentOS init failed:", e);
  showDualAgents.value = false;
} finally {
  isLoading.value = false;
}

// Session separator
if (!window._sessionMarked) {
  window._sessionMarked = true;
  const sep = {
    ts: new Date().toISOString(),
    role: "system",
    msg: "Session started " + new Date().toLocaleString(),
  };
  if (sideMessages.value.length)
    sideMessages.value = [...sideMessages.value, sep];
}

// Render
try {
  render(html`<${App} />`, document.body);
} catch (e) {
  document.body.innerHTML =
    '<div style="padding:40px;font-family:monospace;color:#fff;background:#0a0a0f"><h1>Agent OS Error</h1><pre>' +
    e.message +
    '</pre><button onclick="location.reload()">Reload</button></div>';
}

// Polling
const _clockInterval = setInterval(() => {
  const d = new Date();
  document.querySelectorAll(".clock-display").forEach((el) => {
    el.textContent = d.toLocaleTimeString();
  });
}, 1000);
setInterval(async () => {
  loadAgents();
  loadActivity();
  loadPlan();
  loadFeed();
  loadSignals();
  loadDelegations();
  loadExecutionMap().catch(() => {});
  await loadInbox();
  if (inboxData.value.count > 0 && !inboxData.value.needs_user) {
    const { processInbox } = await import("/api.js");
    processInbox();
  }
}, 15000);
setInterval(() => {
  if (chatCollabMode.value === "duo" && activeDualSession.value) {
    loadDualSession(activeDualSession.value);
  }
}, 3000);

let _lastLiveProjectRefresh = 0;
setInterval(() => {
  const hasActivity = Object.keys(activities.value || {}).length > 0;
  const hasDelegation = Object.values(delegations.value || {}).some((d) =>
    ["pending", "scheduled", "running", "escalated", "deciding"].includes(
      d?.status,
    ),
  );
  if (!isStreaming.value && !hasActivity && !hasDelegation) return;
  loadActivity()
    .then(syncRecoveredActiveRun)
    .catch(() => {});
  const now = Date.now();
  if (now - _lastLiveProjectRefresh > 3000) {
    _lastLiveProjectRefresh = now;
    loadAgents().catch(() => {});
    loadFeed().catch(() => {});
    loadSignals().catch(() => {});
    loadDelegations().catch(() => {});
    loadExecutionMap().catch(() => {});
  }
}, 1000);
