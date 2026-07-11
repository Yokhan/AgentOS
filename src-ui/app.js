// AgentOS main entry — init + keyboard + render
import { render, html, effect } from "/vendor/preact-bundle.mjs";
import { __IS_TAURI, __invoke } from "/bridge.js";
import {
  currentProject,
  sideTitle,
  showSettings,
  showNewProject,
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
  safeMode,
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
  pollDelegationStreams,
  loadModules,
  loadProjectPlan,
  loadGraph,
  loadSignals,
  loadNotifications,
  loadPerms,
  ensureDualSession,
  loadDualSession,
  loadActiveScope,
  loadOrchestrationMap,
  loadExecutionMap,
  loadOperationSnapshot,
  loadAppInfo,
  recordUiDiagnostic,
} from "/api.js";
import { App } from "/views.js";
import { normalizeProjectKey, projectParam } from "/route-state.js";
import { installUiDiagnostics, setSafeModeLocal } from "/resilience.js";

installUiDiagnostics({ recordRemote: recordUiDiagnostic });
window.__AGENTOS_SAFE_MODE__ = !!safeMode.value;
window.__AGENTOS_SET_SAFE_MODE__ = (enabled, reload = true) => {
  safeMode.value = !!enabled;
  setSafeModeLocal(!!enabled, reload);
};

function escapeHtml(value) {
  return String(value || "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function renderStartupError(error, phase = "render") {
  const stack =
    error?.stack || error?.message || String(error || "Unknown error");
  const report = {
    phase,
    version: window.__AGENTOS_VERSION__ || "unknown",
    route: {
      project: currentProject.value || "_orchestrator",
      strategy: !!showStrategy.value,
      plans: !!showPlans.value,
      graph: !!showGraph.value,
      duo: chatCollabMode.value || "solo",
      roomTab: activeRoomTab.value || "chat",
    },
    activeRun: activeRun.value || null,
    error: stack,
    ts: new Date().toISOString(),
  };
  const encodedReport = escapeHtml(JSON.stringify(report, null, 2));
  document.body.innerHTML = `
    <div style="min-height:100vh;padding:32px;background:#070b11;color:#f4f7fb;font-family:ui-monospace,SFMono-Regular,Consolas,monospace">
      <h1 style="margin:0 0 8px 0;font-size:22px">Agent OS startup error</h1>
      <div style="color:#9fb2c7;margin-bottom:16px">Phase: ${escapeHtml(phase)}. The broken view can be bypassed without losing chat files.</div>
      <div style="display:flex;gap:8px;flex-wrap:wrap;margin-bottom:16px">
        <button onclick="location.reload()">Reload</button>
        <button onclick="localStorage.removeItem('agentos_current_project');localStorage.setItem('agentos_active_room_tab','chat');location.reload()">Open main</button>
        <button onclick="navigator.clipboard&&navigator.clipboard.writeText(document.querySelector('#boot-report').textContent)">Copy report</button>
        <button onclick="localStorage.setItem('agentos_active_room_tab','chat');localStorage.removeItem('agentos_workspace_tab');location.reload()">Disable broken view</button>
        <button onclick="localStorage.setItem('agentos_safe_mode','1');localStorage.setItem('agentos_active_room_tab','chat');location.reload()">Enable safe mode</button>
      </div>
      <pre style="white-space:pre-wrap;word-break:break-word;background:#0d1520;border:1px solid #223247;padding:16px;border-radius:10px;max-height:42vh;overflow:auto">${escapeHtml(stack)}</pre>
      <details style="margin-top:16px" open>
        <summary style="cursor:pointer;color:#9fd0ff">diagnostic report</summary>
        <pre id="boot-report" style="white-space:pre-wrap;word-break:break-word;background:#0d1520;border:1px solid #223247;padding:16px;border-radius:10px;max-height:32vh;overflow:auto">${encodedReport}</pre>
      </details>
    </div>`;
}

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
    loadNotifications(),
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
  if (!safeMode.value) {
    Promise.allSettled([
      loadExecutionMap(p || "", activeDualSession.value || null),
      loadOrchestrationMap(p || "", activeDualSession.value || null),
    ]).then((results) => {
      results
        .filter((result) => result.status === "rejected")
        .forEach((result) =>
          console.warn("project execution state load failed:", result.reason),
        );
    });
  }
});

// Theme effect
effect(() => {
  document.documentElement.setAttribute(
    "data-theme",
    theme.value === "light" ? "light" : "",
  );
  localStorage.setItem("theme", theme.value);
});

effect(() => {
  window.__AGENTOS_SAFE_MODE__ = !!safeMode.value;
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
  if (e.key === "Escape") {
    if (showNewProject.value) {
      showNewProject.value = false;
    } else if (showKbHelp.value) {
      showKbHelp.value = false;
    } else if (showGraph.value && graphSelected.value) {
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
  if (
    e.target.tagName === "INPUT" ||
    e.target.tagName === "TEXTAREA" ||
    e.target.isContentEditable
  )
    return;
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
    if (safeMode.value) {
      showToast("Graph is disabled in safe mode", "warn", 2500);
      return;
    }
    showGraph.value = !showGraph.value;
    if (showGraph.value) loadGraph("overview");
    return;
  }
});

function startupTask(label, fn, timeoutMs = 8000) {
  const started = performance.now();
  const task = Promise.resolve()
    .then(fn)
    .then((value) => ({
      label,
      status: "fulfilled",
      value,
      durationMs: Math.round(performance.now() - started),
    }))
    .catch((error) => ({
      label,
      status: "rejected",
      reason: error?.stack || error?.message || String(error),
      durationMs: Math.round(performance.now() - started),
    }));
  let timer = null;
  const timeout = new Promise((resolve) => {
    timer = setTimeout(
      () =>
        resolve({
          label,
          status: "timeout",
          reason: `startup task timed out after ${timeoutMs}ms`,
          durationMs: timeoutMs,
        }),
      timeoutMs,
    );
  });
  return Promise.race([task, timeout]).finally(() => clearTimeout(timer));
}

function markSessionStarted() {
  if (window._sessionMarked) return;
  window._sessionMarked = true;
  const sep = {
    ts: new Date().toISOString(),
    role: "system",
    msg: "Session started " + new Date().toLocaleString(),
  };
  if (sideMessages.value.length)
    sideMessages.value = [...sideMessages.value, sep];
}

async function runStartupLoad() {
  window.__AGENTOS_DEFERRED_STARTUP__ = {};
  const deferStartupTask = (label, fn, timeoutMs) => {
    window.__AGENTOS_DEFERRED_STARTUP__[label] = {
      status: "running",
      startedAt: performance.now(),
    };
    const task = startupTask(label, fn, timeoutMs).then((result) => {
      window.__AGENTOS_DEFERRED_STARTUP__[label] = result;
      if (result.status !== "fulfilled") {
        window.__AGENTOS_INIT_WARNINGS__ = [
          ...(window.__AGENTOS_INIT_WARNINGS__ || []),
          result,
        ];
        console.warn(`AgentOS deferred ${label} skipped:`, result);
      }
      return result;
    });
    return task;
  };
  deferStartupTask("loadAgents", loadAgents, 15000).then((result) => {
    if (result.status === "fulfilled") {
      deferStartupTask("loadPlan", loadPlan, 8000);
    }
  });
  deferStartupTask("loadPerms", loadPerms, 15000);
  const tasks = [
    ["loadSegments", loadSegments],
    ["loadFeed", loadFeed],
    ["loadActivity", loadActivity],
    ["loadQueue", loadQueue],
    ["checkOrch", checkOrch],
    ["chkConn", chkConn],
    ["loadInbox", loadInbox],
    ["loadPlansData", loadPlansData],
    ["loadSignals", loadSignals],
    ["loadNotifications", loadNotifications],
    ["loadAppInfo", loadAppInfo],
    ["loadDelegations", loadDelegations],
  ];
  try {
    const results = await Promise.all(
      tasks.map(([label, fn, timeout]) => startupTask(label, fn, timeout)),
    );
    window.__AGENTOS_STARTUP_RESULTS__ = results;
    const warnings = results.filter((r) => r.status !== "fulfilled");
    if (warnings.length) {
      window.__AGENTOS_INIT_WARNINGS__ = warnings;
      console.warn("AgentOS startup partial load:", warnings);
    }
    syncRecoveredActiveRun();
    const chatResult = await startupTask(
      "loadChat",
      () => loadChat(normalizeProjectKey(currentProject.value || "")),
      8000,
    );
    if (chatResult.status !== "fulfilled") {
      window.__AGENTOS_INIT_WARNINGS__ = [
        ...(window.__AGENTOS_INIT_WARNINGS__ || []),
        chatResult,
      ];
      console.warn("AgentOS startup chat load skipped:", chatResult);
    }
    markSessionStarted();
    if (!safeMode.value) {
      setTimeout(() => {
        Promise.allSettled([
          loadExecutionMap(),
          loadOrchestrationMap(),
          loadOperationSnapshot(),
        ]).catch((e) => console.warn("deferred execution state load failed:", e));
      }, 1200);
    }
  } catch (e) {
    console.error("AgentOS init failed:", e);
    showDualAgents.value = false;
    window.__AGENTOS_INIT_ERROR__ = e?.stack || e?.message || String(e);
  } finally {
    isLoading.value = false;
    window.__AGENTOS_READY_AT__ = performance.now();
    startPolling();
  }
}

// Render
try {
  render(html`<${App} />`, document.body);
} catch (e) {
  renderStartupError(e, "render");
}

runStartupLoad().catch((e) => {
  console.error("AgentOS startup runner failed:", e);
  window.__AGENTOS_INIT_ERROR__ = e?.stack || e?.message || String(e);
  isLoading.value = false;
  startPolling();
});

// Polling
let _lastLiveProjectRefresh = 0;
let _pollingStarted = false;
let _baselinePollInFlight = false;
let _livePollInFlight = false;
const BASELINE_REFRESH_MS = 30000;
const LIVE_HEAVY_REFRESH_MS = 30000;
function isComposerElementActive() {
  const active = document.activeElement;
  return !!(active && active.closest && active.closest(".ch-inp"));
}

function shouldDeferHeavyPolling() {
  const activeUntil = Number(window.__AGENTOS_COMPOSER_ACTIVE_UNTIL || 0);
  return isComposerElementActive() || Date.now() < activeUntil;
}

function startPolling() {
  if (_pollingStarted) return;
  _pollingStarted = true;
  setInterval(async () => {
    if (_baselinePollInFlight) return;
    _baselinePollInFlight = true;
    try {
      const deferHeavy = shouldDeferHeavyPolling() || safeMode.value;
      // Plan consumes the same repository snapshot; refresh it after agents so both calls
      // never contend for the expensive scan lock on an IPC thread.
      await loadAgents();
      await Promise.allSettled([
        chkConn(),
        loadActivity(),
        loadPlan(),
        loadFeed(),
        loadSignals(),
        loadNotifications(),
        loadDelegations(),
        ...(deferHeavy
          ? []
          : [
              loadExecutionMap(),
              loadOrchestrationMap(),
              loadOperationSnapshot(),
            ]),
        loadInbox(),
      ]);
      if (inboxData.value.count > 0 && !inboxData.value.needs_user) {
        const { processInbox } = await import("/api.js");
        processInbox();
      }
    } finally {
      _baselinePollInFlight = false;
    }
  }, BASELINE_REFRESH_MS);
  setInterval(() => {
    if (chatCollabMode.value === "duo" && activeDualSession.value) {
      loadDualSession(activeDualSession.value);
    }
  }, 3000);
  setInterval(async () => {
    if (_livePollInFlight) return;
    const hasActivity = Object.keys(activities.value || {}).length > 0;
    const hasDelegation = Object.values(delegations.value || {}).some((d) =>
      ["pending", "scheduled", "running", "escalated", "deciding"].includes(
        d?.status,
      ),
    );
    if (!isStreaming.value && !hasActivity && !hasDelegation) return;
    _livePollInFlight = true;
    try {
      await loadActivity();
      syncRecoveredActiveRun();
      const now = Date.now();
      const deferHeavy = shouldDeferHeavyPolling() || safeMode.value;
      const refreshHeavy =
        !deferHeavy && now - _lastLiveProjectRefresh > LIVE_HEAVY_REFRESH_MS;
      if (refreshHeavy) _lastLiveProjectRefresh = now;
      await Promise.allSettled([
        ...(deferHeavy ? [] : [pollDelegationStreams()]),
        ...(refreshHeavy
          ? [
              loadDelegations(),
              loadExecutionMap(),
              loadOrchestrationMap(),
              loadOperationSnapshot(),
            ]
          : []),
      ]);
    } finally {
      _livePollInFlight = false;
    }
  }, 2000);
}
