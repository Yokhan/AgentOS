#!/usr/bin/env node
"use strict";

const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const { TextDecoder } = require("node:util");

const decoder = new TextDecoder("utf-8", { fatal: true });
const ROOT = process.cwd();
const MAX_SAMPLE = 140;

const binaryExtensions = new Set([
  ".ico",
  ".jpg",
  ".jpeg",
  ".png",
  ".gif",
  ".webp",
  ".pdf",
  ".zip",
  ".gz",
  ".tar",
  ".res",
]);

const mojibakePatterns = [
  { name: "UTF-8 punctuation decoded as Windows-1251", regex: /\u0432[\u0402\u0404\u20AC\u201E\u2020\u2021\u2030\u0409\u040A\u040C\u040F\u2122]/gu },
  {
    name: "UTF-8 Cyrillic decoded as Windows-1251",
    regex: /(?:\u0420[\u0402\u0403\u0406\u0409-\u040C\u040E-\u040F\u0452-\u045F\u0491\u00A0-\u00BF\u201A-\u201E\u2020-\u2022\u2116]|\u0421[\u0402\u0403\u0406\u0409-\u040C\u040E-\u040F\u0452-\u045F\u0491\u00A0-\u00BF\u201A-\u201E\u2020-\u2022\u2116])/gu,
    minMatches: 2,
  },
  { name: "UTF-8 decoded as CP1252/Latin-1", regex: /(?:\u00E2[\u0080-\u00BF]|\u00C2[\u0080-\u00BF]|\u00C3[\u0080-\u00BF])/gu },
  { name: "replacement character", regex: /\uFFFD/gu },
];

const ownedShellPattern = /^(?:scripts\/.*\.sh|\.claude\/hooks\/.*\.sh|setup\.sh)$/u;
const shellPolicyRules = [
  {
    name: "raw uname call outside platform helper",
    regex: /\buname\b/u,
    allowedFiles: new Set(["scripts/lib/platform.sh"]),
    message: "Use _detect_os/_detect_arch from scripts/lib/platform.sh.",
  },
  {
    name: "hardcoded /tmp path",
    regex: /(^|[^A-Za-z0-9_])\/tmp(?:\/|$)/u,
    allowedFiles: new Set(["scripts/lib/platform.sh"]),
    message: "Use _temp_file/_temp_dir from scripts/lib/platform.sh.",
  },
  {
    name: "raw mktemp call outside platform helper",
    regex: /\bmktemp\b/u,
    allowedFiles: new Set(["scripts/lib/platform.sh"]),
    message: "Use _temp_file/_temp_dir from scripts/lib/platform.sh.",
  },
];

function normalizePath(filePath) {
  return filePath.split(path.sep).join("/");
}

function getTrackedFiles() {
  const output = execFileSync("git", ["ls-files", "-z"], {
    cwd: ROOT,
    encoding: "buffer",
  });
  return output
    .toString("utf8")
    .split("\0")
    .filter(Boolean)
    .map(normalizePath)
    // A tracked file can be intentionally deleted in an unstaged template
    // change. Validate the resulting payload instead of treating that deletion
    // as malformed text; explicit --path inputs still report missing files.
    .filter((filePath) => fs.existsSync(path.join(ROOT, filePath)));
}

function getInputFiles() {
  const pathIndex = process.argv.indexOf("--path");
  if (pathIndex >= 0) {
    return process.argv
      .slice(pathIndex + 1)
      .filter((item) => item && !item.startsWith("--"))
      .map(normalizePath);
  }

  return getTrackedFiles();
}

function isLikelyBinary(filePath, buffer) {
  if (binaryExtensions.has(path.extname(filePath).toLowerCase())) {
    return true;
  }

  return buffer.includes(0);
}

function getLineAndColumn(text, index) {
  const before = text.slice(0, index);
  const lines = before.split("\n");
  return {
    line: lines.length,
    column: lines[lines.length - 1].length + 1,
  };
}

function getSample(text, index) {
  const start = Math.max(0, index - Math.floor(MAX_SAMPLE / 2));
  const end = Math.min(text.length, index + Math.floor(MAX_SAMPLE / 2));
  return text.slice(start, end).replace(/\r/g, "\\r").replace(/\n/g, "\\n");
}

function addIssue(issues, filePath, kind, message, detail = {}) {
  issues.push({ filePath, kind, message, ...detail });
}

function checkUtf8Policy(filePath, buffer, issues) {
  if (buffer.length >= 3 && buffer[0] === 0xef && buffer[1] === 0xbb && buffer[2] === 0xbf) {
    addIssue(issues, filePath, "encoding", "UTF-8 BOM is forbidden.");
  }

  try {
    return decoder.decode(buffer);
  } catch {
    addIssue(issues, filePath, "encoding", "File is not valid UTF-8.");
    return null;
  }
}

function checkLineEndings(filePath, text, issues) {
  if (text.includes("\r\n") && /(^|[^\r])\n/u.test(text)) {
    addIssue(issues, filePath, "line-endings", "Mixed CRLF and LF line endings are forbidden.");
  }
}

function checkMojibake(filePath, text, issues) {
  for (const pattern of mojibakePatterns) {
    pattern.regex.lastIndex = 0;
    const matches = [...text.matchAll(pattern.regex)];
    if (matches.length < (pattern.minMatches || 1)) {
      continue;
    }

    const match = matches[0];
    const location = getLineAndColumn(text, match.index);
    addIssue(issues, filePath, "mojibake", pattern.name, {
      ...location,
      sample: getSample(text, match.index),
    });
  }
}

function checkShellPolicy(filePath, text, issues) {
  if (!ownedShellPattern.test(filePath)) {
    return;
  }

  for (const rule of shellPolicyRules) {
    if (rule.allowedFiles.has(filePath)) {
      continue;
    }

    const match = rule.regex.exec(text);
    if (!match || match.index === undefined) {
      continue;
    }

    addIssue(issues, filePath, "platform", `${rule.name}. ${rule.message}`, {
      ...getLineAndColumn(text, match.index),
      sample: getSample(text, match.index),
    });
  }
}

function checkPlatformHelper(text, issues) {
  const requiredHelpers = ["_detect_os", "_detect_arch", "_is_windows", "_temp_file", "_temp_dir"];
  for (const helper of requiredHelpers) {
    if (!new RegExp(`^${helper}\\(\\)`, "m").test(text)) {
      addIssue(issues, "scripts/lib/platform.sh", "platform", `Missing required platform helper: ${helper}`);
    }
  }
}

function formatIssue(issue) {
  const location = issue.line ? `:${issue.line}:${issue.column}` : "";
  const sample = issue.sample ? `\n    sample: ${issue.sample}` : "";
  return `  ERROR [${issue.kind}] ${issue.filePath}${location} - ${issue.message}${sample}`;
}

function main() {
  const issues = [];
  const files = getInputFiles();

  for (const filePath of files) {
    if (!fs.existsSync(filePath)) {
      addIssue(issues, filePath, "missing", "File does not exist.");
      continue;
    }

    const buffer = fs.readFileSync(filePath);
    if (isLikelyBinary(filePath, buffer)) {
      continue;
    }

    const text = checkUtf8Policy(filePath, buffer, issues);
    if (text === null) {
      continue;
    }

    checkLineEndings(filePath, text, issues);
    checkMojibake(filePath, text, issues);
    checkShellPolicy(filePath, text, issues);

    if (filePath === "scripts/lib/platform.sh") {
      checkPlatformHelper(text, issues);
    }
  }

  if (issues.length > 0) {
    console.error("Text policy validation failed:");
    for (const issue of issues) {
      console.error(formatIssue(issue));
    }
    process.exit(1);
  }

  console.log(`Text policy checks passed (${files.length} files scanned)`);
}

main();
