"use strict";

const fs = require("fs");
const path = require("path");

const STATE_PATH = path.join("tasks", ".active-codex-route.json");

function parseArgs(argv) {
  const values = { taskParts: [] };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--summary" || arg === "--text") values.isSummary = true;
    else if (arg === "--write-state") values.shouldWriteState = true;
    else if (arg === "--discovery-file") {
      values.discoveryPath = getFlagValue(argv, index, arg);
      index += 1;
    } else if (arg === "--decision-file") {
      values.decisionPath = getFlagValue(argv, index, arg);
      index += 1;
    }
    else if (arg.startsWith("--")) throw new Error(`Unknown option: ${arg}`);
    else if (!arg.startsWith("--")) values.taskParts.push(arg);
  }
  return { ...values, task: values.taskParts.join(" ").trim() };
}

function getFlagValue(argv, index, flag) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${flag} requires a file path`);
  }
  return value;
}

function readJson(inputPath) {
  if (!inputPath) return undefined;
  return JSON.parse(fs.readFileSync(path.resolve(inputPath), "utf8"));
}

function writeState(route, statePath = STATE_PATH) {
  fs.mkdirSync(path.dirname(statePath), { recursive: true });
  fs.writeFileSync(statePath, `${JSON.stringify(route, null, 2)}\n`);
}

function runRouteCli(argv, dependencies) {
  const args = parseArgs(argv);
  if (!args.task) {
    throw new Error('Usage: node scripts/codex-route-task.js "<task>" [--discovery-file <json>] [--decision-file <json>] [--summary] [--write-state]');
  }
  const discovery = readJson(args.discoveryPath);
  const changeStrategyDecision = readJson(args.decisionPath);
  const route = dependencies.getRoute(args.task, { discovery, changeStrategyDecision });
  if (args.shouldWriteState) writeState(route);
  return args.isSummary ? dependencies.formatSummary(route) : JSON.stringify(route, null, 2);
}

module.exports = { parseArgs, runRouteCli, writeState };
