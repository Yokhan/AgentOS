// AgentOS chat components — sidebar, messages, delegation, inbox
import {
  html,
  signal,
  useRef,
  useEffect,
  useState,
} from "/vendor/preact-bundle.mjs";
import { esc, md, ft, SC, SL } from "/utils.js";
import { __IS_TAURI, __invoke } from "/bridge.js";
import {
  agents,
  currentProject,
  sideMessages,
  sideTitle,
  composerDraftText,
  chatPageInfo,
  chatHistoryLoading,
  streamText,
  isStreaming,
  streamChain,
  activeRun,
  thinkStart,
  lastUserMsg,
  selectedClaudeModel,
  selectedClaudeEffort,
  selectedCodexModel,
  selectedCodexEffort,
  selectedSoloProvider,
  chatRunMode,
  chatAccessLevel,
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
  inboxData,
  slashOpen,
  slashItems,
  pastedImg,
  delegations,
  delegStreams,
  modules,
  projectPlan,
  feedItems,
  toasts,
  clock,
  activities,
  showToast,
  chatCollabMode,
  activeRoomTab,
  dualSessionData,
  activeDualSession,
  dualBusy,
  activeScope,
  orchestrationMap,
  executionTimeline,
  eventContract,
  permData,
} from "/store.js";
import {
  togVoice,
  hdlFiles,
  rmFile,
  saveDr,
  loadDr,
  clrDr,
  handleSlash,
  execSlash,
  handlePaste,
  loadChat,
  loadOlderChat,
  loadModules,
  loadGraph,
  loadPlansData,
  loadProjectPlan,
  sendMessage,
  approveDel,
  rejectDel,
  processInbox,
  ensureDualSession,
  runDualParticipant,
  runDualRound,
  runDualRoomAction,
  generateStrategy,
  createAdhocPlanFromRoom,
  createRoomProjectSession,
  loadDualSession,
  loadActiveScope,
  loadOrchestrationMap,
  loadExecutionTimeline,
  loadEventContract,
  setDualOrchestrator,
} from "/api.js";
import {
  CLAUDE_EFFORT_OPTIONS,
  CLAUDE_MODEL_OPTIONS,
  codexModelOptionsFromStatus,
  codexEffortOptionsForModel,
} from "/provider-caps.js";

const inpHist = [];
let hIdx = -1;
let draftT = null;

function normalizeDuoView(value) {
  const next = String(value || "chat")
    .trim()
    .toLowerCase();
  if (next === "room") return "collaborate";
  if (next === "work" || next === "reviews") return "execute";
  if (next === "collaborate" || next === "execute") return next;
  return "chat";
}

function fallbackScope(project) {
  const title = project || "_orchestrator";
  return {
    kind: project ? "project" : "global",
    label: project ? "Project" : "Global",
    title,
    project: project || "",
    breadcrumbs: [
      { kind: "global", label: "Global" },
      ...(project ? [{ kind: "project", label: project }] : []),
    ],
    available_actions: [
      { id: "ask_both", label: "Ask both", tone: "neutral" },
      {
        id: project ? "create_plan" : "create_strategy",
        label: project ? "Create plan" : "Create strategy",
        tone: "primary",
      },
    ],
    summary: project
      ? `Duo actions apply to project: ${project}`
      : "Duo is operating at global orchestration level.",
  };
}

function scopeNextTitle(scope, leadLabel) {
  const leader = leadLabel || "Chosen lead";
  if (scope?.kind === "work_item") return `${leader} continues this task`;
  if (scope?.kind === "plan") return `${leader} continues this plan`;
  if (scope?.kind === "strategy") return `${leader} turns strategy into work`;
  if (scope?.kind === "project") return `${leader} scopes project work`;
  return `${leader} coordinates globally`;
}

function shortModelLabel(model) {
  return String(model || "auto").replace(/^gpt-/, "gpt ");
}

function codexModelSourceLabel(codexStatus) {
  if (codexStatus?.models_source) return codexStatus.models_source;
  const sources = new Set(
    (codexStatus?.models || []).map((model) => model?.source).filter(Boolean),
  );
  if (sources.size) return [...sources].join("+");
  return "fallback";
}

function providerAvailability(provider, providerStatus) {
  const info = providerStatus?.providers?.[provider] || {};
  if (info.available === false) return "offline";
  if (provider === "codex") return info.transport || "cli";
  return "ready";
}

function summarizeDelegationsForRoute(allDelegations, project) {
  const entries = Object.entries(allDelegations || {}).filter(([_, item]) =>
    project ? item?.project === project : true,
  );
  const counts = {
    total: entries.length,
    pending: 0,
    running: 0,
    failed: 0,
    done: 0,
  };
  for (const [, item] of entries) {
    const status = item?.status || "pending";
    if (status === "done") counts.done++;
    else if (status === "running" || status === "escalated") counts.running++;
    else if (status === "failed" || status === "error") counts.failed++;
    else counts.pending++;
  }
  return { ...counts, entries: entries.slice(0, 4) };
}

function duoInputLabel(action, lead, target) {
  if (action === "send") return `${lead?.label || "selected lead"} executes`;
  if (action === "challenge") return `challenge ${target?.label || "target"}`;
  if (action === "mention") return `message ${target?.label || "agent"}`;
  if (action === "rebuttal") return `rebut ${target?.label || "agent"}`;
  if (action === "promote_plan") return "create project/global plan";
  if (action === "promote_strategy") return "create strategy";
  if (action === "child_session") return "create project room";
  return "both agents review";
}

function RouteCard({
  route,
  providerOptions,
  modelOptions,
  effortOptions,
  accessOptions,
  onToggleMode,
  onAccess,
  onProvider,
  onModel,
  onEffort,
  onProjectContext,
  onGraphContext,
  onReview,
  onAskBoth,
  onMakePlan,
  onLeadExecute,
}) {
  const run = route.run;
  const warnings = route.warnings || [];
  return html`<div class="route-card ${warnings.length ? "has-warnings" : ""}">
    <div class="route-main">
      <div>
        <div class="route-eyebrow">next message route</div>
        <div class="route-path">
          <span>${route.target}</span><b>-></b><span>${route.inputLabel}</span>
        </div>
      </div>
      <div class="route-pills">
        <span>${route.provider}</span>
        <span>${shortModelLabel(route.model)}</span>
        <span>${route.mode}/${route.access}</span>
        <span>${route.providerState}</span>
      </div>
    </div>
    <div class="route-meta">
      <span>models: ${route.modelCount} ${route.modelSource}</span>
      <span
        >history:
        ${route.historyLoaded}/${route.historyTotal ||
        route.historyLoaded}</span
      >
      <span>scope: ${route.scope}</span>
      <span>
        delegations: ${route.delegations.pending} pending,
        ${route.delegations.running} running, ${route.delegations.failed}
        failed, ${route.delegations.done} done
      </span>
      ${run
        ? html`<span class="route-run ${run.status || "running"}">
            ${run.status || "running"} ${run.phase || "run"} ${run.detail || ""}
          </span>`
        : null}
    </div>
    ${warnings.length
      ? html`<div class="route-warnings">
          ${warnings.map((warning) => html`<span>${warning}</span>`)}
        </div>`
      : null}
    <div class="route-controls">
      ${route.duo
        ? html`<button onClick=${onAskBoth}>ask both</button>
            <button onClick=${onMakePlan}>make plan</button>
            <button
              class="primary"
              disabled=${!route.canLeadExecute}
              onClick=${onLeadExecute}
            >
              lead executes
            </button>`
        : html`<button class="primary" onClick=${onToggleMode}>
              ${route.mode === "plan" ? "switch to act" : "plan first"}
            </button>
            <select
              value=${route.accessRaw}
              disabled=${route.mode === "plan"}
              onChange=${(e) => onAccess(e.target.value)}
            >
              ${accessOptions.map(
                ([value, label]) =>
                  html`<option value=${value}>${label}</option>`,
              )}
            </select>
            <select
              value=${route.providerRaw}
              onChange=${(e) => onProvider(e.target.value)}
            >
              ${providerOptions.map(
                ([value, label]) =>
                  html`<option value=${value}>${label}</option>`,
              )}
            </select>
            <select
              value=${route.modelRaw}
              onChange=${(e) => onModel(e.target.value)}
            >
              ${modelOptions.map(
                ([value, label]) =>
                  html`<option value=${value}>${label}</option>`,
              )}
            </select>
            <select
              value=${route.effortRaw}
              onChange=${(e) => onEffort(e.target.value)}
            >
              ${effortOptions.map(
                ([value, label]) =>
                  html`<option value=${value}>${label}</option>`,
              )}
            </select>`}
      <button onClick=${onProjectContext}>project context</button>
      <button onClick=${onGraphContext}>graph context</button>
      <button onClick=${onReview}>review route</button>
    </div>
    ${route.delegations.entries.length
      ? html`<div class="route-delegations">
          ${route.delegations.entries.map(
            ([id, item]) =>
              html`<button
                title=${id}
                onClick=${() => {
                  if (item?.project) currentProject.value = item.project;
                }}
              >
                ${item?.project || "project"}: ${item?.status || "pending"}
              </button>`,
          )}
        </div>`
      : null}
  </div>`;
}

function Tile({ a }) {
  const act = activities.value[a.name];
  const isActive = !!act;
  const stCls = isActive
    ? "t-working"
    : a.blockers
      ? "t-blocked"
      : a.status === "sleeping"
        ? "t-sleeping"
        : a.managed === false
          ? "t-unmanaged"
          : "";
  const sysText =
    a.managed === false
      ? "UNMANAGED"
      : a.blockers
        ? "SYS.BLOCKED"
        : (a.uncommitted || 0) > 10
          ? "SYS.DIRTY"
          : isActive
            ? "SYS.EXEC"
            : "";
  return html`<div
    class="tile ${stCls}"
    onClick=${() => (currentProject.value = a.name)}
  >
    <div class="top">
      <span class="name"
        ><span
          class="dot ${isActive ? "working" : a.status}"
          style=${isActive ? "animation:pulse .8s infinite" : ""}
        ></span
        >${a.name}${a.blockers
          ? html` <span style="color:var(--accent);font-size:var(--fs-s)"
              >· BLOCKED</span
            >`
          : ""}</span
      >
    </div>
    ${isActive
      ? html`<div
          style="font-size:var(--fs-s);color:var(--green);margin:var(--sp-xs) 0;font-family:var(--font-mono)"
        >
          ⚡ ${act.action}: ${act.detail}
        </div>`
      : html`<div
          style="font-size:var(--fs-s);color:var(--t2);margin:var(--sp-xs) 0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap"
          dangerouslySetInnerHTML=${{
            __html: a.task
              ? md(a.task.substring(0, 80))
              : '<span style="color:var(--t3)">no active task</span>',
          }}
        ></div>`}
    <div class="row">
      ${isActive
        ? html`<span style="color:var(--green)">executing</span>`
        : a.status !== "idle"
          ? html`<span style="color:${SC[a.status] || "var(--t3)"}"
              >${SL[a.status] || a.status}</span
            >`
          : ""}${(a.uncommitted || 0) > 10
        ? html`<span style="color:var(--accent)">${a.uncommitted} dirty</span>`
        : (a.uncommitted || 0) > 0
          ? html`<span>${a.uncommitted} uncommitted</span>`
          : ""}${a.lessons ? html`<span>${a.lessons}L</span>` : ""}
    </div>
    <div class="sys">${sysText}</div>
    <div class="seg">
      ${[0, 1, 2, 3, 4].map((i) => {
        const fill = isActive
          ? 5
          : a.blockers
            ? 1
            : a.task
              ? 3
              : a.days < 7
                ? 2
                : 0;
        const color =
          i < fill
            ? isActive
              ? "var(--green)"
              : a.blockers
                ? "var(--accent)"
                : a.task
                  ? "var(--cyan)"
                  : "var(--green)"
            : "var(--mute)";
        return html`<span style="background:${color}" />`;
      })}
    </div>
  </div>`;
}

function DetailView() {
  const name = currentProject.value;
  const ag = agents.value.find((a) => a.name === name) || {};
  const act = activities.value[name];
  const pp = projectPlan.value;
  return html`<div class="content">
    <div class="back" onClick=${() => (currentProject.value = null)}>
      ← back to dashboard
    </div>
    <h2
      style="font-size:var(--fs-xl);display:flex;align-items:center;gap:var(--sp-s)"
    >
      <span class="dot ${act ? "working" : ag.status}"></span>${name}
    </h2>
    <div
      style="font-size:var(--fs-s);color:var(--t2);margin:var(--sp-s) 0;display:flex;gap:var(--sp-l);flex-wrap:wrap"
    >
      <span style="color:${SC[act ? "working" : ag.status] || "var(--t3)"}"
        >${act ? "executing" : ag.status || ""}</span
      >
      <span>branch: <code>${ag.branch || "—"}</code></span>
      <span>${ag.uncommitted || 0} uncommitted</span>
      ${ag.template_version
        ? html`<span>template ${ag.template_version}</span>`
        : ""}
      <span>${ag.lessons || 0} lessons</span>
      ${ag.blockers
        ? html`<span style="color:var(--accent);font-weight:600">BLOCKED</span>`
        : ""}
    </div>
    ${act
      ? html`<div
          style="padding:var(--sp-s) var(--sp-m);margin:var(--sp-s) 0;background:var(--sf);border-left:2px solid var(--green);font-size:var(--fs-s);color:var(--green);font-family:var(--font-mono)"
        >
          Running: ${act.action} — ${act.detail}
        </div>`
      : ""}
    ${pp && pp.context?.task_title
      ? html`<div
          style="padding:var(--sp-s) var(--sp-m);margin:var(--sp-s) 0;background:var(--sf);border-left:2px solid var(--yellow);font-size:var(--fs-s)"
        >
          <strong>Task:</strong> ${pp.context.task_title}
        </div>`
      : ag.task
        ? html`<div
            style="padding:var(--sp-s) var(--sp-m);margin:var(--sp-s) 0;background:var(--sf);border-left:2px solid var(--yellow);font-size:var(--fs-s)"
          >
            <strong>Task:</strong>
            <span dangerouslySetInnerHTML=${{ __html: md(ag.task) }}></span>
          </div>`
        : ""}
    ${pp && pp.blockers?.length
      ? html`<div
          style="padding:var(--sp-s) var(--sp-m);margin:var(--sp-s) 0;background:var(--sf);border-left:2px solid var(--accent);font-size:var(--fs-s)"
        >
          <strong style="color:var(--accent)">Blockers:</strong>
          ${pp.blockers.map(
            (b) =>
              html`<div
                style="margin:var(--sp-xs) 0;padding-left:var(--sp-m);color:var(--t2)"
              >
                • ${b}
              </div>`,
          )}
        </div>`
      : ""}
    <div class="panels">
      <div class="panel">
        <h3>next steps</h3>
        ${pp && pp.next_steps?.length
          ? pp.next_steps.map(
              (s, i) =>
                html`<div class="mod">
                  <span style="color:var(--t3);min-width:20px">${i + 1}.</span
                  ><span style="color:var(--t2)">${s}</span>
                </div>`,
            )
          : html`<span style="color:var(--t3)"
              >No next steps in tasks/current.md</span
            >`}
      </div>
      <div class="panel">
        <h3>issues</h3>
        ${pp && pp.issues?.length
          ? pp.issues.map(
              (iss) =>
                html`<div class="mod">
                  <span
                    class="pri ${iss.priority}"
                    style="font-size:var(--fs-s);padding:0 4px;border:1px solid"
                    >${iss.priority}</span
                  ><span style="color:var(--t2);margin-left:var(--sp-s)"
                    >${iss.text}</span
                  >
                </div>`,
            )
          : html`<span style="color:var(--green)">No issues detected</span>`}
      </div>
    </div>
    <div class="panels">
      <div class="panel">
        <h3>modules</h3>
        ${modules.value.length
          ? modules.value.map(
              (m) =>
                html`<div class="mod">
                  <span>${m.name}</span
                  ><span class="${m.status}"
                    >${m.status} ${m.files}f/${m.lines}L</span
                  >
                </div>`,
            )
          : html`<span style="color:var(--t3)">Scan via chat</span>`}
      </div>
      <div class="panel">
        <h3>context</h3>
        <div style="font-size:var(--fs-s);color:var(--t2)">
          <div style="margin-bottom:var(--sp-xs)">
            phase: ${pp?.context?.phase || ag.phase || "unknown"}
          </div>
          <div style="margin-bottom:var(--sp-xs)">
            segment: ${ag.segment || "unassigned"}
          </div>
          ${pp?.context?.recent_commits
            ? html`<div style="margin-top:var(--sp-s)">
                <strong>Recent:</strong>${pp.context.recent_commits.map(
                  (cm) =>
                    html`<div
                      style="color:var(--t3);font-family:var(--font-mono);font-size:var(--fs-s);margin:2px 0"
                    >
                      ${cm}
                    </div>`,
                )}
              </div>`
            : ""}
        </div>
      </div>
    </div>
    <div class="panels" style="margin-top:var(--sp-m)">
      <div class="panel">
        <h3>actions</h3>
        <div style="display:flex;flex-wrap:wrap;gap:var(--sp-s)">
          <button
            class="action-btn"
            onClick=${() => {
              showToast("Deploying template...", "info", 3000);
              __invoke("deploy_template", { project: name })
                .then((r) => {
                  showToast(
                    r.status === "ok"
                      ? "Deploy complete"
                      : "Deploy failed: " + (r.error || ""),
                    "success",
                    5000,
                  );
                })
                .catch((e) => showToast("Deploy error: " + e, "error"));
            }}
          >
            sync template
          </button>
          <button
            class="action-btn"
            onClick=${() => {
              showToast("Running health check...", "info", 3000);
              __invoke("health_check", { project: name })
                .then((r) => {
                  showToast(r.result || "No result", "info", 8000);
                })
                .catch((e) => showToast("Error: " + e, "error"));
            }}
          >
            health check
          </button>
          <button
            class="action-btn"
            onClick=${() => {
              if (__IS_TAURI)
                __invoke("plugin:shell|open", {
                  path: "zed://" + ag.path,
                }).catch(() => showToast("Cannot open Zed", "error"));
            }}
          >
            open in Zed
          </button>
        </div>
      </div>
    </div>
  </div>`;
}

function InboxPanel() {
  const d = inboxData.value;
  if (!d || !d.count) return null;
  return html`<div class="inbox-panel">
    <div class="deleg-panel-hdr">
      <span style="color:${d.needs_user ? "var(--accent)" : "var(--green)"}"
        >${d.count} agent result${d.count > 1 ? "s" : ""}</span
      >
      <button
        class="dc-btn"
        style="background:${d.needs_user
          ? "var(--accent)"
          : "var(--green)"};color:var(--bg)"
        onClick=${() => processInbox()}
      >
        ${d.needs_user ? "review" : "send to PA"}
      </button>
    </div>
    ${(d.items || []).map(
      (item) =>
        html`<div class="inbox-item">
          <span class="ii-proj">${item.project}</span>
          <span style="color:${item.needs_user ? "var(--accent)" : "var(--t3)"}"
            >●</span
          >
          <span class="ii-msg">${item.message}</span>
        </div>`,
    )}
  </div>`;
}

function DelegationPanel() {
  const all = Object.entries(delegations.value);
  const active = all.filter(
    ([_, d]) =>
      d.status !== "done" && d.status !== "rejected" && d.status !== "error",
  );
  const pending = all.filter(([_, d]) => d.status === "pending");
  if (!active.length && !pending.length) return null;
  return html`<div class="deleg-panel">
    <div class="deleg-panel-hdr">
      <span
        >${active.length} active
        delegation${active.length !== 1 ? "s" : ""}</span
      >
      ${pending.length > 1
        ? html`<details class="deleg-bulk">
            <summary>bulk</summary>
            <button
              class="dc-btn"
              style="background:var(--green);color:var(--bg)"
              onClick=${async () => {
                for (let i = 0; i < pending.length; i++) {
                  showToast(
                    i +
                      1 +
                      "/" +
                      pending.length +
                      ": " +
                      (pending[i][1]?.project || ""),
                    "info",
                    2000,
                  );
                  await approveDel(pending[i][0]);
                }
              }}
            >
              approve all
            </button>
          </details>`
        : null}
    </div>
    ${active.map(([id, d]) => {
      const el = d._start ? Math.round((Date.now() - d._start) / 1000) : 0;
      const cls =
        d.status === "done"
          ? "dc-done"
          : d.status === "failed"
            ? "dc-failed"
            : d.status === "running" || d.status === "escalated"
              ? "dc-running"
              : "";
      return html`<div class="deleg-card ${cls}">
        <span class="dc-proj">${d.project || "?"}</span>
        <span
          class="dc-status"
          style="color:${d.status === "pending"
            ? "var(--yellow)"
            : d.status === "running"
              ? "var(--cyan)"
              : d.status === "done"
                ? "var(--green)"
                : "var(--accent)"}"
          >${d.status}${el > 2 ? " " + el + "s" : ""}</span
        >
        ${d.status === "pending"
          ? html`<span class="dc-actions"
              ><button
                class="dc-btn"
                style="background:var(--green);color:var(--bg)"
                onClick=${() => approveDel(id)}
              >
                ✓</button
              ><button
                class="dc-btn"
                style="background:var(--accent);color:var(--bg)"
                onClick=${() => rejectDel(id)}
              >
                ✗
              </button></span
            >`
          : null}
        ${d.status === "done"
          ? html`<span
              style="cursor:pointer;color:var(--t3);text-decoration:underline"
              onClick=${() => {
                currentProject.value = d.project;
              }}
              >open</span
            >`
          : null}
      </div>`;
    })}
  </div>`;
}

function RunningBanner() {
  clock.value;
  const proj = currentProject.value || "_orchestrator";
  const act = activities.value[proj];
  if (!act) return null;
  const elapsed = Math.round(Date.now() / 1000 - act.started);
  return html`<div
    style="padding:var(--sp-s) var(--sp-m);background:var(--sf);border-bottom:1px solid var(--border);font-size:var(--fs-s);color:var(--green);font-family:var(--font-mono);display:flex;align-items:center;gap:var(--sp-s)"
  >
    <span style="animation:pulse .8s infinite">⚡</span> ${act.action}:
    ${act.detail} (${elapsed}s)
  </div>`;
}

function LiveRunHud() {
  clock.value;
  const run = activeRun.value;
  if (
    !run ||
    run.project !== (currentProject.value || "_orchestrator") ||
    (!isStreaming.value &&
      !["done", "failed", "cancelled"].includes(run.status))
  ) {
    return null;
  }
  const elapsed = run.startedAt
    ? Math.max(0, Math.round((Date.now() - run.startedAt) / 1000))
    : 0;
  const terminal = ["done", "failed", "cancelled"].includes(run.status);
  const recentTerminal =
    terminal && run.updatedAt && Date.now() - run.updatedAt < 12000;
  if (terminal && !recentTerminal && !isStreaming.value) return null;
  const events = (run.events || []).slice(-6);
  const statusLabel =
    run.status === "done"
      ? "done"
      : run.status === "failed"
        ? "failed"
        : run.status === "cancelled"
          ? "cancelled"
          : run.status || "running";
  return html`<div
    class="live-run ${terminal ? "terminal" : "running"} ${run.status || ""}"
  >
    <div class="live-run-head">
      <div class="live-run-pulse"></div>
      <div class="live-run-main">
        <div class="live-run-title">
          <span>${run.provider || "agent"}</span>
          ${run.model ? html`<em>${run.model}</em>` : null}
          <b>${statusLabel}</b>
        </div>
        <div class="live-run-detail">
          ${run.phase || "run"}: ${run.detail || "working"}
        </div>
      </div>
      <div class="live-run-badges">
        <span>${run.mode || "act"}</span>
        <span>${run.access || "write"}</span>
        <span>${elapsed}s</span>
      </div>
    </div>
    ${events.length
      ? html`<div class="live-run-events">
          ${events.map(
            (evt) =>
              html`<span class=${evt.type === "run_done" ? "final" : ""}>
                ${evt.phase || evt.type}: ${evt.detail || evt.outcome || ""}
              </span>`,
          )}
        </div>`
      : null}
  </div>`;
}

function TranscriptStatusBar({
  route,
  viewportMode,
  unreadLive,
  onFollow,
  onLoadOlder,
}) {
  const page = chatPageInfo.value || {};
  const run = route?.run || null;
  const terminal =
    run && ["done", "failed", "cancelled"].includes(run.status || "");
  const liveLabel = isStreaming.value
    ? run?.phase
      ? `${run.phase}: ${run.detail || "working"}`
      : "streaming"
    : terminal
      ? `${run.status}: ${run.detail || run.outcome || "finished"}`
      : "idle";
  return html`<div
    class="transcript-bar ${viewportMode === "reading"
      ? "reading"
      : "follow"} ${isStreaming.value ? "live" : ""}"
  >
    <div class="transcript-main">
      <span class="transcript-state">
        ${viewportMode === "reading" ? "reading history" : "following live"}
      </span>
      <span class="transcript-meta">
        history
        ${page.loaded || route?.historyLoaded || 0}/${page.total ||
        route?.historyTotal ||
        0}
      </span>
      <span class="transcript-meta">${liveLabel}</span>
      ${unreadLive > 0
        ? html`<span class="transcript-unread"
            >${unreadLive} new update${unreadLive === 1 ? "" : "s"}</span
          >`
        : null}
    </div>
    <div class="transcript-actions">
      ${page.hasMore
        ? html`<button
            disabled=${chatHistoryLoading.value}
            onClick=${onLoadOlder}
            title="Load older messages without losing the current scroll position"
          >
            ${chatHistoryLoading.value ? "loading" : "older"}
          </button>`
        : null}
      <button
        class=${viewportMode === "reading" || unreadLive > 0 ? "primary" : ""}
        onClick=${onFollow}
      >
        latest
      </button>
    </div>
  </div>`;
}

function OrchestrationMapCard({
  map,
  onAttachGraph,
  onOpenGraph,
  onVerifyGraph,
  onOpenPlans,
}) {
  if (!map || map.status !== "ok") return null;
  const big = map.big_plan || {};
  const scope = map.scope || {};
  const graph = map.graph_context || {};
  const delegCounts = map.delegations?.counts || {};
  const plans = map.plans || [];
  const projectSessions = map.project_sessions || [];
  const workItems = map.work_items || [];
  const nextPlan = plans[0] || null;
  const nextStep = nextPlan?.next_step || null;
  const openWork = workItems.filter((item) =>
    ["ready", "queued", "running", "reviewing", "draft"].includes(
      item.status || "",
    ),
  );
  const stageIndex = Number(big.stage_index || 4);
  const stageTotal = Number(big.stage_total || 6);
  return html`<div class="orch-map-card">
    <div class="orch-map-head">
      <div>
        <div class="orch-eyebrow">
          big plan stage ${stageIndex}/${stageTotal}
        </div>
        <div class="orch-title">${big.label || "Orchestration map"}</div>
      </div>
      <div class="orch-stage">
        ${Array.from({ length: stageTotal }).map(
          (_, idx) =>
            html`<span
              class=${idx + 1 < stageIndex
                ? "done"
                : idx + 1 === stageIndex
                  ? "current"
                  : ""}
            ></span>`,
        )}
      </div>
    </div>
    <div class="orch-grid">
      <div>
        <b>scope</b>
        <span
          >${scope.kind || "global"} /
          ${scope.title || map.project || "_orchestrator"}</span
        >
      </div>
      <div>
        <b>project agents</b>
        <span
          >${projectSessions.length}
          session${projectSessions.length === 1 ? "" : "s"}</span
        >
      </div>
      <div>
        <b>work items</b>
        <span>${openWork.length} open / ${workItems.length} shown</span>
      </div>
      <div>
        <b>delegations</b>
        <span>
          ${delegCounts.pending || 0} pending, ${delegCounts.running || 0}
          running, ${delegCounts.failed || 0} failed
        </span>
      </div>
      <div>
        <b>leases</b>
        <span
          >${map.leases?.active || 0} active write
          lease${map.leases?.active === 1 ? "" : "s"}</span
        >
      </div>
      <div class=${graph.available ? "ok" : "warn"}>
        <b>code context</b>
        <span>
          ${graph.available
            ? `${graph.nodes || 0} nodes, ${graph.edges || 0} deps, ${graph.context_chars || 0} ctx chars`
            : graph.reason || "not available"}
        </span>
      </div>
    </div>
    ${nextPlan
      ? html`<div class="orch-next">
          <b>next plan step</b>
          <span>
            ${nextStep
              ? `${nextStep.project || map.project || "project"}: ${nextStep.task || "next step"}`
              : `${nextPlan.title}: no open step`}
          </span>
        </div>`
      : null}
    ${workItems.length
      ? html`<div class="orch-work-strip">
          ${workItems
            .slice(0, 4)
            .map(
              (item) =>
                html`<span class="orch-work ${item.status || ""}">
                  ${item.project}: ${item.status || "open"} ->
                  ${item.title || item.task}
                </span>`,
            )}
        </div>`
      : null}
    <div class="orch-actions">
      <button onClick=${onAttachGraph} disabled=${!map.project}>
        attach code context
      </button>
      <button onClick=${onOpenGraph} disabled=${!map.project}>
        open graph
      </button>
      <button onClick=${onVerifyGraph} disabled=${!map.project}>
        verify graph
      </button>
      <button onClick=${onOpenPlans}>plans</button>
    </div>
  </div>`;
}

function timelineStatusClass(status) {
  const s = String(status || "").toLowerCase();
  if (["failed", "error", "cancelled", "warning"].includes(s)) return "warn";
  if (["running", "started", "pending", "queued", "verifying"].includes(s)) {
    return "live";
  }
  if (["done", "completed", "ok", "success"].includes(s)) return "done";
  return "info";
}

function ExecutionTimelineCard({ timeline, contract, onRefresh }) {
  if (!timeline || timeline.status !== "ok") return null;
  const big = timeline.big_plan || {};
  const schema = contract || timeline.contract || {};
  const items = timeline.items || [];
  const visible = items.slice(-10);
  const counts = timeline.counts || {};
  const stageIndex = Number(big.stage_index || 5);
  const stageTotal = Number(big.stage_total || 6);
  const copySummary = () => {
    const lines = visible.map((item) => {
      const project = item.project ? ` [${item.project}]` : "";
      const detail = item.detail ? ` - ${item.detail}` : "";
      return `${item.status || "info"} ${item.source || "event"}/${item.kind || "event"}${project}: ${item.title || "event"}${detail}`;
    });
    navigator.clipboard
      ?.writeText(lines.join("\n"))
      .then(() => showToast("timeline copied", "success", 1200))
      .catch(() => showToast("copy failed", "error", 2000));
  };
  return html`<div class="exec-timeline-card">
    <div class="exec-head">
      <div>
        <div class="orch-eyebrow">
          execution timeline - stage ${stageIndex}/${stageTotal}
        </div>
        <div class="exec-title">
          ${big.label || "Execution timeline"}
          <span>${counts.items || items.length} events</span>
          ${timeline.schema_version || schema.schema_version
            ? html`<span
                >${timeline.schema_version || schema.schema_version}</span
              >`
            : null}
          ${counts.warnings
            ? html`<em
                >${counts.warnings}
                warning${counts.warnings === 1 ? "" : "s"}</em
              >`
            : null}
        </div>
      </div>
      <div class="exec-actions">
        <button onClick=${onRefresh}>refresh</button>
        <button onClick=${copySummary} disabled=${!visible.length}>copy</button>
      </div>
    </div>
    ${schema.sources?.length
      ? html`<div class="exec-contract-strip">
          ${schema.sources.map(
            (source) =>
              html`<span>
                <b>${source.id}</b>
                ${source.coverage?.length || 0}
                event${source.coverage?.length === 1 ? "" : "s"}
              </span>`,
          )}
        </div>`
      : null}
    ${visible.length
      ? html`<div class="exec-list">
          ${visible.map((item, index) => {
            const cls = timelineStatusClass(item.status);
            return html`<div
              class="exec-row ${cls}"
              key=${`${item.ts || ""}-${index}`}
            >
              <span class="exec-dot"></span>
              <div class="exec-main">
                <div class="exec-row-top">
                  <b>${item.title || "event"}</b>
                  <span
                    >${item.source || "event"} / ${item.kind || "event"}</span
                  >
                </div>
                ${item.detail
                  ? html`<div class="exec-detail">${item.detail}</div>`
                  : null}
              </div>
              <div class="exec-meta">
                <span>${item.status || "info"}</span>
                ${item.project ? html`<span>${item.project}</span>` : null}
                ${item.ts ? html`<span>${ft(item.ts)}</span>` : null}
              </div>
            </div>`;
          })}
        </div>`
      : html`<div class="exec-empty">
          No timeline events yet. Start a chat run, Duo round, or delegation.
        </div>`}
  </div>`;
}

function ChatSidebar() {
  const inputRef = useRef();
  const msgsRef = useRef();
  const fileRef = useRef();
  const prevCount = useRef(0);
  const stickToBottom = useRef(true);
  const [chatWidth, setChatWidth] = useState(() => {
    const saved = Number(localStorage.getItem("agentos.chatWidth") || 0);
    return saved > 0 ? saved : null;
  });
  const showScrollBtn = signal(false);
  const [viewportMode, setViewportMode] = useState("following");
  const [unreadLive, setUnreadLive] = useState(0);
  const lastLiveMarker = useRef("");
  const duoEnabled = chatCollabMode.value === "duo";
  const duoView = duoEnabled ? normalizeDuoView(activeRoomTab.value) : "chat";
  const duoCollaborateMode = duoEnabled && duoView === "collaborate";
  const duoExecuteMode = duoEnabled && duoView === "execute";
  const duoWorkspaceInCanvas = duoCollaborateMode || duoExecuteMode;
  const configuredSoloProvider =
    permData.value?.provider_status?.roles?.orchestrator_provider || "claude";
  const explicitSoloProvider = ["claude", "codex"].includes(
    selectedSoloProvider.value,
  )
    ? selectedSoloProvider.value
    : "";
  const soloProvider = explicitSoloProvider || configuredSoloProvider;
  const soloProviderOptions = [
    ["", "auto: " + configuredSoloProvider],
    ["claude", "claude"],
    ["codex", "codex"],
  ];
  const accessOptions = [
    ["read", "read"],
    ["write", "write"],
    ["full", "full"],
  ];
  const selectedModelValue =
    soloProvider === "codex"
      ? selectedCodexModel.value
      : selectedClaudeModel.value;
  const selectedEffortValue =
    soloProvider === "codex"
      ? selectedCodexEffort.value
      : selectedClaudeEffort.value;
  const codexStatus = permData.value?.provider_status?.providers?.codex || {};
  const modelOptions =
    soloProvider === "codex"
      ? codexModelOptionsFromStatus(codexStatus, selectedModelValue)
      : CLAUDE_MODEL_OPTIONS;
  const effortOptions =
    soloProvider === "codex"
      ? codexEffortOptionsForModel(selectedModelValue, "effort", codexStatus)
      : CLAUDE_EFFORT_OPTIONS;
  const [duoComposerAction, setDuoComposerAction] = useState("ask_both");
  const [duoComposerTarget, setDuoComposerTarget] = useState("");
  const [duoAdvancedOpen, setDuoAdvancedOpen] = useState(false);
  const participants = dualSessionData.value?.session?.participants || [];
  const duoSession = dualSessionData.value?.session || null;
  const activeOrchestratorId = duoSession?.orchestrator_participant_id || "";
  const activeOrchestrator =
    participants.find((p) => p.id === activeOrchestratorId) || null;
  const leadCandidates =
    participants.filter(
      (p) =>
        p?.id &&
        (p.provider === "claude" || p.provider === "codex" || p.write_enabled),
    ) || [];
  const visibleLeadCandidates = leadCandidates.length
    ? leadCandidates
    : participants;
  const roomTarget =
    participants.find((p) => p.id === duoComposerTarget) || null;
  const scope = activeScope.value || fallbackScope(currentProject.value || "");
  const scopeCrumbs = Array.isArray(scope.breadcrumbs)
    ? scope.breadcrumbs
    : fallbackScope(currentProject.value || "").breadcrumbs;
  const scopeActions = (scope.available_actions || [])
    .filter((a) => a?.id && a?.label)
    .slice(0, 3);
  const insertPrompt = (text) => {
    const target = inputRef.current;
    if (!target) return;
    const current = target.value.trim();
    target.value = current ? `${current}\n${text}` : text;
    target.dispatchEvent(new Event("input", { bubbles: true }));
    target.focus();
  };
  const latestDuoRound = (() => {
    if (!duoCollaborateMode || isStreaming.value) {
      return null;
    }
    const sessionId =
      activeDualSession.value || dualSessionData.value?.session?.id;
    if (!sessionId) return null;
    const duoMessages = sideMessages.value.filter(
      (m) => m.room_session_id === sessionId && m.round_id,
    );
    if (!duoMessages.length) return null;
    const lastRoundId = duoMessages[duoMessages.length - 1].round_id;
    const roundMessages = duoMessages.filter((m) => m.round_id === lastRoundId);
    const assistantMessages = roundMessages.filter(
      (m) => m.role === "assistant",
    );
    if (!assistantMessages.length) return null;
    return {
      id: lastRoundId,
      assistants: assistantMessages,
    };
  })();
  const focusDuoAgent = async (participantId) => {
    const participant =
      participants.find((p) => p.id === participantId) || null;
    if (participant?.write_enabled) {
      if (participant.id !== activeOrchestratorId) {
        await useDuoOrchestrator(participant.id);
      } else {
        setDuoComposerAction("send");
        setDuoComposerTarget("");
        activeRoomTab.value = "execute";
      }
      focusComposerSoon();
      return;
    }
    if (participant && !participant.write_enabled) {
      showToast(
        `${participant.label} is review-only in this room. Grant write in Advanced runtime controls if you want execution.`,
        "info",
      );
    }
    setDuoComposerAction("mention");
    setDuoComposerTarget(participantId);
    setDuoAdvancedOpen(true);
    activeRoomTab.value = "collaborate";
  };
  const useDuoOrchestrator = async (participantId) => {
    const sessionId =
      activeDualSession.value || dualSessionData.value?.session?.id;
    if (!sessionId || !participantId) return;
    dualBusy.value = "orchestrator:" + participantId;
    try {
      await setDualOrchestrator(sessionId, participantId);
      setDuoComposerAction("send");
      setDuoComposerTarget("");
      activeRoomTab.value = "execute";
    } catch (e) {
      showToast("Set orchestrator error: " + e, "error");
    } finally {
      dualBusy.value = "";
    }
  };
  const setDuoNextAction = (action) => {
    setDuoComposerAction(action);
    setDuoAdvancedOpen(
      action === "mention" ||
        action === "rebuttal" ||
        action === "promote_strategy" ||
        action === "promote_plan" ||
        action === "child_session",
    );
    activeRoomTab.value = "collaborate";
  };
  const focusComposerSoon = () =>
    setTimeout(() => inputRef.current?.focus(), 0);
  const draftPlanFromDuo = () => {
    setDuoNextAction("promote_plan");
    focusComposerSoon();
  };
  const useProviderAsLead = async (provider) => {
    const candidate =
      visibleLeadCandidates.find(
        (participant) =>
          participant.provider === provider && participant.write_enabled,
      ) ||
      visibleLeadCandidates.find(
        (participant) => participant.provider === provider,
      );
    if (!candidate) {
      showToast(`No ${provider} participant in this Duo room`, "error");
      return;
    }
    await useDuoOrchestrator(candidate.id);
    showToast(
      `${candidate.label} is now orchestrator with write access`,
      "success",
    );
    setDuoComposerAction("send");
    setDuoComposerTarget("");
    setDuoView("execute");
    focusComposerSoon();
  };
  const openExecutionWithLead = async () => {
    const lead =
      activeOrchestrator ||
      visibleLeadCandidates.find((participant) => participant.write_enabled) ||
      visibleLeadCandidates[0] ||
      null;
    if (!lead) {
      showToast("Pick an execution lead first", "error");
      return;
    }
    await useDuoOrchestrator(lead.id);
    setDuoComposerAction("send");
    setDuoComposerTarget("");
    setDuoView("execute");
    focusComposerSoon();
  };
  const runScopeAction = async (actionId) => {
    if (actionId === "ask_both") {
      setDuoComposerAction("ask_both");
      setDuoComposerTarget("");
      setDuoAdvancedOpen(false);
      setDuoView("collaborate");
      setTimeout(() => inputRef.current?.focus(), 0);
      return;
    }
    if (
      actionId === "execute_with_lead" ||
      actionId === "execute_next_step" ||
      actionId === "execute_next"
    ) {
      if (!activeOrchestrator && visibleLeadCandidates[0]) {
        await useDuoOrchestrator(visibleLeadCandidates[0].id);
      }
      setDuoComposerAction("send");
      setDuoComposerTarget("");
      setDuoView("execute");
      setTimeout(() => inputRef.current?.focus(), 0);
      return;
    }
    if (actionId === "create_strategy") {
      setDuoNextAction("promote_strategy");
      focusComposerSoon();
      return;
    }
    if (actionId === "create_plan" || actionId === "replan") {
      draftPlanFromDuo();
      return;
    }
    if (actionId === "queue_task" || actionId === "create_work_item") {
      setDuoNextAction("child_session");
      focusComposerSoon();
      return;
    }
    if (actionId === "review_result") {
      setDuoComposerAction("challenge");
      setDuoComposerTarget(participants[0]?.id || "");
      setDuoAdvancedOpen(false);
      setDuoView("collaborate");
      setTimeout(() => inputRef.current?.focus(), 0);
      return;
    }
    if (actionId === "pick_project") {
      showToast(
        "Pick a project from the main canvas, then Duo will focus it.",
        "info",
      );
      return;
    }
    showToast("Action is not wired yet: " + actionId, "info");
  };
  const setDuoView = (nextView) => {
    activeRoomTab.value = nextView;
    if (nextView !== "chat") {
      ensureDualSession(currentProject.value || "").catch((e) =>
        showToast("Duo init error: " + e, "error"),
      );
    }
  };
  const isNearBottom = (el, threshold = 120) =>
    el.scrollHeight - el.scrollTop - el.clientHeight <= threshold;
  const maybeScrollToBottom = (force = false) => {
    const el = msgsRef.current;
    if (!el) return;
    if (force || stickToBottom.current) {
      requestAnimationFrame(() => {
        const current = msgsRef.current;
        if (!current) return;
        current.scrollTop = current.scrollHeight;
        stickToBottom.current = true;
        showScrollBtn.value = false;
        setViewportMode("following");
        setUnreadLive(0);
      });
    } else {
      showScrollBtn.value = true;
      setViewportMode("reading");
    }
  };
  const liveMarker = [
    sideMessages.value.length,
    streamText.value.length,
    streamChain.value.length,
    activeRun.value?.updatedAt || 0,
    curActivity.value || "",
  ].join(":");
  useEffect(() => {
    const changed =
      lastLiveMarker.current && lastLiveMarker.current !== liveMarker;
    lastLiveMarker.current = liveMarker;
    if (changed && !stickToBottom.current) {
      setUnreadLive((count) => Math.min(999, count + 1));
    }
    maybeScrollToBottom(false);
    prevCount.current = sideMessages.value.length;
  }, [liveMarker]);
  useEffect(() => {
    loadDr();
    stickToBottom.current = true;
    setViewportMode("following");
    setUnreadLive(0);
    setTimeout(() => {
      maybeScrollToBottom(true);
    }, 100);
  }, [currentProject.value]);
  useEffect(() => {
    if (!duoEnabled) return;
    ensureDualSession(currentProject.value || "").catch((e) =>
      console.warn("embedded duo init failed:", e),
    );
  }, [duoEnabled, currentProject.value]);
  useEffect(() => {
    if (!duoEnabled) return;
    loadActiveScope(
      currentProject.value || "",
      activeDualSession.value || null,
    ).catch((e) => console.warn("scope load failed:", e));
  }, [duoEnabled, currentProject.value, activeDualSession.value]);
  useEffect(() => {
    loadOrchestrationMap(
      currentProject.value || "",
      activeDualSession.value || null,
    ).catch((e) => console.warn("orchestration map load failed:", e));
  }, [
    currentProject.value,
    activeDualSession.value,
    dualBusy.value,
    Object.keys(delegations.value || {}).length,
    plansData.value.length,
  ]);
  useEffect(() => {
    loadEventContract().catch((e) =>
      console.warn("event contract load failed:", e),
    );
  }, []);
  useEffect(() => {
    loadExecutionTimeline(
      currentProject.value || "",
      activeDualSession.value || null,
      80,
    ).catch((e) => console.warn("execution timeline load failed:", e));
  }, [
    currentProject.value,
    activeDualSession.value,
    dualBusy.value,
    activeRun.value?.status,
    activeRun.value?.phase,
    activeRun.value?.detail,
    streamChain.value.length,
    Object.keys(delegations.value || {}).length,
  ]);
  useEffect(() => {
    if (!duoEnabled) return;
    const normalized = normalizeDuoView(activeRoomTab.value);
    if (normalized !== activeRoomTab.value) {
      activeRoomTab.value = normalized;
    }
  }, [duoEnabled, activeRoomTab.value]);
  useEffect(() => {
    if (!duoEnabled || !duoCollaborateMode) return;
    if (
      duoComposerAction === "mention" ||
      duoComposerAction === "challenge" ||
      duoComposerAction === "rebuttal"
    ) {
      if (
        !participants.some((p) => p.id === duoComposerTarget) &&
        participants[0]
      ) {
        setDuoComposerTarget(participants[0].id);
      }
    } else if (duoComposerTarget) {
      setDuoComposerTarget("");
    }
  }, [
    duoEnabled,
    duoCollaborateMode,
    duoComposerAction,
    duoComposerTarget,
    participants,
  ]);
  useEffect(() => {
    const validModels = new Set(modelOptions.map(([value]) => value));
    if (!validModels.has(selectedModelValue)) {
      if (soloProvider === "codex") {
        selectedCodexModel.value = "";
      } else {
        selectedClaudeModel.value = "";
      }
    }
    const validEfforts = new Set(effortOptions.map(([value]) => value));
    if (!validEfforts.has(selectedEffortValue)) {
      if (soloProvider === "codex") {
        selectedCodexEffort.value = "";
      } else {
        selectedClaudeEffort.value = "";
      }
    }
  }, [soloProvider, selectedModelValue, selectedEffortValue]);
  useEffect(() => {
    if (!composerDraftText.value) return;
    insertPrompt(composerDraftText.value);
    composerDraftText.value = "";
  }, [composerDraftText.value]);
  const onScroll = () => {
    if (msgsRef.current) {
      const el = msgsRef.current;
      const near = isNearBottom(el);
      stickToBottom.current = near;
      showScrollBtn.value = !near;
      setViewportMode(near ? "following" : "reading");
      if (near) setUnreadLive(0);
    }
  };
  const scrollToBottom = () => {
    if (msgsRef.current) {
      msgsRef.current.scrollTop = msgsRef.current.scrollHeight;
      stickToBottom.current = true;
      showScrollBtn.value = false;
      setViewportMode("following");
      setUnreadLive(0);
    }
  };
  const loadOlder = async () => {
    const el = msgsRef.current;
    const beforeHeight = el?.scrollHeight || 0;
    const ok = await loadOlderChat();
    if (!ok) return;
    requestAnimationFrame(() => {
      const current = msgsRef.current;
      if (!current) return;
      current.scrollTop =
        current.scrollHeight - beforeHeight + current.scrollTop;
      stickToBottom.current = false;
      showScrollBtn.value = true;
    });
  };
  const send = async () => {
    const v = inputRef.current?.value;
    if (!v?.trim()) return;
    if (
      (!duoEnabled || duoView === "chat") &&
      v.startsWith("/") &&
      execSlash(v.trim())
    ) {
      inputRef.current.value = "";
      return;
    }
    const msg = v.trim();
    inputRef.current.value = "";
    inputRef.current.style.height = "36px";
    inpHist.unshift(v);
    hIdx = -1;
    if (pastedImg.value) {
      pastedImg.value = null;
    }
    if (!duoEnabled || duoView === "chat") {
      sendMessage(v);
      return;
    }
    try {
      await ensureDualSession(currentProject.value || "");
      if (duoComposerAction === "send") {
        const lead = activeOrchestrator || visibleLeadCandidates[0] || null;
        if (!lead) {
          showToast("Pick an execution lead first", "error");
          return;
        }
        if (
          !activeOrchestrator ||
          activeOrchestrator.id !== lead.id ||
          !lead.write_enabled
        ) {
          await useDuoOrchestrator(lead.id);
        }
        activeRoomTab.value = "execute";
        await runDualParticipant(lead.id, msg, false);
      } else if (duoComposerAction === "ask_both") {
        await runDualRound(msg, true);
      } else if (duoComposerAction === "mention") {
        if (!duoComposerTarget) {
          showToast("Pick a room target", "error");
          return;
        }
        await runDualRoomAction("mention", msg, duoComposerTarget);
      } else if (duoComposerAction === "challenge") {
        if (!duoComposerTarget) {
          showToast("Pick a challenge target", "error");
          return;
        }
        await runDualRoomAction("challenge", msg, duoComposerTarget);
      } else if (duoComposerAction === "rebuttal") {
        if (!duoComposerTarget) {
          showToast("Pick a rebuttal target", "error");
          return;
        }
        await runDualRoomAction("rebuttal", msg, duoComposerTarget);
      } else if (duoComposerAction === "promote_strategy") {
        dualBusy.value = "promote:strategy";
        try {
          await generateStrategy(msg, "", activeDualSession.value || null);
          if (activeDualSession.value)
            await loadDualSession(activeDualSession.value);
        } finally {
          dualBusy.value = "";
        }
      } else if (duoComposerAction === "promote_plan") {
        dualBusy.value = "promote:plan";
        try {
          await createAdhocPlanFromRoom(
            msg,
            currentProject.value || "",
            activeDualSession.value || null,
          );
          if (activeDualSession.value)
            await loadDualSession(activeDualSession.value);
        } finally {
          dualBusy.value = "";
        }
      } else if (duoComposerAction === "child_session") {
        dualBusy.value = "project-session:create";
        try {
          await createRoomProjectSession(
            msg,
            currentProject.value || "",
            activeDualSession.value || null,
          );
          if (activeDualSession.value)
            await loadDualSession(activeDualSession.value);
        } finally {
          dualBusy.value = "";
        }
      } else {
        await runDualRound(msg, !duoExecuteMode);
      }
    } catch (e) {
      showToast("Room action error: " + e, "error");
    }
  };
  const onKey = (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      send();
    }
    if (e.key === "ArrowUp" && !inputRef.current?.value && inpHist.length) {
      hIdx = Math.min(hIdx + 1, inpHist.length - 1);
      inputRef.current.value = inpHist[hIdx];
    }
    if (e.key === "ArrowDown" && hIdx >= 0) {
      hIdx--;
      inputRef.current.value = hIdx >= 0 ? inpHist[hIdx] : "";
    }
  };
  const onDrop = (e) => {
    e.preventDefault();
    isDrag.value = false;
    if (e.dataTransfer?.files?.length) hdlFiles(e.dataTransfer.files);
  };
  const startResize = (e) => {
    e.preventDefault();
    const max = Math.floor(window.innerWidth * 0.72);
    const min = 360;
    const move = (ev) => {
      const next = Math.max(min, Math.min(max, window.innerWidth - ev.clientX));
      setChatWidth(next);
      localStorage.setItem("agentos.chatWidth", String(next));
    };
    const up = () => {
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
      document.body.classList.remove("chat-resizing");
    };
    document.body.classList.add("chat-resizing");
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  };
  const providerStatusSnapshot = permData.value?.provider_status || {};
  const providerState = providerAvailability(
    soloProvider,
    providerStatusSnapshot,
  );
  const modelSource =
    soloProvider === "codex" ? codexModelSourceLabel(codexStatus) : "static";
  const modelCount =
    soloProvider === "codex"
      ? Number(codexStatus.models_count || 0) ||
        (codexStatus.models || []).length ||
        Math.max(0, modelOptions.length - 1)
      : Math.max(0, modelOptions.length - 1);
  const routeRun =
    activeRun.value &&
    activeRun.value.project === (currentProject.value || "_orchestrator")
      ? activeRun.value
      : null;
  const routeDelegations = summarizeDelegationsForRoute(
    delegations.value,
    currentProject.value || "",
  );
  const routeWarnings = [];
  if (providerState === "offline") {
    routeWarnings.push(`${soloProvider} runtime is offline`);
  }
  if (
    !duoEnabled &&
    chatRunMode.value === "act" &&
    chatAccessLevel.value === "read"
  ) {
    routeWarnings.push("act mode is read-only");
  }
  if (duoExecuteMode && !activeOrchestrator?.write_enabled) {
    routeWarnings.push("execution needs a write-enabled lead");
  }
  const route = {
    duo: duoEnabled,
    target: currentProject.value || "orchestrator",
    scope: `${scope.kind || "global"}:${scope.title || currentProject.value || "orchestrator"}`,
    inputLabel: duoEnabled
      ? duoInputLabel(duoComposerAction, activeOrchestrator, roomTarget)
      : `${soloProvider} ${chatRunMode.value}`,
    provider: duoEnabled ? activeOrchestrator?.provider || "duo" : soloProvider,
    providerRaw: selectedSoloProvider.value,
    providerState,
    model: duoEnabled
      ? activeOrchestrator?.model || "room"
      : selectedModelValue || "auto",
    modelRaw: selectedModelValue,
    effortRaw: selectedEffortValue,
    mode: duoEnabled
      ? duoExecuteMode
        ? "execute"
        : duoCollaborateMode
          ? "review"
          : "chat"
      : chatRunMode.value,
    access: duoEnabled
      ? activeOrchestrator?.write_enabled
        ? "write"
        : "read"
      : chatRunMode.value === "plan"
        ? "read"
        : chatAccessLevel.value || "write",
    accessRaw: chatAccessLevel.value,
    modelSource,
    modelCount,
    historyLoaded: chatPageInfo.value.loaded || sideMessages.value.length,
    historyTotal: chatPageInfo.value.total || sideMessages.value.length,
    delegations: routeDelegations,
    run: routeRun,
    warnings: routeWarnings,
    canLeadExecute: !!activeOrchestrator?.write_enabled,
  };
  return html`<div
    class="chat-side"
    style=${chatWidth ? `width:${chatWidth}px` : ""}
    onDragOver=${(e) => {
      e.preventDefault();
      isDrag.value = true;
    }}
    onDragLeave=${() => (isDrag.value = false)}
    onDrop=${onDrop}
  >
    <div
      class="chat-resize-handle"
      title="Drag to resize chat"
      onMouseDown=${startResize}
    />
    <div
      class="ch-title"
      style="display:flex;justify-content:space-between;align-items:center"
    >
      <span style="display:flex;align-items:center;gap:var(--sp-xs)"
        ><span class="conn-dot ${isOn.value ? "on" : "off"}" /><span
          style="color:var(--t3);font-size:var(--fs-s)"
          >CHAT WITH</span
        ><span
          style="color:${currentProject.value ? "var(--cyan)" : "var(--text)"}"
          >${currentProject.value || "ORCHESTRATOR"}</span
        ></span
      >
      <span style="display:flex;gap:var(--sp-xs);align-items:center">
        <span
          style="display:flex;gap:2px;align-items:center;padding:2px;border:1px solid var(--border);background:var(--bg-soft)"
        >
          <button
            style="background:${chatCollabMode.value === "solo"
              ? "var(--text)"
              : "transparent"};color:${chatCollabMode.value === "solo"
              ? "var(--bg)"
              : "var(--t3)"};border:none;font-size:var(--fs-s);padding:2px 6px;cursor:pointer;font-family:var(--font-mono)"
            onClick=${() => {
              chatCollabMode.value = "solo";
              activeRoomTab.value = "chat";
            }}
          >
            solo
          </button>
          <button
            style="background:${chatCollabMode.value === "duo"
              ? "var(--cyan)"
              : "transparent"};color:${chatCollabMode.value === "duo"
              ? "var(--bg)"
              : "var(--t3)"};border:none;font-size:var(--fs-s);padding:2px 6px;cursor:pointer;font-family:var(--font-mono)"
            onClick=${() => {
              chatCollabMode.value = "duo";
              activeRoomTab.value = "collaborate";
              ensureDualSession(currentProject.value || "").catch((e) =>
                showToast("Duo init error: " + e, "error"),
              );
            }}
          >
            duo
          </button>
        </span>
        <button
          style="background:none;border:1px solid var(--border);color:var(--t3);font-size:var(--fs-s);padding:2px 6px;cursor:pointer;font-family:var(--font-mono)"
          title="Export chat"
          onClick=${() => {
            if (__IS_TAURI) {
              const p = currentProject.value || "_orchestrator";
              __invoke("export_chat", { project: p })
                .then((r) => {
                  const blob = new Blob([r.markdown], {
                    type: "text/markdown",
                  });
                  const url = URL.createObjectURL(blob);
                  const a = document.createElement("a");
                  a.href = url;
                  a.download = p + "-chat.md";
                  a.click();
                  URL.revokeObjectURL(url);
                  showToast("Exported", "success");
                })
                .catch((e) => showToast("Export error: " + e, "error"));
            }
          }}
        >
          ↓
        </button>
        ${duoEnabled
          ? null
          : html`<button
                type="button"
                class="chat-mode-toggle ${chatRunMode.value === "plan"
                  ? "plan"
                  : "act"}"
                title=${chatRunMode.value === "plan"
                  ? "Plan mode: read-only, no AgentOS command execution"
                  : "Act mode: execute with selected access"}
                onClick=${() => {
                  chatRunMode.value =
                    chatRunMode.value === "plan" ? "act" : "plan";
                  showToast(
                    chatRunMode.value === "plan"
                      ? "Plan mode: read-only"
                      : "Act mode",
                    "success",
                    1500,
                  );
                }}
              >
                ${chatRunMode.value === "plan" ? "plan" : "act"}
              </button>
              <select
                value=${chatRunMode.value === "plan"
                  ? "read"
                  : chatAccessLevel.value}
                title="Access level for Act mode"
                disabled=${chatRunMode.value === "plan"}
                class="chat-access-select"
                onChange=${(e) => {
                  chatAccessLevel.value = e.target.value;
                  showToast("access: " + e.target.value, "success", 1500);
                }}
              >
                ${accessOptions.map(
                  ([value, label]) =>
                    html`<option value=${value}>${label}</option>`,
                )}
              </select>
              <select
                value=${selectedSoloProvider.value}
                title="Solo provider"
                style="background:var(--sf);color:var(--t2);border:1px solid var(--border);font-family:var(--font-mono);font-size:var(--fs-s);padding:var(--sp-xs)"
                onChange=${(e) => {
                  selectedSoloProvider.value = e.target.value;
                  showToast(
                    "solo provider: " +
                      (e.target.value || configuredSoloProvider),
                    "success",
                    1500,
                  );
                }}
              >
                ${soloProviderOptions.map(
                  ([value, label]) =>
                    html`<option value=${value}>${label}</option>`,
                )}
              </select>
              <select
                value=${selectedModelValue}
                title=${soloProvider + " model"}
                style="background:var(--sf);color:var(--t2);border:1px solid var(--border);font-family:var(--font-mono);font-size:var(--fs-s);padding:var(--sp-xs)"
                onChange=${(e) => {
                  if (soloProvider === "codex") {
                    selectedCodexModel.value = e.target.value;
                  } else {
                    selectedClaudeModel.value = e.target.value;
                  }
                  showToast(
                    `${soloProvider} model: ` + (e.target.value || "auto"),
                    "success",
                    1500,
                  );
                }}
              >
                ${modelOptions.map(
                  ([value, label]) =>
                    html`<option value=${value}>${label}</option>`,
                )}
              </select>
              <select
                value=${selectedEffortValue}
                title=${soloProvider + " effort"}
                style="background:var(--sf);color:var(--t2);border:1px solid var(--border);font-family:var(--font-mono);font-size:var(--fs-s);padding:var(--sp-xs)"
                onChange=${(e) => {
                  if (soloProvider === "codex") {
                    selectedCodexEffort.value = e.target.value;
                  } else {
                    selectedClaudeEffort.value = e.target.value;
                  }
                  showToast(
                    `${soloProvider} effort: ` + (e.target.value || "default"),
                    "success",
                    1500,
                  );
                }}
              >
                ${effortOptions.map(
                  ([value, label]) =>
                    html`<option value=${value}>${label}</option>`,
                )}
              </select>`}
      </span>
    </div>
    ${isDrag.value ? html`<div class="drop-zone">Drop files here</div>` : null}
    <${RunningBanner} />
    <${LiveRunHud} />
    <${RouteCard}
      route=${route}
      providerOptions=${soloProviderOptions}
      modelOptions=${modelOptions}
      effortOptions=${effortOptions}
      accessOptions=${accessOptions}
      onToggleMode=${() => {
        chatRunMode.value = chatRunMode.value === "plan" ? "act" : "plan";
      }}
      onAccess=${(value) => {
        chatAccessLevel.value = value;
        showToast("access: " + value, "success", 1200);
      }}
      onProvider=${(value) => {
        selectedSoloProvider.value = value;
        showToast(
          "provider: " + (value || configuredSoloProvider),
          "success",
          1200,
        );
      }}
      onModel=${(value) => {
        if (soloProvider === "codex") selectedCodexModel.value = value;
        else selectedClaudeModel.value = value;
        showToast("model: " + (value || "auto"), "success", 1200);
      }}
      onEffort=${(value) => {
        if (soloProvider === "codex") selectedCodexEffort.value = value;
        else selectedClaudeEffort.value = value;
        showToast("effort: " + (value || "default"), "success", 1200);
      }}
      onProjectContext=${() =>
        insertPrompt(
          currentProject.value
            ? `[PROJECT_CONTEXT:${currentProject.value}]\nSummarize current state, blockers, dirty files, and safest next action.`
            : "[DASHBOARD_FULL]\nSummarize the global project state, blockers, and safest next route.",
        )}
      onGraphContext=${() =>
        insertPrompt(
          currentProject.value
            ? `[GRAPH_CONTEXT:${currentProject.value}]\nUse code graph context to identify dependency risks before making changes.`
            : "[GRAPH_CONTEXT:overview]\nUse the project graph to choose the safest orchestration target.",
        )}
      onReview=${() =>
        insertPrompt(
          currentProject.value
            ? "Review this route and project state. If execution is safe, propose the exact next step; otherwise state the blocker."
            : "Review the global orchestration route. Choose the safest project, provider, and execution mode.",
        )}
      onAskBoth=${() => {
        setDuoComposerAction("ask_both");
        setDuoComposerTarget("");
        setDuoView("collaborate");
        focusComposerSoon();
      }}
      onMakePlan=${draftPlanFromDuo}
      onLeadExecute=${openExecutionWithLead}
    />
    <${TranscriptStatusBar}
      route=${route}
      viewportMode=${viewportMode}
      unreadLive=${unreadLive}
      onFollow=${scrollToBottom}
      onLoadOlder=${loadOlder}
    />
    <${OrchestrationMapCard}
      map=${orchestrationMap.value}
      onAttachGraph=${() =>
        insertPrompt(
          currentProject.value
            ? `[GRAPH_CONTEXT:${currentProject.value}]\nUse this code context when planning and executing. Call out dependency risks before edits.`
            : "[GRAPH_CONTEXT:overview]\nUse the project graph to choose the safest orchestration route.",
        )}
      onOpenGraph=${() => loadGraph(currentProject.value || "overview")}
      onVerifyGraph=${() =>
        insertPrompt(
          currentProject.value
            ? `[GRAPH_VERIFY:${currentProject.value}]\nVerify graph health and dependency risks before continuing.`
            : "[GRAPH_CONTEXT:overview]\nVerify global dependency and orchestration risks.",
        )}
      onOpenPlans=${() => {
        showPlans.value = true;
        loadPlansData().catch((e) => console.warn("plans refresh:", e));
      }}
    />
    <${ExecutionTimelineCard}
      timeline=${executionTimeline.value}
      contract=${eventContract.value}
      onRefresh=${() =>
        loadExecutionTimeline(
          currentProject.value || "",
          activeDualSession.value || null,
          80,
        ).catch((e) => console.warn("execution timeline refresh:", e))}
    />
    ${duoEnabled
      ? html`<div class="duo-brief">
          <div class="scope-strip">
            <div class="scope-path">
              ${scopeCrumbs.map(
                (crumb, index) =>
                  html`<span class="scope-crumb">
                    ${index > 0 ? html`<b>/</b>` : null}${crumb.label}
                  </span>`,
              )}
            </div>
            <span class="scope-kind">${scope.label || scope.kind}</span>
          </div>
          <div class="duo-brief-top">
            <div>
              <div class="duo-eyebrow">${scope.kind || "global"} context</div>
              <div class="duo-title">
                ${scope.title ||
                activeOrchestrator?.label ||
                "Choose work area"}
              </div>
              <div class="duo-sub">
                ${scope.summary ||
                (duoExecuteMode
                  ? "Execution board is open in the main canvas."
                  : duoCollaborateMode
                    ? "Ask both agents, then choose who leads the next step."
                    : "Normal chat mode; Duo room is standing by.")}
              </div>
            </div>
            <span class="duo-pill ${duoExecuteMode ? "hot" : ""}">
              ${duoExecuteMode
                ? "execute"
                : duoCollaborateMode
                  ? "review"
                  : "chat"}
            </span>
          </div>
          <div class="scope-actions">
            ${scopeActions.map(
              (action) =>
                html`<button
                  class="scope-action ${action.tone === "primary"
                    ? "primary"
                    : ""}"
                  disabled=${!!dualBusy.value}
                  onClick=${() => runScopeAction(action.id)}
                >
                  ${action.label}
                </button>`,
            )}
          </div>
          <details class="duo-small-controls">
            <summary>lead / mode</summary>
            <div class="duo-control-grid">
              ${visibleLeadCandidates.map(
                (participant) =>
                  html`<button
                    class=${participant.id === activeOrchestratorId
                      ? "lead selected"
                      : "lead"}
                    disabled=${!!dualBusy.value}
                    onClick=${() => useDuoOrchestrator(participant.id)}
                  >
                    lead:
                    ${participant.label}${participant.write_enabled
                      ? ""
                      : " + write"}
                  </button>`,
              )}
              ${[
                ["chat", "plain chat"],
                ["collaborate", "review room"],
                ["execute", "execution board"],
              ].map(
                ([id, label]) =>
                  html`<button
                    class=${duoView === id ? "selected" : ""}
                    onClick=${() => setDuoView(id)}
                  >
                    ${label}
                  </button>`,
              )}
            </div>
          </details>
        </div>`
      : null}
    <${InboxPanel} />
    <${DelegationPanel} />
    <div
      class="ch-msgs"
      ref=${msgsRef}
      onScroll=${onScroll}
      style="position:relative"
    >
      ${!sideMessages.value.length && !isStreaming.value
        ? html`<div class="chat-empty-state">
            <div class="chat-empty-title">Tell the agent what to do.</div>
            <div class="chat-empty-copy">
              Use the header for model, plan mode, and access. Type normally;
              the agent should plan, execute, or report a blocker without extra
              routing steps.
            </div>
            <div class="chat-empty-actions">
              <button
                onClick=${() =>
                  insertPrompt(
                    "Review current project state and propose the safest next step. Do not execute yet.",
                  )}
              >
                review state
              </button>
              <button
                onClick=${() =>
                  insertPrompt("[HEALTH_CHECK:all]\n[DASHBOARD_FULL]")}
              >
                health bundle
              </button>
              <button
                onClick=${() =>
                  insertPrompt(
                    currentProject.value
                      ? `[GRAPH_CONTEXT:${currentProject.value}]\nMap the code structure, risks, and safest implementation path.`
                      : "[GRAPH_CONTEXT:overview]\nMap project dependencies and identify the safest next orchestration target.",
                  )}
              >
                graph context
              </button>
            </div>
          </div>`
        : null}
      ${chatPageInfo.value.hasMore
        ? html`<button
            class="chat-load-older"
            disabled=${chatHistoryLoading.value}
            onClick=${loadOlder}
          >
            ${chatHistoryLoading.value
              ? "loading history..."
              : `load older (${sideMessages.value.length}/${chatPageInfo.value.total})`}
          </button>`
        : null}
      ${sideMessages.value.map((m, i) => html`<${ChatMsg} key=${i} m=${m} />`)}
      ${duoCollaborateMode
        ? html`<${DuoLiveStatus} session=${duoSession} />`
        : null}
      <${StreamBubble} />
    </div>
    ${latestDuoRound
      ? html`<div class="duo-next-card">
          <div>
            <div class="duo-eyebrow">recommended next step</div>
            <div class="duo-next-title">
              ${scopeNextTitle(scope, activeOrchestrator?.label)}
            </div>
            <div class="duo-sub">
              ${latestDuoRound.assistants.length}
              response${latestDuoRound.assistants.length > 1 ? "s" : ""} from
              the last Duo round. Discuss, convert to a plan, then let the
              execution lead delegate child work.
            </div>
          </div>
          <button
            class="duo-primary"
            disabled=${!!dualBusy.value}
            onClick=${draftPlanFromDuo}
          >
            Make plan
          </button>
          <button
            class="duo-primary"
            disabled=${!!dualBusy.value}
            onClick=${openExecutionWithLead}
            title="Use the selected lead as orchestrator and open execution mode."
          >
            Lead executes
          </button>
          <details class="duo-small-controls">
            <summary>other actions</summary>
            <div class="duo-control-grid">
              ${visibleLeadCandidates.map(
                (participant) =>
                  html`<button
                    class=${participant.id === activeOrchestratorId
                      ? "lead selected"
                      : "lead"}
                    disabled=${!!dualBusy.value}
                    onClick=${() => useDuoOrchestrator(participant.id)}
                  >
                    lead:
                    ${participant.label}${participant.write_enabled
                      ? ""
                      : " + write"}
                  </button>`,
              )}
              ${latestDuoRound.assistants.map((msg) => {
                const participant =
                  participants.find((p) => p.id === msg.participant) || null;
                const canWrite = !!participant?.write_enabled;
                const isOrchestrator = participant?.id === activeOrchestratorId;
                const label = (msg.meta || "")
                  .replace(/^\s*[^A-Za-z0-9]+/, "")
                  .trim();
                return html`<button
                  onClick=${() => focusDuoAgent(msg.participant)}
                >
                  ${canWrite ? "continue with" : "ask"}
                  ${label || participant?.label || msg.participant}${canWrite
                    ? isOrchestrator
                      ? " (orchestrator)"
                      : ""
                    : " (review)"}
                </button>`;
              })}
              <button onClick=${() => useProviderAsLead("claude")}>
                prefer Claude lead
              </button>
              <button onClick=${() => useProviderAsLead("codex")}>
                prefer Codex lead
              </button>
              <button onClick=${draftPlanFromDuo}>make plan from round</button>
            </div>
          </details>
        </div>`
      : null}
    ${showScrollBtn.value
      ? html`<button
          onClick=${scrollToBottom}
          class="scroll-catchup"
          title="Jump to latest output"
        >
          ↓
        </button>`
      : null}
    ${attFiles.value.length
      ? html`<div class="attached-files">
          ${attFiles.value.map(
            (fi, i) =>
              html`<div class="attached-file">
                <span>${fi.name}</span
                ><span class="rm" onClick=${() => rmFile(i)}>x</span>
              </div>`,
          )}
        </div>`
      : null}
    <${DelegTracker} />
    ${hasDraft.value
      ? html`<div class="draft-ind">
          draft saved
          <span
            style="cursor:pointer;color:var(--accent);margin-left:var(--sp-s)"
            onClick=${() => {
              clrDr();
              const ta = document.querySelector(".ch-inp textarea");
              if (ta) ta.value = "";
            }}
            >discard</span
          >
        </div>`
      : null}
    ${duoCollaborateMode && duoAdvancedOpen
      ? html`<div
          style="display:flex;gap:var(--sp-xs);flex-wrap:wrap;padding:0 var(--sp-s) var(--sp-s);background:var(--bg-soft)"
        >
          ${[
            ["mention", "@mention"],
            ["rebuttal", "rebuttal"],
            ["promote_strategy", "strategy"],
            ["promote_plan", "make plan"],
            ["child_session", "project room"],
          ].map(
            ([id, label]) =>
              html`<button
                style="background:${duoComposerAction === id
                  ? "var(--yellow)"
                  : "transparent"};color:${duoComposerAction === id
                  ? "var(--bg)"
                  : "var(--t3)"};border:1px solid ${duoComposerAction === id
                  ? "var(--yellow)"
                  : "var(--border)"};font-size:var(--fs-s);padding:4px 8px;cursor:pointer;font-family:var(--font-mono)"
                onClick=${() => {
                  setDuoComposerAction(id);
                  if (id === "mention" || id === "rebuttal") {
                    setDuoComposerTarget(participants[0]?.id || "");
                  } else {
                    setDuoComposerTarget("");
                  }
                }}
              >
                ${label}
              </button>`,
          )}
          ${["mention", "rebuttal"].includes(duoComposerAction)
            ? html`<select
                value=${duoComposerTarget}
                onInput=${(e) => setDuoComposerTarget(e.currentTarget.value)}
                style="background:var(--sf);color:var(--text);border:1px solid var(--border);font-size:var(--fs-s);padding:4px 8px;font-family:var(--font-mono)"
              >
                <option value="">pick target</option>
                ${participants.map(
                  (p) => html`<option value=${p.id}>${p.label}</option>`,
                )}
              </select>`
            : null}
        </div>`
      : null}
    <div class="ch-inp">
      <button
        class="attach-btn"
        onClick=${() => fileRef.current?.click()}
        title="Attach"
      >
        📎
      </button>
      <input
        type="file"
        ref=${fileRef}
        style="display:none"
        multiple
        onChange=${(e) => {
          if (e.target.files.length) hdlFiles(e.target.files);
          e.target.value = "";
        }}
      />
      <textarea
        ref=${inputRef}
        placeholder=${isStreaming.value
          ? "waiting for response..."
          : duoWorkspaceInCanvas
            ? (() => {
                if (duoComposerAction === "send") {
                  return `execute with ${activeOrchestrator?.label || "selected lead"}...`;
                }
                if (duoComposerAction === "mention") {
                  return (
                    "message " + (roomTarget?.label || "selected agent") + "..."
                  );
                }
                if (duoComposerAction === "challenge") {
                  return (
                    "challenge " +
                    (roomTarget?.label || "selected agent") +
                    "..."
                  );
                }
                if (duoComposerAction === "rebuttal") {
                  return (
                    "rebut " + (roomTarget?.label || "selected agent") + "..."
                  );
                }
                if (duoComposerAction === "promote_strategy") {
                  return "describe the goal to promote into strategy...";
                }
                if (duoComposerAction === "promote_plan") {
                  return "write plan steps, e.g. Project: task...";
                }
                if (duoComposerAction === "child_session") {
                  return "write 'project: title' or a project-room title...";
                }
                return "ask both agents for review...";
              })()
            : chatRunMode.value === "plan"
              ? `plan with ${route.inputLabel}...`
              : `tell ${route.inputLabel} what to do...`}
        rows="1"
        style=${isStreaming.value ? "opacity:0.5" : ""}
        onKeyDown=${onKey}
        onInput=${(e) => {
          e.target.style.height = "auto";
          e.target.style.height = Math.min(e.target.scrollHeight, 150) + "px";
          handleSlash(e.target.value);
          clearTimeout(draftT);
          draftT = setTimeout(saveDr, 2000);
        }}
        onPaste=${handlePaste}
      />
      <button
        class="voice-btn ${isRec.value ? "recording" : ""}"
        onClick=${togVoice}
        title="Voice"
      >
        ${isRec.value ? "⏹" : "○"}
      </button>
      ${isStreaming.value
        ? html`<button
            class="send-btn"
            style="background:var(--accent)"
            onClick=${() => {
              __invoke &&
                __invoke("stop_chat", {
                  project: currentProject.value || null,
                })
                  .then((res) => {
                    showToast(
                      res?.killed
                        ? `Stopped provider pid ${res.pid}`
                        : "Stop requested",
                      "info",
                      2000,
                    );
                  })
                  .catch((e) => showToast("Stop error: " + e, "error"));
              curActivity.value = "stopping; preserving visible output...";
              if (activeRun.value) {
                activeRun.value = {
                  ...activeRun.value,
                  status: "stopping",
                  outcome: "",
                  phase: "cancelling",
                  detail: "stop requested; waiting for provider cleanup",
                  updatedAt: Date.now(),
                };
              }
            }}
          >
            ■
          </button>`
        : html`<button class="send-btn" onClick=${send}>↑</button>`}
    </div>
  </div>`;
}

// Tool icon by type (text, no emoji)
const TOOL_ICON = {
  Bash: "$_",
  bash: "$_",
  Read: "[]",
  read_file: "[]",
  Write: "/>",
  write_file: "/>",
  Edit: "/>",
  edit_file: "/>",
  Grep: "?/",
  search: "?/",
  Glob: "*.",
  list_files: "*.",
  Agent: ">>",
  TodoWrite: ">>",
  NotebookEdit: "/>",
};
function toolIcon(name) {
  return TOOL_ICON[name] || ">>";
}
// Smart tool detail extraction
function fmtToolDetail(tool, inp) {
  if (!inp) return "";
  if (tool === "Bash" || tool === "bash")
    return (inp.command || "").substring(0, 100);
  if (tool === "Read" || tool === "read_file") {
    let s = inp.file_path || "";
    if (inp.offset) s += ` :${inp.offset}`;
    return s.split("/").slice(-2).join("/");
  }
  if (
    tool === "Write" ||
    tool === "write_file" ||
    tool === "Edit" ||
    tool === "edit_file"
  )
    return (inp.file_path || "").split("/").slice(-2).join("/");
  if (tool === "Grep" || tool === "search")
    return (
      "/" +
      (inp.pattern || "").substring(0, 50) +
      "/ " +
      (inp.path || "").split("/").slice(-2).join("/")
    );
  if (tool === "Glob" || tool === "list_files") return inp.pattern || "";
  if (tool === "Agent")
    return inp.prompt ? inp.prompt.substring(0, 80) + "..." : "";
  const j = JSON.stringify(inp);
  return j.length > 120 ? j.substring(0, 120) + "..." : j;
}
function ToolCard({ t }) {
  const [open, setOpen] = useState(false);
  const icon = toolIcon(t.tool);
  const detail = fmtToolDetail(t.tool, t.input);
  const elapsed = t.elapsed ? Math.round(t.elapsed / 1000) : 0;
  const statusCls =
    t.status === "started" ? "tc-spin" : t.is_error ? "tc-err" : "tc-ok";
  const statusTxt = t.status === "started" ? "◌" : t.is_error ? "✗" : "✓";
  return html`<div class="tc">
    <div class="tc-hdr" onClick=${() => setOpen(!open)}>
      <span class="tc-icon">${icon}</span>
      <span class="tc-name">${t.tool}</span>
      <span class="tc-sep">—</span>
      <span class="tc-detail">${detail}</span>
      <span class="tc-status ${statusCls}">${statusTxt}</span>
      ${elapsed > 0 ? html`<span class="tc-time">${elapsed}s</span>` : null}
    </div>
    ${open
      ? html`<div class="tc-body">
          <pre class="tc-input">
${t.tool === "Bash" || t.tool === "bash"
              ? t.input?.command || ""
              : JSON.stringify(t.input, null, 2)}</pre
          >
          ${t.result
            ? html`<div class="tc-output ${t.is_error ? "tc-out-err" : ""}">
                ${t.result}
              </div>`
            : null}
        </div>`
      : null}
  </div>`;
}
function ThinkBlock({ b }) {
  const text = b.text || "";
  const visible =
    text.length > 2000
      ? text.substring(0, 2000) + "\n...[thinking continues]"
      : text;
  return html`<details class="think-block" open>
    <summary>
      <span class="think-icon">◆</span
      ><span class="think-label">thinking</span>${b.streaming
        ? html`<span style="color:var(--t3)">...</span>`
        : null}
    </summary>
    <div class="think-body">${visible || "waiting for model reasoning..."}</div>
  </details>`;
}
function ProgressBar({ activity, elapsed }) {
  if (!activity) return null;
  return html`<div class="prog-strip">
    <div class="prog-bar" />
    <span class="prog-label">${activity}</span>
    ${elapsed > 2 ? html`<span class="prog-time">${elapsed}s</span>` : null}
  </div>`;
}

const PA_COMMAND_PATTERN = /\[[A-Z][A-Z0-9_]*(?::[^\]]*)?\]/g;
const PA_COMMAND_LINE = /^\s*\[[A-Z][A-Z0-9_]*(?::[^\]]*)?\]\s*$/;

function isPaFeedbackBlock(block) {
  return (
    block &&
    (block.type === "pa_result" ||
      block.type === "warning" ||
      block.type === "pa_status")
  );
}

function groupChainBlocks(chain) {
  const out = [];
  let paBlocks = [];
  let pendingCommands = [];
  const flushPa = () => {
    if (!paBlocks.length) return;
    out.push({
      type: "pa_trace",
      blocks: paBlocks,
      commands: pendingCommands,
    });
    paBlocks = [];
    pendingCommands = [];
  };
  for (const block of chain || []) {
    if (isPaFeedbackBlock(block)) {
      paBlocks.push(block);
    } else {
      flushPa();
      if (block?.type === "text") {
        pendingCommands = [
          ...pendingCommands,
          ...extractPaCommands(block.text || ""),
        ];
      }
      out.push(block);
    }
  }
  flushPa();
  return out;
}

function stripPaCommandLines(text, shouldStrip) {
  if (!shouldStrip || !text) return text || "";
  const lines = String(text)
    .split(/\r?\n/)
    .filter((line) => !PA_COMMAND_LINE.test(line));
  return lines
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function commandLinesFromText(text) {
  return String(text || "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => PA_COMMAND_LINE.test(line));
}

function isCommandOnlyText(text) {
  const lines = String(text || "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  return lines.length > 0 && lines.every((line) => PA_COMMAND_LINE.test(line));
}

function looksLikeRawDiagnosticDump(text) {
  const lines = String(text || "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  if (lines.length < 4) return false;
  const hits = lines.filter((line) =>
    /===|summary:|warnings?:|errors?:|pending|dirty|clean|no matching|template versions|git status|health check/i.test(
      line,
    ),
  ).length;
  return hits >= 3 && hits / lines.length >= 0.45;
}

function CommandBatchCard({ text }) {
  const commands = commandLinesFromText(text);
  if (!commands.length) return null;
  return html`<details class="command-draft-card">
    <summary>
      <span>command batch</span>
      <b>${commands.length}</b>
      <em>prepared by agent</em>
      <button
        type="button"
        onClick=${(e) => {
          e.preventDefault();
          e.stopPropagation();
          navigator.clipboard.writeText(commands.join("\n"));
          showToast("Commands copied", "success", 1500);
        }}
      >
        copy
      </button>
    </summary>
    <div class="command-draft-list">
      ${commands.map((cmd, i) => html`<code key=${"cmd" + i}>${cmd}</code>`)}
    </div>
  </details>`;
}

function DiagnosticDumpCard({ text }) {
  const lineCount = String(text || "")
    .split(/\r?\n/)
    .filter(Boolean).length;
  return html`<details class="diagnostic-dump-card">
    <summary>
      <span>raw diagnostics hidden</span>
      <b>${lineCount} lines</b>
      <em>covered by run card</em>
      <button
        type="button"
        onClick=${(e) => {
          e.preventDefault();
          e.stopPropagation();
          navigator.clipboard.writeText(text || "");
          showToast("Diagnostics copied", "success", 1500);
        }}
      >
        copy
      </button>
    </summary>
    <pre>${text}</pre>
  </details>`;
}

function extractPaCommand(text) {
  const match = String(text || "").match(PA_COMMAND_PATTERN);
  return match ? match[0] : "";
}

function extractPaCommands(text) {
  return [...String(text || "").matchAll(PA_COMMAND_PATTERN)].map((m) => m[0]);
}

function previewLine(text, limit = 96) {
  const oneLine = String(text || "")
    .replace(/\s+/g, " ")
    .trim();
  return oneLine.length > limit ? oneLine.slice(0, limit) + "..." : oneLine;
}

function cleanTraceText(text) {
  return String(text || "")
    .replace(/\u0432\u045a\u201c/g, "OK")
    .replace(/\u0432\u045a\u2014/g, "FAIL")
    .replace(/\u0432\u0459\u00a0/g, "WARN")
    .replace(/\u0432\u2020\u2019/g, "->")
    .replace(/\u0432\u2020\u2018/g, " ahead")
    .replace(/\u0432\u2020\u201c/g, " behind");
}

function isNoiseResult(text) {
  return /^(no matching delegations\.|no matching log entries\.|no output\.?)$/i.test(
    String(text || "").trim(),
  );
}

function classifyTraceStatus(text, fallback = "done") {
  const t = String(text || "");
  if (/error|failed|blocked|not parsed|permission denied/i.test(t)) {
    return "warning";
  }
  if (/warnings?:\s*[1-9]|errors?:\s*[1-9]/i.test(t)) {
    return "warning";
  }
  return fallback;
}

function traceHint(text) {
  const t = String(text || "");
  if (/Delegation not parsed/i.test(t)) {
    return "Use [DELEGATE:Project]task[/DELEGATE], or run status/log commands without base delegate syntax.";
  }
  if (/permission denied|write access|review-only/i.test(t)) {
    return "Pick a write-enabled lead or grant provider permissions before executing.";
  }
  if (/warnings?:\s*[1-9]|errors?:\s*[1-9]/i.test(t)) {
    return "Inspect warning projects before continuing the rollout.";
  }
  if (/no matching/i.test(t)) {
    return "No matching state found. This is informational unless you expected pending work.";
  }
  return "";
}

function summarizeTraceOutput(text, status) {
  const clean = cleanTraceText(text || "").trim();
  if (!clean) return status === "queued" ? "not executed" : "";
  if (isNoiseResult(clean)) return "no matches";
  const lines = clean
    .split(/\r?\n/)
    .map((line) =>
      line
        .replace(/^#+\s*/, "")
        .replace(/\*\*/g, "")
        .replace(/`/g, "")
        .trim(),
    )
    .filter(Boolean);
  const firstSignal =
    lines.find((line) =>
      /error|failed|warning|blocked|summary|delegations|git status|template|health/i.test(
        line,
      ),
    ) || lines[0];
  return previewLine(firstSignal || clean, 110);
}

function buildPaTraceRows(blocks, commands = []) {
  const rows = [];
  let commandIndex = 0;
  const nextCommand = () => commands[commandIndex++] || "";
  const consumeCommand = (command) => {
    if (!command) return "";
    const exact = commands.indexOf(command, commandIndex);
    if (exact >= commandIndex) commandIndex = exact + 1;
    return command;
  };
  for (const block of blocks || []) {
    const text = cleanTraceText(block.text || "");
    const explicitCommand = block.command || "";
    if (block.type === "pa_status") {
      const command =
        consumeCommand(explicitCommand || extractPaCommand(text)) ||
        nextCommand();
      const lower = text.toLowerCase();
      rows.push({
        type: "status",
        status: lower.startsWith("running")
          ? "running"
          : lower.startsWith("completed")
            ? "done"
            : "info",
        command,
        label: previewLine(
          text.replace(command, "").replace(/^running\s*/i, ""),
        ),
        output: "",
        summary: "",
      });
      continue;
    }
    if (block.type === "pa_result") {
      const resultCommand = explicitCommand || extractPaCommand(text);
      const last = [...rows]
        .reverse()
        .find(
          (row) =>
            row.type === "status" &&
            !row.output &&
            (!resultCommand || row.command === resultCommand),
        );
      if (last) {
        last.status = classifyTraceStatus(text, "done");
        last.output = text;
        last.summary = summarizeTraceOutput(text, last.status);
        last.hint = traceHint(text);
      } else {
        const command = consumeCommand(resultCommand) || nextCommand();
        const status = classifyTraceStatus(text, "done");
        rows.push({
          type: "result",
          status,
          command,
          label: isNoiseResult(text) ? "" : previewLine(text, 72) || "result",
          output: text,
          summary: summarizeTraceOutput(text, status),
          hint: traceHint(text),
        });
      }
      continue;
    }
    const command =
      consumeCommand(explicitCommand || extractPaCommand(text)) ||
      nextCommand();
    rows.push({
      type: "warning",
      status: "warning",
      command,
      label: "warning",
      output: text,
      summary: summarizeTraceOutput(text, "warning"),
      hint: traceHint(text),
    });
  }
  while (commandIndex < commands.length) {
    rows.push({
      type: "status",
      status: "queued",
      command: nextCommand(),
      label: "not executed",
      output: "",
      summary: "not executed",
    });
  }
  return rows;
}

function copyTraceRows(rows) {
  const text = rows
    .map((row, i) => {
      const head = `${i + 1}. ${row.status.toUpperCase()} ${row.command || "PA command"} - ${row.summary || row.label || ""}`;
      return row.output ? `${head}\n${row.output}` : head;
    })
    .join("\n\n");
  navigator.clipboard.writeText(text);
  showToast("Run copied", "success", 1500);
}

function PaTraceRow({ row, index, forceOpen }) {
  const noisy = isNoiseResult(row.output);
  const hasOutput = !!row.output && !noisy;
  const cls =
    row.status === "warning"
      ? "is-warning"
      : row.status === "running"
        ? "is-running"
        : row.status === "done"
          ? "is-done"
          : row.status === "queued"
            ? "is-queued"
            : "";
  return html`<details
    class="run-row ${cls}"
    key=${"run-row" + index}
    open=${forceOpen || row.status === "running"}
  >
    <summary class="run-row-summary">
      <span class="run-index">${index + 1}</span>
      <span class="run-state">${row.status}</span>
      <span class="run-command">${row.command || "PA command"}</span>
      <span class="run-summary">
        ${noisy ? "no matches" : row.summary || row.label || "completed"}
      </span>
      <span class="run-open">${hasOutput ? "details" : ""}</span>
    </summary>
    ${hasOutput
      ? html`${row.hint
            ? html`<div class="run-hint-body">${row.hint}</div>`
            : null}
          <pre
            class="run-output ${row.status === "warning" ? "is-warning" : ""}"
          >
${row.output}</pre
          >`
      : null}
  </details>`;
}

function PaTrace({ blocks, commands }) {
  const [filter, setFilter] = useState("all");
  const [expanded, setExpanded] = useState(false);
  const rows = buildPaTraceRows(blocks, commands);
  if (!rows.length) return null;
  const commandCount = rows.length;
  const warningCount = rows.filter((row) => row.status === "warning").length;
  const doneCount = rows.filter((row) => row.status === "done").length;
  const runningCount = rows.filter((row) => row.status === "running").length;
  const noisyCount = rows.filter((row) => isNoiseResult(row.output)).length;
  const outputCount = rows.filter(
    (row) => row.output && !isNoiseResult(row.output),
  ).length;
  const visibleRows = rows.filter((row) => {
    if (filter === "problems")
      return row.status === "warning" || row.status === "running";
    if (filter === "outputs") return row.output && !isNoiseResult(row.output);
    return true;
  });
  const headline = runningCount
    ? `${runningCount} running`
    : warningCount
      ? `${warningCount} warning${warningCount === 1 ? "" : "s"}`
      : `${doneCount}/${commandCount} complete`;
  return html`<div class="run-card">
    <div class="run-head">
      <div class="run-head-main">
        <span class="run-kicker">run</span>
        <span class="run-title"
          >${commandCount} PA command${commandCount === 1 ? "" : "s"}</span
        >
        <span class="run-headline">${headline}</span>
      </div>
      <div class="run-badges">
        ${noisyCount ? html`<span>${noisyCount} empty</span>` : null}
        ${warningCount
          ? html`<span class="warn">${warningCount} warn</span>`
          : null}
        ${outputCount ? html`<span>${outputCount} output</span>` : null}
        <button
          type="button"
          class=${filter === "all" ? "active" : ""}
          onClick=${() => setFilter("all")}
        >
          all
        </button>
        <button
          type="button"
          class=${filter === "problems" ? "active" : ""}
          onClick=${() => setFilter("problems")}
        >
          issues
        </button>
        <button
          type="button"
          class=${filter === "outputs" ? "active" : ""}
          onClick=${() => setFilter("outputs")}
        >
          outputs
        </button>
        <button type="button" onClick=${() => setExpanded(!expanded)}>
          ${expanded ? "collapse" : "expand"}
        </button>
        <button type="button" onClick=${() => copyTraceRows(rows)}>copy</button>
      </div>
    </div>
    <div class="run-table">
      ${visibleRows.length
        ? visibleRows.map(
            (row, i) =>
              html`<${PaTraceRow}
                row=${row}
                index=${rows.indexOf(row)}
                forceOpen=${expanded}
                key=${"visible-row" + i}
              />`,
          )
        : html`<div class="run-empty-filter">No rows for this filter.</div>`}
    </div>
  </div>`;
}

function TextBlock({ text, stripCommands, prefix }) {
  if (isCommandOnlyText(text)) {
    if (stripCommands) return null;
    return html`<${CommandBatchCard} key=${prefix} text=${text} />`;
  }
  if (stripCommands && looksLikeRawDiagnosticDump(text)) {
    return html`<${DiagnosticDumpCard} key=${prefix} text=${text} />`;
  }
  const t = stripPaCommandLines(text, stripCommands);
  if (!t) return null;
  return html`<div
    key=${prefix}
    dangerouslySetInnerHTML=${{ __html: md(t) }}
  ></div>`;
}
function DuoLiveStatus({ session }) {
  if (!session) return null;
  const participants = session.participants || [];
  const presence = session.presence || {};
  const active = participants.filter((p) => {
    const state = presence[p.id] || "idle";
    return state && state !== "idle";
  });
  if (!active.length) return null;
  return html`${active.map((participant) => {
    const state = presence[participant.id] || "idle";
    const tone =
      participant.provider === "claude" ? "var(--cyan)" : "var(--yellow)";
    return html`<div
      class="m a"
      style="border-left:2px solid ${tone};padding-left:var(--sp-s);opacity:.92"
    >
      <div class="msg-src src-deleg">${participant.label}</div>
      <div style="font-size:var(--fs-s);color:var(--text)">
        ${participant.label} is ${state}
      </div>
    </div>`;
  })}`;
}
function ChatMsg({ m }) {
  if (!m.msg && !m.chain?.length) return null;
  // System messages — enhanced styling
  if (m.role === "system") {
    if (/^Session started/i.test(m.msg || "")) {
      return html`<div class="session-divider">
        <span>${m.msg}</span>
        <em>${ft(m.ts)}</em>
      </div>`;
    }
    const isSuccess =
      (m.msg || "").includes("✓") || (m.msg || "").includes("complete");
    const isFail =
      (m.msg || "").includes("✗") ||
      (m.msg || "").includes("failed") ||
      (m.msg || "").includes("Error");
    return html`<div
      class="m sys-msg ${isSuccess ? "sys-success" : isFail ? "sys-fail" : ""}"
    >
      <div class="msg-src src-sys">SYSTEM</div>
      <div dangerouslySetInnerHTML=${{ __html: md(m.msg) }}></div>
      <div class="ts" title=${m.ts || ""}>${ft(m.ts)}</div>
    </div>`;
  }
  const isViaPA = (m.msg || "").startsWith("[via PA]");
  const isDuoMsg = m.mode === "duo" || !!m.room_session_id;
  const isError =
    (m.msg || "").startsWith("Error:") || (m.msg || "").includes("timed out");
  const canCopy = m.role === "assistant";
  const cls = m.role === "user" ? (isViaPA ? "a" : "u") : "a";
  const dms = [
    ...(m.msg || "").matchAll(/<delegation id="([^"]+)" project="([^"]+)"\/>/g),
  ];
  const clean = (m.msg || "").replace(/<delegation[^>]*\/>/g, "");
  const chain = m.chain || [];
  const groupedChain = groupChainBlocks(chain);
  const hasPaTrace = groupedChain.some((b) => b.type === "pa_trace");
  // If we have a chain, render blocks in order. Otherwise fall back to flat text+tools
  const hasChain = groupedChain.length > 0 && m.role === "assistant";
  const duoAgentLabel = (m.meta || "").replace(/^[\s·]+/, "").trim();
  const srcLabel =
    m.role === "user"
      ? isViaPA
        ? "DELEGATED BY PA"
        : isDuoMsg
          ? "YOU · DUO"
          : "YOU"
      : isDuoMsg && duoAgentLabel
        ? duoAgentLabel
        : currentProject.value
          ? currentProject.value + " AGENT"
          : "ORCHESTRATOR";
  const srcCls =
    m.role === "user"
      ? isViaPA
        ? "src-deleg"
        : "src-user"
      : isDuoMsg
        ? "src-deleg"
        : "src-pa";
  return html`<div
    class="m ${cls}"
    style=${isDuoMsg && m.role === "assistant"
      ? `border-left:${
          duoAgentLabel.includes("Claude")
            ? "2px solid var(--cyan)"
            : duoAgentLabel.includes("Codex")
              ? "2px solid var(--yellow)"
              : "1px solid var(--border)"
        };padding-left:var(--sp-s)`
      : ""}
  >
    <div class="msg-src ${srcCls}">${srcLabel}</div>
    ${hasChain
      ? groupedChain.map((b, i) => {
          if (b.type === "text") {
            return html`<${TextBlock}
              key=${"t" + i}
              text=${b.text || ""}
              stripCommands=${hasPaTrace}
              prefix=${"t" + i}
            />`;
          }
          if (b.type === "tool")
            return html`<${ToolCard} key=${"tc" + i} t=${b} />`;
          if (b.type === "tool_result")
            return html`<div
              key=${"tr" + i}
              class="tc-output ${b.is_error ? "tc-out-err" : ""}"
              style="margin:var(--sp-2xs) 0;font-family:var(--font-mono);font-size:var(--fs-s)"
            >
              ${b.content || ""}
            </div>`;
          if (b.type === "thinking")
            return html`<${ThinkBlock} key=${"th" + i} b=${b} />`;
          if (b.type === "pa_trace")
            return html`<${PaTrace}
              key=${"pat" + i}
              blocks=${b.blocks}
              commands=${b.commands}
            />`;
          if (b.type === "system")
            return html`<div
              key=${"s" + i}
              style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
            >
              ••• ${b.label}
            </div>`;
          if (
            b.type === "pa_result" ||
            b.type === "warning" ||
            b.type === "pa_status"
          ) {
            if (hasPaTrace && looksLikeRawDiagnosticDump(b.text || "")) {
              return html`<${DiagnosticDumpCard}
                key=${"pa-dump" + i}
                text=${b.text || ""}
              />`;
            }
            return html`<div
              key=${"pa" + i}
              class="tc-output ${b.type === "warning" ? "tc-out-err" : ""}"
              style="margin:var(--sp-xs) 0;font-family:var(--font-mono);font-size:var(--fs-s);white-space:pre-wrap"
            >
              ${b.type === "warning"
                ? "warning"
                : b.type === "pa_status"
                  ? "pa status"
                  : "pa result"}:
              ${b.text || ""}
            </div>`;
          }
          if (b.type === "result")
            return html`<div
              key=${"r" + i}
              style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono);margin-top:var(--sp-xs)"
            >
              ⏱ ${b.tokens || "?"}t ·
              ${Math.round((b.duration_ms || 0) / 1000)}s${b.cost
                ? " · $" + b.cost
                : ""}
            </div>`;
          return null;
        })
      : m.role === "assistant"
        ? html`<${TextBlock}
            text=${clean}
            stripCommands=${false}
            prefix="flat"
          />`
        : html`<div
            dangerouslySetInnerHTML=${{
              __html: esc(
                isViaPA ? (m.msg || "").replace("[via PA] ", "") : m.msg,
              ),
            }}
          ></div>`}
    ${canCopy
      ? html`<button
          class="msg-copy"
          onClick=${() => {
            navigator.clipboard.writeText(m.msg || "");
            showToast("Copied", "success", 1500);
          }}
        >
          copy
        </button>`
      : ""}
    ${dms.map((dm) => html`<${DelegBtn} id=${dm[1]} project=${dm[2]} />`)}
    <div class="ts" title=${m.ts || ""}>
      ${ft(m.ts)}${m.meta || ""}${isError && m.role === "assistant"
        ? html` <button
            class="retry-btn"
            onClick=${() => sendMessage(lastUserMsg.value)}
          >
            retry
          </button>`
        : ""}
    </div>
  </div>`;
}

function StreamBubble() {
  const run = activeRun.value;
  const runningRun =
    run &&
    run.project === (currentProject.value || "_orchestrator") &&
    !["done", "failed", "cancelled"].includes(run.status || "");
  if (
    !isStreaming.value &&
    !streamText.value &&
    !streamChain.value.length &&
    !curActivity.value &&
    !runningRun
  ) {
    return null;
  }
  const ch = streamChain.value;
  const groupedCh = groupChainBlocks(ch);
  const hasPaTrace = groupedCh.some((b) => b.type === "pa_trace");
  const el = thinkStart.value
    ? Math.round((Date.now() - thinkStart.value) / 1000)
    : 0;
  const act =
    curActivity.value ||
    (runningRun ? `${run.phase || "run"}: ${run.detail || "working"}` : "");
  // Show progress bar + chain blocks
  return html`<div class="m a stream-chain">
    <${ProgressBar} activity=${act} elapsed=${el} />
    ${groupedCh.map((b, i) => {
      if (b.type === "text")
        return html`<${TextBlock}
          key=${"st" + i}
          text=${b.text || ""}
          stripCommands=${hasPaTrace}
          prefix=${"st" + i}
        />`;
      if (b.type === "tool")
        return html`<${ToolCard} key=${"stc" + i} t=${b} />`;
      if (b.type === "tool_result")
        return html`<div
          key=${"str" + i}
          class="tc-output"
          style="margin:var(--sp-2xs) 0;font-family:var(--font-mono);font-size:var(--fs-s)"
        >
          ${b.content || ""}
        </div>`;
      if (b.type === "thinking")
        return html`<${ThinkBlock} key=${"sth" + i} b=${b} />`;
      if (b.type === "pa_trace")
        return html`<${PaTrace}
          key=${"spat" + i}
          blocks=${b.blocks}
          commands=${b.commands}
        />`;
      if (b.type === "system")
        return html`<div
          key=${"ss" + i}
          style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
        >
          ••• ${b.label}
        </div>`;
      if (
        b.type === "pa_result" ||
        b.type === "warning" ||
        b.type === "pa_status"
      )
        return html`<div
          key=${"spa" + i}
          class="tc-output ${b.type === "warning" ? "tc-out-err" : ""}"
          style="margin:var(--sp-xs) 0;font-family:var(--font-mono);font-size:var(--fs-s);white-space:pre-wrap"
        >
          ${b.type === "warning"
            ? "warning"
            : b.type === "pa_status"
              ? "pa status"
              : "pa result"}:
          ${b.text || ""}
        </div>`;
      return null;
    })}
    ${!streamText.value && !ch.length && !act
      ? html`<${ProgressBar} activity=${"thinking..."} elapsed=${el} />`
      : null}
  </div>`;
}

function DelegBtn({ id, project }) {
  const d = delegations.value[id];
  const el = d?._start ? Math.round((Date.now() - d._start) / 1000) : 0;
  const SI = {
    pending: "○",
    scheduled: "◷",
    running: "⏳",
    escalated: "⚡",
    deciding: "?",
    done: "✓",
    failed: "✗",
    rejected: "✗",
    error: "✗",
    cancelled: "—",
  };
  const s = d?.status || "pending";
  const cls =
    s === "done"
      ? "dc2-done"
      : s === "failed" || s === "error"
        ? "dc2-fail"
        : s === "running"
          ? "dc2-run"
          : s === "escalated"
            ? "dc2-esc"
            : "";
  const stCls =
    s === "done"
      ? "st-done"
      : s === "failed" || s === "error"
        ? "st-failed"
        : s === "running"
          ? "st-running"
          : s === "escalated"
            ? "st-escalated"
            : "st-pending";
  const stream = delegStreams.value[id];
  return html`<div class="dc2 ${cls}">
    <div class="dc2-hdr">
      <span class="dc2-icon">${SI[s] || "○"}</span>
      <span class="dc2-proj">${project}</span>
      <span class="dc2-status ${stCls}"
        >${s}${el > 2 ? " " + el + "s" : ""}${stream
          ? " · " + stream.stage
          : ""}</span
      >
      ${d?.usage
        ? html`<span
            style="font-size:var(--fs-s);color:var(--t3);margin-left:auto"
            >$$${d.usage.cost_usd?.toFixed(4) || "?"}</span
          >`
        : null}
    </div>
    ${d?.task ? html`<div class="dc2-task">${d.task}</div>` : null}
    ${d?.scheduled_at && s === "scheduled"
      ? html`<div
          style="font-size:var(--fs-s);color:var(--t3);margin-top:var(--sp-2xs)"
        >
          @ ${ft(d.scheduled_at)}
        </div>`
      : null}
    ${s === "pending"
      ? html`<div class="dc2-actions">
          <button class="dc2-approve" onClick=${() => approveDel(id)}>
            approve
          </button>
          <button class="dc2-reject" onClick=${() => rejectDel(id)}>
            reject
          </button>
          <button
            style="border:1px solid var(--t3);color:var(--t3);background:var(--sf);padding:var(--sp-2xs) var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s);cursor:pointer"
            onClick=${() => {
              const dt = prompt("Schedule (YYYY-MM-DD HH:MM):");
              if (dt) {
                __invoke("schedule_delegation", {
                  id,
                  scheduledAt: new Date(dt).toISOString(),
                }).then(() => {
                  const d2 = { ...delegations.value };
                  d2[id] = {
                    ...d2[id],
                    status: "scheduled",
                    scheduled_at: new Date(dt).toISOString(),
                  };
                  delegations.value = d2;
                  showToast("Scheduled", "success");
                });
              }
            }}
          >
            schedule
          </button>
        </div>`
      : null}
    ${s === "running" || s === "escalated"
      ? html`<div class="dc2-actions">
          <button
            class="dc2-reject"
            onClick=${() => {
              __invoke("cancel_delegation", { id })
                .then(() => {
                  const d2 = { ...delegations.value };
                  d2[id] = { ...d2[id], status: "cancelled" };
                  delegations.value = d2;
                  showToast("Cancelled", "info");
                })
                .catch((e) => showToast("Cancel: " + e, "error"));
            }}
          >
            cancel
          </button>
        </div>`
      : null}
    ${s === "done"
      ? html`<div style="margin-top:var(--sp-xs)">
          <span
            style="color:var(--t3);cursor:pointer;font-size:var(--fs-s);font-family:var(--font-mono);text-decoration:underline"
            onClick=${() => {
              currentProject.value = project;
            }}
            >open project →</span
          >
        </div>`
      : null}
    ${s === "failed" || s === "error"
      ? html`<div class="dc2-actions">
          <button
            class="dc2-approve"
            onClick=${() => {
              const d2 = { ...delegations.value };
              d2[id] = { ...d2[id], status: "pending" };
              delegations.value = d2;
              approveDel(id);
            }}
          >
            retry
          </button>
        </div>`
      : null}
    ${s === "rejected"
      ? html`<div class="dc2-actions">
          <button
            style="border:1px solid var(--t3);color:var(--t3);background:var(--sf);padding:var(--sp-2xs) var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s);cursor:pointer"
            onClick=${() => {
              const d2 = { ...delegations.value };
              d2[id] = { ...d2[id], status: "pending" };
              delegations.value = d2;
            }}
          >
            re-queue
          </button>
        </div>`
      : null}
    ${d?.git_diff && s === "done"
      ? html`<details
          style="margin-top:var(--sp-xs);font-size:var(--fs-s);color:var(--t3);width:100%"
        >
          <summary style="cursor:pointer;font-family:var(--font-mono)">
            git changes
          </summary>
          <pre
            style="white-space:pre-wrap;max-height:80px;overflow:auto;margin-top:var(--sp-xs);font-size:var(--fs-s)"
          >
${d.git_diff}</pre
          >
        </details>`
      : null}
  </div>`;
}

function DelegTracker() {
  const active = Object.entries(delegations.value).filter(([_, d]) =>
    ["pending", "running", "escalated", "scheduled", "deciding"].includes(
      d?.status,
    ),
  );
  if (!active.length) return null;
  return html`<div class="deleg-tracker">
    ${active.map(
      ([id, d]) => html`<${DelegBtn} id=${id} project=${d.project || "?"} />`,
    )}
  </div>`;
}

function NewProjectModal() {
  if (!showNewProject.value) return null;
  const nameRef = useRef();
  const orchRef = useRef();
  const create = async () => {
    const name = nameRef.current?.value?.trim();
    if (!name) {
      showToast("Enter project name", "error");
      return;
    }
    const isOrch = orchRef.current?.checked;
    try {
      const r = await __invoke("create_project", {
        name,
        orchestrator: isOrch || false,
      });
      if (r.status === "ok") {
        showToast("Project created: " + name, "success");
        showNewProject.value = false;
        loadAgents();
      } else {
        showToast("Error: " + (r.error || "unknown"), "error");
      }
    } catch (e) {
      showToast("Error: " + e, "error");
    }
  };
  return html`<div
    style="position:fixed;inset:0;background:rgba(0,0,0,.7);z-index:1001;display:flex;align-items:center;justify-content:center"
    onClick=${(e) => {
      if (e.target === e.currentTarget) showNewProject.value = false;
    }}
  >
    <div
      style="background:var(--bg);border:1px solid var(--border);padding:var(--sp-xl);min-width:400px;max-width:500px"
    >
      <h3
        style="margin:0 0 var(--sp-l);font-family:var(--font-mono);letter-spacing:2px"
      >
        NEW PROJECT
      </h3>
      <div style="margin-bottom:var(--sp-m)">
        <label
          style="display:block;font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs);font-family:var(--font-mono)"
          >project name</label
        >
        <input
          ref=${nameRef}
          style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-m)"
          placeholder="my-awesome-project"
          onKeyDown=${(e) => {
            if (e.key === "Enter") create();
          }}
        />
      </div>
      <div
        style="margin-bottom:var(--sp-l);display:flex;align-items:center;gap:var(--sp-s)"
      >
        <input type="checkbox" ref=${orchRef} id="orch-check" />
        <label
          for="orch-check"
          style="font-size:var(--fs-s);color:var(--t2);font-family:var(--font-mono)"
          >set as orchestrator</label
        >
      </div>
      <div style="display:flex;gap:var(--sp-s);justify-content:flex-end">
        <button
          class="action-btn"
          onClick=${() => (showNewProject.value = false)}
        >
          cancel
        </button>
        <button
          style="background:var(--green);color:var(--bg);border:none;padding:var(--sp-xs) var(--sp-l);font-family:var(--font-mono);font-size:var(--fs-s);cursor:pointer"
          onClick=${create}
        >
          create
        </button>
      </div>
    </div>
  </div>`;
}

function Toasts() {
  return html`<div class="toasts">
    ${toasts.value.map(
      (t) => html`<div class="toast ${t.type}" key=${t.id}>${t.msg}</div>`,
    )}
  </div>`;
}

function Feed() {
  return html`<div class="feed">
    ${feedItems.value.length
      ? feedItems.value.map(
          (e) =>
            html`<span
              >${ft(e.time)} [${e.type}] <b>${e.project || ""}</b> ${(
                e.message || ""
              ).substring(0, 40)}${"  "}</span
            >`,
        )
      : "no activity"}
  </div>`;
}

export {
  Tile,
  DetailView,
  InboxPanel,
  DelegationPanel,
  RunningBanner,
  ChatSidebar,
  ToolCard,
  ThinkBlock,
  ProgressBar,
  ChatMsg,
  StreamBubble,
  DelegBtn,
  NewProjectModal,
  Toasts,
  Feed,
};
