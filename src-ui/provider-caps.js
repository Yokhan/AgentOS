export const CLAUDE_MODEL_OPTIONS = [
  ["", "auto"],
  ["opus", "opus"],
  ["sonnet", "sonnet"],
  ["haiku", "haiku"],
];

export const CLAUDE_EFFORT_OPTIONS = [
  ["", "effort"],
  ["low", "low"],
  ["medium", "medium"],
  ["high", "high"],
  ["max", "max"],
];

export const CODEX_MODEL_OPTIONS = [
  ["", "auto"],
  ["gpt-5.5", "gpt-5.5"],
  ["gpt-5.4", "gpt-5.4"],
  ["gpt-5.4-mini", "gpt-5.4-mini"],
  ["gpt-5.3-codex", "gpt-5.3-codex"],
  ["gpt-5.3-codex-spark", "gpt-5.3-codex-spark"],
  ["gpt-5.2", "gpt-5.2"],
  ["gpt-5.2-codex", "gpt-5.2-codex"],
  ["gpt-5.1-codex-mini", "gpt-5.1-codex-mini"],
  ["gpt-5.1-codex-max", "gpt-5.1-codex-max"],
  ["gpt-5.1-codex", "gpt-5.1-codex"],
];

function normalizeReasoningEffort(value) {
  if (!value) return "";
  if (typeof value === "string") return value.trim();
  return String(
    value.effort || value.name || value.id || value.value || "",
  ).trim();
}

export function codexModelOptionsFromStatus(codexStatus, selectedModel = "") {
  const options = [["", "auto"]];
  const seen = new Set([""]);
  const add = (value, label = value) => {
    const normalized = String(value || "").trim();
    if (!normalized || seen.has(normalized)) return;
    seen.add(normalized);
    options.push([normalized, String(label || normalized)]);
  };
  for (const model of codexStatus?.models || []) {
    if (typeof model === "string") {
      add(model);
    } else {
      add(
        model.slug || model.id || model.name || model.model,
        model.display_name || model.label,
      );
    }
  }
  for (const [value, label] of CODEX_MODEL_OPTIONS) add(value, label);
  add(selectedModel, selectedModel);
  return options;
}

export function codexEffortOptionsForModel(
  model,
  defaultLabel = "default",
  codexStatus = null,
) {
  const normalized = String(model || "")
    .trim()
    .toLowerCase();
  const head = [["", defaultLabel]];
  const dynamic = (codexStatus?.models || []).find((entry) => {
    if (typeof entry === "string") return entry.toLowerCase() === normalized;
    const slug = String(
      entry.slug || entry.id || entry.name || entry.model || "",
    ).toLowerCase();
    return slug === normalized;
  });
  const dynamicEfforts = (dynamic?.supported_reasoning_levels || [])
    .map(normalizeReasoningEffort)
    .filter(Boolean);
  if (dynamicEfforts.length) {
    return [...head, ...dynamicEfforts.map((effort) => [effort, effort])];
  }
  if (!normalized) {
    return [
      ...head,
      ["none", "none"],
      ["low", "low"],
      ["medium", "medium"],
      ["high", "high"],
      ["xhigh", "xhigh"],
    ];
  }
  if (
    normalized.startsWith("gpt-5.5") ||
    normalized.startsWith("gpt-5.4") ||
    normalized === "gpt-5.2"
  ) {
    return [
      ...head,
      ["none", "none"],
      ["low", "low"],
      ["medium", "medium"],
      ["high", "high"],
      ["xhigh", "xhigh"],
    ];
  }
  if (
    normalized === "gpt-5.3-codex" ||
    normalized === "gpt-5.3-codex-spark" ||
    normalized === "gpt-5.2-codex"
  ) {
    return [
      ...head,
      ["low", "low"],
      ["medium", "medium"],
      ["high", "high"],
      ["xhigh", "xhigh"],
    ];
  }
  if (normalized === "gpt-5.1-codex-max") {
    return [
      ...head,
      ["none", "none"],
      ["medium", "medium"],
      ["high", "high"],
      ["xhigh", "xhigh"],
    ];
  }
  if (normalized.startsWith("gpt-5.1")) {
    return [
      ...head,
      ["none", "none"],
      ["low", "low"],
      ["medium", "medium"],
      ["high", "high"],
    ];
  }
  if (normalized.startsWith("gpt-5")) {
    return [
      ...head,
      ["minimal", "minimal"],
      ["low", "low"],
      ["medium", "medium"],
      ["high", "high"],
    ];
  }
  return [...head, ["low", "low"], ["medium", "medium"], ["high", "high"]];
}

export function normalizeSoloSelection(provider, model, effort) {
  const selectedModel = String(model || "").trim();
  const selectedEffort = String(effort || "").trim();
  if (provider === "codex") {
    const knownModels = new Set(CODEX_MODEL_OPTIONS.map(([value]) => value));
    const normalizedModel =
      knownModels.has(selectedModel) || selectedModel.startsWith("gpt-")
        ? selectedModel
        : "";
    const allowedEfforts = new Set(
      codexEffortOptionsForModel(normalizedModel, "effort").map(
        ([value]) => value,
      ),
    );
    return {
      model: normalizedModel,
      effort: allowedEfforts.has(selectedEffort) ? selectedEffort : "",
    };
  }
  const allowedModels = new Set(CLAUDE_MODEL_OPTIONS.map(([value]) => value));
  const allowedEfforts = new Set(CLAUDE_EFFORT_OPTIONS.map(([value]) => value));
  return {
    model: allowedModels.has(selectedModel) ? selectedModel : "",
    effort: allowedEfforts.has(selectedEffort) ? selectedEffort : "",
  };
}
