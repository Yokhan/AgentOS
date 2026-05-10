const TERMINAL_RUN_STATUSES = new Set(["done", "failed", "cancelled"]);
const PA_COMMAND_PATTERN = /\[[A-Z][A-Z0-9_]*(?::[^\]]*)?\]/g;
const BACKTICK_READONLY_PATTERN =
  /`(DELEGATE_STATUS(?::[^`\]\s]+)?|DELEGATE_LOG(?::[^`\]\s]+)?|DELEGATE_DIFF(?::[^`\]\s]+)?|GIT_STATUS_ALL|TEMPLATE_AUDIT|DASHBOARD_FULL|HEALTH_CHECK:[^`\]\s]+)`/g;
const WRITE_COMMAND_PATTERN =
  /\[(DELEGATE|DELEGATE_BATCH|DELEGATE_CHAIN|DEPLOY|DEPLOY_STATIC|SERVER_EXEC|GIT_BULK_PUSH|GIT_BULK_PULL|MEMORY_DELETE|CRON_CREATE|CRON_EDIT|CRON_DELETE|WORK_ITEM_QUEUE|QUEUE|PLAN|STRATEGY)(?::[^\]]*)?\]/g;

function isTerminalRun(status) {
  return TERMINAL_RUN_STATUSES.has(String(status || ""));
}

function extractPaCommands(text) {
  return [...String(text || "").matchAll(PA_COMMAND_PATTERN)].map((m) => m[0]);
}

function extractBacktickedReadonlyCommands(text) {
  return [...String(text || "").matchAll(BACKTICK_READONLY_PATTERN)].map(
    (m) => `[${m[1]}]`,
  );
}

function extractWriteCommands(text) {
  return [...String(text || "").matchAll(WRITE_COMMAND_PATTERN)].map(
    (m) => m[0],
  );
}

function compactText(text, max = 92) {
  const clean = String(text || "")
    .replace(/\s+/g, " ")
    .trim();
  return clean.length > max ? clean.slice(0, max - 1) + "…" : clean;
}

function runPhaseLabel(run) {
  const phase = String(run?.phase || "");
  const status = String(run?.status || "");
  if (status === "done") return "готово";
  if (status === "failed") return "ошибка";
  if (status === "cancelled") return "остановлено";
  if (status === "warning") return "нужна проверка";
  if (phase === "provider") return "ждём модель";
  if (phase === "tool") return "инструмент";
  if (phase === "agentos") return "AgentOS команда";
  if (phase === "command") return "результат команды";
  if (phase === "waiting_output") return "нет новых событий";
  if (phase === "queued") return "старт";
  if (phase === "startup") return "запуск";
  return phase || status || "работает";
}

function runStuckHint(run, now = Date.now()) {
  if (!run || isTerminalRun(run.status)) return null;
  const ageMs = run.startedAt ? now - Number(run.startedAt) : 0;
  const quietMs = run.updatedAt ? now - Number(run.updatedAt) : ageMs;
  if (String(run.phase || "") === "waiting_output") {
    return {
      severity: "warn",
      title: "Нет новых событий",
      text:
        run.detail ||
        "Процесс жив, но UI давно не получил новых событий. Проверь provider heartbeat или останови запуск.",
    };
  }
  if (quietMs > 45000) {
    return {
      severity: "warn",
      title: "Давно нет обновлений",
      text: `Последнее событие было ${Math.round(quietMs / 1000)}с назад. Если прогресс не появится, жми stop.`,
    };
  }
  if (ageMs > 180000 && String(run.phase || "") === "provider") {
    return {
      severity: "info",
      title: "Долгий ответ модели",
      text: "Модель всё ещё отвечает. Это может быть нормальным для больших задач, но теперь состояние явно видно.",
    };
  }
  return null;
}

function buildComposerPreview({
  route,
  draft,
  duoEnabled,
  duoAction,
  target,
  contextCount = 0,
  fileCount = 0,
}) {
  const text = String(draft || "");
  const strictCommands = extractPaCommands(text);
  const recoveredReadonly = extractBacktickedReadonlyCommands(text).filter(
    (cmd) => !strictCommands.includes(cmd),
  );
  const writeCommands = extractWriteCommands(text);
  const mode = route?.modeRaw || route?.mode || "act";
  const access = route?.accessRaw || route?.access || "write";
  const provider = route?.provider || "agent";
  const model = route?.modelRaw || route?.model || "";
  const destination = duoEnabled
    ? target || route?.inputLabel || "duo"
    : route?.target || "_orchestrator";
  const warnings = [];
  if (mode === "plan" && (strictCommands.length || recoveredReadonly.length)) {
    warnings.push("plan mode: PA-команды не будут выполняться");
  }
  if (mode !== "plan" && access === "read" && writeCommands.length) {
    warnings.push("read access: write-команды будут заблокированы");
  }
  const hasDraft = !!text.trim();
  return {
    hasDraft,
    destination,
    provider,
    model,
    mode,
    access,
    strictCommands,
    recoveredReadonly,
    writeCommands,
    warnings,
    contextCount,
    fileCount,
    headline: `${destination} -> ${provider}${model ? " / " + model : ""} / ${mode}/${mode === "plan" ? "read" : access}`,
    detail:
      strictCommands.length || recoveredReadonly.length
        ? `${strictCommands.length} strict command, ${recoveredReadonly.length} recoverable read-only`
        : compactText(text, 120) || "сообщение ещё не введено",
  };
}

function quietWaitEvent(elapsedSeconds) {
  return {
    type: "frontend_wait",
    phase: "waiting_output",
    status: "running",
    detail: `нет новых событий ${elapsedSeconds}с; ждём provider/tool output`,
  };
}

function runTraceLabel(run) {
  if (!run) return "";
  return [
    run.id ? `run ${String(run.id).slice(-8)}` : "",
    run.provider || "",
    run.model || "",
    run.phase || "",
  ]
    .filter(Boolean)
    .join(" · ");
}

export {
  buildComposerPreview,
  compactText,
  extractBacktickedReadonlyCommands,
  extractPaCommands,
  extractWriteCommands,
  isTerminalRun,
  quietWaitEvent,
  runPhaseLabel,
  runStuckHint,
  runTraceLabel,
};
