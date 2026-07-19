"use strict";

function parseJsonLines(text) {
  return String(text)
    .split(/\r?\n/)
    .filter((line) => line.trim().startsWith("{"))
    .map((line, index) => parseLine(line, index));
}

function parseLine(line, index) {
  try {
    return JSON.parse(line);
  } catch (error) {
    throw new Error(`invalid JSONL at line ${index + 1}: ${error.message}`);
  }
}

function getItem(event) {
  return event && typeof event.item === "object" ? event.item : null;
}

function isCollabTool(event, pattern) {
  const item = getItem(event);
  return item?.type === "collab_tool_call" && pattern.test(String(item.tool || ""));
}

function collectReceiverIds(item) {
  const ids = [];
  if (Array.isArray(item?.receiver_thread_ids)) ids.push(...item.receiver_thread_ids);
  if (typeof item?.receiver_thread_id === "string") ids.push(item.receiver_thread_id);
  if (item?.agents_states && typeof item.agents_states === "object") {
    ids.push(...Object.keys(item.agents_states));
  }
  return Array.from(new Set(ids.filter(Boolean)));
}

function stringifyEvent(event) {
  return JSON.stringify(event).toLowerCase();
}

function includesEvidence(event, expected) {
  return !expected || stringifyEvent(event).includes(expected.toLowerCase());
}

function hasChildActivity(events, childId) {
  return events.some((event) => {
    if (event.thread_id === childId) return true;
    const item = getItem(event);
    if (item?.sender_thread_id === childId) return true;
    const state = item?.agents_states?.[childId];
    return state && /completed|done|shutdown/i.test(JSON.stringify(state));
  });
}

function validateSubagentTrace(events, options = {}) {
  const issues = [];
  const parentId = events.find((event) => event.type === "thread.started")?.thread_id;
  if (!parentId) issues.push("missing parent thread.started event");

  const spawnEvents = events.filter((event) => isCollabTool(event, /spawn/iu));
  if (spawnEvents.length === 0) issues.push("missing genuine spawn tool event");
  const childIds = Array.from(new Set(spawnEvents.flatMap((event) =>
    collectReceiverIds(getItem(event)),
  ))).filter((id) => id !== parentId);
  if (childIds.length === 0) issues.push("spawn event has no distinct child thread id");

  const roleEvent = spawnEvents.find((event) => includesEvidence(event, options.expectedRole));
  if (options.expectedRole && !roleEvent) {
    issues.push(`spawn metadata does not identify role ${options.expectedRole}`);
  }
  const modelEvent = spawnEvents.find((event) => includesEvidence(event, options.expectedModel));
  if (options.expectedModel && !modelEvent) {
    issues.push(`spawn metadata does not identify model ${options.expectedModel}`);
  }

  const waitEvents = events.filter((event) => isCollabTool(event, /wait/iu));
  for (const childId of childIds) {
    const matchingWait = waitEvents.some((event) =>
      collectReceiverIds(getItem(event)).includes(childId),
    );
    if (!matchingWait) issues.push(`no wait event references child ${childId}`);
    if (!hasChildActivity(events, childId)) {
      issues.push(`no child activity or completion evidence for ${childId}`);
    }
  }

  return {
    isValid: issues.length === 0,
    issues,
    parentThreadId: parentId || null,
    childThreadIds: childIds,
    expectedRole: options.expectedRole || null,
    expectedModel: options.expectedModel || null,
  };
}

module.exports = { parseJsonLines, validateSubagentTrace };
