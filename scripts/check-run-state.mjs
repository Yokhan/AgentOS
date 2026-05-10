import {
  isProviderStateSampleEvent,
  runStuckHint,
} from "../src-ui/run-state.js";

const heartbeat = {
  type: "run_heartbeat",
  status: "running",
  phase: "provider",
  detail:
    "Codex subprocess pid=21876 is still running; waiting for provider output (1450s).",
};

if (!isProviderStateSampleEvent(heartbeat)) {
  throw new Error("provider heartbeat must be treated as a state sample");
}

const semanticEvent = {
  type: "tool_use",
  status: "started",
  phase: "tool",
  detail: "Read",
};

if (isProviderStateSampleEvent(semanticEvent)) {
  throw new Error("semantic tool event must not be treated as provider state");
}

const now = Date.now();
const hint = runStuckHint(
  {
    status: "running",
    phase: "provider",
    startedAt: now - 10 * 60 * 1000,
    updatedAt: now,
    heartbeatAt: now - 1000,
    lastSemanticAt: now - 9 * 60 * 1000,
  },
  now,
);

if (!hint || !String(hint.title || "").includes("Модель молчит")) {
  throw new Error("long provider wait must produce a persistent stuck hint");
}

console.log("run state checks ok");
