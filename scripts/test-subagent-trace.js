#!/usr/bin/env node
"use strict";

const assert = require("assert");
const { validateSubagentTrace } = require("./lib/subagent-trace.js");

const PARENT = "parent-thread";
const CHILD = "child-thread";

function event(item) {
  return { type: "item.completed", item };
}

function validTrace() {
  return [
    { type: "thread.started", thread_id: PARENT },
    event({
      type: "collab_tool_call",
      tool: "spawn_agent",
      sender_thread_id: PARENT,
      receiver_thread_ids: [CHILD],
      agent_type: "scout",
      model: "gpt-5.6-luna",
      agents_states: { [CHILD]: { status: "running" } },
    }),
    event({ type: "agent_message", sender_thread_id: CHILD, text: "bounded result" }),
    event({
      type: "collab_tool_call",
      tool: "wait",
      sender_thread_id: PARENT,
      receiver_thread_ids: [CHILD],
      agents_states: { [CHILD]: { status: "completed" } },
    }),
  ];
}

function assertInvalid(events, expectedIssue, options = {}) {
  const result = validateSubagentTrace(events, options);
  assert.strictEqual(result.isValid, false);
  assert(result.issues.some((issue) => issue.includes(expectedIssue)), result.issues);
}

function main() {
  const options = { expectedRole: "scout", expectedModel: "gpt-5.6-luna" };
  assert.strictEqual(validateSubagentTrace(validTrace(), options).isValid, true);
  assertInvalid([
    { type: "thread.started", thread_id: PARENT },
    event({ type: "collab_tool_call", tool: "wait", receiver_thread_ids: [] }),
    event({ type: "agent_message", text: "SCOUT_READY" }),
  ], "missing genuine spawn", options);
  assertInvalid(validTrace().map((entry) => {
    if (entry.item?.tool !== "spawn_agent") return entry;
    return event({ ...entry.item, model: "gpt-5.6-sol" });
  }), "does not identify model", options);
  assertInvalid(validTrace().filter((entry) => entry.item?.tool !== "wait"), "no wait event", options);
  console.log("Subagent trace tests passed");
}

main();
