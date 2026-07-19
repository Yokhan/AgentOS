#!/usr/bin/env node
const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");

const UI_FILE_RE = /\.(css|scss|sass|less|tsx|jsx|vue|svelte|html|astro)$/i;
const DESIGN_CONTEXT_RE = /(^|\/)DESIGN\.md$/i;
const IGNORE_FILE = "design-policy.ignore";
const FAIL_FIXTURE_RE = /^tests\/fixtures\/design-policy\/fail\//;
const MOJIBAKE_MARKERS = [
  "\uFFFD",
  "\u0432\u0402",
  "\u0432\u20AC",
  "\u00D0",
  "\u00D1",
];
const GRADIENT_RE = /(linear-gradient|radial-gradient|conic-gradient)/i;
const TEXT_CLIP_RE = /(-webkit-)?background-clip\s*:\s*text/i;

const RULES = {
  designContextMissing: {
    id: "design/context-starter-missing",
    severity: "HIGH",
    impact:
      "New projects would start without a durable visual context, so agents can drift from product design decisions.",
    next:
      "Restore templates/project-starter/DESIGN.md or update the template contract intentionally.",
  },
  designContextFrontmatter: {
    id: "design/context-frontmatter",
    severity: "HIGH",
    impact:
      "DESIGN.md becomes harder for agents and DESIGN.md-aware tools to parse consistently.",
    next: "Start DESIGN.md with YAML frontmatter delimited by ---.",
  },
  designContextMojibake: {
    id: "design/context-mojibake",
    severity: "CRITICAL",
    impact:
      "Corrupted text in design context teaches agents broken product language and visual rules.",
    next: "Rewrite the file as UTF-8 without BOM and remove mojibake/replacement characters.",
  },
  gradientText: {
    id: "design/no-gradient-text",
    severity: "HIGH",
    impact:
      "Gradient text is a recurring AI-design failure mode that weakens readability and brand discipline.",
    next:
      "Use a solid token color, weight, size, or layout contrast instead of gradient-clipped text.",
  },
};

function toPosix(filePath) {
  return filePath.split(path.sep).join("/");
}

function parseArgs(argv) {
  const options = {
    hook: false,
    json: false,
    paths: [],
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--hook") {
      options.hook = true;
    } else if (arg === "--json") {
      options.json = true;
    } else if (arg === "--path") {
      const next = argv[index + 1];
      if (next) {
        options.paths.push(next);
        index += 1;
      }
    } else if (!arg.startsWith("--")) {
      options.paths.push(arg);
    }
  }

  return options;
}

function runGit(args) {
  try {
    return execFileSync("git", args, { encoding: "utf8" })
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean);
  } catch {
    return [];
  }
}

function getTrackedFiles() {
  return runGit(["ls-files"]);
}

function getChangedFiles() {
  const changed = runGit(["diff", "--name-only"]);
  const untracked = runGit(["ls-files", "--others", "--exclude-standard"]);
  return [...new Set([...changed, ...untracked])];
}

function parseToolInputPaths() {
  const raw = process.env.TOOL_INPUT || process.env.CODEX_TOOL_INPUT || "";
  if (!raw) {
    return [];
  }

  try {
    const data = JSON.parse(raw);
    return [
      data.file_path,
      data.filePath,
      data.path,
      data.target_file,
      data.targetFile,
    ].filter(Boolean);
  } catch {
    return [];
  }
}

function getHookPaths() {
  const direct = [
    process.env.FILE_PATH,
    process.env.CODEX_FILE_PATH,
    ...parseToolInputPaths(),
  ].filter(Boolean);

  const candidates = direct.length > 0 ? direct : getChangedFiles();
  return candidates.filter((filePath) => isRelevantPath(filePath));
}

function isRelevantPath(filePath) {
  const normalized = toPosix(filePath);
  return DESIGN_CONTEXT_RE.test(normalized) || UI_FILE_RE.test(normalized);
}

function isDefaultIgnoredPath(filePath, explicitPaths) {
  const normalized = toPosix(filePath);
  return !explicitPaths && FAIL_FIXTURE_RE.test(normalized);
}

function getScanPaths(options) {
  if (options.hook) {
    return getHookPaths();
  }

  if (options.paths.length > 0) {
    return options.paths;
  }

  return getTrackedFiles().filter(
    (filePath) =>
      isRelevantPath(filePath) && !isDefaultIgnoredPath(filePath, false),
  );
}

function escapeRegex(value) {
  return value.replace(/[|\\{}()[\]^$+?.]/g, "\\$&");
}

function globToRegex(pattern) {
  const normalized = toPosix(pattern.trim());
  const source = escapeRegex(normalized).replace(/\\\*/g, ".*");
  return new RegExp(`^${source}$`);
}

function readIgnoreRules() {
  if (!fs.existsSync(IGNORE_FILE)) {
    return [];
  }

  return fs
    .readFileSync(IGNORE_FILE, "utf8")
    .split(/\r?\n/)
    .map((line, index) => ({ line: line.trim(), index: index + 1 }))
    .filter(({ line }) => line && !line.startsWith("#"))
    .map(({ line, index }) => {
      const [ruleId, pattern, ...reasonParts] = line.split(/\s+/);
      return {
        ruleId,
        pattern,
        reason: reasonParts.join(" "),
        index,
        matcher: pattern ? globToRegex(pattern) : null,
      };
    });
}

function isIgnored(finding, ignoreRules) {
  return ignoreRules.some(
    (rule) =>
      rule.reason &&
      rule.ruleId === finding.ruleId &&
      rule.matcher &&
      rule.matcher.test(finding.file),
  );
}

function createFinding(rule, file, evidence) {
  return {
    severity: rule.severity,
    ruleId: rule.id,
    file,
    evidence,
    impact: rule.impact,
    next: rule.next,
    tune: `Add "${rule.id} ${file} <reason>" to ${IGNORE_FILE} only for intentional exceptions.`,
  };
}

function readFile(filePath) {
  try {
    return fs.readFileSync(filePath, "utf8");
  } catch {
    return "";
  }
}

function hasMojibake(text) {
  return MOJIBAKE_MARKERS.some((marker) => text.includes(marker));
}

function checkDesignContext(paths) {
  const findings = [];
  const starter = "templates/project-starter/DESIGN.md";
  const scanningAll = paths.length === 0;

  if (fs.existsSync("templates/project-starter") && !fs.existsSync(starter)) {
    findings.push(
      createFinding(
        RULES.designContextMissing,
        starter,
        "templates/project-starter exists but starter DESIGN.md is missing",
      ),
    );
  }

  const designFiles = (scanningAll ? getTrackedFiles() : paths).filter((filePath) =>
    DESIGN_CONTEXT_RE.test(toPosix(filePath)),
  );

  for (const filePath of designFiles) {
    if (!fs.existsSync(filePath)) {
      continue;
    }

    const text = readFile(filePath);
    if (!text.startsWith("---")) {
      findings.push(
        createFinding(
          RULES.designContextFrontmatter,
          toPosix(filePath),
          "file does not start with YAML frontmatter",
        ),
      );
    }
    if (hasMojibake(text)) {
      findings.push(
        createFinding(
          RULES.designContextMojibake,
          toPosix(filePath),
          "mojibake or replacement character detected in design context",
        ),
      );
    }
  }

  return findings;
}

function checkGradientText(paths) {
  const findings = [];

  for (const filePath of paths) {
    if (!UI_FILE_RE.test(filePath) || !fs.existsSync(filePath)) {
      continue;
    }

    const text = readFile(filePath);
    if (GRADIENT_RE.test(text) && TEXT_CLIP_RE.test(text)) {
      findings.push(
        createFinding(
          RULES.gradientText,
          toPosix(filePath),
          "gradient background and background-clip:text appear in the same file",
        ),
      );
    }
  }

  return findings;
}

function formatHuman(findings, hook) {
  if (findings.length === 0) {
    return "Design policy checks passed";
  }

  const header = hook
    ? `Design policy notification: ${findings.length} finding(s)`
    : `Design policy failed: ${findings.length} finding(s)`;
  const lines = [header];

  for (const finding of findings) {
    lines.push("");
    lines.push(`[${finding.severity}] ${finding.ruleId}`);
    lines.push(`File: ${finding.file}`);
    lines.push(`Evidence: ${finding.evidence}`);
    lines.push(`Impact: ${finding.impact}`);
    lines.push(`Next: ${finding.next}`);
    lines.push(`Tune: ${finding.tune}`);
  }

  return lines.join("\n");
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const explicitPaths = options.paths.length > 0;
  const scanPaths = getScanPaths(options)
    .map(toPosix)
    .filter(
      (filePath) =>
        options.hook || !isDefaultIgnoredPath(filePath, explicitPaths),
    );
  const ignoreRules = readIgnoreRules();
  const findings = [
    ...checkDesignContext(explicitPaths || options.hook ? scanPaths : []),
    ...checkGradientText(scanPaths),
  ].filter((finding) => !isIgnored(finding, ignoreRules));

  if (options.json) {
    process.stdout.write(`${JSON.stringify({ ok: findings.length === 0, findings }, null, 2)}\n`);
  } else {
    const output = formatHuman(findings, options.hook);
    const stream = findings.length > 0 ? process.stderr : process.stdout;
    stream.write(`${output}\n`);
  }

  if (!options.hook && findings.length > 0) {
    process.exit(1);
  }
}

main();
