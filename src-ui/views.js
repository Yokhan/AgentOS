// AgentOS view components — dashboard, settings, strategy, plans, chrome
import { html, useEffect, useRef, useState } from "/vendor/preact-bundle.mjs";
import { esc, md, ft, SC, SL } from "/utils.js";
import { __IS_TAURI, __invoke } from "/bridge.js";
import {
  agents,
  currentProject,
  selectedClaudeModel,
  selectedClaudeEffort,
  selectedCodexModel,
  selectedCodexEffort,
  showSettings,
  showNewProject,
  showStrategy,
  showPlans,
  showDualAgents,
  chatCollabMode,
  activeRoomTab,
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
  delegations,
  theme,
  isLoading,
  clock,
  segments,
  activeFilter,
  sortBy,
  viewMode,
  searchQuery,
  actionPlan,
  queueTasks,
  showKbHelp,
  showPlan,
  orchOk,
  signalsData,
  activities,
  composerDraftText,
  executionMap,
  appInfo,
  showToast,
} from "/store.js";
import {
  loadAgents,
  loadGoals,
  loadGraph,
  loadSignals,
  ackSignal,
  loadStrategies,
  loadPlansData,
  loadPerms,
  loadFeed,
  loadExecutionMap,
  ensureDualSession,
  sendMessage,
} from "/api.js";
import {
  Tile,
  DetailView,
  ChatSidebar,
  ExecutionMapCard,
  NewProjectModal,
  Toasts,
} from "/chat.js";

let pagesModulePromise = null;
const PAGES_MODULE_URL = "/pages.js?v=20260418b";

function usePagesModule() {
  const [pagesMod, setPagesMod] = useState(null);
  const [pagesErr, setPagesErr] = useState("");
  useEffect(() => {
    let active = true;
    if (!pagesModulePromise) {
      pagesModulePromise = import(PAGES_MODULE_URL);
    }
    pagesModulePromise
      .then((mod) => {
        if (active) setPagesMod(mod);
      })
      .catch((err) => {
        console.error("pages.js load failed:", err);
        if (active) setPagesErr(String(err?.stack || err?.message || err));
      });
    return () => {
      active = false;
    };
  }, []);
  return { pagesMod, pagesErr };
}
function OrchWarning() {
  if (orchOk.value) return null;
  return html`<div
    style="padding:var(--sp-s) var(--sp-l);background:var(--accent-dim);border-bottom:1px solid var(--accent);font-size:var(--fs-s);color:var(--accent);font-family:var(--font-mono);display:flex;align-items:center;gap:var(--sp-s)"
  >
    <span>WARNING:</span> No orchestrator project found. Chat will run in
    template context.
    <span style="color:var(--t2)">Fix: bash setup.sh my-pa --orchestrator</span>
  </div>`;
}

function currentSoloProviderLabel() {
  return !currentProject.value &&
    permData.value?.provider_status?.roles?.orchestrator_provider === "codex"
    ? "codex"
    : "claude";
}

function currentSoloSelectionLabel() {
  const provider = currentSoloProviderLabel();
  const model =
    provider === "codex" ? selectedCodexModel.value : selectedClaudeModel.value;
  const effort =
    provider === "codex"
      ? selectedCodexEffort.value
      : selectedClaudeEffort.value;
  return `${provider}:${model || "auto"}${effort ? "/" + effort : ""}`;
}
function Breadcrumb() {
  const parts = ["Dashboard"];
  if (showPlans.value) parts.push("Plans");
  else if (showStrategy.value) parts.push("Strategy");
  else if (showSettings.value) parts.push("Settings");
  else if (showGraph.value)
    parts.push(
      graphLevel.value === "overview" ? "Graph" : `Graph: ${graphLevel.value}`,
    );
  else if (currentProject.value) parts.push(currentProject.value);
  if (parts.length <= 1) return null;
  return html`<div
    style="padding:var(--sp-2xs) var(--sp-l);font-size:var(--fs-s);font-family:var(--font-mono);color:var(--t3);border-bottom:1px solid var(--border);display:flex;gap:var(--sp-xs)"
  >
    <span
      style="cursor:pointer;color:var(--t2)"
      onClick=${() => {
        currentProject.value = null;
        showPlans.value = false;
        showStrategy.value = false;
        showSettings.value = false;
        showGraph.value = false;
        graphSelected.value = null;
      }}
      >Dashboard</span
    >
    ${parts.length > 1
      ? html`<span>›</span
          ><span style="color:var(--text)">${parts[parts.length - 1]}</span>`
      : null}
  </div>`;
}
function App() {
  const { pagesMod, pagesErr } = usePagesModule();
  const SettingsPage = pagesMod?.SettingsPage || null;
  const PlansView = pagesMod?.PlansView || null;
  const StrategyView = pagesMod?.StrategyView || null;
  const pagesFallback = pagesErr
    ? html`<div class="content">
        <div class="panel" style="margin:var(--sp-l)">
          <h3>UI Module Error</h3>
          <pre
            style="white-space:pre-wrap;word-break:break-word;font-size:var(--fs-s);color:var(--accent)"
          >
${pagesErr}</pre
          >
        </div>
      </div>`
    : html`<div class="content">
        <div class="panel" style="margin:var(--sp-l);color:var(--t3)">
          Loading UI modules...
        </div>
      </div>`;
  return html`<div class="app">
    <${Toasts} /><${KeyboardHelp} /><${OrchWarning} /><${NewProjectModal} /><${Header} /><${Breadcrumb} />
    <div class="main">
      ${showPlans.value && PlansView
        ? html`<${PlansView} />`
        : showStrategy.value && StrategyView
          ? html`<${StrategyView} />`
          : showSettings.value && SettingsPage
            ? html`<${SettingsPage} />`
            : showPlans.value || showStrategy.value || showSettings.value
              ? pagesFallback
              : showGraph.value
                ? html`<${GraphView} />`
                : currentProject.value
                  ? html`<${ProjectWorkbenchView} />`
                  : html`<${DashboardWorkbenchView} />`}<${ChatSidebar} />
    </div>
    <${AnalyticsBar} />
  </div>`;
}

function Header() {
  const pending = Object.values(delegations.value).filter(
    (d) => d.status === "pending",
  ).length;
  const isStrat = showStrategy.value,
    isSet = showSettings.value,
    isGraph = showGraph.value;
  return html`<div class="hdr" data-tauri-drag-region>
    <h1
      data-tauri-drag-region
      style="font-size:var(--fs-m);color:var(--t3);font-weight:400"
    >
      agent os
    </h1>
    <div class="r">
      ${pending ? html`<span class="badge">${pending}</span>` : ""}
      <button
        class=${showPlans.value ? "hdr-active" : ""}
        onClick=${() => {
          showPlans.value = !showPlans.value;
          showStrategy.value = false;
          showSettings.value = false;
          if (showPlans.value) loadPlansData();
        }}
      >
        plans
      </button>
      <button
        class=${isStrat ? "hdr-active" : ""}
        onClick=${() => {
          showStrategy.value = !showStrategy.value;
          showPlans.value = false;
          showDualAgents.value = false;
          showSettings.value = false;
          if (showStrategy.value) {
            loadGoals();
            loadStrategies();
          }
        }}
      >
        strategy
      </button>
      <button
        class=${chatCollabMode.value === "duo" ? "hdr-active" : ""}
        onClick=${() => {
          const next = chatCollabMode.value !== "duo";
          chatCollabMode.value = next ? "duo" : "solo";
          activeRoomTab.value = next ? "collaborate" : "chat";
          showDualAgents.value = false;
          if (next) ensureDualSession(currentProject.value || "");
        }}
      >
        ${chatCollabMode.value === "duo" ? "duo on" : "duo"}
      </button>
      <button
        class=${isGraph ? "hdr-active" : ""}
        onClick=${() => {
          showGraph.value = !showGraph.value;
          showPlans.value = false;
          showStrategy.value = false;
          showDualAgents.value = false;
          showSettings.value = false;
          if (showGraph.value) loadGraph("overview");
        }}
      >
        graph
      </button>
      <button
        onClick=${() =>
          fetch("/api/digest")
            .then((r) => r.json())
            .then((d) => showToast(d.text || "No data", "info", 10000))}
      >
        briefing
      </button>
      <button onClick=${() => (showNewProject.value = true)}>+</button>
      <button
        onClick=${() => {
          loadAgents();
          loadFeed();
        }}
      >
        ↻
      </button>
      <span style="color:var(--border)">│</span>
      <button
        class=${isSet ? "hdr-active" : ""}
        onClick=${() => {
          showSettings.value = !showSettings.value;
          showStrategy.value = false;
          if (showSettings.value) loadPerms();
        }}
      >
        ⚙
      </button>
      <button
        onClick=${() =>
          (theme.value = theme.value === "dark" ? "light" : "dark")}
      >
        ${theme.value === "dark" ? "◐" : "◑"}
      </button>
      <span style="font-size:var(--fs-s);color:var(--t3)">${clock}</span>
      ${__IS_TAURI
        ? html`<span style="color:var(--border)">│</span>
            <div class="win-btns">
              <button
                onClick=${() => {
                  try {
                    window.__TAURI_INTERNALS__.invoke(
                      "plugin:window|minimize",
                      { label: "main" },
                    );
                  } catch (e) {
                    console.error(e);
                  }
                }}
                class="wb"
              >
                −
              </button>
              <button
                onClick=${() => {
                  try {
                    window.__TAURI_INTERNALS__.invoke(
                      "plugin:window|toggle_maximize",
                      { label: "main" },
                    );
                  } catch (e) {
                    console.error(e);
                  }
                }}
                class="wb"
              >
                □
              </button>
              <button
                onClick=${() => {
                  try {
                    window.__TAURI_INTERNALS__.invoke("plugin:window|close", {
                      label: "main",
                    });
                  } catch (e) {
                    console.error(e);
                  }
                }}
                class="wb wb-close"
              >
                ×
              </button>
            </div>`
        : ""}
    </div>
  </div>`;
}

function StatsRow() {
  const a = agents.value;
  const na = a.filter((x) => x.blockers || (x.uncommitted || 0) > 20).length;
  const act = a.filter((x) => x.task).length;
  const h = a.filter((x) => !x.blockers && x.status !== "sleeping").length;
  const st = a.filter((x) => (x.days || 999) > 7).length;
  return html`<div class="stats">
    <div
      class="stat s-alert"
      style="cursor:pointer;${na ? "background:var(--accent-dim)" : ""}"
      onClick=${() =>
        (activeFilter.value =
          activeFilter.value === "attention" ? "" : "attention")}
    >
      <div class="n" style="font-size:var(--fs-xl)">${na}</div>
      <div class="l">attention</div>
    </div>
    <div
      class="stat s-active"
      style="cursor:pointer"
      onClick=${() =>
        (activeFilter.value = activeFilter.value === "active" ? "" : "active")}
    >
      <div class="n" style="font-size:var(--fs-xl)">${act}</div>
      <div class="l">active</div>
    </div>
    <div class="stat">
      <div class="n" style="font-size:var(--fs-xl);color:var(--t3)">${h}</div>
      <div class="l">healthy</div>
    </div>
    <div
      class="stat"
      style="cursor:pointer"
      onClick=${() =>
        (activeFilter.value = activeFilter.value === "stale" ? "" : "stale")}
    >
      <div class="n" style="font-size:var(--fs-xl);color:var(--mute)">
        ${st}
      </div>
      <div class="l">stale</div>
    </div>
    <div class="stat">
      <div class="n" style="font-size:var(--fs-xl);color:var(--t3)">
        ${a.length}
      </div>
      <div class="l">total</div>
    </div>
  </div>`;
}

function EmptyState() {
  return html`<div style="padding:var(--sp-2xl);max-width:600px;margin:0 auto">
    <div
      style="font-size:var(--fs-xl);margin-bottom:var(--sp-xl);font-family:var(--font-mono);letter-spacing:2px"
    >
      SETUP REQUIRED
    </div>
    <div
      style="border:1px solid var(--border);padding:var(--sp-l);margin-bottom:var(--sp-l)"
    >
      <div
        style="font-size:var(--fs-s);color:var(--accent);font-family:var(--font-mono);letter-spacing:1px;margin-bottom:var(--sp-m)"
      >
        STEP 1 — CREATE ORCHESTRATOR
      </div>
      <p style="color:var(--t2);margin-bottom:var(--sp-m)">
        The orchestrator is your central project manager. It delegates tasks to
        other project agents.
      </p>
      <pre
        style="padding:var(--sp-m);background:var(--sf);border:1px solid var(--border);font-size:var(--fs-s);overflow-x:auto"
      >
bash setup.sh my-assistant --orchestrator</pre
      >
    </div>
    <div
      style="border:1px solid var(--border);padding:var(--sp-l);margin-bottom:var(--sp-l)"
    >
      <div
        style="font-size:var(--fs-s);color:var(--green);font-family:var(--font-mono);letter-spacing:1px;margin-bottom:var(--sp-m)"
      >
        STEP 2 — CREATE PROJECTS
      </div>
      <p style="color:var(--t2);margin-bottom:var(--sp-m)">
        Each project gets its own agent with rules, memory, and tools.
      </p>
      <pre
        style="padding:var(--sp-m);background:var(--sf);border:1px solid var(--border);font-size:var(--fs-s);overflow-x:auto"
      >
bash setup.sh my-app</pre
      >
    </div>
    <div style="border:1px solid var(--border);padding:var(--sp-l)">
      <div
        style="font-size:var(--fs-s);color:var(--t2);font-family:var(--font-mono);letter-spacing:1px;margin-bottom:var(--sp-m)"
      >
        STEP 3 — RESTART DASHBOARD
      </div>
      <p style="color:var(--t2)">
        After creating projects, restart: <code>bash start.sh</code>
      </p>
      <p style="color:var(--t3);margin-top:var(--sp-s);font-size:var(--fs-s)">
        Projects must be in your Documents folder (or configure
        <code>n8n/config.json</code>)
      </p>
    </div>
  </div>`;
}

function SearchBar() {
  return html`<div class="search">
    <input
      name="search"
      placeholder="search projects..."
      value=${searchQuery.value}
      onInput=${(e) => (searchQuery.value = e.target.value)}
      onKeyDown=${(e) => {
        if (e.key === "Escape") {
          searchQuery.value = "";
          e.target.blur();
        }
      }}
    />
    <select
      style="background:var(--sf);color:var(--t2);border:1px solid var(--border);font-family:var(--font-mono);font-size:var(--fs-s);padding:var(--sp-xs)"
      onChange=${(e) => (sortBy.value = e.target.value)}
    >
      <option value="">sort: segment</option>
      <option value="name">name</option>
      <option value="uncommitted">dirty files</option>
      <option value="days">activity</option>
      <option value="status">status</option>
    </select>
    <button
      style="background:var(--sf);color:var(--t2);border:1px solid var(--border);padding:var(--sp-xs);cursor:pointer;font-size:var(--fs-s);font-family:var(--font-mono);min-width:28px"
      onClick=${() =>
        (viewMode.value = viewMode.value === "grid" ? "list" : "grid")}
      title="Toggle grid/list"
    >
      ${viewMode.value === "grid" ? "≡" : "⊞"}
    </button>
  </div>`;
}

function ActivePlanCard() {
  const plans = plansData.value.filter((p) => p.status === "active");
  if (!plans.length) return null;
  return html`${plans.map((plan) => {
    const done = (plan.steps || []).filter((s) => s.status === "done").length;
    const total = (plan.steps || []).length;
    const pct = total ? Math.round((done / total) * 100) : 0;
    return html`<div
      class="plan-panel"
      style="margin-bottom:var(--sp-s);cursor:pointer"
      onClick=${() => {
        showPlans.value = true;
        loadPlansData();
      }}
    >
      <div
        class="plan-hdr"
        style="display:flex;align-items:center;gap:var(--sp-s)"
      >
        <span style="color:var(--yellow)">📋</span>
        <span style="flex:1">${plan.title}</span>
        <div
          style="width:80px;height:4px;background:var(--border);border-radius:2px"
        >
          <div
            style="height:100%;background:var(--green);width:${pct}%;border-radius:2px"
          ></div>
        </div>
        <span style="font-size:var(--fs-s);color:var(--t3)"
          >${done}/${total}</span
        >
      </div>
    </div>`;
  })}`;
}

function PlanPanel() {
  const p = actionPlan.value;
  if (!p || !p.high_count) return null;
  const high = showPlan.value
    ? p.plan.filter((x) => x.priority === "HIGH")
    : [];
  return html`<div class="plan-panel">
    <div class="plan-hdr" onClick=${() => (showPlan.value = !showPlan.value)}>
      <span style="color:var(--accent)"
        >⚠ ${p.high_count} project(s) need attention</span
      >
      <span>${showPlan.value ? "▾" : "▸"}</span>
    </div>
    ${high.map(
      (item) =>
        html`<div
          class="plan-item"
          style="cursor:pointer"
          onClick=${() => (currentProject.value = item.project)}
        >
          <span class="proj">${item.project}</span>
          <span class="issue">${item.issues.join(" · ")}</span>
          <span class="act">→</span>
        </div>`,
    )}
  </div>`;
}

function QueuePanel() {
  const q = queueTasks.value.filter((t) => !t.done);
  if (!q.length) return null;
  return html`<div class="plan-panel" style="margin-bottom:var(--sp-m)">
    <div class="plan-hdr">
      <span style="color:var(--yellow)">📋 ${q.length} queued task(s)</span>
    </div>
    ${q.map(
      (t, i) =>
        html`<div
          class="plan-item"
          style="font-size:var(--fs-s);color:var(--t2);padding:var(--sp-xs) var(--sp-m)"
        >
          ${i + 1}. ${t.text}
        </div>`,
    )}
  </div>`;
}

function KeyboardHelp() {
  if (!showKbHelp.value) return null;
  return html`<div
    class="kb-overlay"
    onClick=${() => (showKbHelp.value = false)}
  >
    <div class="kb-box" onClick=${(e) => e.stopPropagation()}>
      <h2>keyboard shortcuts</h2>
      <div class="kb-row">
        <span class="kb-key">/</span
        ><span class="kb-desc">Focus chat input</span>
      </div>
      <div class="kb-row">
        <span class="kb-key">Esc</span
        ><span class="kb-desc">Back to dashboard</span>
      </div>
      <div class="kb-row">
        <span class="kb-key">D</span
        ><span class="kb-desc">Toggle dark/light theme</span>
      </div>
      <div class="kb-row">
        <span class="kb-key">S</span><span class="kb-desc">Focus search</span>
      </div>
      <div class="kb-row">
        <span class="kb-key">R</span><span class="kb-desc">Refresh data</span>
      </div>
      <div class="kb-row">
        <span class="kb-key">P</span
        ><span class="kb-desc">Toggle action plan</span>
      </div>
      <div class="kb-row">
        <span class="kb-key">G</span><span class="kb-desc">Open graph</span>
      </div>
      <div class="kb-row">
        <span class="kb-key">?</span><span class="kb-desc">Show this help</span>
      </div>
    </div>
  </div>`;
}

function AnalyticsBar() {
  const a = agents.value;
  const totalDirty = a.reduce((s, x) => s + (x.uncommitted || 0), 0);
  const totalLessons = a.reduce((s, x) => s + (x.lessons || 0), 0);
  const p = actionPlan.value;
  return html`<div class="analytics-bar">
    <div class="metric">
      <span>projects:</span><span class="v">${a.length}</span>
    </div>
    <div class="metric">
      <span>uncommitted:</span
      ><span class="v${totalDirty > 500 ? " alert" : ""}">${totalDirty}</span>
    </div>
    <div class="metric">
      <span>lessons:</span><span class="v">${totalLessons}</span>
    </div>
    ${p
      ? html`<div class="metric">
          <span>issues:</span
          ><span class="v${p.high_count ? " alert" : ""}"
            >${p.total_issues}</span
          >
        </div>`
      : null}
    ${(() => {
      const dc = Object.values(delegations.value).filter(
        (d) => d.status === "running" || d.status === "escalated",
      ).length;
      return dc
        ? html`<div class="metric">
            <span>delegating:</span
            ><span class="v" style="color:var(--cyan)">${dc}</span>
          </div>`
        : null;
    })()}
    ${inboxData.value.count
      ? html`<div class="metric">
          <span>inbox:</span
          ><span
            class="v"
            style="color:${inboxData.value.needs_user
              ? "var(--accent)"
              : "var(--green)"}"
            >${inboxData.value.count}</span
          >
        </div>`
      : null}
    <div style="flex:1"></div>
    <div class="metric">
      <span style="color:var(--t3)">${currentSoloSelectionLabel()}</span>
    </div>
    <div class="metric" title="Agent OS version">
      <span>v</span><span class="v">${appInfo.value?.version || "dev"}</span>
    </div>
    <div
      class="metric"
      style="cursor:pointer;color:var(--t3)"
      onClick=${() => (showKbHelp.value = true)}
    >
      ?
    </div>
  </div>`;
}

function SignalsPanel() {
  const data = signalsData.value || { signals: [], counts: {} };
  const counts = data.counts || { critical: 0, warn: 0, info: 0 };
  const signals = data.signals || [];
  return html`<div class="plan-panel" style="margin-bottom:var(--sp-m)">
    <div class="plan-hdr">
      <span>signals</span>
      <span style="font-size:var(--fs-s);color:var(--t3)"
        >${counts.critical || 0} critical · ${counts.warn || 0} warn ·
        ${counts.info || 0} info</span
      >
    </div>
    ${signals.length
      ? signals.slice(0, 5).map(
          (sig) =>
            html`<div class="plan-item" style="align-items:flex-start">
              <span class="proj">${sig.project || sig.source || "system"}</span>
              <span class="issue" style="flex:1">${sig.message}</span>
              <button
                class="action-btn"
                style="padding:2px 8px"
                onClick=${() => ackSignal(sig.id)}
              >
                ack
              </button>
            </div>`,
        )
      : html`<div class="plan-item">
          <span class="issue">No active signals</span>
        </div>`}
    <div style="padding:0 var(--sp-m) var(--sp-s)">
      <button class="action-btn" onClick=${() => loadSignals()}>refresh</button>
    </div>
  </div>`;
}

function WorkbenchFocus({ all }) {
  const attention = [...all]
    .filter((x) => x.blockers || (x.uncommitted || 0) > 20)
    .sort((a, b) => (b.uncommitted || 0) - (a.uncommitted || 0))
    .slice(0, 6);
  const active = all.filter((x) => x.task).slice(0, 4);
  const runningDelegations = Object.values(delegations.value || {}).filter(
    (d) => ["running", "escalated", "deciding"].includes(d?.status),
  );
  const pendingDelegations = Object.values(delegations.value || {}).filter(
    (d) => d?.status === "pending",
  );
  const queued = queueTasks.value.filter((t) => !t.done).slice(0, 4);
  const top = attention[0] || active[0] || all[0] || null;
  const focusLabel = top
    ? `${top.name}: ${top.blockers ? "blocked" : top.task || "needs triage"}`
    : "No active project focus";
  return html`<div class="workbench-hero">
    <div class="workbench-hero-main">
      <div class="workbench-eyebrow">operational focus</div>
      <h2>${focusLabel}</h2>
      <p>
        ${attention.length
          ? `${attention.length} project routes need attention. Start by clearing blockers/permissions/failures, then delegate the next safe wave.`
          : active.length
            ? `${active.length} projects are active. Monitor execution, verify outputs, and only open new work when routes are clean.`
            : "No active route is selected. Pick a project or ask the orchestrator for the safest next route."}
      </p>
      <div class="workbench-actions">
        <button
          onClick=${() => {
            activeFilter.value = "attention";
            composerDraftText.value =
              "[DASHBOARD_FULL]\nReview attention projects, explain blockers, and propose the safest unblock order.";
          }}
        >
          ask unblock order
        </button>
        <button
          onClick=${() => {
            chatCollabMode.value = "duo";
            activeRoomTab.value = "collaborate";
            composerDraftText.value =
              "Compare current operational state with both agents. Build a concise plan, then choose the execution lead.";
          }}
        >
          discuss with two
        </button>
        <button
          onClick=${() => {
            showPlans.value = true;
            loadPlansData();
          }}
        >
          open plans
        </button>
      </div>
    </div>
    <div class="workbench-pulse">
      <div>
        <b>${runningDelegations.length}</b>
        <span>running delegations</span>
      </div>
      <div>
        <b>${pendingDelegations.length}</b>
        <span>pending approvals</span>
      </div>
      <div>
        <b>${queued.length}</b>
        <span>queued tasks</span>
      </div>
    </div>
    <div class="workbench-stack">
      <div class="workbench-stack-head">
        <span>attention stack</span>
        <button onClick=${() => (activeFilter.value = "attention")}>
          filter
        </button>
      </div>
      ${(attention.length ? attention : active).slice(0, 6).map(
        (ag) =>
          html`<button
            class="workbench-route"
            onClick=${() => (currentProject.value = ag.name)}
          >
            <span class="dot ${ag.status || "sleeping"}"></span>
            <b>${ag.name}</b>
            <em>${ag.blockers ? "blocked" : ag.task || "triage"}</em>
            <small>${ag.uncommitted || 0} dirty</small>
          </button>`,
      )}
    </div>
  </div>`;
}

function ExecutionFlowStage() {
  const map = executionMap.value;
  useEffect(() => {
    let disposed = false;
    const refresh = () => {
      if (disposed) return;
      loadExecutionMap("", null, 180).catch((e) =>
        console.warn("main execution map refresh failed:", e),
      );
    };
    refresh();
    const timer = setInterval(refresh, 5000);
    return () => {
      disposed = true;
      clearInterval(timer);
    };
  }, []);
  const laneCount = (map?.lanes || []).length;
  const eventCount = (map?.events || []).length;
  return html`<section class="main-execution-stage">
    <div class="main-execution-head">
      <div>
        <div class="workbench-eyebrow">live execution flow</div>
        <h2>Карта веток: оркестратор → проектные агенты → feedback</h2>
      </div>
      <div class="main-execution-meta">
        <span>${laneCount} веток</span>
        <span>${eventCount} событий</span>
        <button onClick=${() => loadExecutionMap("", null, 180)}>
          refresh
        </button>
      </div>
    </div>
    ${map?.status === "ok"
      ? html`<${ExecutionMapCard}
          map=${map}
          onRefresh=${() => loadExecutionMap("", null, 180)}
        />`
      : html`<div class="main-execution-empty">
          <b>Карта исполнения загружается</b>
          <span>
            Здесь будет горизонтальный трек как gitflow: главная ветка
            оркестратора, ниже ветки проектных агентов, стрелки делегаций и
            возвраты feedback.
          </span>
        </div>`}
  </section>`;
}

function projectHasDelegation(project) {
  return Object.values(delegations.value || {}).some(
    (item) =>
      item?.project === project &&
      !["done", "failed", "cancelled", "rejected", "error"].includes(
        item?.status || "",
      ),
  );
}

function projectHasPlan(project) {
  return (plansData.value || []).some(
    (plan) =>
      plan?.project === project ||
      (plan?.steps || []).some((step) => step?.project === project),
  );
}

function applyRailFilter(items) {
  const af = activeFilter.value;
  if (af === "attention") {
    return items.filter((x) => x.blockers || (x.uncommitted || 0) > 20);
  }
  if (af === "active") return items.filter((x) => x.task);
  if (af === "stale") return items.filter((x) => (x.days || 999) > 7);
  if (af === "dirty") return items.filter((x) => (x.uncommitted || 0) > 0);
  if (af === "delegation")
    return items.filter((x) => projectHasDelegation(x.name));
  if (af === "plan") return items.filter((x) => projectHasPlan(x.name));
  return items;
}

function ProjectRail({ items, segMap, useFlat, flatItems }) {
  const groups = useFlat
    ? [["all", flatItems]]
    : Object.entries(segMap).filter(([_, list]) => list.length);
  const railRef = useRef(null);
  const navItems = (useFlat ? flatItems : groups.flatMap(([, list]) => list))
    .filter(Boolean)
    .map((item) => item.name);
  useEffect(() => {
    const el = railRef.current;
    if (!el) return;
    const saved = Number(
      localStorage.getItem("agentos.projectRail.scroll") || 0,
    );
    if (saved > 0) requestAnimationFrame(() => (el.scrollTop = saved));
  }, []);
  const openProject = (project) => {
    const el = railRef.current;
    if (el)
      localStorage.setItem("agentos.projectRail.scroll", String(el.scrollTop));
    currentProject.value = project;
  };
  const moveSelection = (delta) => {
    if (!navItems.length) return;
    const current = currentProject.value;
    const idx = Math.max(0, navItems.indexOf(current));
    const next = navItems[(idx + delta + navItems.length) % navItems.length];
    openProject(next);
  };
  if (!items.length) {
    return html`<div class="project-rail-empty">
      <b>No projects match</b>
      <button
        onClick=${() => {
          activeFilter.value = "";
          searchQuery.value = "";
        }}
      >
        clear filters
      </button>
    </div>`;
  }
  return html`<div
    class="project-rail-list"
    ref=${railRef}
    tabindex="0"
    onScroll=${(e) =>
      localStorage.setItem(
        "agentos.projectRail.scroll",
        String(e.currentTarget.scrollTop),
      )}
    onKeyDown=${(e) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        moveSelection(1);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        moveSelection(-1);
      } else if (e.key === "Escape") {
        currentProject.value = null;
      } else if (e.key === "Enter" && !currentProject.value && navItems[0]) {
        openProject(navItems[0]);
      }
    }}
  >
    ${groups.map(
      ([name, list]) =>
        html`<section class="project-rail-group">
          <div class="project-rail-title">
            <span>${name || "all projects"}</span>
            <em>${list.length}</em>
          </div>
          ${list.map(
            (ag) =>
              html`<button
                class="project-rail-row ${currentProject.value === ag.name
                  ? "selected"
                  : ""}"
                onClick=${() => openProject(ag.name)}
                title=${`${ag.name}: ${ag.task || ag.status || "sleeping"}`}
              >
                <span class="dot ${ag.status || "sleeping"}"></span>
                <span class="project-rail-name">${ag.name}</span>
                <span class="project-rail-state">
                  ${ag.task || ag.status || "sleeping"}
                </span>
                <span class="project-rail-badges">
                  ${(ag.uncommitted || 0) > 0
                    ? html`<span
                        class="project-rail-dirty ${(ag.uncommitted || 0) > 20
                          ? "hot"
                          : ""}"
                        >${ag.uncommitted}</span
                      >`
                    : null}
                  ${(ag.days || 0) > 7
                    ? html`<span class="project-rail-age">${ag.days}d</span>`
                    : null}
                  ${ag.blockers
                    ? html`<span class="project-rail-blocked">block</span>`
                    : null}
                  ${activities.value?.[ag.name]
                    ? html`<span class="project-rail-live">live</span>`
                    : null}
                  ${projectHasDelegation(ag.name)
                    ? html`<span class="project-rail-deleg">deleg</span>`
                    : null}
                  ${projectHasPlan(ag.name)
                    ? html`<span class="project-rail-plan">plan</span>`
                    : null}
                </span>
              </button>`,
          )}
        </section>`,
    )}
  </div>`;
}

function RailQuickFilters({ all, visible }) {
  const counts = {
    attention: all.filter((x) => x.blockers || (x.uncommitted || 0) > 20)
      .length,
    active: all.filter((x) => x.task).length,
    stale: all.filter((x) => (x.days || 999) > 7).length,
    dirty: all.filter((x) => (x.uncommitted || 0) > 0).length,
    delegation: all.filter((x) => projectHasDelegation(x.name)).length,
    plan: all.filter((x) => projectHasPlan(x.name)).length,
  };
  const filterButton = (id, label, count) =>
    html`<button
      class=${activeFilter.value === id ? "active" : ""}
      onClick=${() => {
        activeFilter.value = activeFilter.value === id ? "" : id;
      }}
    >
      ${label}<b>${count}</b>
    </button>`;
  return html`<div class="project-rail-quick">
    <div>
      ${filterButton("attention", "attention", counts.attention)}
      ${filterButton("active", "active", counts.active)}
      ${filterButton("stale", "stale", counts.stale)}
      ${filterButton("dirty", "dirty", counts.dirty)}
      ${filterButton("delegation", "deleg", counts.delegation)}
      ${filterButton("plan", "plan", counts.plan)}
    </div>
    <span>${visible.length}/${all.length}</span>
  </div>`;
}

function graphBounds(nodes) {
  if (!nodes.length) return { x: 0, y: 0, w: 1000, h: 520 };
  const minX = Math.min(...nodes.map((n) => Number(n.x || 0)));
  const minY = Math.min(...nodes.map((n) => Number(n.y || 0)));
  const maxX = Math.max(
    ...nodes.map((n) => Number(n.x || 0) + Number(n.w || 160)),
  );
  const maxY = Math.max(
    ...nodes.map((n) => Number(n.y || 0) + Number(n.h || 28)),
  );
  return {
    x: minX - 40,
    y: minY - 40,
    w: Math.max(760, maxX - minX + 80),
    h: Math.max(420, maxY - minY + 80),
  };
}

function GraphMap({ graph }) {
  const nodes = graph?.nodes || [];
  const edges = graph?.edges || [];
  const nodeById = new Map(nodes.map((node) => [node.id, node]));
  const selected = graphSelected.value;
  const bounds = graphBounds(nodes);
  const selectedIds = new Set();
  if (selected?.id) {
    selectedIds.add(selected.id);
    edges.forEach((edge) => {
      if (edge.source === selected.id) selectedIds.add(edge.target);
      if (edge.target === selected.id) selectedIds.add(edge.source);
    });
  }
  const edgePath = (edge) => {
    const source = nodeById.get(edge.source);
    const target = nodeById.get(edge.target);
    if (!source || !target) return "";
    const sx = Number(source.x || 0) + Number(source.w || 160) / 2;
    const sy = Number(source.y || 0) + Number(source.h || 28) / 2;
    const tx = Number(target.x || 0) + Number(target.w || 160) / 2;
    const ty = Number(target.y || 0) + Number(target.h || 28) / 2;
    const mid = sx + (tx - sx) * 0.5;
    return `M ${sx} ${sy} C ${mid} ${sy}, ${mid} ${ty}, ${tx} ${ty}`;
  };
  return html`<div class="graph-map-wrap">
    <svg
      class="graph-svg"
      viewBox=${`${bounds.x} ${bounds.y} ${bounds.w} ${bounds.h}`}
      role="img"
      aria-label="Project dependency graph"
    >
      <defs>
        <marker
          id="graph-arrow"
          viewBox="0 0 10 10"
          refX="9"
          refY="5"
          markerWidth="5"
          markerHeight="5"
          orient="auto-start-reverse"
        >
          <path d="M 0 0 L 10 5 L 0 10 z" />
        </marker>
      </defs>
      ${(graph.groups || []).map(
        (group) =>
          html`<g class="graph-group">
            <rect
              x=${group.x - 8}
              y=${group.y - 8}
              width=${group.w + 16}
              height=${group.h + 16}
              rx="8"
            />
            <text x=${group.x + 8} y=${group.y + 14}>${group.name}</text>
          </g>`,
      )}
      <g class="graph-edges">
        ${edges.map((edge) => {
          const hot =
            !selected?.id ||
            edge.source === selected.id ||
            edge.target === selected.id;
          const path = edgePath(edge);
          return path
            ? html`<path
                class=${`graph-edge ${edge.kind || "edge"} ${hot ? "hot" : "dim"}`}
                d=${path}
                marker-end="url(#graph-arrow)"
              />`
            : null;
        })}
      </g>
      <g class="graph-nodes">
        ${nodes.map((node) => {
          const w = Number(node.w || 160);
          const h = Number(node.h || 28);
          const active = selected?.id === node.id;
          const hot = !selected?.id || selectedIds.has(node.id);
          return html`<g
            class=${`graph-node ${node.kind || "node"} ${active ? "selected" : ""} ${hot ? "hot" : "dim"}`}
            transform=${`translate(${Number(node.x || 0)} ${Number(node.y || 0)})`}
            onClick=${() => (graphSelected.value = node)}
          >
            <rect width=${w} height=${h} rx="4" />
            <text x="10" y=${Math.min(18, h - 8)}>
              ${node.path || node.label}
            </text>
            ${node.metrics?.in_cycle
              ? html`<circle cx=${w - 12} cy=${h / 2} r="4" />`
              : null}
          </g>`;
        })}
      </g>
    </svg>
  </div>`;
}

function GraphInspector() {
  const graph = graphData.value;
  const node = graphSelected.value;
  if (!graph || !node) return null;
  const outgoing = (graph.edges || []).filter(
    (edge) => edge.source === node.id,
  );
  const incoming = (graph.edges || []).filter(
    (edge) => edge.target === node.id,
  );
  const neighbors = (edges, side) =>
    edges
      .map((edge) =>
        (graph.nodes || []).find((candidate) => candidate.id === edge[side]),
      )
      .filter(Boolean);
  const project = graphLevel.value === "overview" ? "" : graphLevel.value;
  const askOrchestrator = (prompt) => {
    currentProject.value = null;
    showGraph.value = false;
    sendMessage(prompt);
  };
  const attachToChat = (prompt) => {
    currentProject.value = null;
    composerDraftText.value = prompt;
    showGraph.value = false;
    showToast("Graph context attached to chat", "success", 1500);
  };
  return html`<div class="panel" style="min-width:320px;max-width:420px">
    <h3>${node.path || node.label}</h3>
    <div
      style="font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-s)"
    >
      ${node.kind} · ${node.group || "ungrouped"} · Ca ${node.metrics?.ca || 0}
      · Ce ${node.metrics?.ce || 0}
    </div>
    <div
      style="font-size:var(--fs-s);color:var(--t2);margin-bottom:var(--sp-s)"
    >
      ${node.id}
    </div>
    ${graphLevel.value === "overview" && node.kind === "project"
      ? html`<button class="action-btn" onClick=${() => loadGraph(node.label)}>
          open project graph
        </button>`
      : null}
    <div class="graph-inspector-actions">
      ${graphLevel.value === "overview" && node.kind === "project"
        ? html`<button
              class="action-btn"
              onClick=${() => {
                currentProject.value = node.label;
                showGraph.value = false;
              }}
            >
              open project chat
            </button>
            <button
              class="action-btn"
              onClick=${() =>
                attachToChat(
                  `[GRAPH_CONTEXT:${node.label}]\nReview routing and dependency risks for this project before delegation.`,
                )}
            >
              attach graph
            </button>
            <button
              class="action-btn"
              onClick=${() =>
                askOrchestrator(
                  `[GRAPH_CONTEXT:${node.label}]\nReview routing and dependency risks for this project before delegation.`,
                )}
            >
              ask with graph
            </button>`
        : null}
      ${project && node.path
        ? html`<button
              class="action-btn"
              onClick=${() =>
                attachToChat(
                  `[GRAPH_IMPACT:${project}:${node.path}]\nExplain what breaks if I change this file and propose safe delegation steps.`,
                )}
            >
              attach impact
            </button>
            <button
              class="action-btn"
              onClick=${() =>
                askOrchestrator(
                  `[GRAPH_IMPACT:${project}:${node.path}]\nExplain what breaks if I change this file and propose safe delegation steps.`,
                )}
            >
              ask impact
            </button>
            <button
              class="action-btn"
              onClick=${() =>
                askOrchestrator(
                  `[GRAPH_DEPENDENTS:${project}:${node.path}]\nShow who depends on this module and what to verify.`,
                )}
            >
              dependents
            </button>`
        : null}
      ${project
        ? html`<button
              class="action-btn"
              onClick=${() => attachToChat(`[GRAPH_VERIFY:${project}]`)}
            >
              attach verify
            </button>
            <button
              class="action-btn"
              onClick=${() => askOrchestrator(`[GRAPH_VERIFY:${project}]`)}
            >
              verify graph
            </button>`
        : null}
    </div>
    <div style="margin-top:var(--sp-m)">
      <div
        style="font-family:var(--font-mono);font-size:var(--fs-s);margin-bottom:var(--sp-xs)"
      >
        outgoing
      </div>
      ${neighbors(outgoing, "target")
        .slice(0, 8)
        .map(
          (item) =>
            html`<div class="mod" onClick=${() => (graphSelected.value = item)}>
              <span>${item.path || item.label}</span>
            </div>`,
        )}
    </div>
    <div style="margin-top:var(--sp-m)">
      <div
        style="font-family:var(--font-mono);font-size:var(--fs-s);margin-bottom:var(--sp-xs)"
      >
        incoming
      </div>
      ${neighbors(incoming, "source")
        .slice(0, 8)
        .map(
          (item) =>
            html`<div class="mod" onClick=${() => (graphSelected.value = item)}>
              <span>${item.path || item.label}</span>
            </div>`,
        )}
    </div>
  </div>`;
}

function GraphView() {
  const graph = graphData.value;
  const nodes = graph?.nodes || [];
  return html`<div class="content">
    <div
      class="back"
      onClick=${() => {
        showGraph.value = false;
        graphSelected.value = null;
      }}
    >
      ← back to dashboard
    </div>
    <h2
      style="font-size:var(--fs-xl);margin:var(--sp-m) 0;letter-spacing:2px;font-family:var(--font-mono)"
    >
      GRAPH
    </h2>
    <div
      style="display:flex;gap:var(--sp-s);margin-bottom:var(--sp-m);flex-wrap:wrap"
    >
      <button class="action-btn" onClick=${() => loadGraph("overview")}>
        overview
      </button>
      ${agents.value
        .slice(0, 12)
        .map(
          (agent) =>
            html`<button
              class="action-btn"
              onClick=${() => loadGraph(agent.name)}
            >
              ${agent.name}
            </button>`,
        )}
    </div>
    ${graph?.error
      ? html`<div class="panel" style="color:var(--accent)">
          Graph error: ${graph.error}
        </div>`
      : !graph
        ? html`<div class="panel">Loading graph…</div>`
        : html`<div class="panels graph-layout">
            <div class="panel graph-main-panel">
              <h3>
                ${graphLevel.value === "overview"
                  ? "overview"
                  : graphLevel.value}
              </h3>
              <div
                style="font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-s)"
              >
                ${graph.stats?.total_nodes || 0} nodes ·
                ${graph.stats?.total_edges || 0} edges ·
                ${graph.stats?.cycle_count || 0} cycles
              </div>
              <${GraphMap} graph=${graph} />
              <div class="graph-node-list">
                ${nodes.map(
                  (node) =>
                    html`<button
                      class="action-btn"
                      data-active=${graphSelected.value?.id === node.id}
                      onClick=${() => (graphSelected.value = node)}
                    >
                      ${node.path || node.label}
                    </button>`,
                )}
              </div>
            </div>
            <${GraphInspector} />
          </div>`}
  </div>`;
}

function DashboardView() {
  const seg = segments.value;
  let a = agents.value;
  const sq = searchQuery.value.toLowerCase();
  if (sq)
    a = a.filter(
      (x) =>
        x.name.toLowerCase().includes(sq) ||
        x.task?.toLowerCase().includes(sq) ||
        (x.segment || "").toLowerCase().includes(sq),
    );
  const af = activeFilter.value;
  if (af === "attention")
    a = a.filter((x) => x.blockers || (x.uncommitted || 0) > 20);
  if (af === "active") a = a.filter((x) => x.task);
  if (af === "stale") a = a.filter((x) => (x.days || 999) > 7);
  const segMap = {};
  const assigned = new Set();
  const otherItems = [];
  for (const [name, projects] of Object.entries(seg)) {
    const items = a.filter((x) => projects.includes(x.name));
    if (items.length > 2) {
      segMap[name] = items;
    } else {
      otherItems.push(...items);
    }
    projects.forEach((p) => assigned.add(p));
  }
  const unassigned = a.filter((x) => !assigned.has(x.name));
  otherItems.push(...unassigned);
  if (otherItems.length) segMap["Other"] = otherItems;
  if (isLoading.value)
    return html`<div class="content">
      <${StatsRow} />
      <div class="grid">
        ${[0, 1, 2, 3, 4, 5].map(
          (i) => html`<div class="skeleton" key=${i} />`,
        )}
      </div>
    </div>`;
  if (!a.length && !af && !sq)
    return html`<div class="content"><${StatsRow} /><${EmptyState} /></div>`;
  if (!a.length && (af || sq))
    return html`<div class="content">
      <${StatsRow} />
      <div
        style="padding:var(--sp-2xl);text-align:center;color:var(--t3);font-family:var(--font-mono)"
      >
        <div style="font-size:var(--fs-l);margin-bottom:var(--sp-m)">
          No projects match
        </div>
        <button
          class="action-btn"
          onClick=${() => {
            activeFilter.value = "";
            searchQuery.value = "";
          }}
        >
          clear filters
        </button>
      </div>
    </div>`;
  const filterBar = af
    ? html`<div
        style="font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-s)"
      >
        Filter: ${af}
        <span
          style="cursor:pointer;color:var(--accent)"
          onClick=${() => (activeFilter.value = "")}
          >clear</span
        >
      </div>`
    : null;
  // Apply sort
  const sb = sortBy.value;
  if (sb === "name") a.sort((x, y) => x.name.localeCompare(y.name));
  else if (sb === "uncommitted")
    a.sort((x, y) => (y.uncommitted || 0) - (x.uncommitted || 0));
  else if (sb === "days") a.sort((x, y) => (x.days || 999) - (y.days || 999));
  else if (sb === "status") {
    const o = { blocked: 0, working: 1, idle: 2, sleeping: 3 };
    a.sort((x, y) => (o[x.status] ?? 4) - (o[y.status] ?? 4));
  }
  // B3: when sort active, render flat list; when default, render by segment
  const useFlat = sb !== "";
  const flatItems = useFlat ? a : [];
  return html`<div class="content">
    <${StatsRow} /><${ActivePlanCard} /><${SearchBar} /><${PlanPanel} /><${QueuePanel} /><${SignalsPanel} />${filterBar}
    ${useFlat
      ? html`<div class="seg-title">
          <span>All Projects (sorted by ${sb})</span
          ><span style="font-size:var(--fs-s);color:var(--t3)"
            >(${flatItems.length})</span
          >
        </div>`
      : null}
    ${(useFlat
      ? [["", flatItems]]
      : Object.entries(segMap).filter(([_, i]) => i.length)
    ).map(
      ([name, items]) => html`
        ${!useFlat
          ? html`<div class="seg-title">
              <span>${name}</span
              ><span style="font-size:var(--fs-s);color:var(--t3)"
                >(${items.length})</span
              >
            </div>`
          : null}
        ${viewMode.value === "list"
          ? html`<div style="margin-bottom:var(--sp-m)">
              ${items.map(
                (ag) =>
                  html`<div
                    style="display:flex;align-items:center;gap:var(--sp-s);padding:var(--sp-2xs) var(--sp-s);border-bottom:1px solid var(--border);cursor:pointer;font-size:var(--fs-s)"
                    onClick=${() => (currentProject.value = ag.name)}
                  >
                    <span
                      class="dot ${ag.status}"
                      style="width:6px;height:6px"
                    ></span>
                    <span style="min-width:160px;font-weight:500"
                      >${ag.name}</span
                    >
                    <span
                      style="color:var(--t3);min-width:100px;font-family:var(--font-mono);font-size:var(--fs-s)"
                      >${ag.branch || "—"}</span
                    >
                    <span
                      style="color:${(ag.uncommitted || 0) > 10
                        ? "var(--accent)"
                        : "var(--t3)"};min-width:60px;font-family:var(--font-mono);font-size:var(--fs-s)"
                      >${ag.uncommitted || 0} dirty</span
                    >
                    <span
                      style="color:var(--t3);font-size:var(--fs-s);flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap"
                      >${ag.task || ""}</span
                    >
                  </div>`,
              )}
            </div>`
          : html`<div class="grid">
              ${items.map((ag) => html`<${Tile} a=${ag} />`)}
            </div>`}
      `,
    )}
  </div>`;
}

function DashboardWorkbenchView() {
  const seg = segments.value;
  const allAgents = agents.value;
  let visibleAgents = allAgents;
  const sq = searchQuery.value.toLowerCase();
  if (sq) {
    visibleAgents = visibleAgents.filter(
      (x) =>
        x.name.toLowerCase().includes(sq) ||
        x.task?.toLowerCase().includes(sq) ||
        (x.segment || "").toLowerCase().includes(sq),
    );
  }
  const af = activeFilter.value;
  visibleAgents = applyRailFilter(visibleAgents);

  const sb = sortBy.value;
  if (sb === "name") {
    visibleAgents = [...visibleAgents].sort((x, y) =>
      x.name.localeCompare(y.name),
    );
  } else if (sb === "uncommitted") {
    visibleAgents = [...visibleAgents].sort(
      (x, y) => (y.uncommitted || 0) - (x.uncommitted || 0),
    );
  } else if (sb === "days") {
    visibleAgents = [...visibleAgents].sort(
      (x, y) => (x.days || 999) - (y.days || 999),
    );
  } else if (sb === "status") {
    const o = { blocked: 0, working: 1, idle: 2, sleeping: 3 };
    visibleAgents = [...visibleAgents].sort(
      (x, y) => (o[x.status] ?? 4) - (o[y.status] ?? 4),
    );
  }

  const segMap = {};
  const assigned = new Set();
  const otherItems = [];
  for (const [name, projects] of Object.entries(seg)) {
    const items = visibleAgents.filter((x) => projects.includes(x.name));
    if (items.length > 2) segMap[name] = items;
    else otherItems.push(...items);
    projects.forEach((p) => assigned.add(p));
  }
  const unassigned = visibleAgents.filter((x) => !assigned.has(x.name));
  otherItems.push(...unassigned);
  if (otherItems.length) segMap["Other"] = otherItems;
  const useFlat = sb !== "";
  const flatItems = useFlat ? visibleAgents : [];
  const filterBar = af
    ? html`<div class="project-rail-filter">
        Filter: ${af}
        <span onClick=${() => (activeFilter.value = "")}>clear</span>
      </div>`
    : null;

  if (isLoading.value) {
    return html`<div class="content workbench-content">
      <section class="workbench-primary">
        <${StatsRow} />
        <div class="grid">
          ${[0, 1, 2, 3, 4, 5].map(
            (i) => html`<div class="skeleton" key=${i} />`,
          )}
        </div>
      </section>
    </div>`;
  }
  if (!allAgents.length && !af && !sq) {
    return html`<div class="content"><${StatsRow} /><${EmptyState} /></div>`;
  }

  return html`<div class="content workbench-content">
    <aside class="project-rail">
      <div class="project-rail-head">
        <div>
          <div class="workbench-eyebrow">project navigation</div>
          <b>${visibleAgents.length}/${allAgents.length} visible</b>
        </div>
        <button
          onClick=${() => {
            activeFilter.value = "";
            searchQuery.value = "";
            sortBy.value = "";
          }}
        >
          reset
        </button>
      </div>
      <${SearchBar} />
      <${RailQuickFilters} all=${allAgents} visible=${visibleAgents} />
      ${filterBar}
      <${ProjectRail}
        items=${visibleAgents}
        segMap=${segMap}
        useFlat=${useFlat}
        flatItems=${flatItems}
      />
    </aside>
    <section class="workbench-primary">
      <${StatsRow} />
      <${ExecutionFlowStage} />
      <${WorkbenchFocus} all=${allAgents} />
      <div class="workbench-panels">
        <${ActivePlanCard} />
        <${PlanPanel} />
        <${QueuePanel} />
        <${SignalsPanel} />
      </div>
    </section>
  </div>`;
}

function ProjectWorkbenchView() {
  const seg = segments.value;
  const allAgents = agents.value;
  let visibleAgents = allAgents;
  const sq = searchQuery.value.toLowerCase();
  if (sq) {
    visibleAgents = visibleAgents.filter(
      (x) =>
        x.name.toLowerCase().includes(sq) ||
        x.task?.toLowerCase().includes(sq) ||
        (x.segment || "").toLowerCase().includes(sq),
    );
  }
  const af = activeFilter.value;
  visibleAgents = applyRailFilter(visibleAgents);
  const sb = sortBy.value;
  if (sb === "name") {
    visibleAgents = [...visibleAgents].sort((x, y) =>
      x.name.localeCompare(y.name),
    );
  } else if (sb === "uncommitted") {
    visibleAgents = [...visibleAgents].sort(
      (x, y) => (y.uncommitted || 0) - (x.uncommitted || 0),
    );
  } else if (sb === "days") {
    visibleAgents = [...visibleAgents].sort(
      (x, y) => (x.days || 999) - (y.days || 999),
    );
  } else if (sb === "status") {
    const o = { blocked: 0, working: 1, idle: 2, sleeping: 3 };
    visibleAgents = [...visibleAgents].sort(
      (x, y) => (o[x.status] ?? 4) - (o[y.status] ?? 4),
    );
  }
  const segMap = {};
  const assigned = new Set();
  const otherItems = [];
  for (const [name, projects] of Object.entries(seg)) {
    const items = visibleAgents.filter((x) => projects.includes(x.name));
    if (items.length > 2) segMap[name] = items;
    else otherItems.push(...items);
    projects.forEach((p) => assigned.add(p));
  }
  const unassigned = visibleAgents.filter((x) => !assigned.has(x.name));
  otherItems.push(...unassigned);
  if (otherItems.length) segMap["Other"] = otherItems;
  const useFlat = sb !== "";
  const flatItems = useFlat ? visibleAgents : [];
  const filterBar = af
    ? html`<div class="project-rail-filter">
        Filter: ${af}
        <span onClick=${() => (activeFilter.value = "")}>clear</span>
      </div>`
    : null;
  return html`<div class="content workbench-content project-workbench-content">
    <aside class="project-rail">
      <div class="project-rail-head">
        <div>
          <div class="workbench-eyebrow">project navigation</div>
          <b>${visibleAgents.length}/${allAgents.length} visible</b>
        </div>
        <button onClick=${() => (currentProject.value = null)}>
          dashboard
        </button>
      </div>
      <${SearchBar} />
      <${RailQuickFilters} all=${allAgents} visible=${visibleAgents} />
      ${filterBar}
      <${ProjectRail}
        items=${visibleAgents}
        segMap=${segMap}
        useFlat=${useFlat}
        flatItems=${flatItems}
      />
    </aside>
    <section class="workbench-primary project-workbench-primary">
      <${DetailView} />
    </section>
  </div>`;
}

export {
  OrchWarning,
  Breadcrumb,
  App,
  Header,
  StatsRow,
  EmptyState,
  SearchBar,
  ActivePlanCard,
  PlanPanel,
  QueuePanel,
  KeyboardHelp,
  AnalyticsBar,
  DashboardView,
  DashboardWorkbenchView,
  ProjectWorkbenchView,
};
