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
  sideMessages,
  projectPlan,
  permData,
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
  loadModules,
  loadProjectPlan,
  loadGraph,
  loadSignals,
  loadPerms,
  ensureDualSession,
  loadDualSession,
} from "/api.js";
import { App } from "/views.js";

// Project change effect
effect(() => {
  const p = currentProject.value;
  sideTitle.value = p ? p + " agent" : "orchestrator";
  loadChat(p || "_orchestrator");
  if (p) {
    loadModules(p);
    loadProjectPlan(p);
  } else {
    projectPlan.value = null;
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

// Keyboard shortcuts
document.addEventListener("keydown", (e) => {
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
    loadAgents();
    loadPlan();
    loadFeed();
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
  ]);
  await loadChat("_orchestrator");
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
  await loadInbox();
  const { inboxData } = await import("/store.js");
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
