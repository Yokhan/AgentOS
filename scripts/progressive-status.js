#!/usr/bin/env node
const fs = require("fs");
const path = require("path");
const crypto = require("crypto");
const childProcess = require("child_process");

const ROOT = process.cwd();
const CACHE_FILE = path.join(ROOT, ".session-cache", "progressive-status.json");
const HEADER_RE = /<!--\s*PROGRESSIVE_STATUS\s*([\s\S]*?)-->/m;
const STATUS_VALUES = new Set(["planned", "active", "partial", "blocked", "done", "stale"]);
const NUMERIC_FIELDS = ["readiness", "plan", "inventory", "production", "cleanup"];
const IGNORED_DIRS = new Set([
  ".git",
  ".session-cache",
  ".cache",
  "node_modules",
  "dist",
  "build",
  "coverage",
  ".pytest_cache",
  ".mypy_cache",
  ".ruff_cache",
]);

function parseArgs(argv) {
  const args = {
    check: false,
    json: false,
    noCache: false,
    paths: [],
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--check") args.check = true;
    else if (arg === "--json") args.json = true;
    else if (arg === "--no-cache") args.noCache = true;
    else if (arg === "--path") {
      index += 1;
      if (!argv[index]) throw new Error("--path requires a value");
      args.paths.push(argv[index]);
    } else if (arg === "--help" || arg === "-h") {
      printHelp();
      process.exit(0);
    } else if (!arg.startsWith("-")) {
      args.paths.push(arg);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return args;
}

function printHelp() {
  console.log(`Usage: node scripts/progressive-status.js [--check] [--json] [--path <file-or-dir>]

Scans PROGRESSIVE_STATUS headers in working markdown docs.

Header format:
<!-- PROGRESSIVE_STATUS
id: template-v4.9.0-progressive-status
status: active
updated: 2026-07-06
readiness: 60
plan: 100
inventory: 80
production: 40
cleanup: 60
tags: progressive-jpeg,template
next: release gate
-->

--check fails when a tracked header document changed but its header did not.`);
}

function runGit(args) {
  try {
    return childProcess.execFileSync("git", args, {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });
  } catch {
    return "";
  }
}

function toPosix(relativePath) {
  return relativePath.split(path.sep).join("/");
}

function isMarkdown(filePath) {
  return /\.(md|md\.template)$/i.test(filePath) || path.basename(filePath) === "SKILL.md";
}

function listFiles(startPath) {
  const absolute = path.resolve(ROOT, startPath);
  if (!fs.existsSync(absolute)) return [];
  const stat = fs.statSync(absolute);
  if (stat.isFile()) return isMarkdown(absolute) ? [absolute] : [];
  if (!stat.isDirectory()) return [];
  const files = [];
  const entries = fs.readdirSync(absolute, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.isDirectory()) {
      if (IGNORED_DIRS.has(entry.name)) continue;
      files.push(...listFiles(path.join(absolute, entry.name)));
    } else {
      const fullPath = path.join(absolute, entry.name);
      if (isMarkdown(fullPath)) files.push(fullPath);
    }
  }
  return files;
}

function candidateFiles(paths) {
  const roots = paths.length > 0
    ? paths
    : [
      "tasks",
      "docs",
      "AGENTS.md",
      "CLAUDE.md",
      "PROJECT_SPEC.md",
      "README.md",
      "SETUP_GUIDE.md",
      ".claude/library",
      ".agents/skills",
      "templates/project-starter/tasks",
    ];
  return Array.from(new Set(roots.flatMap(listFiles))).sort();
}

function hashText(text) {
  return crypto.createHash("sha256").update(text).digest("hex");
}

function parseHeader(content) {
  const match = content.match(HEADER_RE);
  if (!match) return null;
  if (content.slice(0, match.index).trim().length > 0) return null;
  const raw = match[0];
  const fields = {};
  for (const line of match[1].split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const colon = trimmed.indexOf(":");
    if (colon === -1) continue;
    const key = trimmed.slice(0, colon).trim().toLowerCase();
    const value = trimmed.slice(colon + 1).trim();
    fields[key] = value;
  }
  return { raw, fields };
}

function parseDoc(filePath) {
  const content = fs.readFileSync(filePath, "utf8");
  const header = parseHeader(content);
  if (!header) return null;
  const relativePath = toPosix(path.relative(ROOT, filePath));
  const doc = {
    file: relativePath,
    hash: hashText(content),
    headerHash: hashText(header.raw),
    header: header.fields,
  };
  for (const field of NUMERIC_FIELDS) {
    const raw = header.fields[field];
    doc[field] = raw === undefined ? null : Number(raw);
  }
  return doc;
}

function validateDoc(doc) {
  const issues = [];
  for (const field of ["id", "status", "updated", "readiness", "next"]) {
    if (!doc.header[field]) issues.push(`${doc.file}: missing ${field}`);
  }
  if (doc.header.status && !STATUS_VALUES.has(doc.header.status)) {
    issues.push(`${doc.file}: invalid status ${doc.header.status}`);
  }
  for (const field of NUMERIC_FIELDS) {
    if (doc[field] === null) continue;
    if (!Number.isInteger(doc[field]) || doc[field] < 0 || doc[field] > 100) {
      issues.push(`${doc.file}: ${field} must be an integer from 0 to 100`);
    }
  }
  if (doc.header.updated && !/^\d{4}-\d{2}-\d{2}$/.test(doc.header.updated)) {
    issues.push(`${doc.file}: updated must use YYYY-MM-DD`);
  }
  return issues;
}

function getHeadHeader(file) {
  const content = runGit(["show", `HEAD:${file}`]);
  if (!content) return null;
  return parseHeader(content);
}

function isChanged(file) {
  return runGit(["status", "--porcelain", "--", file]).trim().length > 0;
}

function validateChangedHeader(doc) {
  if (!isChanged(doc.file)) return [];
  const headHeader = getHeadHeader(doc.file);
  if (!headHeader) return [];
  if (hashText(headHeader.raw) === doc.headerHash) {
    return [`${doc.file}: content changed but PROGRESSIVE_STATUS header did not change`];
  }
  return [];
}

function loadCache() {
  if (!fs.existsSync(CACHE_FILE)) return null;
  try {
    return JSON.parse(fs.readFileSync(CACHE_FILE, "utf8"));
  } catch {
    return null;
  }
}

function writeCache(docs) {
  fs.mkdirSync(path.dirname(CACHE_FILE), { recursive: true });
  fs.writeFileSync(CACHE_FILE, JSON.stringify({
    generatedAt: new Date().toISOString(),
    docs: docs.map((doc) => ({
      file: doc.file,
      hash: doc.hash,
      headerHash: doc.headerHash,
      status: doc.header.status,
      readiness: doc.readiness,
      updated: doc.header.updated,
    })),
  }, null, 2), "utf8");
}

function bar(value, width = 20) {
  const safeValue = Number.isFinite(value) ? Math.max(0, Math.min(100, value)) : 0;
  const filled = Math.round((safeValue / 100) * width);
  return `[${"#".repeat(filled)}${"-".repeat(width - filled)}]`;
}

function pad(value, width) {
  const stringValue = String(value ?? "");
  return stringValue.length >= width ? stringValue.slice(0, width) : stringValue.padEnd(width, " ");
}

function average(values) {
  const numbers = values.filter((value) => Number.isFinite(value));
  if (numbers.length === 0) return 0;
  return Math.round(numbers.reduce((sum, value) => sum + value, 0) / numbers.length);
}

function printSlice(docs, cache) {
  const changedSinceCache = new Set();
  if (cache && Array.isArray(cache.docs)) {
    const cachedByFile = new Map(cache.docs.map((doc) => [doc.file, doc.hash]));
    for (const doc of docs) {
      if (cachedByFile.has(doc.file) && cachedByFile.get(doc.file) !== doc.hash) {
        changedSinceCache.add(doc.file);
      }
    }
  }

  const readiness = average(docs.map((doc) => doc.readiness));
  const plan = average(docs.map((doc) => doc.plan));
  const inventory = average(docs.map((doc) => doc.inventory));
  const production = average(docs.map((doc) => doc.production));
  const cleanup = average(docs.map((doc) => doc.cleanup));
  const fileWidth = Math.min(42, Math.max(12, ...docs.map((doc) => doc.file.length)));

  console.log("Progressive JPEG Project Slice");
  console.log(`Docs: ${docs.length} | Cache: ${cache ? "hit" : "cold"} | Changed since cache: ${changedSinceCache.size}`);
  console.log("");
  console.log("```text");
  console.log(`${pad("dimension", 12)} ${pad("bar", 22)} pct`);
  console.log(`${pad("readiness", 12)} ${bar(readiness)} ${String(readiness).padStart(3)}%`);
  console.log(`${pad("plan", 12)} ${bar(plan)} ${String(plan).padStart(3)}%`);
  console.log(`${pad("inventory", 12)} ${bar(inventory)} ${String(inventory).padStart(3)}%`);
  console.log(`${pad("production", 12)} ${bar(production)} ${String(production).padStart(3)}%`);
  console.log(`${pad("cleanup", 12)} ${bar(cleanup)} ${String(cleanup).padStart(3)}%`);
  console.log("");
  console.log(`${pad("file", fileWidth)} ${pad("status", 8)} ${pad("ready", 22)} next`);
  for (const doc of docs) {
    const marker = changedSinceCache.has(doc.file) ? "*" : " ";
    console.log(`${marker}${pad(doc.file, fileWidth)} ${pad(doc.header.status, 8)} ${bar(doc.readiness)} ${doc.header.next || ""}`);
  }
  console.log("```");
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const docs = candidateFiles(args.paths)
    .map(parseDoc)
    .filter(Boolean);
  const cache = loadCache();
  const issues = docs.flatMap((doc) => [
    ...validateDoc(doc),
    ...(args.check ? validateChangedHeader(doc) : []),
  ]);

  if (!args.noCache) writeCache(docs);

  if (args.json) {
    console.log(JSON.stringify({ docs, issues }, null, 2));
  } else {
    printSlice(docs, cache);
    if (issues.length > 0) {
      console.error("");
      console.error("Progressive status issues:");
      for (const issue of issues) console.error(`- ${issue}`);
    }
  }

  if (issues.length > 0) process.exit(1);
}

main();
