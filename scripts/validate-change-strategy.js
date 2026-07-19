#!/usr/bin/env node
"use strict";

const fs = require("fs");
const { validateChangeStrategy } = require("./lib/change-strategy-policy.js");

function main() {
  const file = process.argv[2];
  if (!file) throw new Error("Usage: node scripts/validate-change-strategy.js <decision.json>");
  const decision = JSON.parse(fs.readFileSync(file, "utf8"));
  const result = validateChangeStrategy(decision);
  console.log(JSON.stringify(result, null, 2));
  if (!result.isValid || result.blocked) process.exit(1);
}

main();
