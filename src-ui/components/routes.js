import { html, useState } from "/vendor/preact-bundle.mjs";
import {
  activeDualSession,
  composerDraftText,
  currentProject,
  showGraph,
  showPlans,
  showSettings,
  showStrategy,
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

function routeNeedsDecision(route) {
  const progress = route?.progress || {};
  return (
    route?.route_state === "needs_user" ||
    route?.route_state === "blocked" ||
    progress.needs_user ||
    route?.has_blockers
  );
}

function routeDelegationId(route) {
  const progress = route?.progress || {};
  return (
    progress.active_delegation_id ||
    route?.active_delegation_id ||
    (route?.blocker_delegation_ids || [])[0] ||
    (progress.blocker_delegation_ids || [])[0] ||
    ""
  );
}

async function executeRouteCommand(text, label = "command") {
  if (!text?.trim()) return;
  if (!__IS_TAURI || !__invoke) {
    composerDraftText.value = text;
    showToast("Команда положена в чат", "info", 1600);
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
    composerDraftText.value = text;
    showToast(`Не удалось выполнить: ${e}`, "error", 2600);
  }
}

async function approveRouteDelegation(id, action) {
  if (!id) return;
  if (action === "reject") {
    await rejectDel(id);
  } else {
    await approveDel(id);
  }
  await Promise.allSettled([
    loadDelegations(),
    loadExecutionMap("", activeDualSession.value || null, 180),
    loadOrchestrationMap("", activeDualSession.value || null),
  ]);
}

function openProjectAgentChat(project) {
  const name = String(project || "").trim();
  if (!name || name === "project" || name === "_orchestrator") return;
  currentProject.value = name;
  showSettings.value = false;
  showStrategy.value = false;
  showPlans.value = false;
  showGraph.value = false;
  showToast(`chat: ${name}`, "success", 1200);
}

function routeCommands() {
  return {
    status: (id, fallback) => `[DELEGATE_STATUS:${id || fallback || "?failed"}]`,
    cleanup: `[DELEGATE_CLEANUP:1]\n[DELEGATE_STATUS:?failed]\n[DASHBOARD_FULL]`,
    retry: (id) =>
      `[DELEGATE_RETRY:${id}]Repeat through the currently available provider. If Claude is unavailable, use Codex. Check diff/health first, then return the factual result without unrelated changes.[/DELEGATE_RETRY]`,
    health: (project) => `[HEALTH_CHECK:${project || "all"}]`,
  };
}

function RouteCard({ route, compact = false }) {
  const id = routeDelegationId(route);
  const counts = route.counts || {};
  const commands = routeCommands();
  const title = route.title || route.progress?.label || "Route blocked";
  if (compact) {
    return html`<article class="route-decision-chip">
      <b>${route.project || "project"}</b>
      <span>${route.route_state || route.progress?.phase || "blocked"}</span>
      <em>${counts.blocked || 0} blockers</em>
      <button
        onClick=${() =>
          executeRouteCommand(commands.status(id, route.project), "status")}
      >
        status
      </button>
      <button
        disabled=${!id}
        onClick=${() => executeRouteCommand(commands.retry(id), "retry")}
      >
        retry
      </button>
      <button onClick=${() => openProjectAgentChat(route.project)}>chat</button>
    </article>`;
  }
  return html`<article class="route-decision-card">
    <div class="route-decision-title">
      <b>${route.project || "project"}</b>
      <span>${route.route_state || route.progress?.phase || "blocked"}</span>
    </div>
    <p>${title}</p>
    <div class="route-decision-meta">
      ${id ? html`<code>${id}</code>` : html`<code>no delegation id</code>`}
      <span>${counts.blocked || 0} blockers</span>
      <span>${route.executor_provider || "agent"}</span>
    </div>
    <div class="route-decision-actions">
      <button
        onClick=${() =>
          executeRouteCommand(commands.status(id, route.project), "status")}
      >
        status
      </button>
      <button
        disabled=${!id}
        onClick=${() => executeRouteCommand(commands.retry(id), "retry")}
      >
        retry
      </button>
      <button onClick=${() => executeRouteCommand(commands.cleanup, "cleanup")}>
        archive terminal
      </button>
      <button
        onClick=${() =>
          executeRouteCommand(commands.health(route.project), "health")}
      >
        health
      </button>
      <button onClick=${() => openProjectAgentChat(route.project)}>chat</button>
    </div>
  </article>`;
}

function WaitingCard({ item, compact = false }) {
  const commands = routeCommands();
  const canApprove =
    item.action === "approve" ||
    item.status === "pending" ||
    item.status === "needs_permission";
  if (compact) {
    return html`<article class="route-decision-chip">
      <b>${item.project || "project"}</b>
      <span>${item.status || item.action || "waiting"}</span>
      <em>${item.action || "review"}</em>
      ${canApprove
        ? html`<button
              class="primary"
              onClick=${() => approveRouteDelegation(item.id, "approve")}
            >
              approve
            </button>
            <button onClick=${() => approveRouteDelegation(item.id, "reject")}>
              reject
            </button>`
        : html`<button
            onClick=${() =>
              executeRouteCommand(commands.status(item.id), "status")}
          >
            status
          </button>`}
      <button onClick=${() => openProjectAgentChat(item.project)}>chat</button>
    </article>`;
  }
  return html`<article class="route-decision-card">
    <div class="route-decision-title">
      <b>${item.project || "project"}</b>
      <span>${item.status || item.action || "waiting"}</span>
    </div>
    <p>${item.task || "Delegation waits for a decision"}</p>
    <div class="route-decision-meta">
      <code>${item.id}</code>
      <span>${item.action || "review"}</span>
    </div>
    <div class="route-decision-actions">
      ${canApprove
        ? html`<button
              class="primary"
              onClick=${() => approveRouteDelegation(item.id, "approve")}
            >
              approve
            </button>
            <button onClick=${() => approveRouteDelegation(item.id, "reject")}>
              reject
            </button>`
        : null}
      <button
        onClick=${() => executeRouteCommand(commands.status(item.id), "status")}
      >
        status
      </button>
      <button
        disabled=${!item.id || canApprove}
        onClick=${() => executeRouteCommand(commands.retry(item.id), "retry")}
      >
        retry
      </button>
      <button onClick=${() => executeRouteCommand(commands.cleanup, "cleanup")}>
        archive terminal
      </button>
      <button onClick=${() => openProjectAgentChat(item.project)}>chat</button>
    </div>
  </article>`;
}

export function RouteDecisionPanelCompact({ map, execution }) {
  const [showDetails, setShowDetails] = useState(false);
  const allRoutes = (map?.project_agent_routes || []).filter(routeNeedsDecision);
  const allWaiting = execution?.waiting_for_user || [];
  const routeLimit = showDetails ? 5 : 3;
  const routes = allRoutes.slice(0, routeLimit);
  const waiting = allWaiting.slice(0, routeLimit);
  const totalWaiting = allRoutes.length + allWaiting.length;
  if (!totalWaiting) return null;

  const routeProgress = map?.route_progress || {};
  const headline =
    routeProgress.headline ||
    (allRoutes.length
      ? `${allRoutes.length} route need decision`
      : `${allWaiting.length} delegation decisions`);
  const refresh = () =>
    Promise.allSettled([
      loadExecutionMap("", activeDualSession.value || null, 180),
      loadOrchestrationMap("", activeDualSession.value || null),
    ]);

  return html`<section
    class=${`route-decision-panel ${showDetails ? "" : "compact"}`}
  >
    <div class="route-decision-head">
      <div>
        <span>needs your decision</span>
        <b>${headline}</b>
      </div>
      <div class="route-decision-head-actions">
        <button onClick=${() => setShowDetails((value) => !value)}>
          ${showDetails ? "collapse" : "details"}
        </button>
        <button onClick=${refresh}>refresh</button>
      </div>
    </div>
    ${showDetails
      ? html`<div class="route-decision-grid">
          ${routes.map((route) => html`<${RouteCard} route=${route} />`)}
          ${waiting.map((item) => html`<${WaitingCard} item=${item} />`)}
        </div>`
      : html`<div class="route-decision-strip">
          ${routes.map(
            (route) => html`<${RouteCard} route=${route} compact=${true} />`,
          )}
          ${waiting.map(
            (item) => html`<${WaitingCard} item=${item} compact=${true} />`,
          )}
          ${totalWaiting > routes.length + waiting.length
            ? html`<article class="route-decision-chip muted">
                +${totalWaiting - routes.length - waiting.length} more
                <button onClick=${() => setShowDetails(true)}>details</button>
              </article>`
            : null}
        </div>`}
  </section>`;
}
