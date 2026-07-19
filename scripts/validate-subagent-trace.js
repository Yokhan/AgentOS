#!/usr/bin/env node
"use strict";

const fs = require("fs");
const { parseJsonLines, validateSubagentTrace } = require("./lib/subagent-trace.js");

function parseArgs(argv) {
  const options = { file: "", expectedRole: "", expectedModel: "" };
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index];
    const value = argv[index + 1];
    if (key === "--file") options.file = value;
    else if (key === "--expected-role") options.expectedRole = value;
    else if (key === "--expected-model") options.expectedModel = value;
    else throw new Error(`unknown argument: ${key}`);
    index += 1;
  }
  if (!options.file) throw new Error("--file is required");
  return options;
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const events = parseJsonLines(fs.readFileSync(options.file, "utf8"));
  const result = validateSubagentTrace(events, options);
  console.log(JSON.stringify(result, null, 2));
  if (!result.isValid) process.exit(1);
}

main();
