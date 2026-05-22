import { html, useEffect, useState } from "/vendor/preact-bundle.mjs";
import { notificationsData, showToast } from "/store.js";
import { clearNotifications, loadNotifications } from "/api.js";

function contextLabels(item) {
  return [
    item.project ? `project:${item.project}` : "",
    item.route_id ? `route:${item.route_id}` : "",
    item.delegation_id ? `delegation:${item.delegation_id}` : "",
    item.run_id ? `run:${item.run_id}` : "",
  ].filter(Boolean);
}

function NotificationGroup({ title, items, tone }) {
  return html`<section class=${`notification-group ${tone || ""}`}>
    <div class="notification-group-head">
      <b>${title}</b>
      <span>${items.length}</span>
    </div>
    ${items.length
      ? items.slice(-40).map(
          (item) =>
            html`<article class="notification-row">
              <div class="notification-row-top">
                <b>${item.title || item.kind || "AgentOS"}</b>
                <span>${item.severity || "info"}</span>
                <em>${item.ts || ""}</em>
              </div>
              <p>${item.message || ""}</p>
              <div class="notification-row-meta">
                ${item.command ? html`<code>${item.command}</code>` : null}
                ${contextLabels(item).map((label) => html`<span>${label}</span>`)}
                <span>${item.source || "system"} / ${item.kind || "event"}</span>
              </div>
            </article>`,
        )
      : html`<div class="notification-empty">Пусто</div>`}
  </section>`;
}

export function NotificationsWorkspace() {
  const [severityFilter, setSeverityFilter] = useState("all");
  const [sourceFilter, setSourceFilter] = useState("all");
  const [projectFilter, setProjectFilter] = useState("all");
  useEffect(() => {
    loadNotifications();
  }, []);

  const data = notificationsData.value || { items: [], counts: {}, count: 0 };
  const items = data.items || [];
  const counts = data.counts || {};
  const sourceOptions = [
    ...new Set(items.map((item) => item.source || "system").filter(Boolean)),
  ].sort();
  const projectOptions = [
    ...new Set(items.map((item) => item.project || "").filter(Boolean)),
  ].sort();
  const visibleItems = items.filter((item) => {
    const severity = item.severity || "info";
    const source = item.source || "system";
    const project = item.project || "";
    return (
      (severityFilter === "all" || severity === severityFilter) &&
      (sourceFilter === "all" || source === sourceFilter) &&
      (projectFilter === "all" || project === projectFilter)
    );
  });
  const bySeverity = (severity) =>
    visibleItems.filter((item) => (item.severity || "info") === severity);

  return html`<section class="workspace-panel notifications-workspace">
    <div class="workspace-panel-head">
      <div>
        <div class="workbench-eyebrow">notification center</div>
        <h2>Журнал системы</h2>
        <p>
          Сюда вынесены PA/status/warning сообщения. Чат остается разговором,
          карта исполнения показывает только смысловые события.
        </p>
      </div>
      <div class="workspace-actions">
        <span>${visibleItems.length}/${data.count || items.length} событий</span>
        <span>${counts.warning || 0} warning</span>
        <button onClick=${() => loadNotifications()}>refresh</button>
        <button
          disabled=${!items.length}
          onClick=${() =>
            clearNotifications().catch((e) =>
              showToast("Clear notifications failed: " + e, "error", 3000),
            )}
        >
          clear
        </button>
      </div>
    </div>
    <div class="notification-filters">
      <label>
        severity
        <select
          value=${severityFilter}
          onChange=${(e) => setSeverityFilter(e.target.value)}
        >
          <option value="all">all</option>
          <option value="warning">warning</option>
          <option value="success">success</option>
          <option value="info">info</option>
        </select>
      </label>
      <label>
        source
        <select
          value=${sourceFilter}
          onChange=${(e) => setSourceFilter(e.target.value)}
        >
          <option value="all">all</option>
          ${sourceOptions.map(
            (source) => html`<option value=${source}>${source}</option>`,
          )}
        </select>
      </label>
      <label>
        project
        <select
          value=${projectFilter}
          onChange=${(e) => setProjectFilter(e.target.value)}
        >
          <option value="all">all</option>
          ${projectOptions.map(
            (project) => html`<option value=${project}>${project}</option>`,
          )}
        </select>
      </label>
      <button
        onClick=${() => {
          setSeverityFilter("all");
          setSourceFilter("all");
          setProjectFilter("all");
        }}
      >
        reset filters
      </button>
    </div>
    <div class="notification-grid">
      <${NotificationGroup}
        title="Требуют внимания"
        items=${bySeverity("warning")}
        tone="warning"
      />
      <${NotificationGroup}
        title="Результаты команд"
        items=${bySeverity("success")}
        tone="success"
      />
      <${NotificationGroup} title="Инфо" items=${bySeverity("info")} />
    </div>
  </section>`;
}
