const report = {
  type: "e2e_report",
  severity: "info",
  startedAt: new Date().toISOString(),
  userAgent: navigator.userAgent,
  viewport: { width: innerWidth, height: innerHeight, dpr: devicePixelRatio },
  startup: null,
  apiLatencyMs: null,
  steps: [],
  errors: [],
  passed: false,
};

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function errorText(error) {
  return error?.stack || error?.message || String(error);
}

window.addEventListener("error", (event) => {
  report.errors.push({ type: "error", message: errorText(event.error || event.message) });
});
window.addEventListener("unhandledrejection", (event) => {
  report.errors.push({ type: "unhandledrejection", message: errorText(event.reason) });
});

async function waitFor(predicate, label, timeoutMs = 12000) {
  const deadline = performance.now() + timeoutMs;
  while (performance.now() < deadline) {
    const value = predicate();
    if (value) return value;
    await sleep(50);
  }
  throw new Error(`Timed out waiting for ${label}`);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function settle() {
  await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
  await sleep(120);
}

async function step(name, action) {
  const started = performance.now();
  try {
    const detail = await action();
    report.steps.push({ name, status: "passed", durationMs: Math.round(performance.now() - started), detail });
  } catch (error) {
    report.steps.push({ name, status: "failed", durationMs: Math.round(performance.now() - started), error: errorText(error) });
  }
}

async function clickView(id, expectedBreadcrumb) {
  const button = document.querySelector(`[data-e2e="${id}"]`);
  assert(button, `Missing ${id} button`);
  button.click();
  await settle();
  assert(button.classList.contains("hdr-active"), `${id} did not become active`);
  assert(document.body.innerText.includes(expectedBreadcrumb), `${expectedBreadcrumb} view is not visible`);
  button.click();
  await settle();
}

async function measureHealthLatency() {
  const invoke = window.__TAURI_INTERNALS__?.invoke;
  assert(invoke, "Tauri invoke bridge is unavailable");
  const samples = [];
  for (let index = 0; index < 12; index += 1) {
    const started = performance.now();
    const health = await invoke("get_health");
    assert(health && typeof health === "object", "get_health returned invalid data");
    samples.push(performance.now() - started);
  }
  samples.sort((a, b) => a - b);
  return {
    samples: samples.map((value) => Math.round(value * 10) / 10),
    p50: Math.round(samples[Math.floor(samples.length * 0.5)] * 10) / 10,
    p95: Math.round(samples[Math.floor(samples.length * 0.95)] * 10) / 10,
    max: Math.round(samples.at(-1) * 10) / 10,
  };
}

async function run() {
  await waitFor(() => document.querySelector(".app"), "application shell");
  await waitFor(() => window.__AGENTOS_READY_AT__, "startup completion");
  report.startup = {
    readyMs: Math.round(window.__AGENTOS_READY_AT__),
    tasks: window.__AGENTOS_STARTUP_RESULTS__ || [],
    deferred: window.__AGENTOS_DEFERRED_STARTUP__ || {},
    warnings: window.__AGENTOS_INIT_WARNINGS__ || [],
    error: window.__AGENTOS_INIT_ERROR__ || null,
  };

  await step("startup", async () => {
    assert(!report.startup.error, `Startup failed: ${report.startup.error}`);
    assert(!document.body.innerText.includes("Agent OS startup error"), "Startup error screen is visible");
    assert(document.querySelector(".hdr"), "Header is missing");
    assert(document.querySelector(".main"), "Main workspace is missing");
    return { readyMs: report.startup.readyMs, warnings: report.startup.warnings.length };
  });

  report.apiLatencyMs = {};
  await step("health latency under startup load", async () => {
    report.apiLatencyMs.startupLoad = await measureHealthLatency();
    assert(report.apiLatencyMs.startupLoad.p95 < 600, `Health p95 under startup load is ${report.apiLatencyMs.startupLoad.p95}ms`);
    return report.apiLatencyMs.startupLoad;
  });

  await waitFor(
    () => Object.values(window.__AGENTOS_DEFERRED_STARTUP__ || {}).every((item) => item.status !== "running"),
    "deferred startup tasks",
    16000,
  );
  await step("health latency steady state", async () => {
    report.apiLatencyMs.steady = await measureHealthLatency();
    assert(report.apiLatencyMs.steady.p95 < 300, `Steady health p95 is ${report.apiLatencyMs.steady.p95}ms`);
    return report.apiLatencyMs.steady;
  });

  await step("verified subagent execution tree", async () => {
    const map = await window.__TAURI_INTERNALS__.invoke("get_execution_map", {
      project: null,
      roomSessionId: null,
      limit: 180,
    });
    const childLane = (map.lanes || []).find(
      (lane) => lane.kind === "agent_run" && lane.runtime_evidence === true,
    );
    assert(childLane, "Verified child run is missing from execution map");
    assert(childLane.model === "gpt-5.6-luna", "Child model is not visible");
    assert(childLane.access === "read-only", "Child sandbox is not visible");
    const [{ h, render }, { ExecutionMapCard }] = await Promise.all([
      import("/vendor/preact-bundle.mjs"),
      import("/chat.js"),
    ]);
    const host = document.createElement("div");
    host.dataset.e2e = "execution-map-fixture";
    document.body.appendChild(host);
    render(h(ExecutionMapCard, { map, variant: "stage" }), host);
    await settle();
    const badge = document.querySelector('[data-e2e="verified-subagent-trace"]');
    const renderedLanes = [...document.querySelectorAll(".exec-map-lane-label")].map(
      (node) => node.innerText,
    );
    assert(
      badge,
      `Verified badge did not render: ${JSON.stringify({
        renderedLanes,
        mapCards: document.querySelectorAll(".exec-map-card").length,
      })}`,
    );
    render(null, host);
    host.remove();
    return { lane: childLane.id, model: childLane.model, access: childLane.access };
  });

  await step("plans navigation", () => clickView("plans", "Plans"));
  await step("strategy navigation", () => clickView("strategy", "Strategy"));
  await step("graph navigation", () => clickView("graph", "Graph"));
  await step("settings navigation", () => clickView("settings", "Settings"));

  await step("safe mode guard", async () => {
    const safe = document.querySelector('[data-e2e="safe-mode"]');
    const graph = document.querySelector('[data-e2e="graph"]');
    assert(safe && graph, "Safe mode controls are missing");
    safe.click();
    await settle();
    assert(document.querySelector('[data-e2e="safe-mode"]').classList.contains("hdr-active"), "Safe mode did not activate");
    graph.click();
    await settle();
    assert(!graph.classList.contains("hdr-active"), "Graph opened while safe mode was active");
    document.querySelector('[data-e2e="safe-mode"]').click();
    await settle();
  });

  await step("new project modal escape", async () => {
    document.querySelector('[data-e2e="new-project"]').click();
    const modal = await waitFor(() => document.querySelector('[data-e2e="new-project-modal"]'), "new project modal");
    const input = modal.querySelector("input");
    assert(input, "New project input is missing");
    input.focus();
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await settle();
    assert(!document.querySelector('[data-e2e="new-project-modal"]'), "Escape did not close the new project modal");
  });

  await step("keyboard help", async () => {
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "?", bubbles: true }));
    await settle();
    assert(document.querySelector(".kb-overlay"), "Keyboard help did not open");
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await settle();
    assert(!document.querySelector(".kb-overlay"), "Keyboard help did not close");
  });

  await step("theme toggle", async () => {
    const button = document.querySelector('[data-e2e="theme"]');
    assert(button, "Theme button is missing");
    const before = document.documentElement.getAttribute("data-theme");
    button.click();
    await settle();
    const after = document.documentElement.getAttribute("data-theme");
    assert(before !== after, "Theme did not change");
    button.click();
    await settle();
  });

  await step("layout overflow", async () => {
    const overflowX = document.documentElement.scrollWidth - document.documentElement.clientWidth;
    assert(overflowX <= 2, `Horizontal overflow is ${overflowX}px`);
    const visibleButtons = [...document.querySelectorAll("button")].filter((button) => {
      const rect = button.getBoundingClientRect();
      return rect.width > 0 && rect.height > 0;
    });
    const undersized = visibleButtons.filter((button) => {
      const rect = button.getBoundingClientRect();
      return rect.width < 20 || rect.height < 20;
    });
    assert(undersized.length === 0, `${undersized.length} visible buttons are smaller than 20px`);
    return { overflowX, visibleButtons: visibleButtons.length };
  });

  report.finishedAt = new Date().toISOString();
  report.durationMs = Math.round(performance.now());
  report.passed = report.steps.every((item) => item.status === "passed") && report.errors.length === 0;
  report.severity = report.passed ? "info" : "error";
  window.__AGENTOS_E2E_REPORT__ = report;
  document.documentElement.dataset.e2eStatus = report.passed ? "passed" : "failed";
  await window.__TAURI_INTERNALS__.invoke("record_ui_diagnostic", { event: report });
}

run().catch(async (error) => {
  report.errors.push({ type: "runner", message: errorText(error) });
  report.finishedAt = new Date().toISOString();
  report.durationMs = Math.round(performance.now());
  report.severity = "error";
  window.__AGENTOS_E2E_REPORT__ = report;
  document.documentElement.dataset.e2eStatus = "failed";
  try {
    await window.__TAURI_INTERNALS__?.invoke("record_ui_diagnostic", { event: report });
  } catch (persistError) {
    console.error("Cannot persist E2E report", persistError);
  }
});
