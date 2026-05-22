import { html, useEffect, useState } from "/vendor/preact-bundle.mjs";
import {
  activeDualSession,
  activeWorkspaceTab,
  currentProject,
  delegations,
  delegStreams,
  showToast,
} from "/store.js";
import {
  approveDel,
  loadDelegations,
  loadExecutionMap,
  loadFeed,
  loadOrchestrationMap,
  rejectDel,
} from "/api.js";
import { __IS_TAURI, __invoke } from "/bridge.js";

const RUNNING_STATUSES = new Set([
  "running",
  "escalated",
  "deciding",
  "verifying",
]);
const APPROVAL_STATUSES = new Set(["pending", "needs_permission", "scheduled"]);
const TERMINAL_STATUSES = new Set([
  "done",
  "failed",
  "rejected",
  "cancelled",
  "error",
]);

function statusOf(delegation) {
  return String(delegation?.status || "pending")
    .trim()
    .toLowerCase();
}

function needsUser(delegation) {
  const status = statusOf(delegation);
  return (
    APPROVAL_STATUSES.has(status) ||
    status === "failed" ||
    status === "error" ||
    (delegation?.terminal === false && delegation?.blockers)
  );
}

function timestampOf(delegation) {
  return delegation?.started_at || delegation?.ts || "";
}

function ageOf(delegation) {
  const ts = timestampOf(delegation);
  const time = ts ? Date.parse(ts) : 0;
  if (!time || Number.isNaN(time)) return "";
  const minutes = Math.max(0, Math.round((Date.now() - time) / 60000));
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.round(minutes / 60);
  if (hours < 48) return `${hours}h`;
  return `${Math.round(hours / 24)}d`;
}

function toneOf(delegation) {
  const status = statusOf(delegation);
  if (status === "done") return "done";
  if (status === "failed" || status === "error") return "failed";
  if (status === "rejected" || status === "cancelled") return "muted";
  if (RUNNING_STATUSES.has(status)) return "running";
  if (APPROVAL_STATUSES.has(status)) return "needs-user";
  return "default";
}

function sortedDelegations() {
  return Object.entries(delegations.value || {})
    .map(([id, value]) => ({ id, ...(value || {}) }))
    .sort((a, b) => {
      const aNeeds = needsUser(a) ? 1 : 0;
      const bNeeds = needsUser(b) ? 1 : 0;
      if (aNeeds !== bNeeds) return bNeeds - aNeeds;
      const aRunning = RUNNING_STATUSES.has(statusOf(a)) ? 1 : 0;
      const bRunning = RUNNING_STATUSES.has(statusOf(b)) ? 1 : 0;
      if (aRunning !== bRunning) return bRunning - aRunning;
      return timestampOf(b).localeCompare(timestampOf(a));
    });
}

function providerSummary(delegation) {
  return [
    delegation.executor_provider
      ? `executor: ${delegation.executor_provider}`
      : "",
    delegation.reviewer_provider
      ? `reviewer: ${delegation.reviewer_provider}`
      : "",
    delegation.priority ? `priority: ${delegation.priority}` : "",
    delegation.timeout_secs ? `timeout: ${delegation.timeout_secs}s` : "",
  ].filter(Boolean);
}

function linkSummary(delegation) {
  return [
    delegation.batch_id ? `batch ${delegation.batch_id}` : "",
    delegation.plan_id ? `plan ${delegation.plan_id}` : "",
    delegation.work_item_id ? `work ${delegation.work_item_id}` : "",
    delegation.project_session_id ? `session ${delegation.project_session_id}` : "",
    delegation.room_session_id ? `room ${delegation.room_session_id}` : "",
  ].filter(Boolean);
}

function healthSummary(delegation) {
  const gate = delegation.gate_result?.status || delegation.gate_result?.result || "";
  const review = delegation.review_verdict?.status || "";
  const usage = delegation.usage?.cost_usd
    ? `cost $${Number(delegation.usage.cost_usd).toFixed(4)}`
    : "";
  return [
    gate ? `gate: ${gate}` : "",
    review ? `review: ${review}` : "",
    usage,
  ].filter(Boolean);
}

async function executeDelegationCommand(text, label = "command") {
  if (!text?.trim()) return;
  if (!__IS_TAURI || !__invoke) {
    showToast("Команда доступна только в desktop app", "error", 2200);
    return;
  }
  try {
    const res = await __invoke("execute_pa_text", { text });
    const errorCount = res?.errors?.length || 0;
    const commandCount = res?.commands?.length || 0;
    showToast(
      errorCount
        ? `${label}: ${commandCount} выполнено, ${errorCount} ошибок`
        : `${label}: выполнено`,
      errorCount ? "error" : "success",
      2200,
    );
    await Promise.allSettled([
      loadDelegations(),
      loadFeed(),
      loadExecutionMap("", activeDualSession.value || null, 180),
      loadOrchestrationMap("", activeDualSession.value || null),
    ]);
  } catch (e) {
    showToast(`Не удалось выполнить: ${e}`, "error", 2600);
  }
}

function DelegationCard({ item }) {
  const status = statusOf(item);
  const stream = delegStreams.value?.[item.id];
  const provider = providerSummary(item);
  const links = linkSummary(item);
  const health = healthSummary(item);
  const isApproval = APPROVAL_STATUSES.has(status);
  const isRunning = RUNNING_STATUSES.has(status);
  const canRetry = status === "failed" || status === "error";
  const shortId = String(item.id || "").slice(0, 18);
  return html`<article class=${`delegation-work-card ${toneOf(item)}`}>
    <div class="delegation-work-top">
      <div>
        <b>${item.project || "unknown project"}</b>
        <span>${status}${ageOf(item) ? ` / ${ageOf(item)}` : ""}</span>
      </div>
      <code>${shortId}</code>
    </div>
    <p>${item.task || "No task text captured"}</p>
    <div class="delegation-work-meta">
      ${provider.map((label) => html`<span>${label}</span>`)}
      ${links.map((label) => html`<span>${label}</span>`)}
      ${health.map((label) => html`<span>${label}</span>`)}
      ${stream ? html`<span>stream: ${stream.stage || "running"}</span>` : null}
    </div>
    ${stream?.events?.length
      ? html`<details class="delegation-stream-details">
          <summary>live stream</summary>
          ${(stream.events || []).slice(-6).map(
            (evt) =>
              html`<div>
                <b>${evt.type || "event"}</b>
                <span>
                  ${evt.stage ||
                  evt.label ||
                  evt.tool ||
                  evt.reason ||
                  evt.response ||
                  evt.text ||
                  ""}
                </span>
              </div>`,
          )}
        </details>`
      : null}
    <div class="delegation-work-actions">
      ${isApproval
        ? html`<button class="primary" onClick=${() => approveDel(item.id)}>
              approve
            </button>
            <button onClick=${() => rejectDel(item.id)}>reject</button>`
        : null}
      ${isRunning
        ? html`<button
            onClick=${() =>
              executeDelegationCommand(`[DELEGATE_CANCEL:${item.id}]`, "cancel")}
          >
            cancel
          </button>`
        : null}
      ${canRetry
        ? html`<button
            class="primary"
            onClick=${() =>
              executeDelegationCommand(
                `[DELEGATE_RETRY:${item.id}]Повтори делегацию через доступный provider. Сначала проверь diff/health, затем верни фактический результат и блокеры.[/DELEGATE_RETRY]`,
                "retry",
              )}
          >
            retry
          </button>`
        : null}
      <button
        onClick=${() =>
          executeDelegationCommand(`[DELEGATE_STATUS:${item.id}]`, "status")}
      >
        status
      </button>
      <button
        onClick=${() => {
          currentProject.value = item.project || currentProject.value;
          activeWorkspaceTab.value = "projects";
        }}
      >
        open project
      </button>
    </div>
  </article>`;
}

export function DelegationsWorkspace() {
  const [filter, setFilter] = useState("open");
  useEffect(() => {
    loadDelegations();
  }, []);
  const rows = sortedDelegations();
  const counts = {
    needs: rows.filter(needsUser).length,
    running: rows.filter((item) => RUNNING_STATUSES.has(statusOf(item))).length,
    pending: rows.filter((item) => APPROVAL_STATUSES.has(statusOf(item))).length,
    failed: rows.filter((item) => ["failed", "error"].includes(statusOf(item))).length,
    done: rows.filter((item) => statusOf(item) === "done").length,
  };
  const visible = rows.filter((item) => {
    const status = statusOf(item);
    if (filter === "all") return true;
    if (filter === "needs") return needsUser(item);
    if (filter === "running") return RUNNING_STATUSES.has(status);
    if (filter === "pending") return APPROVAL_STATUSES.has(status);
    if (filter === "failed") return status === "failed" || status === "error";
    if (filter === "done") return status === "done";
    return !TERMINAL_STATUSES.has(status);
  });
  const pending = rows.filter((item) => APPROVAL_STATUSES.has(statusOf(item)));
  return html`<section class="workspace-panel delegations-workspace">
    <div class="workspace-panel-head">
      <div>
        <div class="workbench-eyebrow">delegation control</div>
        <h2>Делегации и проектные агенты</h2>
        <p>
          Единое место для аппрувов, запущенных подагентов, failed-route и
          свежих результатов. Чат остается разговором, а очередь работы
          управляется здесь.
        </p>
      </div>
      <div class="workspace-actions">
        <span>${rows.length} visible</span>
        <span>${counts.needs} need you</span>
        <button onClick=${() => loadDelegations()}>refresh</button>
        <button
          disabled=${!pending.length}
          onClick=${async () => {
            if (!confirm(`Approve ${pending.length} pending delegation(s)?`))
              return;
            for (const item of pending) {
              await approveDel(item.id);
            }
            await loadDelegations();
          }}
        >
          approve pending
        </button>
      </div>
    </div>
    <div class="delegation-summary-grid">
      <button
        class=${filter === "needs" ? "active" : ""}
        onClick=${() => setFilter("needs")}
      >
        <b>${counts.needs}</b><span>нужен ты</span>
      </button>
      <button
        class=${filter === "running" ? "active" : ""}
        onClick=${() => setFilter("running")}
      >
        <b>${counts.running}</b><span>работают</span>
      </button>
      <button
        class=${filter === "pending" ? "active" : ""}
        onClick=${() => setFilter("pending")}
      >
        <b>${counts.pending}</b><span>аппрувы</span>
      </button>
      <button
        class=${filter === "failed" ? "active" : ""}
        onClick=${() => setFilter("failed")}
      >
        <b>${counts.failed}</b><span>упали</span>
      </button>
      <button
        class=${filter === "done" ? "active" : ""}
        onClick=${() => setFilter("done")}
      >
        <b>${counts.done}</b><span>готово</span>
      </button>
      <button
        class=${filter === "all" ? "active" : ""}
        onClick=${() => setFilter("all")}
      >
        <b>${rows.length}</b><span>все</span>
      </button>
    </div>
    ${visible.length
      ? html`<div class="delegation-work-grid">
          ${visible.map((item) => html`<${DelegationCard} item=${item} />`)}
        </div>`
      : html`<div class="workspace-empty">
          <b>В этом фильтре делегаций нет</b>
          <span>
            Если агент говорит, что отправил работу, а здесь пусто, значит он
            не выдал исполняемый PA-тег или route не создал delegation.
          </span>
          <button onClick=${() => setFilter("all")}>показать все</button>
        </div>`}
  </section>`;
}
