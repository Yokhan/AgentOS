// Tauri bridge — auto-installs fetch interceptor on import
// ===== TAURI BRIDGE =====
// Maps fetch('/api/...') calls to Tauri invoke() commands.
// In browser mode (no Tauri), fetch works normally against Python server.
const __IS_TAURI = !!window.__TAURI_INTERNALS__;
const __invoke = __IS_TAURI ? window.__TAURI_INTERNALS__.invoke : null;
const __listen = __IS_TAURI ? window.__TAURI_INTERNALS__.event?.listen : null;
// In browser mode, find HTTP API server (Tauri app runs it on 3333-3335)
let __API_BASE = "";
if (!__IS_TAURI) {
  (async () => {
    for (const p of [3333, 3334, 3335]) {
      try {
        const r = await fetch("http://localhost:" + p + "/api/health", {
          signal: AbortSignal.timeout(1000),
        });
        if (r.ok) {
          __API_BASE = "http://localhost:" + p;
          console.log("API:", __API_BASE);
          break;
        }
      } catch {}
    }
  })();
}

// Route map: URL pattern → { command, extract(url, opts) → args }
const __API_ROUTES = __IS_TAURI
  ? {
      "GET /api/health": { cmd: "get_health" },
      "GET /api/agents": { cmd: "get_agents" },
      "POST /api/agents": { cmd: "get_agents" },
      "GET /api/feed": { cmd: "get_feed" },
      "POST /api/feed": { cmd: "get_feed" },
      "GET /api/activity": { cmd: "get_activity" },
      "GET /api/segments": { cmd: "get_segments" },
      "GET /api/plan": { cmd: "get_plan" },
      "GET /api/digest": { cmd: "get_digest" },
      "GET /api/permissions": { cmd: "get_permissions" },
      "GET /api/chats": { cmd: "get_chats" },
      "GET /api/analytics": { cmd: "get_analytics" },
      "GET /api/health-history": { cmd: "get_health_history" },
      "GET /api/queue": { cmd: "get_queue" },
      "GET /api/inbox": { cmd: "get_inbox" },
      "POST /api/inbox/process": { cmd: "process_inbox" },
      "POST /api/inbox/clear": { cmd: "clear_inbox" },
      "GET /api/goals": { cmd: "get_goals" },
      "GET /api/strategies": { cmd: "get_strategies" },
      "GET /api/config": { cmd: "get_config" },
    }
  : null;

// Fake Response wrapper for invoke results
function __fakeResponse(data) {
  const json = JSON.stringify(data);
  return {
    ok: true,
    status: 200,
    json: () => Promise.resolve(data),
    text: () => Promise.resolve(json),
  };
}

const _origFetch = window.fetch;
window.fetch = function (url, opts = {}) {
  if (!__IS_TAURI) {
    // Browser mode: redirect /api/* to HTTP API server
    if (typeof url === "string" && url.startsWith("/api") && __API_BASE) {
      return _origFetch.call(this, __API_BASE + url, opts);
    }
    return _origFetch.call(this, url, opts);
  }
  if (typeof url !== "string" || !url.startsWith("/api")) {
    return _origFetch.call(this, url, opts);
  }

  const method = (opts.method || "GET").toUpperCase();
  let body = {};
  if (opts.body) {
    try {
      body = JSON.parse(opts.body);
    } catch {}
  }

  // Exact route match
  const key = `${method} ${url.split("?")[0]}`;
  if (__API_ROUTES[key]) {
    return __invoke(__API_ROUTES[key].cmd, body)
      .then(__fakeResponse)
      .catch((e) => __fakeResponse({ error: String(e) }));
  }

  // Parameterized routes
  const path = url.split("?")[0];
  if (path.startsWith("/api/chat/") && method === "GET") {
    const project = decodeURIComponent(path.split("/api/chat/")[1]);
    const params = new URLSearchParams(url.split("?")[1] || "");
    const beforeRaw = params.get("before");
    const limitRaw = params.get("limit");
    const before =
      beforeRaw && /^\d+$/.test(beforeRaw) ? Number(beforeRaw) : null;
    const limit = limitRaw && /^\d+$/.test(limitRaw) ? Number(limitRaw) : null;
    return __invoke("get_chat_history", { project, before, limit }).then(
      __fakeResponse,
    );
  }
  if (path.startsWith("/api/modules/")) {
    const project = decodeURIComponent(path.split("/api/modules/")[1]);
    return __invoke("get_modules", { project }).then(__fakeResponse);
  }
  if (path.startsWith("/api/project-plan/")) {
    const project = decodeURIComponent(path.split("/api/project-plan/")[1]);
    return __invoke("get_project_plan", { project }).then(__fakeResponse);
  }
  if (path.startsWith("/api/impact/")) {
    const project = decodeURIComponent(path.split("/api/impact/")[1]);
    return __invoke("get_impact", { project }).then(__fakeResponse);
  }
  if (path.startsWith("/api/health-history/")) {
    const project = decodeURIComponent(path.split("/api/health-history/")[1]);
    return __invoke("get_health_history", { project }).then(__fakeResponse);
  }
  if (path.startsWith("/api/approve/") && method === "POST") {
    const id = decodeURIComponent(path.split("/api/approve/")[1]);
    return __invoke("approve_delegation", { id }).then(__fakeResponse);
  }
  if (path.startsWith("/api/reject/") && method === "POST") {
    const id = decodeURIComponent(path.split("/api/reject/")[1]);
    return __invoke("reject_delegation", { id }).then(__fakeResponse);
  }
  if (path.startsWith("/api/action/")) {
    const name = path.split("/api/action/")[1];
    return __invoke("run_action", { name }).then(__fakeResponse);
  }

  // POST routes with body
  if (path === "/api/chat" && method === "POST") {
    return __invoke("send_chat", body).then(__fakeResponse);
  }
  if (path === "/api/permissions" && method === "POST") {
    return __invoke("set_permission", body).then(__fakeResponse);
  }
  if (path === "/api/delegations" && method === "POST") {
    return __invoke("get_delegations", {}).then(__fakeResponse);
  }

  // Chat: use send_chat directly (blocking but reliable)
  if (path === "/api/chat-stream" && method === "POST") {
    return Promise.resolve({
      ok: true,
      status: 200,
      body: { getReader: () => ({ read: () => new Promise(() => {}) }) },
      _tauriStream: true,
    });
  }

  // Webhook proxy
  if (path.startsWith("/webhook/")) {
    return __invoke("proxy_webhook", {
      path,
      method,
      body: opts.body || null,
    }).then((r) => __fakeResponse(r.data || r));
  }

  // Fallback to real fetch
  return _origFetch.call(this, url, opts);
};

export { __IS_TAURI, __invoke };
