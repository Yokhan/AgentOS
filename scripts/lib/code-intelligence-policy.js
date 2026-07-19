"use strict";

const fs = require("fs");
const path = require("path");

const CODE_STACKS = new Set([
  "javascript", "typescript", "python", "go", "rust", "csharp", "shell",
]);
const INSTALL_KINDS = new Set([
  "system", "existing-bootstrap", "local-npm", "npm-global", "uv-tool",
  "github-release-binary",
]);
const ACTIVATION_MODES = new Set(["always", "code", "stacks", "on-demand"]);

const STACK_PATTERNS = [
  ["typescript", /(?:^|\/)(?:tsconfig[^/]*\.json|[^/]+\.tsx?)$/i],
  ["javascript", /(?:^|\/)(?:package\.json|[^/]+\.(?:jsx?|mjs|cjs))$/i],
  ["python", /(?:^|\/)(?:pyproject\.toml|requirements[^/]*\.txt|[^/]+\.py)$/i],
  ["go", /(?:^|\/)(?:go\.mod|[^/]+\.go)$/i],
  ["rust", /(?:^|\/)(?:Cargo\.toml|[^/]+\.rs)$/i],
  ["csharp", /(?:^|\/)(?:[^/]+\.(?:sln|csproj|cs))$/i],
  ["shell", /(?:^|\/)[^/]+\.(?:sh|bash)$/i],
];

function readCatalog(rootDir) {
  const catalogPath = path.join(rootDir, "_reference", "code-intelligence-tools.json");
  return JSON.parse(fs.readFileSync(catalogPath, "utf8"));
}

function hasPinnedVersion(tool, prohibited) {
  const value = String(tool?.version || "").toLowerCase();
  return value.length > 0 && !prohibited.includes(value);
}

function validateTool(tool, catalog) {
  const issues = [];
  const prohibited = catalog?.policy?.prohibited_version_labels || [];
  if (!tool?.id || !tool?.name || !tool?.role) issues.push("identity is incomplete");
  if (!hasPinnedVersion(tool, prohibited)) issues.push("version is not pinned");
  if (!tool?.license || !tool?.source) issues.push("license/source is incomplete");
  if (!tool?.command || !Array.isArray(tool?.health_args)) issues.push("health contract is incomplete");
  if (!INSTALL_KINDS.has(tool?.install?.kind)) issues.push("install kind is invalid");
  if (tool?.install?.kind === "github-release-binary" &&
      (!tool.install.repository || !tool.install.tag || !tool.install.asset || !tool.install.checksums_asset || !tool.install.binary)) {
    issues.push("GitHub release install contract is incomplete");
  }
  if (!ACTIVATION_MODES.has(tool?.activation?.mode)) issues.push("activation mode is invalid");
  if (!Array.isArray(tool?.activation?.stacks)) issues.push("activation stacks must be an array");
  return issues.map((issue) => `${tool?.id || "unknown"}: ${issue}`);
}

function validateCatalog(catalog) {
  const issues = [];
  const tools = Array.isArray(catalog?.tools) ? catalog.tools : [];
  if (catalog?.schema_version !== 1) issues.push("schema_version must be 1");
  if (tools.length !== catalog?.policy?.arsenal_size || tools.length !== 10) {
    issues.push("catalog must contain exactly ten tools");
  }
  const ids = tools.map((tool) => tool.id);
  if (new Set(ids).size !== ids.length) issues.push("tool ids must be unique");
  const prohibitedLicenses = catalog?.policy?.prohibited_default_licenses || [];
  const blocked = tools.filter((tool) => prohibitedLicenses.includes(tool.license));
  if (blocked.length > 0) issues.push(`prohibited default licenses: ${blocked.map((tool) => tool.id).join(", ")}`);
  return [...issues, ...tools.flatMap((tool) => validateTool(tool, catalog))];
}

function detectStacks(files) {
  const normalized = files.map((file) => String(file).replaceAll("\\", "/"));
  return STACK_PATTERNS
    .filter(([, pattern]) => normalized.some((file) => pattern.test(file)))
    .map(([stack]) => stack);
}

function hasCodeStack(stacks) {
  return stacks.some((stack) => CODE_STACKS.has(stack));
}

function isAutoSelected(tool, stacks) {
  const mode = tool.activation.mode;
  if (mode === "always") return true;
  if (mode === "code") return hasCodeStack(stacks);
  if (mode === "stacks") return tool.activation.stacks.some((stack) => stacks.includes(stack));
  return false;
}

function selectTools(catalog, profile, stacks) {
  if (profile === "full") return [...catalog.tools];
  if (profile === "core") return catalog.tools.filter((tool) => tool.core);
  if (profile === "auto") return catalog.tools.filter((tool) => isAutoSelected(tool, stacks));
  throw new Error(`Unknown profile: ${profile}`);
}

function selectActiveMcpTools(catalog, stacks) {
  return catalog.tools.filter((tool) => tool.mcp && isAutoSelected(tool, stacks));
}

function executableName(name, platform) {
  if (platform !== "win32") return name;
  if (name === "npm") return "npm.cmd";
  if (name === "uv") return "uv.exe";
  return name;
}

function getInstallCommand(tool, platform) {
  const install = tool.install;
  if (install.kind === "npm-global") {
    return { command: executableName("npm", platform), args: ["install", "--global", `${install.package}@${install.version}`] };
  }
  if (install.kind === "uv-tool") {
    const pythonArgs = install.python ? ["--python", install.python] : [];
    return { command: executableName("uv", platform), args: ["tool", "install", ...pythonArgs, `${install.package}==${install.version}`] };
  }
  if (install.kind === "local-npm") {
    return { command: executableName("npm", platform), args: ["ci"], cwd: install.path };
  }
  return null;
}

function getBenchmarkGate(catalog) {
  return { ...catalog.policy.benchmark_gate };
}

function includesAny(value, patterns) {
  return patterns.some((pattern) => pattern.test(value));
}

function getToolWorkflow(task, stacks = [], modes = []) {
  const value = String(task || "").toLowerCase();
  const isJs = stacks.some((stack) => ["javascript", "typescript"].includes(stack));
  const workflow = (id, tools, reason, guards = []) => ({ id, tools, reason, guards });
  const nonCodeModes = new Set([
    "write", "docs", "marketing", "writing-literary", "writing-informational", "writing-communication",
  ]);
  const hasCodeSignal = includesAny(value, [/\bcode\b|\bapi\b|\bcli\b|source|repository|file|function|class|module|код|исходник|репозитор|файл|функц|класс|модул/]);
  if (modes.length > 0 && modes.every((mode) => nonCodeModes.has(mode)) && !hasCodeSignal) {
    return workflow(
      "no-code-intelligence",
      [],
      "This task does not need repository code intelligence.",
      ["Do not invoke code tools merely because they are installed."],
    );
  }

  if (includesAny(value, [/continue|resume|handoff|decision|context restore|продолж|решени|контекст|хендофф|передач/])) {
    return workflow(
      "restore-and-handoff",
      ["engram", "codebase-memory", "repomix"],
      "Restore durable decisions first, validate current structure from the graph, and create a bounded pack only for handoff.",
      ["Do not treat Engram observations as current source truth.", "Run Gitleaks before sharing a generated pack."],
    );
  }
  if (includesAny(value, [/secret|credential|token leak|security|vulnerab|audit|release|deploy|секрет|ключ|безопас|уязв|релиз|деплой/])) {
    return workflow(
      "security-and-release",
      ["codebase-memory", "semgrep", "gitleaks", "ripgrep"],
      "Use the graph to bound reachable code, Semgrep for structural policy, Gitleaks for secrets, and rg to verify exact evidence.",
      ["Semgrep and Gitleaks are findings, not proof of exploitability.", "Never upload source by default."],
    );
  }
  if (includesAny(value, [/rename|reference|symbol|signature|extract method|refactor|переимен|ссылк|символ|сигнатур|рефактор/])) {
    return workflow(
      "symbol-refactor",
      ["codebase-memory", "serena", "ripgrep"],
      "Map impact once, use LSP only for exact symbol operations, then verify residual text references.",
      ["Start Serena only for this task and stop it afterwards.", "Run native project tests after edits."],
    );
  }
  if (includesAny(value, [/codemod|mass migration|structural search|rewrite pattern|deprecated api|массов|миграц|структурн|устаревш.*api/])) {
    return workflow(
      "structural-migration",
      ["codebase-memory", "ast-grep", "semgrep", "ripgrep"],
      "Use the graph for scope, ast-grep for an auditable rewrite, Semgrep for the postcondition, and rg for absence checks.",
      ["Preview rewrites before applying them.", "Prefer project-native tests over tool-only validation."],
    );
  }
  if (includesAny(value, [/dependency|cycle|boundary|layer violation|import graph|зависим|цикл|границ|сло[йя]|импорт/])) {
    return workflow(
      "dependency-boundaries",
      isJs
        ? ["codebase-memory", "dependency-cruiser", "ripgrep"]
        : ["codebase-memory", "ast-grep", "ripgrep"],
      isJs
        ? "Use the cross-language graph for impact and dependency-cruiser for enforceable JS/TS boundaries."
        : "Use the cross-language graph for impact; dependency-cruiser is intentionally skipped outside JS/TS.",
      ["Do not infer runtime reachability from imports alone."],
    );
  }
  if (includesAny(value, [/unindexed|stale index|bootstrap|unknown repo|first look|без индекс|устаревш.*индекс|перв.*знаком/])) {
    return workflow(
      "zero-index-bootstrap",
      ["probe", "ripgrep", "codebase-memory"],
      "Get bounded AST-backed context without waiting for an index, verify exact hits, then build or refresh the persistent graph only if the task continues.",
      ["Probe is release-candidate software and remains on-demand.", "Do not run a second persistent index."],
    );
  }
  if (includesAny(value, [/exact|literal|config|log|error text|filename|найди|точн|литерал|конфиг|лог|текст ошиб|имя файл/])) {
    return workflow(
      "exact-search",
      ["ripgrep", "probe"],
      "Start with the cheapest exact evidence; add bounded AST context only when text matches are ambiguous.",
      ["Do not build or refresh a graph for a one-off literal lookup."],
    );
  }
  if (includesAny(value, [/architecture|call path|blast radius|impact|entry point|route|understand|debug|архитект|цепочк.*вызов|радиус|влияни|точк.*вход|маршрут|разбер|отлад/])) {
    return workflow(
      "graph-exploration",
      ["codebase-memory", "ripgrep"],
      "Use the persistent graph for topology and change impact, then validate decisive claims against source text.",
      ["Refresh a stale graph before relying on impact results.", "Do not quote vendor token claims as local results."],
    );
  }
  return workflow(
    "default-code-task",
    ["codebase-memory", "ripgrep"],
    "Use the graph for bounded structure and rg for exact source verification.",
    ["If the graph is unavailable, switch to Probe rather than reading the repository wholesale."],
  );
}

module.exports = {
  detectStacks,
  getBenchmarkGate,
  getInstallCommand,
  getToolWorkflow,
  readCatalog,
  selectActiveMcpTools,
  selectTools,
  validateCatalog,
};
