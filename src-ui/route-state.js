const ORCHESTRATOR_KEY = "_orchestrator";
const ORCHESTRATOR_LABEL = "orchestrator";

function normalizeProjectKey(project) {
  const value = String(project || "").trim();
  if (!value || value === ORCHESTRATOR_KEY || value === ORCHESTRATOR_LABEL) {
    return ORCHESTRATOR_KEY;
  }
  return value;
}

function projectParam(project) {
  const key = normalizeProjectKey(project);
  return key === ORCHESTRATOR_KEY ? "" : key;
}

function displayProject(project) {
  const key = normalizeProjectKey(project);
  return key === ORCHESTRATOR_KEY ? ORCHESTRATOR_LABEL : key;
}

function buildRouteState({
  currentProject = "",
  activeScope = null,
  activeRun = null,
  chatPageInfo = null,
  activeDualSession = null,
  dualSessionData = null,
} = {}) {
  const key = normalizeProjectKey(currentProject);
  const label = displayProject(key);
  const scope = activeScope || {};
  const scopeProject = scope.project || scope.target_project || "";
  const chatProject = chatPageInfo?.project || "";
  const runProject = activeRun?.project || "";
  const sessionProject = dualSessionData?.session?.project || dualSessionData?.project || "";
  const mismatches = [];
  const check = (name, value) => {
    if (!value) return;
    const other = normalizeProjectKey(value);
    if (other !== key) {
      mismatches.push(`${name} points to ${displayProject(other)}`);
    }
  };
  check("chat history", chatProject);
  check("active run", runProject);
  check("scope", scopeProject);
  check("duo session", sessionProject);

  return {
    key,
    label,
    projectParam: projectParam(key),
    isGlobal: key === ORCHESTRATOR_KEY,
    scopeKind: scope.kind || (key === ORCHESTRATOR_KEY ? "global" : "project"),
    scopeTitle: scope.title || label,
    scopeLabel: `${scope.kind || (key === ORCHESTRATOR_KEY ? "global" : "project")}:${scope.title || label}`,
    activeDualSession: activeDualSession || "",
    mismatches,
  };
}

export {
  ORCHESTRATOR_KEY,
  ORCHESTRATOR_LABEL,
  normalizeProjectKey,
  projectParam,
  displayProject,
  buildRouteState,
};
