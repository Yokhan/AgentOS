const SAFE_MODE_KEY = "agentos_safe_mode";
const DIAGNOSTIC_KEY = "agentos_ui_diagnostics";
const MAX_LOCAL_DIAGNOSTICS = 120;
const LONG_TASK_THRESHOLD_MS = 220;
const EVENT_LOOP_LAG_THRESHOLD_MS = 1800;
const REMOTE_THROTTLE_MS = 5000;

let installed = false;
const lastRemoteByType = new Map();

function nowIso() {
  return new Date().toISOString();
}

function readLocalDiagnostics() {
  try {
    const data = JSON.parse(localStorage.getItem(DIAGNOSTIC_KEY) || "[]");
    return Array.isArray(data) ? data : [];
  } catch {
    return [];
  }
}

function writeLocalDiagnostics(items) {
  try {
    localStorage.setItem(
      DIAGNOSTIC_KEY,
      JSON.stringify(items.slice(-MAX_LOCAL_DIAGNOSTICS)),
    );
  } catch {}
}

function shouldSendRemote(event) {
  const type = event.type || "unknown";
  const now = Date.now();
  const last = Number(lastRemoteByType.get(type) || 0);
  if (now - last < REMOTE_THROTTLE_MS) return false;
  lastRemoteByType.set(type, now);
  return true;
}

function normalizeError(error) {
  if (!error) return "";
  return error.stack || error.message || String(error);
}

function buildContext() {
  return {
    href: location.href,
    project: localStorage.getItem("agentos_current_project") || "",
    workspace_tab: localStorage.getItem("agentos_workspace_tab") || "",
    room_tab: localStorage.getItem("agentos_active_room_tab") || "",
    safe_mode: localStorage.getItem(SAFE_MODE_KEY) === "1",
    visibility: document.visibilityState || "",
  };
}

function recordUiDiagnosticLocal(event, recordRemote = null) {
  const normalized = {
    ts: event.ts || nowIso(),
    type: event.type || "unknown",
    severity: event.severity || "warn",
    ...event,
    context: { ...buildContext(), ...(event.context || {}) },
  };
  writeLocalDiagnostics([...readLocalDiagnostics(), normalized]);
  if (recordRemote && shouldSendRemote(normalized)) {
    Promise.resolve(recordRemote(normalized)).catch((err) => {
      console.warn("ui diagnostic remote write failed:", err);
    });
  }
  return normalized;
}

function installLongTaskObserver(record) {
  if (!("PerformanceObserver" in window)) return;
  try {
    const observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        if (entry.duration < LONG_TASK_THRESHOLD_MS) continue;
        record({
          type: "long_task",
          severity: entry.duration > 800 ? "error" : "warn",
          duration_ms: Math.round(entry.duration),
          start_ms: Math.round(entry.startTime),
          name: entry.name || "longtask",
        });
      }
    });
    observer.observe({ entryTypes: ["longtask"] });
  } catch (err) {
    record({
      type: "diagnostics_install_failed",
      severity: "warn",
      source: "longtask",
      error: normalizeError(err),
    });
  }
}

function installEventLoopWatchdog(record) {
  let expected = performance.now() + 1000;
  setInterval(() => {
    const now = performance.now();
    const lag = now - expected;
    expected = now + 1000;
    if (lag < EVENT_LOOP_LAG_THRESHOLD_MS) return;
    record({
      type: "event_loop_lag",
      severity: lag > 6000 ? "error" : "warn",
      lag_ms: Math.round(lag),
    });
  }, 1000);
}

function installErrorCapture(record) {
  window.addEventListener("error", (event) => {
    record({
      type: "window_error",
      severity: "error",
      message: event.message || "",
      source: event.filename || "",
      line: event.lineno || 0,
      col: event.colno || 0,
      error: normalizeError(event.error),
    });
  });
  window.addEventListener("unhandledrejection", (event) => {
    record({
      type: "unhandled_rejection",
      severity: "error",
      error: normalizeError(event.reason),
    });
  });
}

function setSafeModeLocal(enabled, reload = false) {
  localStorage.setItem(SAFE_MODE_KEY, enabled ? "1" : "0");
  window.__AGENTOS_SAFE_MODE__ = !!enabled;
  if (reload) location.reload();
}

function installUiDiagnostics({ recordRemote = null } = {}) {
  if (installed) return;
  installed = true;
  const record = (event) => recordUiDiagnosticLocal(event, recordRemote);
  window.__AGENTOS_UI_DIAGNOSTICS__ = {
    get: readLocalDiagnostics,
    clear: () => writeLocalDiagnostics([]),
    record,
    enableSafeMode: () => setSafeModeLocal(true, true),
    disableSafeMode: () => setSafeModeLocal(false, true),
  };
  window.__AGENTOS_SAFE_MODE__ = localStorage.getItem(SAFE_MODE_KEY) === "1";
  installErrorCapture(record);
  installLongTaskObserver(record);
  installEventLoopWatchdog(record);
  record({ type: "diagnostics_installed", severity: "info" });
}

function isSafeModeLocal() {
  return localStorage.getItem(SAFE_MODE_KEY) === "1";
}

export {
  installUiDiagnostics,
  recordUiDiagnosticLocal,
  readLocalDiagnostics,
  setSafeModeLocal,
  isSafeModeLocal,
};
