// AgentOS API layer — data fetching, message sending, delegation approval
import {} from "/vendor/preact-bundle.mjs";
import { beep, SLASH_CMDS } from "/utils.js";
import { __IS_TAURI, __invoke } from "/bridge.js";
import {
  agents,
  currentProject,
  sideMessages,
  streamText,
  isStreaming,
  streamChain,
  thinkStart,
  lastUserMsg,
  selectedClaudeModel,
  selectedClaudeEffort,
  selectedCodexModel,
  selectedCodexEffort,
  selectedSoloProvider,
  isRec,
  attFiles,
  isOn,
  isDrag,
  hasDraft,
  curActivity,
  lastStats,
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
  feedItems,
  signalsData,
  segments,
  modules,
  actionPlan,
  queueTasks,
  orchOk,
  projectPlan,
  activities,
  chatMode,
  showGraph,
  graphData,
  graphLevel,
  graphSelected,
  showDualAgents,
  activeDualSession,
  dualSessionData,
  dualHistories,
  dualBusy,
  activeScope,
  showToast,
} from "/store.js";
import { normalizeSoloSelection } from "/provider-caps.js";
// ===== API =====

let recog = null;
function togVoice() {
  if (
    !("webkitSpeechRecognition" in window) &&
    !("SpeechRecognition" in window)
  ) {
    showToast("No speech API", "error");
    return;
  }
  if (isRec.value) {
    recog?.stop();
    isRec.value = false;
    return;
  }
  const S = window.SpeechRecognition || window.webkitSpeechRecognition;
  recog = new S();
  recog.continuous = false;
  recog.interimResults = false;
  recog.lang = navigator.language || "en-US";
  recog.onresult = (e) => {
    const t = e.results[0][0].transcript;
    const ta = document.querySelector(".ch-inp textarea");
    if (ta) {
      ta.value = (ta.value ? ta.value + " " : "") + t;
      ta.dispatchEvent(new Event("input"));
    }
  };
  recog.onend = () => (isRec.value = false);
  recog.onerror = (e) => {
    isRec.value = false;
  };
  recog.start();
  isRec.value = true;
}
async function hdlFiles(fl) {
  const nf = [...attFiles.value];
  for (const f of fl) {
    if (__IS_TAURI) {
      try {
        const buf = await f.arrayBuffer();
        const data = Array.from(new Uint8Array(buf));
        const r = await __invoke("save_attachment", { name: f.name, data });
        nf.push({ name: f.name, size: f.size, path: r.path || "" });
      } catch (e) {
        showToast("Attach error: " + e, "error");
      }
    } else {
      nf.push({ name: f.name, size: f.size, path: "" });
    }
  }
  attFiles.value = nf;
}
function rmFile(i) {
  attFiles.value = attFiles.value.filter((_, j) => j !== i);
}
async function chkConn() {
  try {
    const r = await fetch("/api/health");
    isOn.value = r.ok;
  } catch {
    isOn.value = false;
  }
}
setInterval(chkConn, 15000);
let draftT = null;
function saveDr() {
  const ta = document.querySelector(".ch-inp textarea");
  if (ta?.value?.trim()) {
    localStorage.setItem("dr_" + (currentProject.value || "o"), ta.value);
    hasDraft.value = true;
  } else hasDraft.value = false;
}
function loadDr() {
  const d = localStorage.getItem("dr_" + (currentProject.value || "o"));
  if (d) {
    const ta = document.querySelector(".ch-inp textarea");
    if (ta) {
      ta.value = d;
      hasDraft.value = true;
    }
  }
}
function clrDr() {
  localStorage.removeItem("dr_" + (currentProject.value || "o"));
  hasDraft.value = false;
}

function normalizedSoloProviderSelection() {
  const configuredProvider =
    permData.value?.provider_status?.roles?.orchestrator_provider || "claude";
  const explicitProvider = ["claude", "codex"].includes(
    selectedSoloProvider.value,
  )
    ? selectedSoloProvider.value
    : "";
  const soloProvider = explicitProvider || configuredProvider;
  return {
    provider: soloProvider,
    ...normalizeSoloSelection(
      soloProvider,
      soloProvider === "codex"
        ? selectedCodexModel.value
        : selectedClaudeModel.value,
      soloProvider === "codex"
        ? selectedCodexEffort.value
        : selectedClaudeEffort.value,
    ),
  };
}
// beep() imported from utils.js

function handleSlash(val) {
  if (val.startsWith("/")) {
    const q = val.slice(1).toLowerCase();
    const m = SLASH_CMDS.filter((c) => c.cmd.slice(1).includes(q));
    slashItems.value = m;
    slashOpen.value = m.length > 0;
  } else slashOpen.value = false;
}
function execSlash(cmd) {
  slashOpen.value = false;
  if (cmd === "/clear") {
    sideMessages.value = [];
    showToast("Cleared", "success");
    return true;
  }
  if (cmd === "/help") {
    showToast(
      SLASH_CMDS.map((c) => c.cmd + " - " + c.desc).join(", "),
      "info",
      8000,
    );
    return true;
  }
  if (cmd === "/briefing") {
    fetch("/api/digest")
      .then((r) => r.json())
      .then((d) => showToast(d.text, "info", 8000));
    return true;
  }
  if (cmd === "/status") {
    loadActivity();
    showToast("Status refreshed", "success");
    return true;
  }
  if (cmd === "/health") {
    if (__IS_TAURI)
      __invoke("health_check", { project: "all" }).then((r) =>
        showToast(JSON.stringify(r).substring(0, 300), "info", 8000),
      );
    return true;
  }
  if (cmd.startsWith("/mode-")) {
    const mode = cmd.replace("/mode-", "");
    chatMode.value = mode;
    showToast(
      "Mode: " + mode + " (next message will include mode prefix)",
      "success",
      3000,
    );
    return true;
  }
  return false;
}
function handlePaste(e) {
  const items = e.clipboardData?.items;
  if (!items) return;
  for (const item of items) {
    if (item.type.startsWith("image/")) {
      e.preventDefault();
      const blob = item.getAsFile();
      const r = new FileReader();
      r.onload = () => {
        pastedImg.value = { data: r.result, name: "screenshot.png" };
      };
      r.readAsDataURL(blob);
    }
  }
}
async function loadAgents() {
  try {
    const r = await fetch("/api/agents");
    if (!r.ok) throw new Error("API error " + r.status);
    const d = await r.json();
    if (d.agents) agents.value = d.agents;
    if (d.error) showToast(d.error, "error");
  } catch (e) {
    showToast("Cannot reach dashboard server", "error");
  }
}
async function checkOrch() {
  try {
    const r = await fetch("/api/health");
    const d = await r.json();
    orchOk.value = !!d.orchestrator;
  } catch (e) {
    console.warn("checkOrch:", e);
  }
}
async function loadSegments() {
  try {
    const r = await fetch("/api/segments");
    const d = await r.json();
    if (d.segments) segments.value = d.segments;
  } catch (e) {
    console.warn("loadSegments:", e);
  }
}
let _chatLoadId = 0;
const PA_COMMAND_TOKEN = /\[[A-Z][A-Z0-9_]*(?::[^\]]*)?\]/;

function messageContainsPaCommand(msg) {
  return PA_COMMAND_TOKEN.test(msg || "");
}

function legacyPaFeedbackType(msg) {
  const text = msg || "";
  if (/^(Running|Completed)\s+\[[A-Z][A-Z0-9_]*(?::[^\]]*)?\]/i.test(text)) {
    return "pa_status";
  }
  if (/warning|malformed|invalid|error/i.test(text)) {
    return "warning";
  }
  return "pa_result";
}

function appendPaFeedbackTo(prev, type, text, command = "") {
  prev.chain = prev.chain?.length
    ? [...prev.chain]
    : prev.msg
      ? [{ type: "text", text: prev.msg }]
      : [];
  prev.chain.push({
    type,
    text: text || "",
    command: command || "",
  });
}

async function loadChat(p) {
  const myId = ++_chatLoadId;
  try {
    const r = await fetch("/api/chat/" + encodeURIComponent(p));
    if (myId !== _chatLoadId) return; // stale — another loadChat started
    const d = await r.json();
    const rawMsgs = (d.messages || []).map((m) => {
      if (m.tools && m.tools.length && !m.chain) {
        const chain = [];
        for (const t of m.tools) {
          chain.push({
            type: "tool",
            tool: t.tool,
            input: t.input || {},
            status: "complete",
          });
        }
        if (m.msg) chain.push({ type: "text", text: m.msg });
        m.chain = chain;
      }
      return m;
    });
    const msgs = [];
    for (const m of rawMsgs) {
      if (m.kind === "pa_feedback") {
        const prev = msgs[msgs.length - 1];
        if (prev && prev.role === "assistant") {
          appendPaFeedbackTo(
            prev,
            m.pa_type || "pa_result",
            m.msg || "",
            m.pa_command || "",
          );
          continue;
        }
      }
      if (m.role === "system") {
        const prev = msgs[msgs.length - 1];
        const prevIsPaTurn =
          prev &&
          prev.role === "assistant" &&
          (messageContainsPaCommand(prev.msg) ||
            (prev.chain || []).some((b) =>
              ["pa_status", "pa_result", "warning"].includes(b.type),
            ));
        if (prevIsPaTurn) {
          appendPaFeedbackTo(prev, legacyPaFeedbackType(m.msg), m.msg || "");
          continue;
        }
      }
      msgs.push(m);
    }
    sideMessages.value = msgs;
    const newDel = { ...delegations.value };
    for (const m of msgs) {
      const dms = [
        ...(m.msg || "").matchAll(
          /<delegation id="([^"]+)" project="([^"]+)"\/>/g,
        ),
      ];
      for (const dm of dms) {
        if (!newDel[dm[1]]) {
          newDel[dm[1]] = {
            project: dm[2],
            status: m.msg.includes("✓") ? "done" : "pending",
          };
        }
      }
    }
    delegations.value = newDel;
  } catch (e) {
    if (myId === _chatLoadId) {
      console.warn("loadChat:", e);
      sideMessages.value = [];
    }
  }
}
async function loadModules(p) {
  try {
    const r = await fetch("/api/modules/" + encodeURIComponent(p));
    const d = await r.json();
    modules.value = d.modules || [];
  } catch (e) {
    console.warn("loadModules:", e);
    modules.value = [];
  }
}
async function loadPlan() {
  try {
    const r = await fetch("/api/plan");
    const d = await r.json();
    actionPlan.value = d;
  } catch (e) {
    console.warn("loadPlan:", e);
  }
}
async function loadQueue() {
  try {
    if (__IS_TAURI) {
      const r = await __invoke("get_queue");
      queueTasks.value = r.tasks || [];
    }
  } catch (e) {
    console.warn("loadQueue:", e);
  }
}
async function loadGoals() {
  try {
    if (__IS_TAURI) {
      const r = await __invoke("get_goals");
      goals.value = r.goals || [];
    }
  } catch (e) {
    console.warn("loadGoals:", e);
  }
}
async function loadGraph(level = "overview") {
  graphLevel.value = level || "overview";
  graphSelected.value = null;
  graphData.value = null;
  try {
    graphData.value =
      level === "overview"
        ? await __invoke("get_overview_graph")
        : await __invoke("get_project_graph", { project: level });
    showGraph.value = true;
  } catch (e) {
    graphData.value = { error: String(e) };
  }
}
async function loadSignals() {
  if (!__IS_TAURI) {
    signalsData.value = {
      signals: [],
      counts: { critical: 0, warn: 0, info: 0 },
    };
    return;
  }
  try {
    const result = await __invoke("get_signals");
    signalsData.value = result || {
      signals: [],
      counts: { critical: 0, warn: 0, info: 0 },
    };
  } catch (e) {
    console.warn("loadSignals:", e);
  }
}
async function ackSignal(id) {
  if (!__IS_TAURI || !id) {
    return;
  }
  try {
    await __invoke("ack_signal", { id });
    await loadSignals();
  } catch (e) {
    showToast("Signal ack error: " + e, "error");
  }
}
async function loadStrategies() {
  try {
    if (__IS_TAURI) {
      const r = await __invoke("get_strategies");
      strategies.value = r.strategies || [];
      loadActiveScope().catch((e) => console.warn("scope refresh:", e));
    }
  } catch (e) {
    console.warn("loadStrategies:", e);
  }
}
async function loadPlansData() {
  try {
    if (__IS_TAURI) {
      const r = await __invoke("get_plans");
      plansData.value = r.plans || [];
      loadActiveScope().catch((e) => console.warn("scope refresh:", e));
    }
  } catch (e) {
    console.warn("loadPlans:", e);
  }
}

async function loadActiveScope(
  project = currentProject.value || "",
  sessionId = activeDualSession.value || null,
) {
  const fallbackProject = project || "";
  if (!__IS_TAURI) {
    activeScope.value = {
      kind: fallbackProject ? "project" : "global",
      label: fallbackProject ? "Project" : "Global",
      title: fallbackProject || "_orchestrator",
      project: fallbackProject,
      breadcrumbs: [
        { kind: "global", label: "Global" },
        ...(fallbackProject
          ? [{ kind: "project", label: fallbackProject }]
          : []),
      ],
      available_actions: [
        { id: "ask_both", label: "Ask both", tone: "neutral" },
      ],
      summary: fallbackProject
        ? `Duo actions apply to project: ${fallbackProject}`
        : "Duo is operating at global orchestration level.",
    };
    return activeScope.value;
  }
  const res = await __invoke("get_active_scope", {
    project: fallbackProject || null,
    roomSessionId: sessionId || null,
  });
  if (res?.status === "ok" && res.scope) {
    activeScope.value = res.scope;
    return res.scope;
  }
  throw new Error(res?.error || "Cannot resolve active scope");
}
async function generateStrategy(goalText, ctx, roomSessionId = null) {
  strategyLoading.value = true;
  try {
    const r = await __invoke("generate_strategy", {
      goal: goalText,
      context: ctx || null,
      roomSessionId: roomSessionId || null,
    });
    if (r.status === "ok" && r.strategy) {
      strategies.value = [...strategies.value, r.strategy];
      activeStrategy.value = r.strategy;
      showToast("Strategy generated", "success");
    } else {
      showToast("Error: " + (r.error || r.raw || "unknown"), "error", 8000);
    }
  } catch (e) {
    showToast("Error: " + e, "error");
  }
  strategyLoading.value = false;
}

function parseAdhocPlanSteps(rawText, fallbackProject = "") {
  const lines = String(rawText || "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  const knownProjects = new Set((agents.value || []).map((a) => a.name));
  const steps = [];
  for (const line of lines) {
    const cleaned = line.replace(/^[-*]\s+/, "").trim();
    if (!cleaned) continue;
    const colonIdx = cleaned.indexOf(":");
    if (colonIdx > 0) {
      const maybeProject = cleaned.slice(0, colonIdx).trim();
      const task = cleaned.slice(colonIdx + 1).trim();
      if (task && (knownProjects.has(maybeProject) || !fallbackProject)) {
        steps.push({ project: maybeProject, task });
        continue;
      }
    }
    if (!fallbackProject) {
      throw new Error(
        "Each line must be 'project: task' when room has no active project",
      );
    }
    steps.push({ project: fallbackProject, task: cleaned });
  }
  if (!steps.length) {
    throw new Error("No plan steps parsed");
  }
  return steps;
}

function parseSingleRoomTask(rawText, fallbackProject = "") {
  const text = String(rawText || "").trim();
  if (!text) {
    throw new Error("Empty task");
  }
  const cleaned = text.replace(/^[-*]\s+/, "").trim();
  const colonIdx = cleaned.indexOf(":");
  const knownProjects = new Set((agents.value || []).map((a) => a.name));
  if (colonIdx > 0) {
    const maybeProject = cleaned.slice(0, colonIdx).trim();
    const task = cleaned.slice(colonIdx + 1).trim();
    if (task && (knownProjects.has(maybeProject) || !fallbackProject)) {
      return { project: maybeProject, task };
    }
  }
  if (!fallbackProject) {
    throw new Error("Use 'project: task' when room has no active project");
  }
  return { project: fallbackProject, task: cleaned };
}

async function createAdhocPlanFromRoom(
  rawText,
  fallbackProject = "",
  roomSessionId = null,
) {
  if (!__IS_TAURI) return null;
  const steps = parseAdhocPlanSteps(rawText, fallbackProject);
  const firstTask = (steps[0]?.task || "room plan").slice(0, 72);
  const title =
    steps.length === 1
      ? `Ad-hoc: ${firstTask}`
      : `Ad-hoc batch (${steps.length} steps): ${firstTask}`;
  const res = await __invoke("create_plan", {
    title,
    steps,
    roomSessionId: roomSessionId || null,
  });
  if (res?.status === "ok") {
    await loadPlansData();
    showToast("Ad-hoc plan created", "success");
    return res;
  }
  throw new Error(res?.error || "Plan creation failed");
}

async function queueRoomAgentTask(
  rawText,
  fallbackProject = "",
  sessionId = null,
  projectSessionId = null,
) {
  if (!__IS_TAURI) return null;
  const currentSessionId = sessionId || activeDualSession.value;
  if (!currentSessionId) {
    throw new Error("No active room session");
  }
  const { project, task } = parseSingleRoomTask(rawText, fallbackProject);
  const res = await __invoke("queue_session_delegation", {
    sessionId: currentSessionId,
    projectSessionId: projectSessionId || null,
    project,
    task,
  });
  if (res?.status === "ok") {
    const updated = { ...delegations.value };
    updated[res.delegation_id] = { project, status: "pending" };
    delegations.value = updated;
    await loadInbox();
    await loadSignals();
    showToast("Agent task queued", "success");
    return res;
  }
  throw new Error(res?.error || "Delegation queue failed");
}
async function createRoomProjectSession(
  rawText,
  fallbackProject = "",
  sessionId = null,
) {
  if (!__IS_TAURI) return null;
  const currentSessionId = sessionId || activeDualSession.value;
  if (!currentSessionId) {
    throw new Error("No active room session");
  }
  const { project, task } = parseSingleRoomTask(rawText, fallbackProject);
  const title = task || project + " work session";
  const res = await __invoke("create_project_session", {
    parentRoomSessionId: currentSessionId,
    project,
    title,
    executorProvider: null,
    reviewerProvider: null,
  });
  if (res?.status === "ok") {
    showToast("Project session created", "success");
    return res;
  }
  throw new Error(res?.error || "Project session creation failed");
}
async function createRoomWorkItem({
  project,
  title,
  task,
  assignee = "agent",
  writeIntent = "read_only",
  declaredPaths = [],
  verify = null,
  sessionId = null,
  projectSessionId = null,
  executorProvider = null,
  reviewerProvider = null,
  sourceKind = null,
  sourceId = null,
}) {
  if (!__IS_TAURI) return null;
  const currentSessionId = sessionId || activeDualSession.value;
  if (!currentSessionId) {
    throw new Error("No active room session");
  }
  const res = await __invoke("create_work_item", {
    parentRoomSessionId: currentSessionId,
    projectSessionId: projectSessionId || null,
    project,
    title: title || null,
    task,
    assignee,
    writeIntent,
    declaredPaths,
    verify,
    executorProvider: executorProvider || null,
    reviewerProvider: reviewerProvider || null,
    sourceKind: sourceKind || null,
    sourceId: sourceId || null,
  });
  if (res?.status === "ok") {
    showToast("Work item created", "success");
    return res;
  }
  throw new Error(res?.error || "Work item creation failed");
}
async function createPlanStepWorkItem(
  planId,
  stepIndex,
  sessionId = null,
  projectSessionId = null,
  executorProvider = null,
  reviewerProvider = null,
) {
  if (!__IS_TAURI) return null;
  const currentSessionId = sessionId || activeDualSession.value;
  const res = await __invoke("create_plan_step_work_item", {
    planId,
    stepIndex,
    roomSessionId: currentSessionId || null,
    projectSessionId: projectSessionId || null,
    executorProvider: executorProvider || null,
    reviewerProvider: reviewerProvider || null,
  });
  if (res?.status === "ok") {
    await loadPlansData();
    showToast("Plan step linked as work item", "success");
    return res;
  }
  throw new Error(res?.error || "Create plan step work item failed");
}
async function queueWorkItemExecution(workItemId) {
  if (!__IS_TAURI) return null;
  const res = await __invoke("queue_work_item_execution", { workItemId });
  if (res?.status === "ok") {
    const updated = { ...delegations.value };
    updated[res.delegation_id] = { project: res.project, status: "pending" };
    delegations.value = updated;
    await loadPlansData();
    await loadInbox();
    await loadSignals();
    showToast("Work item queued", "success");
    return res;
  }
  throw new Error(res?.error || "Queue execution failed");
}
async function queueParallelWorkItems(sessionId, workItemIds = []) {
  if (!__IS_TAURI) return null;
  const res = await __invoke("queue_parallel_work_items", {
    sessionId,
    workItemIds,
  });
  if (res?.status === "ok" || res?.status === "partial") {
    await loadPlansData();
    await loadInbox();
    await loadSignals();
    showToast(
      res?.status === "partial"
        ? "Parallel batch queued with partial failures"
        : "Parallel batch queued",
      res?.status === "partial" ? "warning" : "success",
    );
    return res;
  }
  throw new Error(res?.error || "Parallel batch queue failed");
}
async function queueProviderParallelRound(sessionId, provider) {
  if (!__IS_TAURI) return null;
  const res = await __invoke("queue_provider_parallel_round", {
    sessionId,
    provider,
  });
  if (res?.status === "ok" || res?.status === "partial") {
    await loadPlansData();
    await loadInbox();
    await loadSignals();
    showToast(
      res?.status === "partial"
        ? `${provider} round queued with partial failures`
        : `${provider} round queued`,
      res?.status === "partial" ? "warning" : "success",
    );
    return res;
  }
  throw new Error(res?.error || `${provider} provider round failed`);
}
async function completeUserWorkItem(workItemId, result = "") {
  if (!__IS_TAURI) return null;
  const res = await __invoke("complete_user_work_item", {
    workItemId,
    result: result || null,
  });
  if (res?.status === "ok") {
    await loadPlansData();
    showToast("User work item completed", "success");
    return res;
  }
  throw new Error(res?.error || "Complete user work item failed");
}
async function acquireWorkItemLease(workItemId, participantId = null) {
  if (!__IS_TAURI) return null;
  const res = await __invoke("acquire_work_item_lease_manual", {
    workItemId,
    participantId: participantId || null,
  });
  if (res?.status === "ok") {
    showToast("Lease acquired", "success");
    return res;
  }
  throw new Error(res?.error || "Acquire lease failed");
}
async function releaseFileLease(leaseId, force = false) {
  if (!__IS_TAURI) return null;
  const res = await __invoke("release_file_lease", {
    leaseId,
    force,
  });
  if (res?.status === "ok") {
    showToast(force ? "Lease force-released" : "Lease released", "success");
    return res;
  }
  throw new Error(res?.error || "Release lease failed");
}
async function approveSteps(stratId, stepIds) {
  try {
    await __invoke("approve_strategy_steps", {
      strategyId: stratId,
      approvedSteps: stepIds,
    });
    showToast("Steps approved", "success");
    loadStrategies();
  } catch (e) {
    showToast("Error: " + e, "error");
  }
}
async function executeNextStep(stratId) {
  try {
    const r = await __invoke("execute_strategy_step", { strategyId: stratId });
    if (r.status === "complete") {
      showToast("Strategy complete!", "success");
    } else if (r.status === "step_done") {
      showToast(`${r.project}: done`, "success");
      loadStrategies();
    } else {
      showToast("Error: " + (r.error || ""), "error");
    }
  } catch (e) {
    showToast("Error: " + e, "error");
  }
}
async function loadProjectPlan(p) {
  try {
    const r = await fetch("/api/project-plan/" + encodeURIComponent(p));
    projectPlan.value = await r.json();
  } catch {
    projectPlan.value = null;
  }
}
async function loadInbox() {
  if (__IS_TAURI) {
    try {
      const r = await __invoke("get_inbox");
      inboxData.value = r || { items: [], count: 0 };
    } catch {}
  }
}
async function processInbox() {
  if (__IS_TAURI) {
    try {
      showToast("Processing inbox...", "info");
      const r = await __invoke("process_inbox");
      if (r.status === "needs_user") {
        showToast("Some items need your review", "error", 5000);
      } else if (r.status === "processed") {
        showToast("PA processed " + r.count + " results", "success", 4000);
        await loadChat(currentProject.value || "_orchestrator");
      } else {
        showToast(r.message || "Empty inbox", "info");
      }
      inboxData.value = { items: [], count: 0 };
    } catch (e) {
      showToast("Error: " + e, "error");
    }
  }
}
async function loadPerms() {
  try {
    const [r, c, ps] = await Promise.all([
      fetch("/api/permissions"),
      __IS_TAURI
        ? __invoke("get_config").catch(() => ({}))
        : Promise.resolve({}),
      __IS_TAURI
        ? __invoke("get_provider_status").catch(() => ({}))
        : Promise.resolve({}),
    ]);
    const pd = await r.json();
    pd.config = c || {};
    pd.provider_status = ps || {};
    permData.value = pd;
  } catch (e) {
    console.warn("loadPerms:", e);
  }
}
async function authenticateCodexAcp(methodId = null) {
  if (!__IS_TAURI) return null;
  const res = await __invoke("codex_acp_authenticate", {
    methodId: methodId || null,
  });
  await loadPerms();
  if (res?.status === "ok") {
    showToast("Codex ACP authentication complete", "success");
    return res;
  }
  throw new Error(res?.error || "Codex ACP authentication failed");
}
async function setPermission(proj, profile) {
  try {
    await fetch("/api/permissions", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ project: proj, profile }),
    });
    showToast(proj + " -> " + profile, "success");
    loadPerms();
  } catch {
    showToast("Error setting permission", "error");
  }
}
async function loadFeed() {
  try {
    const r = await fetch("/api/feed");
    const d = await r.json();
    feedItems.value = d.feed || [];
  } catch {}
}
async function loadActivity() {
  try {
    const r = await fetch("/api/activity");
    const d = await r.json();
    activities.value = d.activities || {};
    // Show recovery notification if tasks were running when page loaded
    const acts = d.activities || {};
    const pending = Object.entries(acts);
    if (pending.length && !window._recoveryShown) {
      window._recoveryShown = true;
      const names = pending
        .map(([k, v]) => k + " (" + v.action + ")")
        .join(", ");
      console.log("Recovering running tasks:", names);
    }
  } catch {}
}

async function sendMessage(msg) {
  if (!msg.trim()) return;
  if (isStreaming.value) {
    showToast("Wait for current response to complete", "error", 2000);
    return;
  }
  const proj = currentProject.value || "";
  lastUserMsg.value = msg;
  // Apply chat mode prefix
  if (chatMode.value) {
    const modeMap = {
      code: "[MODE: Focus on code. Write clean, tested code. No unnecessary explanations.]",
      design:
        "[MODE: Focus on architecture and design. Think about patterns, structure, trade-offs.]",
      review:
        "[MODE: Code review mode. Find bugs, suggest improvements, check edge cases.]",
      fix: "[MODE: Bug fix mode. Diagnose the issue, find root cause, fix it.]",
    };
    const prefix = modeMap[chatMode.value] || "";
    if (prefix) msg = prefix + "\n\n" + msg;
    chatMode.value = "";
  }
  if (attFiles.value.length) {
    const paths = attFiles.value
      .filter((f) => f.path)
      .map((f) => "[file: " + f.path + "]");
    if (paths.length) msg += "\n\nAttached files:\n" + paths.join("\n");
    else msg += " | Attached: " + attFiles.value.map((f) => f.name).join(", ");
  }
  attFiles.value = [];
  clrDr();
  sideMessages.value = [
    ...sideMessages.value,
    { ts: new Date().toISOString(), role: "user", msg },
  ];
  isStreaming.value = true;
  streamText.value = "";
  streamChain.value = [];
  thinkStart.value = Date.now();
  try {
    let full = "";
    const tools = [];
    if (__IS_TAURI) {
      const normalizedSelection = normalizedSoloProviderSelection();
      // Tauri mode: invoke stream_chat + poll file-based buffer
      __invoke("stream_chat", {
        project: proj,
        message: msg,
        provider: normalizedSelection.provider || null,
        model: normalizedSelection.model || null,
        reasoningEffort: normalizedSelection.effort || null,
      }).catch((e) => {
        console.error("stream_chat error:", e);
        showToast("Chat error: " + e, "error");
        isStreaming.value = false;
      });
      let offset = 0;
      let done = false;
      let pollCount = 0;
      const MAX_POLLS = 1200; // 5 min max
      // Collect ordered chain of blocks for rendering
      const chain = [];
      while (!done && pollCount < MAX_POLLS) {
        pollCount++;
        await new Promise((r) => setTimeout(r, 250));
        try {
          const poll = await __invoke("poll_stream", { offset });
          if (poll.events) {
            for (const evt of poll.events) {
              // Text (complete block from assistant event)
              if (evt.type === "text") {
                full = evt.text;
                streamText.value = full;
                // Merge with last text block if exists (text_deltas already created it)
                const lastT = chain[chain.length - 1];
                if (lastT && lastT.type === "text") {
                  lastT.text = evt.text;
                } else {
                  chain.push({ type: "text", text: evt.text });
                }
                streamChain.value = [...chain];
              }
              // Text delta (streaming partial) — update last text block in chain or create new one
              if (evt.type === "text_delta") {
                const newFull = evt.full || full;
                streamText.value = newFull;
                full = newFull;
                const last = chain[chain.length - 1];
                if (last && last.type === "text") {
                  last.text = newFull;
                } else {
                  chain.push({ type: "text", text: newFull });
                }
                streamChain.value = [...chain];
              }
              // Thinking events
              if (evt.type === "thinking_start") {
                chain.push({ type: "thinking", text: "", streaming: true });
                curActivity.value = "thinking...";
                streamChain.value = [...chain];
              }
              if (evt.type === "thinking_delta") {
                const lt = chain.findLast((b) => b.type === "thinking");
                if (lt) lt.text += evt.text || "";
                streamChain.value = [...chain];
              }
              if (evt.type === "thinking_stop") {
                const lt = chain.findLast((b) => b.type === "thinking");
                if (lt) lt.streaming = false;
                streamChain.value = [...chain];
              }
              // Tool use (started or complete)
              if (evt.type === "tool_use") {
                const tb = {
                  type: "tool",
                  tool: evt.tool,
                  input: evt.input || {},
                  status: evt.status,
                  startedAt: Date.now(),
                };
                if (evt.status === "started") {
                  curActivity.value = "▸ " + evt.tool + "...";
                }
                const last = chain[chain.length - 1];
                if (
                  last &&
                  last.type === "tool" &&
                  last.tool === evt.tool &&
                  last.status === "started" &&
                  evt.status === "complete"
                ) {
                  last.input = evt.input;
                  last.status = "complete";
                  last.elapsed = Date.now() - last.startedAt;
                } else {
                  chain.push(tb);
                }
                tools.push({ tool: evt.tool, input: evt.input || {} });
                streamChain.value = [...chain];
              }
              // Tool result — attach to last tool without result, or push standalone
              if (evt.type === "tool_result") {
                const lt = chain.findLast(
                  (b) => b.type === "tool" && !b.result,
                );
                if (lt) {
                  lt.result = evt.content || "";
                  lt.is_error = evt.is_error || false;
                } else {
                  chain.push({
                    type: "tool_result",
                    content: evt.content || "",
                  });
                }
                curActivity.value = "";
                streamChain.value = [...chain];
              }
              // System
              if (evt.type === "system" && evt.system) {
                curActivity.value = evt.system;
                chain.push({ type: "system", label: evt.system });
                streamChain.value = [...chain];
              }
              // PA command execution feedback
              if (
                evt.type === "pa_result" ||
                evt.type === "warning" ||
                evt.type === "pa_status"
              ) {
                chain.push({
                  type: evt.type,
                  text: evt.text || "",
                  command: evt.command || "",
                });
                streamChain.value = [...chain];
              }
              // Result stats
              if (evt.type === "result") {
                lastStats.value = {
                  duration_ms: evt.duration_ms,
                  cost: evt.cost,
                  tokens: evt.tokens,
                };
                chain.push({
                  type: "result",
                  cost: evt.cost,
                  duration_ms: evt.duration_ms,
                  tokens: evt.tokens,
                });
              }
              // Delegation
              if (evt.type === "delegation") {
                full +=
                  '\n\n<delegation id="' +
                  evt.id +
                  '" project="' +
                  evt.project +
                  '"/>';
                chain.push({
                  type: "delegation",
                  id: evt.id,
                  project: evt.project,
                  task: evt.task,
                });
              }
              // Done
              if (evt.type === "done") {
                done = true;
                if (evt.text && !full) full = evt.text;
              }
            }
            offset = poll.offset;
          }
        } catch (e) {
          console.warn("poll_stream:", e);
          done = true;
        }
      }
      if (!done && pollCount >= MAX_POLLS) {
        showToast(
          "Response timed out (5 min). Try again or click Stop.",
          "error",
          8000,
        );
      }
      // Unblock chat input immediately after poll loop exits
      isStreaming.value = false;
      streamText.value = "";
      curActivity.value = "";
    } else {
      // Browser mode: fetch SSE stream from serve.py
      const r = await fetch("/api/chat-stream", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ project: proj, message: msg }),
      });
      const reader = r.body.getReader();
      const dec = new TextDecoder();
      let buf = "";
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        buf += dec.decode(value, { stream: true });
        const lines = buf.split("\n");
        buf = lines.pop() || "";
        for (const ln of lines) {
          if (!ln.startsWith("data: ")) continue;
          const p = ln.slice(6);
          if (p === "[DONE]") break;
          try {
            const d = JSON.parse(p);
            if (d.text) {
              full += d.text;
              streamText.value = full;
            }
            if (d.activity) curActivity.value = d.activity;
            if (d.stats) {
              lastStats.value = d.stats;
              curActivity.value = "";
            }
          } catch {}
        }
      }
    }
    isStreaming.value = false;
    const dms = [
      ...full.matchAll(/<delegation id="([^"]+)" project="([^"]+)"\/>/g),
    ];
    if (dms.length) {
      const nd = { ...delegations.value };
      for (const dm of dms) {
        nd[dm[1]] = { project: dm[2], status: "pending" };
      }
      delegations.value = nd;
    }
    beep();
    // Desktop notification on completion (if window not focused)
    if (__IS_TAURI && !document.hasFocus()) {
      try {
        __invoke("plugin:notification|notify", {
          options: {
            title: "Agent OS",
            body: (proj || "Orchestrator") + ": response ready",
          },
        });
      } catch {}
    }
    curActivity.value = "";
    lastStats.value = null;
    streamText.value = "";
    streamChain.value = [];
    // Reload chat from JSONL (single source of truth) — prevents duplicates
    await loadChat(currentProject.value || "_orchestrator");
    loadFeed();
  } catch (e) {
    isStreaming.value = false;
    sideMessages.value = [
      ...sideMessages.value,
      {
        ts: new Date().toISOString(),
        role: "assistant",
        msg: "Error: " + e.message,
      },
    ];
  }
}

async function approveDel(id) {
  const d = { ...delegations.value };
  d[id] = { ...d[id], status: "running", _start: Date.now() };
  delegations.value = d;
  // Re-render timer every second while running
  const timer = setInterval(() => {
    delegations.value = { ...delegations.value };
  }, 1000);
  try {
    const r = await fetch("/api/approve/" + id, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: "{}",
    });
    const res = await r.json();
    clearInterval(timer);
    const finalStatus =
      res.status === "complete"
        ? "done"
        : res.status === "failed"
          ? "failed"
          : "error";
    const d2 = { ...delegations.value };
    d2[id] = { ...d2[id], status: finalStatus };
    delegations.value = d2;
    const proj = res.project || "";
    const icon = finalStatus === "done" ? "✓" : "✗";
    showToast(
      icon + " " + proj + ": " + finalStatus,
      finalStatus === "done" ? "success" : "error",
      4000,
    );
    // Reload current chat from JSONL (source of truth) — don't append to sideMessages directly
    const active = currentProject.value || "_orchestrator";
    await loadChat(active);
  } catch (e) {
    clearInterval(timer);
    const d2 = { ...delegations.value };
    d2[id] = { ...d2[id], status: "error" };
    delegations.value = d2;
    showToast("Delegation error: " + e, "error");
  }
}
async function rejectDel(id) {
  const d = { ...delegations.value };
  d[id] = { ...d[id], status: "rejected" };
  delegations.value = d;
  fetch("/api/reject/" + id, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: "{}",
  });
}

async function loadDualSession(sessionId) {
  if (!__IS_TAURI || !sessionId) return;
  const res = await __invoke("get_multi_agent_session", { sessionId });
  if (res?.status === "ok" && res.session) {
    activeDualSession.value = sessionId;
    dualSessionData.value = res;
    const histories = {};
    for (const p of res.session.participants || []) {
      const h = await __invoke("get_session_agent_history", {
        sessionId,
        participantId: p.id,
      });
      histories[p.id] = h.messages || [];
    }
    dualHistories.value = histories;
    await loadActiveScope(
      res.session.project || currentProject.value || "",
      sessionId,
    );
  } else {
    showToast(res?.error || "Cannot load session", "error");
  }
}

async function setDualWriter(sessionId, participantId) {
  if (!__IS_TAURI || !sessionId || !participantId) return null;
  const res = await __invoke("set_session_writer", {
    sessionId,
    participantId,
  });
  if (res?.status === "ok") {
    await loadDualSession(sessionId);
    showToast("Write access granted", "success");
    return res;
  }
  if (Array.isArray(res?.leases) && res.leases.length) {
    const leaseHint = res.leases
      .map(
        (lease) => `${lease.participant_id}: ${(lease.paths || []).join(", ")}`,
      )
      .join(" ; ");
    throw new Error((res?.error || "Cannot set writer") + " -> " + leaseHint);
  }
  throw new Error(res?.error || "Cannot set writer");
}
async function setDualOrchestrator(sessionId, participantId) {
  if (!__IS_TAURI || !sessionId || !participantId) return null;
  const res = await __invoke("set_session_orchestrator", {
    sessionId,
    participantId,
  });
  if (res?.status === "ok") {
    await loadDualSession(sessionId);
    showToast("Orchestrator switched", "success");
    return res;
  }
  throw new Error(res?.error || "Cannot set orchestrator");
}
async function revokeDualWriter(sessionId, participantId) {
  if (!__IS_TAURI || !sessionId || !participantId) return null;
  const res = await __invoke("revoke_session_writer", {
    sessionId,
    participantId,
  });
  if (res?.status === "ok") {
    await loadDualSession(sessionId);
    showToast("Write access revoked", "success");
    return res;
  }
  if (Array.isArray(res?.leases) && res.leases.length) {
    const leaseHint = res.leases
      .map(
        (lease) => `${lease.participant_id}: ${(lease.paths || []).join(", ")}`,
      )
      .join(" ; ");
    throw new Error((res?.error || "Cannot revoke write") + " -> " + leaseHint);
  }
  throw new Error(res?.error || "Cannot revoke write");
}

async function createDualSession(project = "") {
  if (!__IS_TAURI) return;
  const res = await __invoke("create_multi_agent_session", {
    project: project || null,
    mode: "review",
  });
  if (res?.status === "ok" && res.session?.id) {
    await loadDualSession(res.session.id);
    showToast("Dual-agent session created", "success");
  } else {
    showToast(res?.error || "Cannot create session", "error");
  }
}

async function ensureDualSession(project = "") {
  if (!__IS_TAURI) return null;
  const normalized = project || "";
  if (
    dualSessionData.value?.session &&
    (dualSessionData.value.session.project || "") === normalized
  ) {
    loadActiveScope(normalized, dualSessionData.value.session.id).catch((e) =>
      console.warn("scope refresh:", e),
    );
    return dualSessionData.value.session;
  }
  try {
    const listed = await __invoke("list_multi_agent_sessions");
    const sessions = (listed?.sessions || []).filter(
      (s) => (s.project || "") === normalized && s.status !== "closed",
    );
    if (sessions.length) {
      await loadDualSession(sessions[0].id);
      return sessions[0];
    }
  } catch (e) {
    console.warn("ensureDualSession list:", e);
  }
  await createDualSession(normalized);
  return dualSessionData.value?.session || null;
}

async function runDualParticipant(participantId, message, analysisOnly = true) {
  if (!__IS_TAURI) return;
  const msg = (message || "").trim();
  if (!msg) {
    showToast("Enter a prompt", "error");
    return;
  }
  let sessionId = activeDualSession.value;
  if (!sessionId) {
    await ensureDualSession(currentProject.value || "");
    sessionId = activeDualSession.value;
  }
  if (!sessionId) return;
  dualBusy.value = participantId;
  try {
    const res = await __invoke("run_session_agent", {
      sessionId,
      participantId,
      message: msg,
      model: null,
      reasoningEffort: null,
      analysisOnly,
    });
    if (res?.status === "complete") {
      await loadDualSession(sessionId);
      await loadChat(currentProject.value || "_orchestrator");
    } else {
      showToast(res?.error || "Participant run failed", "error");
    }
  } catch (e) {
    showToast("Participant error: " + e, "error");
  } finally {
    dualBusy.value = "";
  }
}

async function runDualRound(message, analysisOnly = true) {
  if (!__IS_TAURI) return;
  const msg = (message || "").trim();
  if (!msg) {
    showToast("Enter a prompt", "error");
    return;
  }
  let sessionId = activeDualSession.value;
  if (!sessionId) {
    await ensureDualSession(currentProject.value || "");
    sessionId = activeDualSession.value;
  }
  if (!sessionId) return;
  dualBusy.value = "round";
  try {
    const res = await __invoke("run_session_round", {
      sessionId,
      message: msg,
      model: null,
      reasoningEffort: null,
      analysisOnly,
    });
    if (res?.status === "complete" || res?.status === "partial") {
      await loadDualSession(sessionId);
      await loadChat(currentProject.value || "_orchestrator");
      if (res?.status === "partial") {
        showToast("Round completed with errors", "error");
      }
    } else {
      showToast(res?.error || "Round failed", "error");
    }
  } catch (e) {
    showToast("Round error: " + e, "error");
  } finally {
    dualBusy.value = "";
  }
}

async function runDualRoomAction(action, message, targetParticipantId) {
  if (!__IS_TAURI) return;
  const msg = (message || "").trim();
  let sessionId = activeDualSession.value;
  if (!sessionId) {
    await ensureDualSession(currentProject.value || "");
    sessionId = activeDualSession.value;
  }
  if (!sessionId) return;
  const busyKey = action + ":" + (targetParticipantId || "room");
  dualBusy.value = busyKey;
  try {
    const res = await __invoke("run_session_room_action", {
      sessionId,
      action,
      message: msg,
      targetParticipantId: targetParticipantId || null,
    });
    if (res?.status === "complete") {
      await loadDualSession(sessionId);
      await loadChat(currentProject.value || "_orchestrator");
    } else {
      showToast(res?.error || "Room action failed", "error");
    }
  } catch (e) {
    showToast("Room action error: " + e, "error");
  } finally {
    dualBusy.value = "";
  }
}

// ===== HELPERS =====
// esc, md, ft, SC, SL imported from utils.js

// ===== COMPONENTS =====

export {
  togVoice,
  hdlFiles,
  rmFile,
  chkConn,
  saveDr,
  loadDr,
  clrDr,
  handleSlash,
  execSlash,
  handlePaste,
  loadAgents,
  checkOrch,
  loadSegments,
  loadChat,
  loadModules,
  loadPlan,
  loadQueue,
  loadGoals,
  loadGraph,
  loadSignals,
  ackSignal,
  loadStrategies,
  loadPlansData,
  loadActiveScope,
  generateStrategy,
  createAdhocPlanFromRoom,
  createRoomProjectSession,
  createRoomWorkItem,
  createPlanStepWorkItem,
  queueWorkItemExecution,
  queueParallelWorkItems,
  queueProviderParallelRound,
  completeUserWorkItem,
  acquireWorkItemLease,
  releaseFileLease,
  queueRoomAgentTask,
  approveSteps,
  executeNextStep,
  loadProjectPlan,
  loadPerms,
  authenticateCodexAcp,
  setPermission,
  loadFeed,
  loadActivity,
  sendMessage,
  approveDel,
  rejectDel,
  createDualSession,
  setDualWriter,
  setDualOrchestrator,
  revokeDualWriter,
  ensureDualSession,
  loadDualSession,
  runDualParticipant,
  runDualRound,
  runDualRoomAction,
  loadInbox,
  processInbox,
};
