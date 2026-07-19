import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { join } from "path";
import { existsSync } from "fs";
import { routeKeywords, getLibraryMap } from "./router.js";
import { loadRules, getProjectContext } from "./context.js";
import { getState, updateState, restoreState } from "./state.js";
import { runResearch, runVerify, runPlanScaffold } from "./research.js";
import { searchMemory, getEngramStatus } from "./engram.js";
import { runPipeline, listPipelines, pipelineStatus } from "./n8n.js";
import { appendWritingContract } from "./writing-contract.js";
import { buildActiveRulesOutput } from "./active-rules.js";

const server = new McpServer({
  name: "context-router",
  version: "1.7.0",
});

// --- Tool: get_context ---
server.tool(
  "get_context",
  "Route task to relevant rules and return context. Call on EVERY new task. Default depth=brief (cheap). Use depth=full for L/XL tasks.",
  {
    keywords: z
      .string()
      .max(500)
      .describe(
        'Original user task. Preserve its language and explicit output-language request.',
      ),
    depth: z
      .enum(["brief", "normal", "full"])
      .optional()
      .default("brief")
      .describe(
        "brief=route only (~50 tokens), normal=+rules text, full=+lessons+git+registry+ecosystem",
      ),
  },
  async ({ keywords, depth }) => {
    const route = routeKeywords(keywords);
    updateState(route.modes, route.files, keywords);

    const sections: string[] = [];
    sections.push(`MODE: ${route.modes.join("+")}`);
    sections.push(`AGENT: ${route.agent}`);
    sections.push(`RULES: ${route.files.length} files`);
    sections.push(`CODEX_SKILLS: ${route.codexSkills.join(", ") || "none"}`);
    sections.push(
      `CODEX_SUBAGENTS: ${route.codexSubagents.join(", ") || "none"}`,
    );
    sections.push(`PIPELINE: ${route.pipeline}`);
    sections.push(`RISK: ${route.risk}`);
    sections.push(`CODE_INTELLIGENCE: ${route.codeIntelligence.id} | ${route.codeIntelligence.tools.join(" -> ")}`);
    appendWritingContract(sections, route);
    if (route.needsFreshDocs) sections.push("FRESH_DOCS: required");

    // BRIEF: just routing info + file list (for XS/S tasks and subagents)
    if (depth === "brief") {
      sections.push("");
      sections.push("FILES (read only if needed):");
      for (const f of route.files) {
        sections.push(`  .claude/library/${f}`);
      }
      return {
        content: [{ type: "text" as const, text: sections.join("\n") }],
      };
    }

    // NORMAL: + full rules text
    const { text: rulesText, lines: rulesLines } = await loadRules(route.files);
    sections[2] = `RULES: ${route.files.length} files, ${rulesLines} lines`;
    sections.push("");
    sections.push("=== RULES ===");
    sections.push(rulesText);

    if (depth === "normal") {
      return {
        content: [{ type: "text" as const, text: sections.join("\n") }],
      };
    }

    // FULL: + lessons + git + current task + registry + ecosystem + research
    const context = await getProjectContext(keywords);

    if (context.lessons) {
      sections.push("");
      sections.push("=== RELEVANT LESSONS ===");
      sections.push(context.lessons);
    }

    if (context.gitLog) {
      sections.push("");
      sections.push("=== RECENT GIT ===");
      sections.push(context.gitLog);
    }

    if (context.currentTask) {
      sections.push("");
      sections.push("=== CURRENT TASK ===");
      sections.push(context.currentTask);
    }

    if (context.toolRegistry) {
      sections.push("");
      sections.push("=== TOOL REGISTRY (matches) ===");
      sections.push(context.toolRegistry);
    }

    if (context.ecosystem) {
      sections.push("");
      sections.push("=== ECOSYSTEM (cross-project deps) ===");
      sections.push(context.ecosystem);
    }

    if (context.research) {
      sections.push("");
      sections.push("=== RESEARCH CACHE ===");
      sections.push(context.research);
    }

    // Engram memory search (full depth only)
    const memory = await searchMemory(keywords);
    if (memory) {
      sections.push("");
      sections.push(memory);
    }

    return { content: [{ type: "text" as const, text: sections.join("\n") }] };
  },
);

// --- Tool: switch_context ---
server.tool(
  "switch_context",
  "Switch to a different task context mid-conversation. Returns brief by default.",
  {
    keywords: z.string().max(500).describe("New task keywords"),
    depth: z.enum(["brief", "normal", "full"]).optional().default("brief"),
  },
  async ({ keywords, depth }) => {
    const previousState = getState();
    const previousMode = previousState.currentModes.join("+") || "none";

    const route = routeKeywords(keywords);
    const previousRules = new Set(previousState.activeRules);
    const added = route.files.filter((f) => !previousRules.has(f));
    const dropped = previousState.activeRules.filter(
      (f) => !new Set(route.files).has(f),
    );

    updateState(route.modes, route.files, keywords);

    const sections: string[] = [];
    sections.push(`SWITCHED: ${previousMode} → ${route.modes.join("+")}`);
    sections.push(`AGENT: ${route.agent}`);
    sections.push(
      `RULES: ${route.files.length} files (+${added.length} new, -${dropped.length} dropped)`,
    );
    sections.push(`CODEX_SKILLS: ${route.codexSkills.join(", ") || "none"}`);
    sections.push(
      `CODEX_SUBAGENTS: ${route.codexSubagents.join(", ") || "none"}`,
    );
    sections.push(`PIPELINE: ${route.pipeline}`);
    sections.push(`RISK: ${route.risk}`);
    sections.push(`CODE_INTELLIGENCE: ${route.codeIntelligence.id} | ${route.codeIntelligence.tools.join(" -> ")}`);
    appendWritingContract(sections, route);
    if (route.needsFreshDocs) sections.push("FRESH_DOCS: required");

    if (depth === "brief") {
      sections.push("");
      sections.push("NEW FILES:");
      for (const f of added) sections.push(`  + .claude/library/${f}`);
      for (const f of dropped) sections.push(`  - .claude/library/${f}`);
      return {
        content: [{ type: "text" as const, text: sections.join("\n") }],
      };
    }

    // normal/full: include rule text
    const { text: rulesText, lines: rulesLines } = await loadRules(route.files);
    sections[2] = `RULES: ${route.files.length} files, ${rulesLines} lines (+${added.length} new, -${dropped.length} dropped)`;
    sections.push("");
    sections.push("=== RULES ===");
    sections.push(rulesText);

    if (depth === "full") {
      const context = await getProjectContext(keywords);
      if (context.lessons) {
        sections.push("");
        sections.push("=== LESSONS ===");
        sections.push(context.lessons);
      }
      if (context.currentTask) {
        sections.push("");
        sections.push("=== CURRENT TASK ===");
        sections.push(context.currentTask);
      }
    }

    return { content: [{ type: "text" as const, text: sections.join("\n") }] };
  },
);

// --- Tool: get_active_rules ---
server.tool(
  "get_active_rules",
  "Get current active rules state. Use after compaction to restore context.",
  {},
  async () => {
    let state = getState();
    if (!state.lastRouteTime) {
      const restored = await restoreState();
      if (restored) state = restored;
    }

    if (!state.lastRouteTime) {
      return {
        content: [
          {
            type: "text" as const,
            text: 'No active rules. Run get_context(keywords="...") to route a task first.',
          },
        ],
      };
    }

    const engramStatus = await getEngramStatus();
    const text = buildActiveRulesOutput(state, engramStatus);
    return { content: [{ type: "text" as const, text }] };
  },
);

// --- Tool: research ---
server.tool(
  "research",
  "Auto research protocol: reads target files, importers, git log, lessons, registry, ecosystem, cache. Replaces 6+ manual tool calls.",
  {
    target: z
      .string()
      .describe(
        'File or directory path to research, e.g. "src/features/auth/"',
      ),
  },
  async ({ target }) => {
    const result = await runResearch(target);
    return { content: [{ type: "text" as const, text: result }] };
  },
);

// --- Tool: verify ---
server.tool(
  "verify",
  "Auto verification checklist: file sizes, syntax, tests, gates by task size. Replaces manual verification.",
  {
    size: z
      .enum(["XS", "S", "M", "L", "XL"])
      .optional()
      .default("M")
      .describe("Task size for gate selection"),
  },
  async ({ size }) => {
    const result = await runVerify(size);
    return { content: [{ type: "text" as const, text: result }] };
  },
);

// --- Tool: plan_scaffold ---
server.tool(
  "plan_scaffold",
  "Generate plan skeleton: finds affected files, estimates size, creates implementation template.",
  {
    task: z
      .string()
      .describe('Task description, e.g. "add OAuth flow to auth module"'),
  },
  async ({ task }) => {
    const result = await runPlanScaffold(task);
    return { content: [{ type: "text" as const, text: result }] };
  },
);

// --- Tool: run_pipeline (n8n) ---
server.tool(
  "run_pipeline",
  "Trigger n8n workflow pipeline via webhook. Returns execution result. Requires n8n running at N8N_URL.",
  {
    name: z
      .string()
      .describe('Pipeline/webhook name, e.g. "briefing", "health", "scan"'),
    params: z
      .record(z.string())
      .optional()
      .describe("Optional key-value parameters"),
  },
  async ({ name, params }) => {
    const result = await runPipeline(name, params);
    return { content: [{ type: "text" as const, text: result }] };
  },
);

// --- Tool: list_pipelines (n8n) ---
server.tool(
  "list_pipelines",
  "List active n8n workflows. Requires n8n running with API key.",
  {},
  async () => {
    const result = await listPipelines();
    return { content: [{ type: "text" as const, text: result }] };
  },
);

// --- Tool: pipeline_status (n8n) ---
server.tool(
  "pipeline_status",
  "Get execution status of an n8n pipeline run.",
  {
    execution_id: z.string().describe("n8n execution ID"),
  },
  async ({ execution_id }) => {
    const result = await pipelineStatus(execution_id);
    return { content: [{ type: "text" as const, text: result }] };
  },
);

// --- Resource: library-map ---
server.resource(
  "library-map",
  "context://library-map",
  { description: "Map of all available rules with keywords and file paths" },
  async () => ({
    contents: [
      {
        uri: "context://library-map",
        mimeType: "text/markdown",
        text: getLibraryMap(),
      },
    ],
  }),
);

// --- Resource: current-state ---
server.resource(
  "current-state",
  "context://current-state",
  { description: "Current routing state: mode, active rules, last route time" },
  async () => {
    const state = getState();
    const text = JSON.stringify(state, null, 2);
    return {
      contents: [
        { uri: "context://current-state", mimeType: "application/json", text },
      ],
    };
  },
);

// --- Start ---
async function main() {
  const libraryPath = join(process.cwd(), ".claude", "library");
  if (!existsSync(libraryPath)) {
    console.error(`WARNING: ${libraryPath} not found. Rules won't load.`);
  }
  await restoreState();
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch(console.error);
