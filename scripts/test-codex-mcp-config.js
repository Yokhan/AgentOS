#!/usr/bin/env node
"use strict";

const assert = require("assert");
const { BEGIN, END, mergeConfig, parseArgs } = require("./configure-codex-mcp.js");

const block = `${BEGIN}\n[mcp_servers.engram]\ncommand = "engram"\n${END}\n`;

function main() {
  const added = mergeConfig("[agents]\nmax_depth = 1\n", block);
  assert.strictEqual(added.status, "added");
  assert(added.text.includes("[agents]"));
  assert(added.text.includes("[mcp_servers.engram]"));

  const current = `[agents]\nmax_depth = 1\n\n${BEGIN}\n[mcp_servers.engram]\ncommand = "old"\n${END}\n`;
  const updated = mergeConfig(current, block);
  assert.strictEqual(updated.status, "updated");
  assert(updated.text.includes('command = "engram"'));
  assert(!updated.text.includes('command = "old"'));

  const unchanged = mergeConfig(updated.text, block);
  assert.strictEqual(unchanged.status, "ok");
  assert.strictEqual(unchanged.changed, false);

  assert.throws(
    () => mergeConfig('[mcp_servers.engram]\ncommand = "custom"\n', block),
    /unmanaged \[mcp_servers\.engram\] conflicts/u,
  );
  assert.deepStrictEqual(parseArgs(["--check", "--root", "."]).mode, "check");
  assert(parseArgs(["--reference", "_reference/codex-mcp-config.toml"]).reference.endsWith("codex-mcp-config.toml"));
  console.log("Codex MCP config tests passed");
}

main();
