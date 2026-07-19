#!/usr/bin/env node
"use strict";

const fs = require("fs");
const { validateProgressivePlan } = require("./lib/progressive-plan.js");

function main() {
  const file = process.argv[2];
  if (!file) throw new Error("Usage: node scripts/validate-progressive-plan.js <plan.json>");
  const plan = JSON.parse(fs.readFileSync(file, "utf8"));
  const result = validateProgressivePlan(plan);
  console.log(JSON.stringify(result, null, 2));
  if (!result.isValid) process.exit(1);
}

main();
