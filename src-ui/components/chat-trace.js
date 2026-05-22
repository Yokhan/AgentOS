// Chat trace rendering: PA command runs, tool cards, thinking blocks and noise filters.
import { html, useState } from "/vendor/preact-bundle.mjs";
import { md } from "/utils.js";
import { showToast } from "/store.js";

const PA_TRACE_COMPACT_ROW_LIMIT = 3;
const PA_COMMAND_PATTERN = /\[[A-Z][A-Z0-9_]*(?::[^\]]*)?\]/g;
const PA_COMMAND_LINE = /^\s*\[[A-Z][A-Z0-9_]*(?::[^\]]*)?\]\s*$/;

const TOOL_ICON = {
  Bash: "$_",
  bash: "$_",
  Read: "[]",
  read_file: "[]",
  Write: "/>",
  write_file: "/>",
  Edit: "/>",
  edit_file: "/>",
  Grep: "?/",
  search: "?/",
  Glob: "*.",
  list_files: "*.",
  Agent: ">>",
  TodoWrite: ">>",
  NotebookEdit: "/>",
};

function toolIcon(name) {
  return TOOL_ICON[name] || ">>";
}

function fmtToolDetail(tool, inp) {
  if (!inp) return "";
  if (tool === "Bash" || tool === "bash") {
    return (inp.command || "").substring(0, 100);
  }
  if (tool === "Read" || tool === "read_file") {
    let s = inp.file_path || "";
    if (inp.offset) s += ` :${inp.offset}`;
    return s.split("/").slice(-2).join("/");
  }
  if (
    tool === "Write" ||
    tool === "write_file" ||
    tool === "Edit" ||
    tool === "edit_file"
  ) {
    return (inp.file_path || "").split("/").slice(-2).join("/");
  }
  if (tool === "Grep" || tool === "search") {
    return (
      "/" +
      (inp.pattern || "").substring(0, 50) +
      "/ " +
      (inp.path || "").split("/").slice(-2).join("/")
    );
  }
  if (tool === "Glob" || tool === "list_files") return inp.pattern || "";
  if (tool === "Agent") {
    return inp.prompt ? inp.prompt.substring(0, 80) + "..." : "";
  }
  const j = JSON.stringify(inp);
  return j.length > 120 ? j.substring(0, 120) + "..." : j;
}

function ToolCard({ t }) {
  const [open, setOpen] = useState(false);
  const icon = toolIcon(t.tool);
  const detail = fmtToolDetail(t.tool, t.input);
  const elapsed = t.elapsed ? Math.round(t.elapsed / 1000) : 0;
  const statusCls =
    t.status === "started" ? "tc-spin" : t.is_error ? "tc-err" : "tc-ok";
  const statusTxt = t.status === "started" ? "..." : t.is_error ? "x" : "ok";
  return html`<div class="tc">
    <div class="tc-hdr" onClick=${() => setOpen(!open)}>
      <span class="tc-icon">${icon}</span>
      <span class="tc-name">${t.tool}</span>
      <span class="tc-sep">-</span>
      <span class="tc-detail">${detail}</span>
      <span class="tc-status ${statusCls}">${statusTxt}</span>
      ${elapsed > 0 ? html`<span class="tc-time">${elapsed}s</span>` : null}
    </div>
    ${open
      ? html`<div class="tc-body">
          <pre class="tc-input">
${t.tool === "Bash" || t.tool === "bash"
              ? t.input?.command || ""
              : JSON.stringify(t.input, null, 2)}</pre
          >
          ${t.result
            ? html`<div class="tc-output ${t.is_error ? "tc-out-err" : ""}">
                ${t.result}
              </div>`
            : null}
        </div>`
      : null}
  </div>`;
}

function ThinkBlock({ b }) {
  const text = b.text || "";
  const visible =
    text.length > 2000
      ? text.substring(0, 2000) + "\n...[thinking continues]"
      : text;
  return html`<details class="think-block" open>
    <summary>
      <span class="think-icon">think</span
      ><span class="think-label">thinking</span>${b.streaming
        ? html`<span style="color:var(--t3)">...</span>`
        : null}
    </summary>
    <div class="think-body">${visible || "waiting for model reasoning..."}</div>
  </details>`;
}

function ProgressBar({ activity, elapsed }) {
  if (!activity) return null;
  return html`<div class="prog-strip">
    <div class="prog-bar" />
    <span class="prog-label">${activity}</span>
    ${elapsed > 2 ? html`<span class="prog-time">${elapsed}s</span>` : null}
  </div>`;
}

function isPaFeedbackBlock(block) {
  return (
    block &&
    (block.type === "pa_result" ||
      block.type === "warning" ||
      block.type === "pa_status")
  );
}

function extractPaCommand(text) {
  const match = String(text || "").match(PA_COMMAND_PATTERN);
  return match ? match[0] : "";
}

function extractPaCommands(text) {
  return [...String(text || "").matchAll(PA_COMMAND_PATTERN)].map((m) => m[0]);
}

function groupChainBlocks(chain) {
  const out = [];
  let paBlocks = [];
  let pendingCommands = [];
  const flushPa = () => {
    if (!paBlocks.length) return;
    out.push({
      type: "pa_trace",
      blocks: paBlocks,
      commands: pendingCommands,
    });
    paBlocks = [];
    pendingCommands = [];
  };
  for (const block of chain || []) {
    if (isPaFeedbackBlock(block)) {
      paBlocks.push(block);
    } else {
      flushPa();
      if (block?.type === "text") {
        pendingCommands = [
          ...pendingCommands,
          ...extractPaCommands(block.text || ""),
        ];
      }
      out.push(block);
    }
  }
  flushPa();
  return out;
}

function stripPaCommandLines(text, shouldStrip) {
  if (!shouldStrip || !text) return text || "";
  const lines = String(text)
    .split(/\r?\n/)
    .filter((line) => !PA_COMMAND_LINE.test(line));
  return lines
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function commandLinesFromText(text) {
  return String(text || "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => PA_COMMAND_LINE.test(line));
}

function isCommandOnlyText(text) {
  const lines = String(text || "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  return lines.length > 0 && lines.every((line) => PA_COMMAND_LINE.test(line));
}

function looksLikeRawDiagnosticDump(text) {
  const lines = String(text || "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  if (lines.length < 4) return false;
  const hits = lines.filter((line) =>
    /===|summary:|warnings?:|errors?:|pending|dirty|clean|no matching|template versions|git status|health check/i.test(
      line,
    ),
  ).length;
  return hits >= 3 && hits / lines.length >= 0.45;
}

function CommandBatchCard({ text }) {
  const commands = commandLinesFromText(text);
  if (!commands.length) return null;
  return html`<details class="command-draft-card">
    <summary>
      <span>command batch</span>
      <b>${commands.length}</b>
      <em>prepared by agent</em>
      <button
        type="button"
        onClick=${(e) => {
          e.preventDefault();
          e.stopPropagation();
          navigator.clipboard.writeText(commands.join("\n"));
          showToast("Commands copied", "success", 1500);
        }}
      >
        copy
      </button>
    </summary>
    <div class="command-draft-list">
      ${commands.map((cmd, i) => html`<code key=${"cmd" + i}>${cmd}</code>`)}
    </div>
  </details>`;
}

function DiagnosticDumpCard({ text }) {
  const lineCount = String(text || "")
    .split(/\r?\n/)
    .filter(Boolean).length;
  return html`<details class="diagnostic-dump-card">
    <summary>
      <span>raw diagnostics hidden</span>
      <b>${lineCount} lines</b>
      <em>covered by run card</em>
      <button
        type="button"
        onClick=${(e) => {
          e.preventDefault();
          e.stopPropagation();
          navigator.clipboard.writeText(text || "");
          showToast("Diagnostics copied", "success", 1500);
        }}
      >
        copy
      </button>
    </summary>
    <pre>${text}</pre>
  </details>`;
}

function previewLine(text, limit = 96) {
  const oneLine = String(text || "")
    .replace(/\s+/g, " ")
    .trim();
  return oneLine.length > limit ? oneLine.slice(0, limit) + "..." : oneLine;
}

function cleanTraceText(text) {
  return String(text || "")
    .replace(/\u0432\u045a\u201c/g, "OK")
    .replace(/\u0432\u045a\u2014/g, "FAIL")
    .replace(/\u0432\u0459\u00a0/g, "WARN")
    .replace(/\u0432\u2020\u2019/g, "->")
    .replace(/\u0432\u2020\u2018/g, " ahead")
    .replace(/\u0432\u2020\u201c/g, " behind");
}

function isNoiseResult(text) {
  return /^(no matching delegations\.|no matching log entries\.|no output\.?)$/i.test(
    String(text || "").trim(),
  );
}

function isRoutineSystemTraceMessage(text) {
  const t = String(text || "")
    .replace(/\s+/g, " ")
    .trim();
  if (!t) return false;
  if (
    /warning|error|failed|permission|needs user|approve|confirm|blocked|not parsed|denied/i.test(
      t,
    )
  ) {
    return false;
  }
  return [
    /^auto-continuing after \d+ agentos actions?/i,
    /^waiting coordinator:/i,
    /^provider (is )?alive/i,
    /^heartbeat:/i,
    /^beat #\d+/i,
    /^codex subprocess .*still running/i,
    /^claude subprocess .*still running/i,
  ].some((pattern) => pattern.test(t));
}

function classifyTraceStatus(text, fallback = "done") {
  const t = String(text || "");
  if (/error|failed|blocked|not parsed|permission denied/i.test(t)) {
    return "warning";
  }
  if (/warnings?:\s*[1-9]|errors?:\s*[1-9]/i.test(t)) {
    return "warning";
  }
  return fallback;
}

function traceHint(text) {
  const t = String(text || "");
  if (/Delegation not parsed/i.test(t)) {
    return "Use [DELEGATE:Project]task[/DELEGATE], or run status/log commands without base delegate syntax.";
  }
  if (/permission denied|write access|review-only/i.test(t)) {
    return "Pick a write-enabled lead or grant provider permissions before executing.";
  }
  if (/warnings?:\s*[1-9]|errors?:\s*[1-9]/i.test(t)) {
    return "Inspect warning projects before continuing the rollout.";
  }
  if (/no matching/i.test(t)) {
    return "No matching state found. This is informational unless you expected pending work.";
  }
  return "";
}

function summarizeTraceOutput(text, status) {
  const clean = cleanTraceText(text || "").trim();
  if (!clean) return status === "queued" ? "not executed" : "";
  if (isNoiseResult(clean)) return "no matches";
  const lines = clean
    .split(/\r?\n/)
    .map((line) =>
      line
        .replace(/^#+\s*/, "")
        .replace(/\*\*/g, "")
        .replace(/`/g, "")
        .trim(),
    )
    .filter(Boolean);
  const firstSignal =
    lines.find((line) =>
      /error|failed|warning|blocked|summary|delegations|git status|template|health/i.test(
        line,
      ),
    ) || lines[0];
  return previewLine(firstSignal || clean, 110);
}

function buildPaTraceRows(blocks, commands = []) {
  const rows = [];
  let commandIndex = 0;
  const nextCommand = () => commands[commandIndex++] || "";
  const consumeCommand = (command) => {
    if (!command) return "";
    const exact = commands.indexOf(command, commandIndex);
    if (exact >= commandIndex) commandIndex = exact + 1;
    return command;
  };
  for (const block of blocks || []) {
    const text = cleanTraceText(block.text || "");
    const explicitCommand = block.command || "";
    if (block.type === "pa_status") {
      const command =
        consumeCommand(explicitCommand || extractPaCommand(text)) ||
        nextCommand();
      const lower = text.toLowerCase();
      rows.push({
        type: "status",
        status: lower.startsWith("running")
          ? "running"
          : lower.startsWith("completed")
            ? "done"
            : "info",
        command,
        label: previewLine(
          text.replace(command, "").replace(/^running\s*/i, ""),
        ),
        output: "",
        summary: "",
      });
      continue;
    }
    if (block.type === "pa_result") {
      const resultCommand = explicitCommand || extractPaCommand(text);
      const last = [...rows]
        .reverse()
        .find(
          (row) =>
            row.type === "status" &&
            !row.output &&
            (!resultCommand || row.command === resultCommand),
        );
      if (last) {
        last.status = classifyTraceStatus(text, "done");
        last.output = text;
        last.summary = summarizeTraceOutput(text, last.status);
        last.hint = traceHint(text);
      } else {
        const command = consumeCommand(resultCommand) || nextCommand();
        const status = classifyTraceStatus(text, "done");
        rows.push({
          type: "result",
          status,
          command,
          label: isNoiseResult(text) ? "" : previewLine(text, 72) || "result",
          output: text,
          summary: summarizeTraceOutput(text, status),
          hint: traceHint(text),
        });
      }
      continue;
    }
    const command =
      consumeCommand(explicitCommand || extractPaCommand(text)) ||
      nextCommand();
    rows.push({
      type: "warning",
      status: "warning",
      command,
      label: "warning",
      output: text,
      summary: summarizeTraceOutput(text, "warning"),
      hint: traceHint(text),
    });
  }
  while (commandIndex < commands.length) {
    rows.push({
      type: "status",
      status: "queued",
      command: nextCommand(),
      label: "not executed",
      output: "",
      summary: "not executed",
    });
  }
  return rows;
}

function copyTraceRows(rows) {
  const text = rows
    .map((row, i) => {
      const head = `${i + 1}. ${row.status.toUpperCase()} ${row.command || "PA command"} - ${row.summary || row.label || ""}`;
      return row.output ? `${head}\n${row.output}` : head;
    })
    .join("\n\n");
  navigator.clipboard.writeText(text);
  showToast("Run copied", "success", 1500);
}

function PaTraceRow({ row, index, forceOpen }) {
  const noisy = isNoiseResult(row.output);
  const hasOutput = !!row.output && !noisy;
  const cls =
    row.status === "warning"
      ? "is-warning"
      : row.status === "running"
        ? "is-running"
        : row.status === "done"
          ? "is-done"
          : row.status === "queued"
            ? "is-queued"
            : "";
  return html`<details
    class="run-row ${cls}"
    key=${"run-row" + index}
    open=${forceOpen || row.status === "running"}
  >
    <summary class="run-row-summary">
      <span class="run-index">${index + 1}</span>
      <span class="run-state">${row.status}</span>
      <span class="run-command">${row.command || "PA command"}</span>
      <span class="run-summary">
        ${noisy ? "no matches" : row.summary || row.label || "completed"}
      </span>
      <span class="run-open">${hasOutput ? "details" : ""}</span>
    </summary>
    ${hasOutput
      ? html`${row.hint
            ? html`<div class="run-hint-body">${row.hint}</div>`
            : null}
          <pre
            class="run-output ${row.status === "warning" ? "is-warning" : ""}"
          >
${row.output}</pre
          >`
      : null}
  </details>`;
}

function PaTrace({ blocks, commands, compact = false }) {
  const [expanded, setExpanded] = useState(false);
  const rows = buildPaTraceRows(blocks, commands);
  const [filter, setFilter] = useState(() =>
    compact &&
    rows.some((row) => row.status === "warning" || row.status === "running")
      ? "problems"
      : "all",
  );
  const [showRows, setShowRows] = useState(!compact);
  if (!rows.length) return null;
  const commandCount = rows.length;
  const warningCount = rows.filter((row) => row.status === "warning").length;
  const doneCount = rows.filter((row) => row.status === "done").length;
  const runningCount = rows.filter((row) => row.status === "running").length;
  const noisyCount = rows.filter((row) => isNoiseResult(row.output)).length;
  const outputCount = rows.filter(
    (row) => row.output && !isNoiseResult(row.output),
  ).length;
  const visibleRows = rows.filter((row) => {
    if (filter === "problems") {
      return row.status === "warning" || row.status === "running";
    }
    if (filter === "outputs") return row.output && !isNoiseResult(row.output);
    return true;
  });
  const compactRows =
    compact && !showRows
      ? visibleRows.slice(0, PA_TRACE_COMPACT_ROW_LIMIT)
      : visibleRows;
  const hiddenRowCount = Math.max(0, visibleRows.length - compactRows.length);
  const headline = runningCount
    ? `${runningCount} running`
    : warningCount
      ? `${warningCount} warning${warningCount === 1 ? "" : "s"}`
      : `${doneCount}/${commandCount} complete`;
  return html`<div class=${`run-card ${compact ? "compact" : ""}`}>
    <div class="run-head">
      <div class="run-head-main">
        <span class="run-kicker">run</span>
        <span class="run-title"
          >${commandCount} PA command${commandCount === 1 ? "" : "s"}</span
        >
        <span class="run-headline">${headline}</span>
      </div>
      <div class="run-badges">
        ${noisyCount ? html`<span>${noisyCount} empty</span>` : null}
        ${warningCount
          ? html`<span class="warn">${warningCount} warn</span>`
          : null}
        ${outputCount ? html`<span>${outputCount} output</span>` : null}
        <button
          type="button"
          class=${filter === "all" ? "active" : ""}
          onClick=${() => setFilter("all")}
        >
          all
        </button>
        <button
          type="button"
          class=${filter === "problems" ? "active" : ""}
          onClick=${() => setFilter("problems")}
        >
          issues
        </button>
        <button
          type="button"
          class=${filter === "outputs" ? "active" : ""}
          onClick=${() => setFilter("outputs")}
        >
          outputs
        </button>
        <button type="button" onClick=${() => setExpanded(!expanded)}>
          ${expanded ? "collapse" : "expand"}
        </button>
        ${compact
          ? html`<button type="button" onClick=${() => setShowRows(!showRows)}>
              ${showRows ? "hide rows" : "show rows"}
            </button>`
          : null}
        <button type="button" onClick=${() => copyTraceRows(rows)}>copy</button>
      </div>
    </div>
    <div class="run-table">
      ${compactRows.length
        ? compactRows.map(
            (row, i) =>
              html`<${PaTraceRow}
                row=${row}
                index=${rows.indexOf(row)}
                forceOpen=${expanded}
                key=${"visible-row" + i}
              />`,
          )
        : html`<div class="run-empty-filter">No rows for this filter.</div>`}
      ${hiddenRowCount
        ? html`<button class="run-show-more" onClick=${() => setShowRows(true)}>
            show ${hiddenRowCount} more row${hiddenRowCount === 1 ? "" : "s"}
          </button>`
        : null}
    </div>
  </div>`;
}

function TextBlock({ text, stripCommands, prefix }) {
  if (isCommandOnlyText(text)) {
    if (stripCommands) return null;
    return html`<${CommandBatchCard} key=${prefix} text=${text} />`;
  }
  if (stripCommands && looksLikeRawDiagnosticDump(text)) {
    return html`<${DiagnosticDumpCard} key=${prefix} text=${text} />`;
  }
  const t = stripPaCommandLines(text, stripCommands);
  if (!t) return null;
  return html`<div
    key=${prefix}
    dangerouslySetInnerHTML=${{ __html: md(t) }}
  ></div>`;
}

export {
  DiagnosticDumpCard,
  ProgressBar,
  PaTrace,
  TextBlock,
  ThinkBlock,
  ToolCard,
  groupChainBlocks,
  isRoutineSystemTraceMessage,
  looksLikeRawDiagnosticDump,
};
