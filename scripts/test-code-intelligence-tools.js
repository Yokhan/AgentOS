#!/usr/bin/env node
"use strict";

const assert = require("assert");
const path = require("path");
const {
  detectStacks,
  getBenchmarkGate,
  getInstallCommand,
  getToolWorkflow,
  readCatalog,
  selectActiveMcpTools,
  selectTools,
  validateCatalog,
} = require("./lib/code-intelligence-policy.js");
const { getGithubReleaseAsset, getReport, normalizeVersion, parseArgs } = require("./code-intelligence-tools.js");

function main() {
  const rootDir = path.join(__dirname, "..");
  const catalog = readCatalog(rootDir);
  assert.deepStrictEqual(validateCatalog(catalog), []);
  assert.strictEqual(selectTools(catalog, "full", []).length, 10);
  assert.strictEqual(parseArgs([]).profile, "full");
  assert.deepStrictEqual(parseArgs(["install", "--only", "gitleaks,probe"]).only, ["gitleaks", "probe"]);
  assert.strictEqual(selectTools(catalog, "core", []).length, 5);
  assert.strictEqual(selectTools(catalog, "auto", ["markdown"]).length, 2);

  const stacks = detectStacks([
    "apps/web/tsconfig.json",
    "service/pyproject.toml",
    "native/Cargo.toml",
    "game/game.csproj",
  ]);
  assert.deepStrictEqual(stacks, ["typescript", "python", "rust", "csharp"]);

  const webTools = selectTools(catalog, "auto", ["typescript", "javascript"]);
  assert(webTools.some((tool) => tool.id === "dependency-cruiser"));
  assert(!webTools.some((tool) => tool.id === "semgrep"));
  assert(!webTools.some((tool) => tool.id === "probe"));

  const activeMcp = selectActiveMcpTools(catalog, ["typescript"]);
  assert.deepStrictEqual(activeMcp.map((tool) => tool.id), ["engram", "codebase-memory"]);

  assert.deepStrictEqual(
    getToolWorkflow("rename the authentication symbol", ["typescript"]).tools,
    ["codebase-memory", "serena", "ripgrep"],
  );
  assert.deepStrictEqual(
    getToolWorkflow("find an exact error text in logs", ["typescript"]).tools,
    ["ripgrep", "probe"],
  );
  assert.deepStrictEqual(
    getToolWorkflow("check dependency cycles", ["typescript"]).tools,
    ["codebase-memory", "dependency-cruiser", "ripgrep"],
  );
  assert.deepStrictEqual(
    getToolWorkflow("security release audit", ["python"]).tools,
    ["codebase-memory", "semgrep", "gitleaks", "ripgrep"],
  );
  assert.deepStrictEqual(
    getToolWorkflow("write a customer email", [], ["writing-communication"]).tools,
    [],
  );

  const graph = catalog.tools.find((tool) => tool.id === "codebase-memory");
  const install = getInstallCommand(graph, "win32");
  assert.deepStrictEqual(install, {
    command: "npm.cmd",
    args: ["install", "--global", "codebase-memory-mcp@0.9.0"],
  });

  const ripgrep = catalog.tools.find((tool) => tool.id === "ripgrep");
  assert.strictEqual(
    getGithubReleaseAsset(ripgrep, "linux", "x64").asset,
    "ripgrep-15.2.0-x86_64-unknown-linux-musl.tar.gz",
  );
  assert.strictEqual(
    normalizeVersion("cache/probe-0.6.0-rc325.lock\nprobe-code 0.6.0", "0.6.0"),
    "0.6.0",
  );

  assert.strictEqual(getBenchmarkGate(catalog).minimum_median_token_reduction, 0.5);
  assert(!catalog.tools.some((tool) => tool.id === "codegraphcontext"));
  assert(!catalog.tools.some((tool) => tool.id === "gitnexus"));
  assert(!catalog.tools.some((tool) => tool.id === "context-router"));
  assert(!catalog.tools.some((tool) => tool.id === "codesight"));
  assert.deepStrictEqual(getReport(catalog, { command: "validate" }), { status: "ok", tools: 10 });
  console.log("Code intelligence tool tests passed");
}

main();
