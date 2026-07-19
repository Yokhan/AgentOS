#!/usr/bin/env node
"use strict";

const fs = require("fs");
const crypto = require("crypto");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
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

const COMMANDS = new Set(["validate", "plan", "route", "check", "install", "benchmark-plan"]);
const BENCHMARK_QUERIES = [
  "Summarize architecture, entry points, and major boundaries.",
  "Locate the owner and references for one representative symbol.",
  "List inbound and outbound callers for one representative function.",
  "Inventory application routes and their handlers.",
  "Estimate the blast radius of one uncommitted representative change.",
];

function parseArgs(argv) {
  const options = { command: "plan", profile: "full", root: process.cwd(), task: "", only: [], json: false, dryRun: false };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (COMMANDS.has(arg)) options.command = arg;
    else if (arg === "--profile") options.profile = argv[++index];
    else if (arg === "--root") options.root = path.resolve(argv[++index]);
    else if (arg === "--task") options.task = argv[++index];
    else if (arg === "--only") options.only.push(...argv[++index].split(",").filter(Boolean));
    else if (arg === "--json") options.json = true;
    else if (arg === "--dry-run") options.dryRun = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }
  return options;
}

function getTrackedFiles(rootDir) {
  const result = spawnSync("git", ["-C", rootDir, "ls-files"], { encoding: "utf8" });
  if (result.status !== 0) return fs.readdirSync(rootDir, { withFileTypes: true }).map((entry) => entry.name);
  return result.stdout.split(/\r?\n/).filter(Boolean);
}

function normalizeVersion(value, preferred = "") {
  const matches = [...String(value || "").matchAll(/\bv?(\d+\.\d+\.\d+(?:[-+][\w.-]+)?)\b/gi)]
    .map((match) => match[1]);
  if (preferred && matches.includes(preferred)) return preferred;
  return matches.at(-1) || "unknown";
}

function compareVersions(left, right) {
  const a = left.split(".").map(Number);
  const b = right.split(".").map(Number);
  for (let index = 0; index < Math.max(a.length, b.length); index += 1) {
    if ((a[index] || 0) !== (b[index] || 0)) return (a[index] || 0) - (b[index] || 0);
  }
  return 0;
}

function resolveCommand(command) {
  const localName = process.platform === "win32" && !command.endsWith(".exe") ? `${command}.exe` : command;
  const localPath = path.join(process.env.CODE_INTELLIGENCE_BIN_DIR || path.join(os.homedir(), ".local", "bin"), localName);
  if (fs.existsSync(localPath)) return localPath;
  const finder = process.platform === "win32"
    ? spawnSync("where.exe", [command], { encoding: "utf8", timeout: 3000 })
    : spawnSync("sh", ["-c", 'command -v -- "$1"', "sh", command], { encoding: "utf8", timeout: 3000 });
  if (finder.status !== 0) {
    return null;
  }
  const candidates = finder.stdout.split(/\r?\n/).filter(Boolean);
  if (process.platform === "win32") {
    return candidates.find((candidate) => /\.(?:exe|cmd|bat)$/i.test(candidate)) || candidates[0] || null;
  }
  return candidates[0] || null;
}

function spawnCommand(command, args, options) {
  const isWindowsBatch = process.platform === "win32" && /\.(?:cmd|bat)$/i.test(command);
  return isWindowsBatch
    ? spawnSync("cmd.exe", ["/d", "/s", "/c", command, ...args], options)
    : spawnSync(command, args, options);
}

function getLocalNpmHealth(tool, rootDir) {
  const packagePath = path.join(rootDir, tool.install.path, "package.json");
  const distPath = path.join(rootDir, tool.install.path, "dist", "index.js");
  if (!fs.existsSync(packagePath)) return { status: "missing", version: "unknown", detail: packagePath };
  const version = JSON.parse(fs.readFileSync(packagePath, "utf8")).version;
  const status = fs.existsSync(distPath) && version === tool.version ? "ok" : "drift";
  return { status, version, detail: fs.existsSync(distPath) ? "built" : "dist missing" };
}

function getCommandHealth(tool, rootDir) {
  const executable = resolveCommand(tool.command);
  if (!executable) return { status: "missing", version: "unknown", detail: "command not found" };
  const result = spawnCommand(executable, tool.health_args, {
    cwd: rootDir,
    encoding: "utf8",
    timeout: tool.health_timeout_ms || 5000,
  });
  if (result.error || result.status !== 0) return { status: "missing", version: "unknown", detail: result.error?.message || result.stderr.trim() };
  const expectedVersion = tool.health_version || tool.version;
  const version = normalizeVersion(`${result.stdout}\n${result.stderr}`, expectedVersion);
  const minimum = tool.install.minimum_version;
  const isCompatibleSystem = tool.install.kind === "system" && minimum && compareVersions(version, minimum) >= 0;
  const status = version === expectedVersion ? "ok" : isCompatibleSystem ? "compatible" : "drift";
  return { status, version, detail: status === "compatible" ? `host-managed; catalog target ${tool.version}` : "command responded" };
}

function getHealth(tool, rootDir) {
  return tool.install.kind === "local-npm"
    ? getLocalNpmHealth(tool, rootDir)
    : getCommandHealth(tool, rootDir);
}

function getPlan(catalog, options) {
  const stacks = detectStacks(getTrackedFiles(options.root));
  const selectedByProfile = selectTools(catalog, options.profile, stacks);
  const unknown = (options.only || []).filter((id) => !catalog.tools.some((tool) => tool.id === id));
  if (unknown.length > 0) throw new Error(`Unknown tool ids: ${unknown.join(", ")}`);
  const selected = (options.only || []).length > 0
    ? selectedByProfile.filter((tool) => options.only.includes(tool.id))
    : selectedByProfile;
  const activeMcp = selectActiveMcpTools(catalog, stacks).map((tool) => tool.id);
  return {
    root: options.root,
    profile: options.profile,
    stacks,
    selected: selected.map((tool) => ({ id: tool.id, version: tool.version, role: tool.role, mcpActive: activeMcp.includes(tool.id) })),
    activeMcp,
    note: "Installed capability does not imply an always-active MCP surface.",
  };
}

function getWorkflowReport(catalog, options) {
  const stacks = detectStacks(getTrackedFiles(options.root));
  const workflow = getToolWorkflow(options.task, stacks);
  const known = new Set(catalog.tools.map((tool) => tool.id));
  const unknown = workflow.tools.filter((tool) => !known.has(tool));
  if (unknown.length > 0) throw new Error(`Workflow ${workflow.id} references unknown tools: ${unknown.join(", ")}`);
  return { root: options.root, task: options.task, stacks, ...workflow };
}

function getHealthReport(catalog, options) {
  const plan = getPlan(catalog, options);
  const selected = plan.selected.map(({ id }) => catalog.tools.find((tool) => tool.id === id));
  return { ...plan, tools: selected.map((tool) => ({ id: tool.id, expected: tool.health_version || tool.version, ...getHealth(tool, options.root) })) };
}

function runInstallCommand(command, rootDir, dryRun) {
  if (dryRun) return { status: "planned", command: [command.command, ...command.args].join(" ") };
  const cwd = command.cwd ? path.join(rootDir, command.cwd) : rootDir;
  const result = spawnCommand(command.command, command.args, { cwd, encoding: "utf8", stdio: "inherit" });
  const detail = result.error?.message || `exit ${result.status}`;
  return { status: result.status === 0 ? "installed" : "failed", command: [command.command, ...command.args].join(" "), detail };
}

function getGithubReleaseAsset(tool, platform = process.platform, architecture = process.arch) {
  const osName = { win32: "windows", linux: "linux", darwin: "darwin" }[platform];
  const archName = { x64: "x64", arm64: "arm64" }[architecture];
  if (!osName || !archName) throw new Error(`Unsupported release platform: ${platform}/${architecture}`);
  const ext = platform === "win32" ? "zip" : "tar.gz";
  const target = {
    win32: { x64: "x86_64-pc-windows-msvc", arm64: "aarch64-pc-windows-msvc" },
    linux: { x64: "x86_64-unknown-linux-gnu", arm64: "aarch64-unknown-linux-gnu" },
    darwin: { x64: "x86_64-apple-darwin", arm64: "aarch64-apple-darwin" },
  }[platform][architecture];
  const releaseTarget = tool.install.targets?.[`${osName}-${archName}`] || target;
  const replace = (value) => value
    .replaceAll("{version}", tool.version)
    .replaceAll("{os}", osName)
    .replaceAll("{arch}", archName)
    .replaceAll("{target}", releaseTarget)
    .replaceAll("{ext}", ext);
  const base = `https://github.com/${tool.install.repository}/releases/download/${tool.install.tag}`;
  const asset = replace(tool.install.asset);
  const checksums = replace(tool.install.checksums_asset);
  return { asset, checksums, assetUrl: `${base}/${asset}`, checksumsUrl: `${base}/${checksums}` };
}

async function fetchBuffer(url, attempts = 3) {
  let lastError;
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      const response = await fetch(url, {
        headers: { "user-agent": "agent-project-template" },
        signal: AbortSignal.timeout(60000),
      });
      if (!response.ok) {
        const error = new Error(`download failed ${response.status}: ${url}`);
        if (response.status < 500 && response.status !== 429) throw error;
        lastError = error;
      } else {
        return Buffer.from(await response.arrayBuffer());
      }
    } catch (error) {
      lastError = error;
      if (/download failed 4\d\d/.test(error.message) && !/429/.test(error.message)) throw error;
    }
    if (attempt < attempts) await new Promise((resolve) => setTimeout(resolve, attempt * 500));
  }
  throw lastError || new Error(`download failed: ${url}`);
}

function findFile(root, name) {
  const entries = fs.readdirSync(root, { withFileTypes: true });
  for (const entry of entries) {
    const candidate = path.join(root, entry.name);
    if (entry.isFile() && entry.name === name) return candidate;
    if (entry.isDirectory()) {
      const nested = findFile(candidate, name);
      if (nested) return nested;
    }
  }
  return null;
}

async function installGithubReleaseBinary(tool, dryRun) {
  const release = getGithubReleaseAsset(tool);
  if (dryRun) return { id: tool.id, status: "planned", command: `verified download ${release.assetUrl}` };
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "code-intelligence-"));
  try {
    const archivePath = path.join(tempDir, release.asset);
    const extractDir = path.join(tempDir, "extract");
    fs.mkdirSync(extractDir);
    const [archive, checksums] = await Promise.all([fetchBuffer(release.assetUrl), fetchBuffer(release.checksumsUrl)]);
    const checksumText = checksums.toString("utf8");
    const checksumLines = checksumText.split(/\r?\n/).filter(Boolean);
    const expectedLine = checksumLines.find((line) => line.trim().endsWith(release.asset));
    const expected = (expectedLine?.match(/[a-f0-9]{64}/i) || checksumText.match(/[a-f0-9]{64}/i))?.[0].toLowerCase();
    if (!/^[a-f0-9]{64}$/.test(expected || "")) throw new Error(`checksum missing for ${release.asset}`);
    const actual = crypto.createHash("sha256").update(archive).digest("hex");
    if (actual !== expected) throw new Error(`checksum mismatch for ${release.asset}`);
    fs.writeFileSync(archivePath, archive);
    const psQuote = (value) => `'${String(value).replaceAll("'", "''")}'`;
    const extraction = process.platform === "win32"
      ? spawnSync("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", `Expand-Archive -LiteralPath ${psQuote(archivePath)} -DestinationPath ${psQuote(extractDir)} -Force`], { encoding: "utf8" })
      : spawnSync("tar", ["-xzf", archivePath, "-C", extractDir], { encoding: "utf8" });
    if (extraction.status !== 0) throw new Error(extraction.stderr || `archive extraction exit ${extraction.status}`);
    const binaryName = process.platform === "win32" ? `${tool.install.binary}.exe` : tool.install.binary;
    const source = findFile(extractDir, binaryName);
    if (!source) throw new Error(`${binaryName} missing from ${release.asset}`);
    const binDir = process.env.CODE_INTELLIGENCE_BIN_DIR || path.join(os.homedir(), ".local", "bin");
    fs.mkdirSync(binDir, { recursive: true });
    const target = path.join(binDir, binaryName);
    fs.copyFileSync(source, target);
    if (process.platform !== "win32") fs.chmodSync(target, 0o755);
    return { id: tool.id, status: "installed", command: `verified download ${release.assetUrl}`, detail: target };
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
}

async function ensureUvRuntime(catalog, dryRun) {
  const present = resolveCommand("uv");
  if (present) return { id: "uv-runtime", status: "present", detail: present };
  const version = catalog.policy.runtime_dependencies.uv.version;
  const pseudoTool = {
    id: "uv-runtime",
    version,
    install: {
      kind: "github-release-binary",
      repository: "astral-sh/uv",
      tag: version,
      asset: "uv-{target}.{ext}",
      checksums_asset: "uv-{target}.{ext}.sha256",
      binary: "uv",
    },
  };
  return installGithubReleaseBinary(pseudoTool, dryRun);
}

async function installTool(tool, rootDir, dryRun) {
  if (!dryRun) {
    const before = getHealth(tool, rootDir);
    if (["ok", "compatible"].includes(before.status)) return { id: tool.id, status: "present", version: before.version };
  }
  if (tool.install.kind === "github-release-binary") return installGithubReleaseBinary(tool, dryRun);
  if (dryRun) {
    const command = getInstallCommand(tool, process.platform);
    return command
      ? { id: tool.id, status: "planned", command: [command.command, ...command.args].join(" ") }
      : { id: tool.id, status: "external", detail: `managed by ${tool.install.kind}` };
  }
  const command = getInstallCommand(tool, process.platform);
  if (!command) return { id: tool.id, status: "external", detail: `managed by ${tool.install.kind}` };
  if (tool.install.kind === "uv-tool") command.command = resolveCommand("uv") || command.command;
  const primary = runInstallCommand(command, rootDir, false);
  if (primary.status === "failed" || tool.install.kind !== "local-npm") return { id: tool.id, ...primary };
  const build = runInstallCommand({ command: process.platform === "win32" ? "npm.cmd" : "npm", args: ["run", "build"], cwd: tool.install.path }, rootDir, false);
  return { id: tool.id, status: build.status, command: `${primary.command} && ${build.command}` };
}

async function getInstallReport(catalog, options) {
  const plan = getPlan(catalog, options);
  const selected = plan.selected.map(({ id }) => catalog.tools.find((tool) => tool.id === id));
  const runtimes = selected.some((tool) => tool.install.kind === "uv-tool")
    ? [await ensureUvRuntime(catalog, options.dryRun)]
    : [];
  const tools = [];
  for (const tool of selected) tools.push(await installTool(tool, options.root, options.dryRun));
  return { ...plan, dryRun: options.dryRun, runtimes, tools };
}

function getBenchmarkPlan(catalog, options) {
  return {
    repositories: ["typescript-javascript", "python", "go-or-rust", "csharp"],
    variants: ["baseline-rg-read", "graph", "graph-plus-lsp"],
    queries: BENCHMARK_QUERIES,
    metrics: ["answer-recall", "input-output-tokens", "tool-calls", "wall-time", "index-time", "peak-memory"],
    thresholds: getBenchmarkGate(catalog),
    rule: "Promote a tool only when the same gold answers and environment are used for every variant.",
    root: options.root,
  };
}

function printHuman(report) {
  if (report.stacks) console.log(`Stacks: ${report.stacks.join(", ") || "docs-or-unknown"}`);
  if (report.profile) console.log(`Profile: ${report.profile}`);
  if (report.activeMcp) console.log(`Active MCP: ${report.activeMcp.join(", ") || "none"}`);
  if (report.id && report.tools && report.reason) {
    console.log(`Workflow: ${report.id}`);
    console.log(`Tools: ${report.tools.join(" -> ")}`);
    console.log(`Why: ${report.reason}`);
    for (const guard of report.guards || []) console.log(`Guard: ${guard}`);
    return;
  }
  for (const tool of report.tools || report.selected || []) {
    const state = tool.status ? `${tool.status}${tool.version ? ` (${tool.version})` : ""}` : `selected ${tool.version}`;
    console.log(`${tool.id}: ${state}`);
  }
  if (report.thresholds) console.log(JSON.stringify(report, null, 2));
}

function getReport(catalog, options) {
  if (options.command === "validate") return { status: "ok", tools: catalog.tools.length };
  if (options.command === "plan") return getPlan(catalog, options);
  if (options.command === "route") return getWorkflowReport(catalog, options);
  if (options.command === "check") return getHealthReport(catalog, options);
  if (options.command === "install") return getInstallReport(catalog, options);
  return getBenchmarkPlan(catalog, options);
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const catalog = readCatalog(options.root);
  const issues = validateCatalog(catalog);
  if (issues.length > 0) throw new Error(issues.join("\n"));
  const report = await getReport(catalog, options);
  if (options.json || options.command === "validate") console.log(JSON.stringify(report, null, 2));
  else printHuman(report);
  if (options.command === "check" && report.tools.some((tool) => !["ok", "compatible"].includes(tool.status))) process.exitCode = 1;
  if (options.command === "install" && report.tools.some((tool) => tool.status === "failed")) process.exitCode = 1;
}

if (require.main === module) {
  try {
    main().catch((error) => {
      console.error(error instanceof Error ? error.message : String(error));
      process.exitCode = 1;
    });
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}

module.exports = { getGithubReleaseAsset, getReport, normalizeVersion, parseArgs };
