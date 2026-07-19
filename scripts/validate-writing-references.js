#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { validateProjectWritingRegistry, validateWritingReferenceRegistry } = require("./lib/writing-reference-policy.js");

function getOption(args, name, fallback) {
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] : fallback;
}

function main(args) {
  const root = process.cwd();
  const registryPath = getOption(args, "--registry", ".claude/library/technical/writing-reference-registry.json");
  const today = getOption(args, "--today", new Date().toISOString().slice(0, 10));
  const registry = JSON.parse(fs.readFileSync(path.join(root, registryPath), "utf8"));
  const projectPath = getOption(args, "--project-registry", "brain/03-knowledge/writing/reference-registry.json");
  const projectRegistry = fs.existsSync(path.join(root, projectPath))
    ? JSON.parse(fs.readFileSync(path.join(root, projectPath), "utf8"))
    : null;
  const errors = projectRegistry
    ? validateProjectWritingRegistry(projectRegistry, registry, { root, today })
    : validateWritingReferenceRegistry(registry, { root, today });
  if (errors.length) {
    console.error(errors.map((error) => `- ${error}`).join("\n"));
    process.exit(1);
  }
  console.log(`Writing reference registry passed (${registry.sources.length} template sources, ${registry.profiles.length} template profiles, project overlay ${projectRegistry ? "loaded" : "absent"})`);
}

main(process.argv.slice(2));
