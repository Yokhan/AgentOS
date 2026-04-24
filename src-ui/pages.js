// AgentOS page views — Settings, Plans, Strategy
import { html, useEffect, useState } from "/vendor/preact-bundle.mjs";
import { ft, md } from "/utils.js";
import { __IS_TAURI, __invoke } from "/bridge.js";
import {
  agents,
  currentProject,
  showSettings,
  showPlans,
  showStrategy,
  plansData,
  goals,
  strategies,
  strategyLoading,
  permData,
  activeDualSession,
  dualSessionData,
  dualBusy,
  sideMessages,
  chatCollabMode,
  activeRoomTab,
  activeScope,
  showToast,
} from "/store.js";
import {
  loadGoals,
  loadStrategies,
  loadPlansData,
  loadPerms,
  loadInbox,
  loadSignals,
  generateStrategy,
  createAdhocPlanFromRoom,
  createRoomProjectSession,
  createRoomWorkItem,
  createPlanStepWorkItem,
  queueWorkItemExecution,
  queueParallelWorkItems,
  queueProviderParallelRound,
  completeUserWorkItem,
  acquireWorkItemLease,
  releaseFileLease,
  queueRoomAgentTask,
  approveSteps,
  executeNextStep,
  approveDel,
  setPermission,
  authenticateCodexAcp,
  setDualWriter,
  setDualOrchestrator,
  revokeDualWriter,
  ensureDualSession,
  loadDualSession,
  runDualRound,
  runDualRoomAction,
} from "/api.js";
import {
  codexModelOptionsFromStatus,
  codexEffortOptionsForModel,
} from "/provider-caps.js";

function SettingsPage() {
  const pd = permData.value;
  const a = agents.value;
  const ps = pd?.provider_status || {};
  const prov = ps.providers || {};
  const profCount = (k) => pd?.profiles?.[k]?.permissions?.allow?.length || 0;
  const codexStatus = prov?.codex || {};
  const codexModel = pd?.config?.codex_model || "";
  const codexEffort = pd?.config?.codex_effort || "";
  const codexModelOptions = codexModelOptionsFromStatus(
    codexStatus,
    codexModel,
  );
  const codexEffortOptions = codexEffortOptionsForModel(
    codexModel,
    "default",
    codexStatus,
  );
  const codexTransport =
    codexStatus?.transport || pd?.config?.codex_transport || "cli";
  const codexReady = !!codexStatus.ready;
  const codexAvailable = !!codexStatus.available;
  const codexAuthRequired = !!codexStatus.auth_required;
  const codexStateLabel = codexReady
    ? "Connected"
    : !codexAvailable
      ? codexTransport === "acp"
        ? "Runtime not found"
        : "CLI not found"
      : codexTransport === "acp" && codexAuthRequired
        ? "Needs sign-in"
        : "Needs setup";
  const codexStateColor = codexReady
    ? "var(--green)"
    : codexAvailable
      ? "var(--yellow)"
      : "var(--accent)";
  const codexHelperText = codexReady
    ? "Codex is connected and ready for duo mode."
    : !codexAvailable
      ? codexTransport === "acp"
        ? "AgentOS cannot see a Codex ACP runtime yet. Point it to your codex-acp command in Advanced."
        : "AgentOS cannot see a standalone Codex CLI yet."
      : codexTransport === "acp" && codexAuthRequired
        ? "The runtime is reachable. Finish login with ChatGPT and then refresh status."
        : codexTransport === "acp"
          ? "ACP is reachable but not ready yet. Refresh after login or adapter startup."
          : "CLI is reachable. Pick a model and AgentOS can use the official codex exec flow directly.";
  return html`<div class="content">
    <div
      class="back"
      style="font-size:var(--fs-s);color:var(--t3);cursor:pointer;padding:var(--sp-s) 0;text-transform:uppercase;letter-spacing:1px"
      onClick=${() => (showSettings.value = false)}
    >
      ← back
    </div>
    <h2
      style="font-size:var(--fs-xl);font-family:var(--font-mono);letter-spacing:3px;margin-bottom:var(--sp-xl)"
    >
      SETTINGS
    </h2>

    <div
      style="display:grid;grid-template-columns:1fr 1fr;gap:var(--sp-l);margin-bottom:var(--sp-xl)"
    >
      <div class="panel">
        <h3>General</h3>
        <div style="display:flex;flex-direction:column;gap:var(--sp-m)">
          <div>
            <label
              style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs);text-transform:uppercase;letter-spacing:1px"
              >documents directory</label
            >
            <input
              style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
              value="${pd?.config?.documents_dir || ""}"
              onChange=${(e) => {
                __invoke("set_config", {
                  key: "documents_dir",
                  value: e.target.value,
                }).then(() => showToast("Saved. Restart to apply.", "info"));
              }}
            />
          </div>
          <div>
            <label
              style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs);text-transform:uppercase;letter-spacing:1px"
              >orchestrator project</label
            >
            <select
              style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
              onChange=${(e) => {
                __invoke("set_config", {
                  key: "orchestrator_project",
                  value: e.target.value,
                }).then(() => showToast("Set. Restart to apply.", "info"));
              }}
            >
              <option value="">-- select --</option>
              ${a.map(
                (ag) =>
                  html`<option
                    value=${ag.name}
                    selected=${ag.name ===
                    (pd?.config?.orchestrator_project || "")}
                  >
                    ${ag.name}
                  </option>`,
              )}
            </select>
          </div>
          <div
            style="font-size:var(--fs-s);color:var(--t2);margin-top:var(--sp-xs)"
          >
            Normal chat stays primary. Duo mode auto-creates or reuses a hidden
            shared thread for Claude and Codex.
          </div>
        </div>
      </div>
      <div class="panel">
        <h3>Permission Profiles</h3>
        <div style="display:flex;flex-direction:column;gap:var(--sp-s)">
          <div
            style="display:flex;justify-content:space-between;padding:var(--sp-xs) 0;border-bottom:1px solid var(--border)"
          >
            <span
              style="color:var(--accent);font-family:var(--font-mono);font-size:var(--fs-s)"
              >RESTRICTIVE</span
            >
            <span style="color:var(--t3);font-size:var(--fs-s)"
              >${profCount("restrictive")} tools</span
            >
          </div>
          <div
            style="display:flex;justify-content:space-between;padding:var(--sp-xs) 0;border-bottom:1px solid var(--border)"
          >
            <span
              style="color:var(--yellow);font-family:var(--font-mono);font-size:var(--fs-s)"
              >BALANCED</span
            >
            <span style="color:var(--t3);font-size:var(--fs-s)"
              >${profCount("balanced")} tools</span
            >
          </div>
          <div
            style="display:flex;justify-content:space-between;padding:var(--sp-xs) 0"
          >
            <span
              style="color:var(--green);font-family:var(--font-mono);font-size:var(--fs-s)"
              >PERMISSIVE</span
            >
            <span style="color:var(--t3);font-size:var(--fs-s)"
              >${profCount("permissive")} tools</span
            >
          </div>
        </div>
      </div>
    </div>

    <div
      style="display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:var(--sp-l);margin-bottom:var(--sp-xl)"
    >
      <div class="panel">
        <h3>Claude Orchestrator</h3>
        <div style="display:flex;flex-direction:column;gap:var(--sp-s)">
          <div>
            <label
              style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
              >model</label
            >
            <select
              style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
              value=${pd?.config?.orchestrator_model || ""}
              onChange=${(e) => {
                __invoke("set_config", {
                  key: "orchestrator_model",
                  value: e.target.value,
                }).then(() => {
                  showToast("Saved", "success");
                  loadPerms();
                });
              }}
            >
              <option value="">auto</option>
              <option value="opus">opus</option>
              <option value="sonnet">sonnet</option>
              <option value="haiku">haiku</option>
            </select>
          </div>
          <div>
            <label
              style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
              >effort</label
            >
            <select
              style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
              value=${pd?.config?.orchestrator_effort || ""}
              onChange=${(e) => {
                __invoke("set_config", {
                  key: "orchestrator_effort",
                  value: e.target.value,
                }).then(() => {
                  showToast("Saved", "success");
                  loadPerms();
                });
              }}
            >
              <option value="">default</option>
              <option value="low">low</option>
              <option value="medium">medium</option>
              <option value="high">high</option>
              <option value="max">max</option>
            </select>
          </div>
        </div>
      </div>
      <div class="panel">
        <h3>Claude Delegation</h3>
        <div style="display:flex;flex-direction:column;gap:var(--sp-s)">
          <div>
            <label
              style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
              >model</label
            >
            <select
              style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
              value=${pd?.config?.delegation_model || ""}
              onChange=${(e) => {
                __invoke("set_config", {
                  key: "delegation_model",
                  value: e.target.value,
                }).then(() => {
                  showToast("Saved", "success");
                  loadPerms();
                });
              }}
            >
              <option value="">auto</option>
              <option value="opus">opus</option>
              <option value="sonnet">sonnet</option>
              <option value="haiku">haiku</option>
            </select>
          </div>
          <div>
            <label
              style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
              >effort</label
            >
            <select
              style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
              value=${pd?.config?.delegation_effort || ""}
              onChange=${(e) => {
                __invoke("set_config", {
                  key: "delegation_effort",
                  value: e.target.value,
                }).then(() => {
                  showToast("Saved", "success");
                  loadPerms();
                });
              }}
            >
              <option value="">default</option>
              <option value="low">low</option>
              <option value="medium">medium</option>
              <option value="high">high</option>
              <option value="max">max</option>
            </select>
          </div>
        </div>
      </div>
      <div class="panel">
        <h3>Codex</h3>
        <div style="display:flex;flex-direction:column;gap:var(--sp-s)">
          <div>
            <label
              style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
              >model</label
            >
            <select
              style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
              value=${codexModel}
              onChange=${(e) => {
                const nextModel = e.target.value;
                const nextEffortOptions = codexEffortOptionsForModel(
                  nextModel,
                  "default",
                  codexStatus,
                );
                const allowedEfforts = new Set(
                  nextEffortOptions.map(([value]) => value),
                );
                Promise.all([
                  __invoke("set_config", {
                    key: "codex_model",
                    value: nextModel,
                  }),
                  !allowedEfforts.has(codexEffort)
                    ? __invoke("set_config", {
                        key: "codex_effort",
                        value: "",
                      })
                    : Promise.resolve(),
                ]).then(() => {
                  showToast("Saved", "success");
                  loadPerms();
                });
              }}
            >
              <option value="">auto</option>
              ${codexModelOptions.map(
                ([value, label]) =>
                  html`<option value=${value}>${label}</option>`,
              )}
            </select>
            <input
              style="margin-top:var(--sp-xs);width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
              placeholder="custom model, e.g. gpt-5.5"
              value=${codexModel}
              onChange=${(e) => {
                const nextModel = e.target.value.trim();
                __invoke("set_config", {
                  key: "codex_model",
                  value: nextModel,
                }).then(() => {
                  showToast("Saved", "success");
                  loadPerms();
                });
              }}
            />
          </div>
          <div>
            <label
              style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
              >reasoning effort</label
            >
            <select
              style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
              value=${codexEffort}
              onChange=${(e) => {
                __invoke("set_config", {
                  key: "codex_effort",
                  value: e.target.value,
                }).then(() => {
                  showToast("Saved", "success");
                  loadPerms();
                });
              }}
            >
              ${codexEffortOptions.map(
                ([value, label]) =>
                  html`<option value=${value}>${label}</option>`,
              )}
            </select>
          </div>
          <div style="font-size:var(--fs-s);color:var(--t2)">
            These are current OpenAI/Codex-style model names. Your ChatGPT plan
            may not allow every option.
          </div>
          <div style="font-size:var(--fs-s);color:var(--t3)">
            Effort choices are model-aware now. GPT-5.4 and GPT-5.2 support
            <code>none..xhigh</code>, Codex-specific models usually support
            <code>low..xhigh</code>, and GPT-5.1 family stays at
            <code>none..high</code>.
          </div>
        </div>
      </div>
    </div>

    <div
      style="display:grid;grid-template-columns:1fr 1fr;gap:var(--sp-l);margin-bottom:var(--sp-xl)"
    >
      <div class="panel">
        <h3>Dual-Agent Roles</h3>
        <div style="display:flex;flex-direction:column;gap:var(--sp-s)">
          <div>
            <label
              style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
              >orchestrator provider</label
            >
            <select
              style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
              value=${pd?.config?.orchestrator_provider || "claude"}
              onChange=${(e) => {
                __invoke("set_config", {
                  key: "orchestrator_provider",
                  value: e.target.value,
                }).then(() => {
                  showToast("Saved", "success");
                  loadPerms();
                });
              }}
            >
              <option value="claude">claude</option>
              <option value="codex">codex</option>
            </select>
          </div>
          <div>
            <label
              style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
              >technical reviewer provider</label
            >
            <select
              style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
              value=${pd?.config?.technical_reviewer_provider || "codex"}
              onChange=${(e) => {
                __invoke("set_config", {
                  key: "technical_reviewer_provider",
                  value: e.target.value,
                }).then(() => {
                  showToast("Saved", "success");
                  loadPerms();
                });
              }}
            >
              <option value="codex">codex</option>
              <option value="claude">claude</option>
            </select>
          </div>
          <div
            style="font-size:var(--fs-s);color:var(--t2);display:flex;flex-direction:column;gap:var(--sp-xs)"
          >
            <div>
              claude:
              <span
                style="color:${prov?.claude?.available
                  ? "var(--green)"
                  : "var(--accent)"}"
                >${prov?.claude?.available ? "available" : "missing"}</span
              >
            </div>
            <div>
              codex:
              <span
                style="color:${prov?.codex?.ready
                  ? "var(--green)"
                  : prov?.codex?.available
                    ? "var(--yellow)"
                    : "var(--accent)"}"
                >${prov?.codex?.ready
                  ? "ready"
                  : prov?.codex?.available
                    ? prov?.codex?.transport === "acp" &&
                      prov?.codex?.auth_required
                      ? "sign in required"
                      : "needs setup"
                    : "missing"}</span
              >
            </div>
          </div>
        </div>
      </div>
      <div class="panel">
        <h3>Codex Connection</h3>
        <div style="display:flex;flex-direction:column;gap:var(--sp-s)">
          <div
            style="display:flex;justify-content:space-between;gap:var(--sp-s);align-items:flex-start;flex-wrap:wrap"
          >
            <div>
              <div
                style="font-family:var(--font-mono);font-size:var(--fs-s);color:${codexStateColor}"
              >
                ${codexStateLabel}
              </div>
              <div
                style="font-size:var(--fs-s);color:var(--t2);margin-top:var(--sp-xs)"
              >
                ${codexHelperText}
              </div>
            </div>
            <div style="display:flex;gap:var(--sp-xs);flex-wrap:wrap">
              ${codexTransport === "acp" && codexAvailable && codexAuthRequired
                ? html`<button
                    class="action-btn"
                    onClick=${async () => {
                      try {
                        const methodId =
                          codexStatus?.auth_methods?.[0]?.id || null;
                        await authenticateCodexAcp(methodId);
                      } catch (e) {
                        showToast("Codex ACP auth error: " + e, "error", 8000);
                      }
                    }}
                  >
                    sign in with ChatGPT
                  </button>`
                : null}
              <button class="action-btn" onClick=${() => loadPerms()}>
                ${codexReady ? "refresh" : "check status"}
              </button>
            </div>
          </div>
          <details>
            <summary
              style="cursor:pointer;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3)"
            >
              advanced
            </summary>
            <div
              style="display:flex;flex-direction:column;gap:var(--sp-s);margin-top:var(--sp-s)"
            >
              <div style="font-size:var(--fs-s);color:var(--t2)">
                ACP is recommended when you want ChatGPT subscription auth and
                agent-managed login. CLI mode is for standalone Codex setups.
              </div>
              <div>
                <label
                  style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
                  >transport</label
                >
                <select
                  style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
                  value=${codexTransport}
                  onChange=${(e) => {
                    __invoke("set_config", {
                      key: "codex_transport",
                      value: e.target.value,
                    }).then(() => {
                      showToast("Saved", "success");
                      loadPerms();
                    });
                  }}
                >
                  <option value="acp">acp</option>
                  <option value="cli">cli</option>
                </select>
              </div>
              ${codexTransport === "acp"
                ? html`<div>
                      <label
                        style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
                        >acp command</label
                      >
                      <input
                        style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
                        value="${pd?.config?.codex_acp_command || ""}"
                        onChange=${(e) => {
                          __invoke("set_config", {
                            key: "codex_acp_command",
                            value: e.target.value,
                          }).then(() => {
                            showToast("Saved", "success");
                            loadPerms();
                          });
                        }}
                      />
                    </div>
                    <div>
                      <label
                        style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
                        >acp args</label
                      >
                      <textarea
                        rows="2"
                        style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s);resize:vertical"
                        onChange=${(e) => {
                          __invoke("set_config", {
                            key: "codex_acp_args",
                            value: e.target.value,
                          }).then(() => {
                            showToast("Saved", "success");
                            loadPerms();
                          });
                        }}
                      >
${pd?.config?.codex_acp_args || "[]"}</textarea
                      >
                    </div>
                    <div style="font-size:var(--fs-s);color:var(--t3)">
                      ${codexStatus?.probe ||
                      "ACP mode expects an external Codex ACP adapter command."}
                    </div>
                    <div style="font-size:var(--fs-s);color:var(--t3)">
                      ${codexStatus?.auth_methods?.length
                        ? "Auth methods: " +
                          codexStatus.auth_methods
                            .map((m) => m.label || m.id || "unknown")
                            .join(", ")
                        : "No auth methods advertised yet."}
                    </div>`
                : html`<div>
                      <label
                        style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
                        >binary</label
                      >
                      <input
                        style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s)"
                        value="${pd?.config?.codex_binary || ""}"
                        onChange=${(e) => {
                          __invoke("set_config", {
                            key: "codex_binary",
                            value: e.target.value,
                          }).then(() => {
                            showToast("Saved", "success");
                            loadPerms();
                          });
                        }}
                      />
                    </div>
                    <div>
                      <label
                        style="display:block;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
                        >command template</label
                      >
                      <textarea
                        rows="3"
                        style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s);resize:vertical"
                        onChange=${(e) => {
                          __invoke("set_config", {
                            key: "codex_command_template",
                            value: e.target.value,
                          }).then(() => {
                            showToast("Saved", "success");
                            loadPerms();
                          });
                        }}
                      >
${pd?.config?.codex_command_template || ""}</textarea
                      >
                    </div>
                    <div style="font-size:var(--fs-s);color:var(--t3)">
                      ${codexStatus?.probe ||
                      "Use placeholders {prompt_file}, {model}, {effort} in the template."}
                    </div>
                    <div style="font-size:var(--fs-s);color:var(--t3)">
                      Example JSON template:
                      ["exec","-m","{model}","-c","model_reasoning_effort="{effort}"","{prompt_file}"]
                    </div>`}
            </div>
          </details>
        </div>
      </div>
    </div>

    <div class="panel" style="margin-bottom:var(--sp-xl)">
      <h3>Per-Project Permissions</h3>
      <div
        style="display:grid;grid-template-columns:repeat(auto-fill,minmax(300px,1fr));gap:var(--sp-xs)"
      >
        ${a.map((ag) => {
          const cur = pd?.project_permissions?.[ag.name] || "balanced";
          return html`<div
            style="display:flex;justify-content:space-between;align-items:center;padding:var(--sp-xs) var(--sp-s);border-bottom:1px solid var(--border)"
          >
            <span
              style="font-size:var(--fs-s);display:flex;align-items:center;gap:var(--sp-xs)"
              ><span class="dot ${ag.status}"></span>${ag.name}</span
            >
            <div style="display:flex;gap:var(--sp-2xs)">
              <button
                class="perm-btn ${cur === "restrictive" ? "active" : ""}"
                onClick=${() => setPermission(ag.name, "restrictive")}
                style="font-size:var(--fs-s);padding:var(--sp-2xs) var(--sp-xs)"
              >
                R
              </button>
              <button
                class="perm-btn ${cur === "balanced" ? "active-yellow" : ""}"
                onClick=${() => setPermission(ag.name, "balanced")}
                style="font-size:var(--fs-s);padding:var(--sp-2xs) var(--sp-xs)"
              >
                B
              </button>
              <button
                class="perm-btn ${cur === "permissive" ? "active-green" : ""}"
                onClick=${() => setPermission(ag.name, "permissive")}
                style="font-size:var(--fs-s);padding:var(--sp-2xs) var(--sp-xs)"
              >
                P
              </button>
            </div>
          </div>`;
        })}
      </div>
    </div>
  </div>`;
}

function PlansView() {
  const plans = plansData.value;
  const openPlanInDuo = async (plan) => {
    const project =
      (plan.steps || []).find(
        (step) => step.project && step.project !== "_orchestrator",
      )?.project || "";
    if (project) currentProject.value = project;
    activeScope.value = {
      kind: "plan",
      label: "Plan",
      title: plan.title,
      project,
      plan_id: plan.id,
      counts: {
        steps: (plan.steps || []).length,
        active_steps: (plan.steps || []).filter(
          (step) => !["done", "failed", "cancelled"].includes(step.status),
        ).length,
      },
      breadcrumbs: [
        { kind: "global", label: "Global" },
        ...(project ? [{ kind: "project", label: project }] : []),
        { kind: "plan", label: plan.title },
      ],
      available_actions: [
        { id: "ask_both", label: "Ask both", tone: "neutral" },
        {
          id: "execute_next_step",
          label: "Execute next step",
          tone: "primary",
        },
        { id: "create_work_item", label: "Create task", tone: "neutral" },
        { id: "replan", label: "Replan", tone: "neutral" },
      ],
      summary: `Duo actions apply to plan: ${plan.title}`,
    };
    showPlans.value = false;
    chatCollabMode.value = true;
    activeRoomTab.value = "execute";
    try {
      const session = await ensureDualSession(project);
      if (session?.id) await loadDualSession(session.id);
    } catch (e) {
      showToast("Open plan in Duo error: " + e, "error");
    }
  };
  return html`<div class="content">
    <div class="back" onClick=${() => (showPlans.value = false)}>
      ← back to dashboard
    </div>
    <h2
      style="font-size:var(--fs-xl);margin:var(--sp-m) 0;letter-spacing:2px;font-family:var(--font-mono)"
    >
      PLANS
    </h2>
    ${!plans.length
      ? html`<div
          style="color:var(--t3);font-family:var(--font-mono);padding:var(--sp-xl)"
        >
          No plans yet. PA will create plans automatically when you give it
          multi-step tasks.
        </div>`
      : null}
    ${plans.map((plan) => {
      const done = plan.steps?.filter((s) => s.status === "done").length || 0;
      const failed =
        plan.steps?.filter((s) => s.status === "failed").length || 0;
      const total = plan.steps?.length || 0;
      const pct = total ? Math.round((done / total) * 100) : 0;
      return html`<div
        style="border:1px solid var(--border);margin-bottom:var(--sp-m);padding:var(--sp-m)"
      >
        <div
          style="display:flex;justify-content:space-between;align-items:center;margin-bottom:var(--sp-s)"
        >
          <strong style="font-size:var(--fs-m)">${plan.title}</strong>
          <div
            style="display:flex;gap:var(--sp-xs);align-items:center;flex-wrap:wrap"
          >
            <button
              class="action-btn"
              style="font-size:var(--fs-s);padding:3px 8px"
              onClick=${() => openPlanInDuo(plan)}
            >
              open in Duo execute
            </button>
            <span
              style="font-size:var(--fs-s);padding:2px 8px;border:1px solid;color:${plan.status ===
              "completed"
                ? "var(--green)"
                : plan.status === "active"
                  ? "var(--yellow)"
                  : "var(--t3)"}"
              >${plan.status}</span
            >
          </div>
        </div>
        <div
          style="display:flex;align-items:center;gap:var(--sp-s);margin-bottom:var(--sp-s)"
        >
          <div
            style="flex:1;height:4px;background:var(--border);border-radius:2px"
          >
            <div
              style="height:100%;background:var(--green);width:${pct}%;border-radius:2px;transition:width .3s"
            ></div>
          </div>
          <span
            style="font-size:var(--fs-s);font-family:var(--font-mono);color:var(--t2)"
            >${done}/${total}${failed ? " (" + failed + " failed)" : ""}</span
          >
        </div>
        ${(plan.steps || []).map((step, i) => {
          const icon =
            step.status === "done"
              ? "✓"
              : step.status === "failed"
                ? "✗"
                : step.status === "running"
                  ? "⏳"
                  : "○";
          const color =
            step.status === "done"
              ? "var(--green)"
              : step.status === "failed"
                ? "var(--accent)"
                : step.status === "running"
                  ? "var(--yellow)"
                  : "var(--t3)";
          return html`<div
            style="display:flex;align-items:center;gap:var(--sp-s);padding:var(--sp-xs) 0;border-bottom:1px solid var(--border);font-size:var(--fs-s);cursor:pointer"
            onClick=${() => {
              if (step.project && step.project !== "_orchestrator") {
                currentProject.value = step.project;
                showPlans.value = false;
              }
            }}
          >
            <span style="color:${color};min-width:20px">${icon}</span>
            <span
              style="font-family:var(--font-mono);min-width:120px;color:var(--text)"
              >${step.project}</span
            >
            <span style="color:var(--t2);flex:1">${step.task}</span>
            ${step.result
              ? html`<span
                  style="font-size:var(--fs-s);color:var(--t3);max-width:200px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap"
                  >${step.result}</span
                >`
              : null}
          </div>`;
        })}
        <div
          style="font-size:var(--fs-s);color:var(--t3);margin-top:var(--sp-xs)"
        >
          created
          ${ft(plan.created)}${plan.updated !== plan.created
            ? " · updated " + ft(plan.updated)
            : ""}
        </div>
      </div>`;
    })}
  </div>`;
}

function StrategyView() {
  const goalRef = useRef();
  const ctxRef = useRef();
  const strats = strategies.value;
  const g = goals.value;
  const as_ = activeStrategy.value;
  const SC_ = { HIGH: "var(--accent)", MED: "var(--yellow)", LOW: "var(--t3)" };
  return html`<div class="content">
    <div class="back" onClick=${() => (showStrategy.value = false)}>
      ← back to dashboard
    </div>
    <h2
      style="font-size:var(--fs-xl);margin:var(--sp-m) 0;letter-spacing:2px;font-family:var(--font-mono)"
    >
      STRATEGY
    </h2>

    <div class="panel" style="margin-bottom:var(--sp-l)">
      <h3>set goal</h3>
      <input
        ref=${goalRef}
        style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--text);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-m);margin-bottom:var(--sp-s)"
        placeholder="e.g., Launch fitness bot MVP by Friday"
      />
      <textarea
        ref=${ctxRef}
        rows="2"
        style="width:100%;background:var(--sf);border:1px solid var(--border);color:var(--t2);padding:var(--sp-s);font-family:var(--font-mono);font-size:var(--fs-s);margin-bottom:var(--sp-s);resize:vertical"
        placeholder="Context: constraints, deadline, priorities (optional)"
      />
      <button
        style="background:var(--green);color:var(--bg);border:none;padding:var(--sp-xs) var(--sp-l);font-family:var(--font-mono);cursor:pointer"
        onClick=${() => {
          const g = goalRef.current?.value;
          if (!g?.trim()) {
            showToast("Enter a goal", "error");
            return;
          }
          generateStrategy(g, ctxRef.current?.value);
        }}
        disabled=${strategyLoading.value}
      >
        ${strategyLoading.value ? "generating..." : "generate strategy"}
      </button>
    </div>

    ${g.length
      ? html`<div class="panel" style="margin-bottom:var(--sp-l)">
          <h3>active goals</h3>
          ${g
            .filter((x) => x.status === "active")
            .map(
              (gl) =>
                html`<div
                  style="padding:var(--sp-xs) 0;border-bottom:1px solid var(--border);display:flex;justify-content:space-between;align-items:center"
                >
                  <div>
                    <strong>${gl.title}</strong
                    ><span
                      style="color:var(--t3);margin-left:var(--sp-s);font-size:var(--fs-s)"
                      >${gl.deadline || ""}</span
                    >
                  </div>
                  <div style="font-size:var(--fs-s);color:var(--t3)">
                    ${gl.projects.join(", ")}
                  </div>
                </div>`,
            )}
        </div>`
      : ""}
    ${strats.length
      ? html`<div class="panel">
          <h3>strategies</h3>
          ${strats.map(
            (s) =>
              html`<div
                style="margin-bottom:var(--sp-l);border:1px solid var(--border);padding:var(--sp-m)"
              >
                <div
                  style="display:flex;justify-content:space-between;align-items:center;margin-bottom:var(--sp-s)"
                >
                  <strong style="font-size:var(--fs-m)">${s.title}</strong>
                  <span
                    style="font-size:var(--fs-s);padding:2px 8px;border:1px solid;color:${s.status ===
                    "done"
                      ? "var(--green)"
                      : s.status === "executing"
                        ? "var(--yellow)"
                        : "var(--t3)"}"
                    >${s.status}</span
                  >
                </div>

                ${s.plans.map(
                  (plan) =>
                    html`<div
                      style="margin:var(--sp-s) 0;padding:var(--sp-s);background:var(--sf)"
                    >
                      <div
                        style="display:flex;justify-content:space-between;align-items:center;margin-bottom:var(--sp-xs)"
                      >
                        <span
                          style="font-family:var(--font-mono);font-weight:600"
                          >${plan.project}</span
                        >
                        <span
                          style="font-size:var(--fs-s);color:${SC_[
                            plan.priority
                          ] || "var(--t3)"}"
                          >${plan.priority}</span
                        >
                      </div>
                      ${plan.depends_on.length
                        ? html`<div
                            style="font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-xs)"
                          >
                            depends on: ${plan.depends_on.join(", ")}
                          </div>`
                        : ""}
                      ${plan.steps.map(
                        (step) =>
                          html`<div
                            style="display:flex;align-items:center;gap:var(--sp-s);padding:var(--sp-xs) 0;font-size:var(--fs-s)"
                          >
                            <input
                              type="checkbox"
                              checked=${step.status === "approved" ||
                              step.status === "done" ||
                              step.status === "running"}
                              disabled=${step.status === "done" ||
                              step.status === "running" ||
                              step.status === "failed"}
                              onChange=${(e) => {
                                const ids = s.plans.flatMap((p) =>
                                  p.steps
                                    .filter(
                                      (st) =>
                                        st.status === "approved" ||
                                        st.status === "done",
                                    )
                                    .map((st) => st.id),
                                );
                                if (e.target.checked) ids.push(step.id);
                                else {
                                  const i = ids.indexOf(step.id);
                                  if (i >= 0) ids.splice(i, 1);
                                }
                                approveSteps(s.id, ids);
                              }}
                            />
                            <span
                              style="color:${step.status === "done"
                                ? "var(--green)"
                                : step.status === "running"
                                  ? "var(--yellow)"
                                  : step.status === "failed"
                                    ? "var(--accent)"
                                    : "var(--t2)"}"
                              >${step.task}</span
                            >
                            ${step.status === "done"
                              ? html`<span
                                  style="color:var(--green);font-size:var(--fs-s)"
                                  >✓</span
                                >`
                              : ""}
                            ${step.status === "running"
                              ? html`<span
                                  style="color:var(--yellow);font-size:var(--fs-s)"
                                  >⏳</span
                                >`
                              : ""}
                            ${step.status === "failed"
                              ? html`<span
                                  style="color:var(--accent);font-size:var(--fs-s)"
                                  >✗</span
                                >`
                              : ""}
                          </div>`,
                      )}
                    </div>`,
                )}
                ${s.status === "approved" || s.status === "executing"
                  ? html`<button
                      class="action-btn"
                      style="margin-top:var(--sp-s);border-color:var(--green);color:var(--green)"
                      onClick=${() => executeNextStep(s.id)}
                    >
                      execute next step
                    </button>`
                  : ""}
                ${s.status === "draft"
                  ? html`<button
                      class="action-btn"
                      style="margin-top:var(--sp-s);border-color:var(--yellow);color:var(--yellow)"
                      onClick=${() => {
                        const allIds = s.plans.flatMap((p) =>
                          p.steps.map((st) => st.id),
                        );
                        approveSteps(s.id, allIds);
                      }}
                    >
                      approve all steps
                    </button>`
                  : ""}
              </div>`,
          )}
        </div>`
      : ""}
  </div>`;
}

function normalizeDuoPanelTab(value) {
  const next = String(value || "collaborate")
    .trim()
    .toLowerCase();
  if (next === "room") return "collaborate";
  if (next === "work" || next === "reviews") return "execute";
  if (next === "execute" || next === "chat") return next;
  return "collaborate";
}

function EmbeddedDualAgentsPanel({ tab = "collaborate" }) {
  const [todoProject, setTodoProject] = useState("");
  const [todoTitle, setTodoTitle] = useState("");
  const [todoTask, setTodoTask] = useState("");
  const [todoAssignee, setTodoAssignee] = useState("agent");
  const [todoWriteIntent, setTodoWriteIntent] = useState("read_only");
  const [todoDeclaredPaths, setTodoDeclaredPaths] = useState("");
  const [todoExecutorProvider, setTodoExecutorProvider] = useState("");
  const [todoReviewerProvider, setTodoReviewerProvider] = useState("");
  const [selectedParallelItems, setSelectedParallelItems] = useState([]);
  const data = dualSessionData.value;
  const session = data?.session || null;
  const events = data?.events || [];
  const participants = session?.participants || [];
  const presence = session?.presence || {};
  const workingSet = session?.current_working_set || [];
  const activeWriters = data?.active_writers || [];
  const activeOrchestrator = data?.active_orchestrator || null;
  const activeOrchestratorId =
    session?.orchestrator_participant_id || activeOrchestrator?.id || "";
  const scope = activeScope.value || {
    kind: session?.project ? "project" : "global",
    label: session?.project ? "Project" : "Global",
    title: session?.project || "_orchestrator",
    breadcrumbs: [
      { kind: "global", label: "Global" },
      ...(session?.project
        ? [{ kind: "project", label: session.project }]
        : []),
    ],
    summary: session?.project
      ? `Duo actions apply to project: ${session.project}`
      : "Duo is operating at global orchestration level.",
  };
  const scopeCrumbs = Array.isArray(scope.breadcrumbs)
    ? scope.breadcrumbs
    : [{ kind: "global", label: "Global" }];
  const writeConflicts = data?.write_conflicts || [];
  const activeLeases = data?.active_leases || [];
  const linkedProjectSessions = data?.linked_project_sessions || [];
  const linkedWorkItems = data?.linked_work_items || [];
  const linkedDelegations = data?.linked_delegations || [];
  const linkedSignals = data?.linked_signals || [];
  const linkedInboxItems = data?.linked_inbox_items || [];
  const parallelBatches = data?.parallel_batches || [];
  const sharedRoomMessages = sideMessages.value.filter(
    (msg) => msg.room_session_id === session?.id,
  );
  const reviewerWorkItems = linkedWorkItems.filter(
    (item) => item.source_kind === "delegation_review",
  );
  const readyWorkItems = linkedWorkItems.filter(
    (item) => item.status === "ready",
  );
  const runningWorkItems = linkedWorkItems.filter(
    (item) => item.status === "running",
  );
  const blockedWorkItems = linkedWorkItems.filter(
    (item) => item.status === "blocked" || item.status === "failed",
  );
  const reviewWarnings = linkedWorkItems.filter(
    (item) => item.review_verdict?.status === "warn",
  );
  const reviewFailures = linkedWorkItems.filter(
    (item) => item.review_verdict?.status === "fail",
  );
  const reviewApprovals = linkedWorkItems.filter(
    (item) => item.review_verdict?.status === "approve",
  );
  const pendingReviews = reviewerWorkItems.filter(
    (item) => !item.review_verdict?.status,
  );
  const pendingApprovalDelegations = linkedDelegations.filter(
    (delegation) => delegation.status === "pending",
  );
  const attentionSignals = linkedSignals.length + linkedInboxItems.length;
  const codex = permData.value?.provider_status?.providers?.codex || {};
  const activeTab = normalizeDuoPanelTab(tab);
  const byId = Object.fromEntries(participants.map((p) => [p.id, p]));
  const activeParticipants = participants.filter((participant) => {
    const state = presence[participant.id] || "idle";
    return state && state !== "idle";
  });
  const latestDisagreement = [...events]
    .reverse()
    .find(
      (evt) =>
        evt.kind === "challenge_requested" || evt.kind === "rebuttal_requested",
    );
  const collaborationStatus = activeParticipants.length
    ? activeParticipants
        .map((participant) => {
          const state = presence[participant.id] || "idle";
          return `${participant.label} ${state}`;
        })
        .join(" · ")
    : "quiet";
  const conflictSummary = writeConflicts.length
    ? `blocked on ${writeConflicts.length} write scope${writeConflicts.length > 1 ? "s" : ""}`
    : activeLeases.length
      ? `${activeLeases.length} active lease${activeLeases.length > 1 ? "s" : ""}`
      : "no write conflicts";
  useEffect(() => {
    if (!todoProject) {
      setTodoProject(session?.project || currentProject.value || "");
    }
  }, [session?.project, currentProject.value]);
  useEffect(() => {
    setSelectedParallelItems((current) =>
      current.filter((id) => linkedWorkItems.some((item) => item.id === id)),
    );
  }, [linkedWorkItems]);
  const clearTodoComposer = () => {
    setTodoProject(session?.project || currentProject.value || "");
    setTodoTitle("");
    setTodoTask("");
    setTodoAssignee("agent");
    setTodoWriteIntent("read_only");
    setTodoDeclaredPaths("");
    setTodoExecutorProvider("");
    setTodoReviewerProvider("");
  };
  const participantLabel = (participantId) =>
    byId[participantId]?.label || participantId || "unknown";
  const latestProjectSessionFor = (project) =>
    linkedProjectSessions.find((item) => item.project === project);
  const parseDeclaredPaths = (raw) => {
    const normalized = [];
    for (const item of String(raw || "")
      .split(/[\n,]/)
      .map((entry) => entry.trim().replaceAll("\\", "/").replace(/^\.\//, ""))
      .filter(Boolean)) {
      if (
        item &&
        !item.startsWith("/") &&
        !item.includes("..") &&
        !normalized.includes(item)
      ) {
        normalized.push(item);
      }
    }
    return normalized;
  };
  const eventBody = (evt) =>
    evt.payload?.response || evt.payload?.message || evt.payload?.summary || "";
  const roomTimeline = (() => {
    const buckets = [];
    const byRound = new Map();
    for (const msg of sharedRoomMessages) {
      const roundId = msg.round_id || "";
      if (!roundId) {
        buckets.push({
          id: `direct-${msg.ts}-${msg.role}-${msg.participant || "user"}`,
          roundId: null,
          messages: [msg],
          startedAt: msg.ts || "",
        });
        continue;
      }
      if (!byRound.has(roundId)) {
        byRound.set(roundId, {
          id: roundId,
          roundId,
          messages: [],
          startedAt: msg.ts || "",
        });
        buckets.push(byRound.get(roundId));
      }
      const bucket = byRound.get(roundId);
      bucket.messages.push(msg);
      if ((msg.ts || "") < bucket.startedAt) bucket.startedAt = msg.ts || "";
    }
    buckets.sort((a, b) =>
      (a.startedAt || "").localeCompare(b.startedAt || ""),
    );
    for (const bucket of buckets) {
      bucket.messages.sort((a, b) => (a.ts || "").localeCompare(b.ts || ""));
    }
    return buckets;
  })();
  const toneFor = (evt) => {
    if (evt.actor === "user") return "var(--text)";
    if (evt.actor === "system") return "var(--t3)";
    if (evt.kind === "challenge_requested" || evt.kind === "rebuttal_requested")
      return "var(--yellow)";
    if (evt.kind === "gate_result" || evt.kind === "review_verdict")
      return "var(--accent)";
    if (evt.kind === "delegation_linked" || evt.kind === "plan_linked")
      return "var(--green)";
    return "var(--cyan)";
  };
  const shortTs = (value) => {
    const ts = String(value || "").trim();
    if (!ts) return "";
    if (ts.includes("T") && ts.length >= 16) return ts.slice(11, 16);
    return ts.slice(0, 16);
  };
  const compactText = (value, max = 140) => {
    const text = String(value || "")
      .replace(/\s+/g, " ")
      .trim();
    if (!text) return "";
    return text.length > max ? text.slice(0, max - 3) + "..." : text;
  };
  const eventKindLabel = (kind) => {
    switch (kind) {
      case "agent_presence":
        return "presence";
      case "challenge_requested":
        return "challenge";
      case "rebuttal_requested":
        return "rebuttal";
      case "review_verdict":
        return "review";
      case "gate_result":
        return "gate";
      case "delegation_linked":
        return "delegation";
      case "plan_linked":
        return "plan";
      case "lease_acquired":
        return "lease";
      case "lease_released":
        return "release";
      case "session_created":
        return "session";
      default:
        return String(kind || "event").replaceAll("_", " ");
    }
  };
  const presenceTone = (state) => {
    if (state === "replying") return "var(--green)";
    if (state === "thinking") return "var(--yellow)";
    if (state === "blocked") return "var(--accent)";
    return "var(--t3)";
  };
  const workTone = (status) => {
    if (status === "running") return "var(--green)";
    if (status === "ready") return "var(--cyan)";
    if (status === "blocked" || status === "failed") return "var(--accent)";
    if (status === "done") return "var(--green)";
    return "var(--t3)";
  };
  const reviewTone = (status) => {
    if (status === "approve") return "var(--green)";
    if (status === "warn") return "var(--yellow)";
    if (status === "fail") return "var(--accent)";
    return "var(--t3)";
  };
  const participantTelemetry = participants.map((participant) => {
    const provider = participant.provider || "";
    const state = presence[participant.id] || "idle";
    const canWrite = activeWriters.some(
      (writer) => writer.participant_id === participant.id,
    );
    const executorItems = linkedWorkItems.filter(
      (item) => item.executor_provider === provider,
    );
    const reviewerItems = linkedWorkItems.filter(
      (item) => item.reviewer_provider === provider,
    );
    const recentEvent =
      [...events].reverse().find((evt) => evt.actor === participant.id) || null;
    const recentMessage =
      [...sharedRoomMessages]
        .reverse()
        .find((msg) => msg.participant === participant.id) || null;
    const lastEventTs = recentEvent?.ts || "";
    const lastMessageTs = recentMessage?.ts || "";
    const preferMessage =
      lastMessageTs && (!lastEventTs || lastMessageTs >= lastEventTs);
    return {
      participant,
      provider,
      state,
      canWrite,
      readyCount: executorItems.filter((item) => item.status === "ready")
        .length,
      runningCount: executorItems.filter((item) => item.status === "running")
        .length,
      blockedCount: executorItems.filter(
        (item) => item.status === "blocked" || item.status === "failed",
      ).length,
      reviewCount: reviewerItems.filter((item) => !item.review_verdict?.status)
        .length,
      eventCount: events.filter((evt) => evt.actor === participant.id).length,
      messageCount: sharedRoomMessages.filter(
        (msg) => msg.participant === participant.id,
      ).length,
      lastLabel: preferMessage
        ? "last reply"
        : recentEvent
          ? eventKindLabel(recentEvent.kind)
          : "waiting",
      lastSummary: preferMessage
        ? compactText(recentMessage?.msg || "", 120)
        : compactText(
            (recentEvent ? eventBody(recentEvent) : "") ||
              recentEvent?.payload?.summary ||
              "",
            120,
          ),
      lastTs: preferMessage ? lastMessageTs : lastEventTs,
    };
  });
  const activityFeed = [
    ...events.map((evt) => ({
      id: evt.id,
      ts: evt.ts || "",
      actor:
        evt.actor === "user"
          ? "You"
          : evt.actor === "system"
            ? "System"
            : participantLabel(evt.actor),
      label: eventKindLabel(evt.kind),
      summary: compactText(
        eventBody(evt) || JSON.stringify(evt.payload || {}),
        160,
      ),
      tone: toneFor(evt),
      surface: "event",
    })),
    ...sharedRoomMessages.map((msg) => ({
      id: `msg-${msg.ts || ""}-${msg.role}-${msg.participant || "user"}`,
      ts: msg.ts || "",
      actor: msg.role === "user" ? "You" : participantLabel(msg.participant),
      label: msg.role === "user" ? "instruction" : "reply",
      summary: compactText(msg.msg || "", 160),
      tone: msg.role === "user" ? "var(--text)" : "var(--cyan)",
      surface: msg.round_id ? "round" : "direct",
    })),
  ]
    .filter((entry) => entry.summary)
    .sort((a, b) => (b.ts || "").localeCompare(a.ts || ""))
    .slice(0, 12);
  const pipelineCounts = [
    { label: "ready", count: readyWorkItems.length, color: "var(--cyan)" },
    { label: "running", count: runningWorkItems.length, color: "var(--green)" },
    { label: "review", count: pendingReviews.length, color: "var(--yellow)" },
    {
      label: "approved",
      count: reviewApprovals.length,
      color: "var(--green)",
    },
    {
      label: "risk",
      count:
        reviewWarnings.length + reviewFailures.length + blockedWorkItems.length,
      color: "var(--accent)",
    },
  ];
  const executionHighlights = [
    ...linkedWorkItems
      .filter(
        (item) =>
          item.status === "running" ||
          item.status === "blocked" ||
          item.status === "failed" ||
          !!item.review_verdict?.status,
      )
      .map((item) => ({
        id: `work-${item.id}`,
        ts: item.updated_at || item.created_at || "",
        actor:
          item.executor_provider ||
          item.reviewer_provider ||
          item.assignee ||
          "task",
        label: item.review_verdict?.status
          ? `review ${item.review_verdict.status}`
          : item.status,
        summary: compactText(item.title || item.task || item.id, 80),
        detail: compactText(
          item.review_verdict?.summary || item.result || item.task || "",
          140,
        ),
        tone: item.review_verdict?.status
          ? reviewTone(item.review_verdict.status)
          : workTone(item.status),
      })),
    ...linkedSignals.map((signal, index) => ({
      id: `signal-${index}`,
      ts: signal.created_at || "",
      actor: signal.source || "signal",
      label: signal.level || "signal",
      summary: compactText(signal.title || signal.message || "signal", 80),
      detail: compactText(signal.message || signal.title || "", 140),
      tone:
        signal.level === "error" || signal.level === "critical"
          ? "var(--accent)"
          : "var(--yellow)",
    })),
    ...linkedInboxItems.map((item, index) => ({
      id: `inbox-${index}`,
      ts: item.created_at || item.ts || "",
      actor: item.project || "inbox",
      label: "attention",
      summary: compactText(item.message || "inbox item", 80),
      detail: compactText(item.message || "", 140),
      tone: "var(--accent)",
    })),
    ...activeLeases.map((lease) => ({
      id: `lease-${lease.id}`,
      ts: lease.updated_at || lease.created_at || "",
      actor: participantLabel(lease.participant_id),
      label: "lease",
      summary: compactText(lease.work_item_id || lease.id, 80),
      detail: compactText((lease.paths || []).join(", ") || "no paths", 140),
      tone: "var(--yellow)",
    })),
    ...parallelBatches.map((batch) => ({
      id: `batch-${batch.batch_id}`,
      ts: batch.updated_at || batch.created_at || "",
      actor: "parallel",
      label: batch.status || "batch",
      summary: compactText(batch.batch_id || "safe parallel", 80),
      detail: `queued ${batch.pending || 0} | running ${batch.running || 0} | done ${batch.done || 0} | failed ${(batch.failed || 0) + (batch.rejected || 0)}`,
      tone: batch.failed || batch.rejected ? "var(--accent)" : "var(--cyan)",
    })),
  ]
    .sort((a, b) => (b.ts || "").localeCompare(a.ts || ""))
    .slice(0, 10);
  const createTodo = async () => {
    const project = (
      todoProject ||
      session?.project ||
      currentProject.value ||
      ""
    ).trim();
    const task = todoTask.trim();
    const declaredPaths = parseDeclaredPaths(todoDeclaredPaths);
    if (!project) {
      showToast("Set a project for the work item", "error");
      return;
    }
    if (!task) {
      showToast("Enter task details", "error");
      return;
    }
    if (todoWriteIntent !== "read_only" && !declaredPaths.length) {
      showToast("Write work items require declared paths", "error");
      return;
    }
    dualBusy.value = "work-item:create";
    try {
      const linkedProjectSession = latestProjectSessionFor(project);
      await createRoomWorkItem({
        project,
        title: todoTitle.trim(),
        task,
        assignee: todoAssignee,
        writeIntent: todoWriteIntent,
        declaredPaths,
        verify: null,
        sessionId: activeDualSession.value || null,
        projectSessionId: linkedProjectSession?.id || null,
        executorProvider: todoExecutorProvider || null,
        reviewerProvider: todoReviewerProvider || null,
      });
      if (activeDualSession.value)
        await loadDualSession(activeDualSession.value);
      clearTodoComposer();
    } catch (e) {
      showToast("Create work item error: " + e, "error");
    } finally {
      dualBusy.value = "";
    }
  };
  const queueExistingWorkItem = async (item) => {
    dualBusy.value = "work-item:queue:" + item.id;
    try {
      await queueWorkItemExecution(item.id);
      if (activeDualSession.value)
        await loadDualSession(activeDualSession.value);
    } catch (e) {
      showToast("Queue work item error: " + e, "error");
    } finally {
      dualBusy.value = "";
    }
  };
  const completeExistingUserWorkItem = async (item) => {
    const note = window.prompt(
      "Completion note for user work item",
      item.result || "",
    );
    if (note === null) return;
    dualBusy.value = "work-item:complete:" + item.id;
    try {
      await completeUserWorkItem(item.id, note);
      if (activeDualSession.value)
        await loadDualSession(activeDualSession.value);
    } catch (e) {
      showToast("Complete user work item error: " + e, "error");
    } finally {
      dualBusy.value = "";
    }
  };
  const toggleParallelItem = (itemId) => {
    setSelectedParallelItems((current) =>
      current.includes(itemId)
        ? current.filter((id) => id !== itemId)
        : [...current, itemId],
    );
  };
  const queueParallelBatch = async () => {
    if (!activeDualSession.value || selectedParallelItems.length < 2) {
      showToast("Select at least two queueable work items", "error");
      return;
    }
    dualBusy.value = "parallel:queue";
    try {
      await queueParallelWorkItems(
        activeDualSession.value,
        selectedParallelItems,
      );
      setSelectedParallelItems([]);
      if (activeDualSession.value)
        await loadDualSession(activeDualSession.value);
    } catch (e) {
      showToast("Parallel batch error: " + e, "error");
    } finally {
      dualBusy.value = "";
    }
  };
  const queueProviderRound = async (provider) => {
    if (!activeDualSession.value) return;
    dualBusy.value = "parallel:provider:" + provider;
    try {
      await queueProviderParallelRound(activeDualSession.value, provider);
      setSelectedParallelItems([]);
      if (activeDualSession.value)
        await loadDualSession(activeDualSession.value);
    } catch (e) {
      showToast(provider + " round error: " + e, "error");
    } finally {
      dualBusy.value = "";
    }
  };
  const grantWriter = async (participant) => {
    dualBusy.value = "writer:" + participant.id;
    try {
      await setDualWriter(activeDualSession.value || null, participant.id);
    } catch (e) {
      showToast("Set writer error: " + e, "error");
    } finally {
      dualBusy.value = "";
    }
  };
  const setOrchestrator = async (participant) => {
    dualBusy.value = "orchestrator:" + participant.id;
    try {
      await setDualOrchestrator(
        activeDualSession.value || null,
        participant.id,
      );
    } catch (e) {
      showToast("Set orchestrator error: " + e, "error");
    } finally {
      dualBusy.value = "";
    }
  };
  const revokeWriter = async (participant) => {
    dualBusy.value = "writer:revoke:" + participant.id;
    try {
      await revokeDualWriter(activeDualSession.value || null, participant.id);
    } catch (e) {
      showToast("Revoke write error: " + e, "error");
    } finally {
      dualBusy.value = "";
    }
  };
  const acquireLease = async (item) => {
    dualBusy.value = "lease:acquire:" + item.id;
    try {
      await acquireWorkItemLease(item.id, null);
      if (activeDualSession.value)
        await loadDualSession(activeDualSession.value);
    } catch (e) {
      showToast("Acquire lease error: " + e, "error");
    } finally {
      dualBusy.value = "";
    }
  };
  const releaseLease = async (lease, force = false) => {
    dualBusy.value = (force ? "lease:force:" : "lease:release:") + lease.id;
    try {
      await releaseFileLease(lease.id, force);
      if (activeDualSession.value)
        await loadDualSession(activeDualSession.value);
    } catch (e) {
      showToast(
        (force ? "Force release" : "Release") + " lease error: " + e,
        "error",
      );
    } finally {
      dualBusy.value = "";
    }
  };
  if (!session) {
    return html`<div class="panel" style="margin:var(--sp-s);color:var(--t3)">
      No active duo session for this context.
    </div>`;
  }
  if (activeTab === "collaborate") {
    return html`<div style="display:flex;flex-direction:column;gap:var(--sp-s)">
      <div class="panel">
        <div class="scope-strip" style="margin-bottom:var(--sp-s)">
          <div class="scope-path">
            ${scopeCrumbs.map(
              (crumb, index) =>
                html`<span class="scope-crumb">
                  ${index > 0 ? html`<b>/</b>` : null}${crumb.label}
                </span>`,
            )}
          </div>
          <span class="scope-kind">${scope.label || scope.kind}</span>
        </div>
        <div
          style="display:flex;justify-content:space-between;gap:var(--sp-s);align-items:flex-start;flex-wrap:wrap"
        >
          <div>
            <div
              style="font-family:var(--font-mono);font-size:var(--fs-s);color:var(--cyan)"
            >
              collaborate
            </div>
            <div style="font-size:var(--fs-s);color:var(--t3)">
              visible second-agent workflow for
              ${scope.title || session.project || "_orchestrator"}
            </div>
          </div>
          <div
            style="font-size:var(--fs-s);color:var(--t3);max-width:320px;text-align:right"
          >
            Use the shared composer in Collaborate for ask both, challenge, and
            secondary actions. The same history continues in Chat mode.
          </div>
        </div>
        <div
          style="display:flex;gap:var(--sp-xs);flex-wrap:wrap;margin-top:var(--sp-s)"
        >
          <button
            class="action-btn"
            onClick=${() => {
              activeRoomTab.value = "chat";
            }}
          >
            back to chat
          </button>
          <button
            class="action-btn"
            onClick=${() => {
              activeRoomTab.value = "execute";
            }}
          >
            open execute
          </button>
          <button
            class="action-btn"
            onClick=${() => {
              chatCollabMode.value = "solo";
              activeRoomTab.value = "chat";
            }}
          >
            switch to solo
          </button>
        </div>
        <div
          style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:var(--sp-xs);margin-top:var(--sp-s)"
        >
          ${[
            ["status", collaborationStatus, "var(--cyan)"],
            [
              "working set",
              workingSet.length
                ? `${workingSet.length} path${workingSet.length > 1 ? "s" : ""}`
                : "empty",
              "var(--yellow)",
            ],
            [
              "disagreement",
              latestDisagreement?.payload?.summary || "none",
              latestDisagreement ? "var(--yellow)" : "var(--t3)",
            ],
            [
              "conflicts",
              conflictSummary,
              writeConflicts.length ? "var(--accent)" : "var(--t3)",
            ],
            [
              "codex",
              codex.ready ? "ready" : "needs setup",
              codex.ready ? "var(--green)" : "var(--accent)",
            ],
          ].map(
            ([label, value, color]) =>
              html`<div
                style="padding:var(--sp-s);border:1px solid var(--border);background:var(--bg-soft)"
              >
                <div
                  style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                >
                  ${label}
                </div>
                <div
                  style="font-size:var(--fs-s);color:${color};margin-top:4px;white-space:pre-wrap"
                >
                  ${value}
                </div>
              </div>`,
          )}
        </div>
        <div
          style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:var(--sp-xs);margin-top:var(--sp-s)"
        >
          ${participantTelemetry.map((entry) => {
            return html`<div
              style="padding:var(--sp-s);border:1px solid var(--border);background:var(--bg-soft)"
            >
              <div
                style="display:flex;justify-content:space-between;gap:var(--sp-xs);align-items:flex-start"
              >
                <div style="font-size:var(--fs-s);color:var(--text)">
                  ${entry.participant.label}
                </div>
                <div
                  style="font-size:var(--fs-s);color:${presenceTone(
                    entry.state,
                  )};font-family:var(--font-mono)"
                >
                  ${entry.lastTs ? shortTs(entry.lastTs) : "waiting"}
                </div>
              </div>
              <div
                style="font-size:var(--fs-s);color:${presenceTone(
                  entry.state,
                )};font-family:var(--font-mono)"
              >
                ${entry.provider} |
                ${entry.state}${entry.canWrite ? " | writer" : ""}
              </div>
              <div
                style="display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:6px;margin-top:var(--sp-xs)"
              >
                ${[
                  ["ready", entry.readyCount, "var(--cyan)"],
                  ["running", entry.runningCount, "var(--green)"],
                  ["review", entry.reviewCount, "var(--yellow)"],
                  ["blocked", entry.blockedCount, "var(--accent)"],
                ].map(
                  ([label, count, color]) =>
                    html`<div
                      style="padding:6px;border:1px solid var(--border);background:var(--bg)"
                    >
                      <div
                        style="font-size:11px;color:var(--t3);font-family:var(--font-mono)"
                      >
                        ${label}
                      </div>
                      <div
                        style="font-size:var(--fs-s);color:${color};font-family:var(--font-mono)"
                      >
                        ${count}
                      </div>
                    </div>`,
                )}
              </div>
              <div
                style="margin-top:var(--sp-xs);padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg)"
              >
                <div
                  style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                >
                  ${entry.lastLabel}
                </div>
                <div
                  style="margin-top:4px;font-size:var(--fs-s);color:var(--text);white-space:pre-wrap"
                >
                  ${entry.lastSummary || "No visible activity yet."}
                </div>
              </div>
              <div
                style="display:flex;gap:6px;flex-wrap:wrap;margin-top:var(--sp-xs)"
              >
                ${[
                  `${entry.messageCount} repl${entry.messageCount === 1 ? "y" : "ies"}`,
                  `${entry.eventCount} event${entry.eventCount === 1 ? "" : "s"}`,
                ].map(
                  (label) =>
                    html`<span
                      style="padding:3px 8px;border:1px solid var(--border);background:var(--bg);font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                    >
                      ${label}
                    </span>`,
                )}
              </div>
            </div>`;
          })}
        </div>
        <div
          style="margin-top:var(--sp-s);padding:var(--sp-s);border:1px dashed var(--border);background:var(--bg-soft);font-size:var(--fs-s);color:var(--t3)"
        >
          Collaborate is now a grouped view of the same shared thread. Chat and
          Collaborate no longer split history.
        </div>
      </div>
      <div class="panel">
        <div
          style="font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
        >
          activity stream
        </div>
        <div
          style="display:flex;flex-direction:column;gap:var(--sp-xs);max-height:320px;overflow:auto"
        >
          ${activityFeed.length
            ? activityFeed.map(
                (entry) =>
                  html`<div
                    style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                  >
                    <div
                      style="display:flex;justify-content:space-between;gap:var(--sp-xs);align-items:flex-start"
                    >
                      <div
                        style="font-size:var(--fs-s);color:${entry.tone};font-family:var(--font-mono)"
                      >
                        ${entry.actor} | ${entry.label}
                      </div>
                      <div
                        style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                      >
                        ${shortTs(entry.ts)}${entry.surface
                          ? ` | ${entry.surface}`
                          : ""}
                      </div>
                    </div>
                    <div
                      style="margin-top:4px;font-size:var(--fs-s);color:var(--text);white-space:pre-wrap"
                    >
                      ${entry.summary}
                    </div>
                  </div>`,
              )
            : html`<div style="font-size:var(--fs-s);color:var(--t3)">
                No activity yet.
              </div>`}
        </div>
      </div>
      ${workingSet.length
        ? html`<details class="panel">
            <summary
              style="cursor:pointer;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow)"
            >
              working set
            </summary>
            <div
              style="display:flex;flex-wrap:wrap;gap:var(--sp-xs);margin-top:var(--sp-s)"
            >
              ${workingSet.map(
                (path) =>
                  html`<span
                    style="padding:4px 8px;border:1px solid var(--border);background:var(--bg-soft);font-size:var(--fs-s);font-family:var(--font-mono)"
                  >
                    ${path}
                  </span>`,
              )}
            </div>
          </details>`
        : null}
      <div class="panel">
        <div
          style="font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
        >
          room feed
        </div>
        <div
          style="display:flex;flex-direction:column;gap:var(--sp-xs);max-height:420px;overflow:auto"
        >
          ${roomTimeline.length
            ? roomTimeline.map((bucket, idx) => {
                const title = bucket.roundId
                  ? `round ${idx + 1}`
                  : "direct turn";
                return html`<div
                  style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                >
                  <div
                    style="display:flex;justify-content:space-between;gap:var(--sp-xs);font-size:var(--fs-s);font-family:var(--font-mono);color:var(--yellow)"
                  >
                    <span>${title}</span>
                    <span>${bucket.startedAt || ""}</span>
                  </div>
                  <div
                    style="display:flex;flex-direction:column;gap:var(--sp-xs);margin-top:var(--sp-xs)"
                  >
                    ${bucket.messages.map((msg) => {
                      const actor =
                        msg.role === "user"
                          ? "You"
                          : (msg.meta || "")
                              .replace(/^\s*[^A-Za-z0-9]+/, "")
                              .trim() || participantLabel(msg.participant);
                      const color =
                        msg.role === "user" ? "var(--text)" : "var(--cyan)";
                      return html`<div
                        style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg);"
                      >
                        <div
                          style="display:flex;justify-content:space-between;gap:var(--sp-xs);font-size:var(--fs-s);font-family:var(--font-mono);color:${color}"
                        >
                          <span>${actor}</span>
                          <span>${msg.role}</span>
                        </div>
                        <div
                          style="margin-top:4px;font-size:var(--fs-s);color:var(--text);white-space:pre-wrap"
                        >
                          ${msg.msg || ""}
                        </div>
                      </div>`;
                    })}
                  </div>
                </div>`;
              })
            : html`<div style="font-size:var(--fs-s);color:var(--t3)">
                No collaboration rounds yet.
              </div>`}
        </div>
      </div>
      ${events.length
        ? html`<details class="panel">
            <summary
              style="cursor:pointer;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--t3)"
            >
              advanced: raw room events
            </summary>
            <div
              style="display:flex;flex-direction:column;gap:var(--sp-xs);max-height:280px;overflow:auto;margin-top:var(--sp-s)"
            >
              ${events.slice(-40).map((evt) => {
                const actor =
                  evt.actor === "user"
                    ? "You"
                    : evt.actor === "system"
                      ? "System"
                      : participantLabel(evt.actor);
                return html`<div
                  style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                >
                  <div
                    style="display:flex;justify-content:space-between;gap:var(--sp-xs);font-size:var(--fs-s);font-family:var(--font-mono);color:${toneFor(
                      evt,
                    )}"
                  >
                    <span>${actor}</span>
                    <span>${evt.kind}</span>
                  </div>
                  <div
                    style="margin-top:4px;font-size:var(--fs-s);color:var(--text);white-space:pre-wrap"
                  >
                    ${eventBody(evt) || JSON.stringify(evt.payload || {})}
                  </div>
                </div>`;
              })}
            </div>
          </details>`
        : null}
    </div>`;
  }
  if (activeTab === "execute") {
    return html`<div style="display:flex;flex-direction:column;gap:var(--sp-s)">
      <div class="panel">
        <div class="scope-strip" style="margin-bottom:var(--sp-s)">
          <div class="scope-path">
            ${scopeCrumbs.map(
              (crumb, index) =>
                html`<span class="scope-crumb">
                  ${index > 0 ? html`<b>/</b>` : null}${crumb.label}
                </span>`,
            )}
          </div>
          <span class="scope-kind">${scope.label || scope.kind}</span>
        </div>
        <div
          style="display:flex;justify-content:space-between;gap:var(--sp-s);align-items:flex-start;flex-wrap:wrap"
        >
          <div>
            <div
              style="font-family:var(--font-mono);font-size:var(--fs-s);color:var(--cyan)"
            >
              execute
            </div>
            <div style="font-size:var(--fs-s);color:var(--t3)">
              run work, inspect results, and open low-level controls only when
              needed
            </div>
          </div>
          <div style="display:flex;gap:var(--sp-xs);flex-wrap:wrap">
            <button
              class="action-btn"
              disabled=${!!dualBusy.value}
              onClick=${() => queueProviderRound("claude")}
            >
              run all Claude tasks
            </button>
            <button
              class="action-btn"
              disabled=${!!dualBusy.value}
              onClick=${() => queueProviderRound("codex")}
            >
              run all Codex tasks
            </button>
            <button
              class="action-btn"
              disabled=${!!dualBusy.value}
              onClick=${queueParallelBatch}
            >
              run safe
              parallel${selectedParallelItems.length
                ? ` (${selectedParallelItems.length})`
                : ""}
            </button>
          </div>
        </div>
        <div
          style="display:grid;grid-template-columns:repeat(auto-fit,minmax(120px,1fr));gap:var(--sp-xs);margin-top:var(--sp-s)"
        >
          ${[
            ["ready", readyWorkItems.length, "var(--cyan)"],
            ["running", runningWorkItems.length, "var(--green)"],
            ["blocked", blockedWorkItems.length, "var(--accent)"],
            ["warn", reviewWarnings.length, "var(--yellow)"],
            ["fail", reviewFailures.length, "var(--accent)"],
          ].map(
            ([label, count, color]) =>
              html`<div
                style="padding:var(--sp-s);border:1px solid var(--border);background:var(--bg-soft)"
              >
                <div
                  style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                >
                  ${label}
                </div>
                <div
                  style="font-size:var(--fs-l);color:${color};font-family:var(--font-mono)"
                >
                  ${count}
                </div>
              </div>`,
          )}
        </div>
        <div style="margin-top:var(--sp-s)">
          <div
            style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
          >
            execution pulse
          </div>
          <div
            style="display:flex;gap:6px;align-items:stretch;margin-top:var(--sp-xs)"
          >
            ${pipelineCounts.map(
              (segment) =>
                html`<div
                  style="flex:${Math.max(
                    segment.count,
                    1,
                  )};min-height:14px;border:1px solid var(--border);background:${segment.count
                    ? segment.color
                    : "var(--bg)"};opacity:${segment.count ? 1 : 0.2}"
                  title=${`${segment.label}: ${segment.count}`}
                ></div>`,
            )}
          </div>
          <div
            style="display:flex;gap:6px;flex-wrap:wrap;margin-top:var(--sp-xs)"
          >
            ${pipelineCounts.map(
              (segment) =>
                html`<span
                  style="padding:3px 8px;border:1px solid var(--border);background:var(--bg-soft);font-size:var(--fs-s);color:${segment.color};font-family:var(--font-mono)"
                >
                  ${segment.label} ${segment.count}
                </span>`,
            )}
          </div>
        </div>
        <div
          style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:var(--sp-xs);margin-top:var(--sp-s)"
        >
          ${participantTelemetry.map(
            (entry) =>
              html`<div
                style="padding:var(--sp-s);border:1px solid var(--border);background:var(--bg-soft)"
              >
                <div
                  style="display:flex;justify-content:space-between;gap:var(--sp-xs);align-items:flex-start"
                >
                  <div style="font-size:var(--fs-s);color:var(--text)">
                    ${entry.participant.label}
                  </div>
                  <div
                    style="font-size:var(--fs-s);color:${presenceTone(
                      entry.state,
                    )};font-family:var(--font-mono)"
                  >
                    ${entry.state}
                  </div>
                </div>
                <div
                  style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                >
                  ${entry.provider}${entry.canWrite
                    ? " | writer"
                    : ""}${entry.lastTs ? " | " + shortTs(entry.lastTs) : ""}
                </div>
                <div
                  style="display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:6px;margin-top:var(--sp-xs)"
                >
                  ${[
                    ["ready", entry.readyCount, "var(--cyan)"],
                    ["running", entry.runningCount, "var(--green)"],
                    ["review", entry.reviewCount, "var(--yellow)"],
                    ["blocked", entry.blockedCount, "var(--accent)"],
                  ].map(
                    ([label, count, color]) =>
                      html`<div
                        style="padding:6px;border:1px solid var(--border);background:var(--bg)"
                      >
                        <div
                          style="font-size:11px;color:var(--t3);font-family:var(--font-mono)"
                        >
                          ${label}
                        </div>
                        <div
                          style="font-size:var(--fs-s);color:${color};font-family:var(--font-mono)"
                        >
                          ${count}
                        </div>
                      </div>`,
                  )}
                </div>
              </div>`,
          )}
        </div>
        ${writeConflicts.length
          ? html`<div
              style="margin-top:var(--sp-s);padding:var(--sp-s);border:1px solid var(--accent);background:var(--accent-dim);font-size:var(--fs-s);color:var(--accent)"
            >
              blocked by write conflict on ${writeConflicts.length}
              scope${writeConflicts.length > 1 ? "s" : ""}
            </div>`
          : null}
        <details style="margin-top:var(--sp-s)">
          <summary
            style="cursor:pointer;font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
          >
            advanced runtime controls
          </summary>
          <div
            style="margin-top:var(--sp-s);font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
          >
            active orchestrator: ${activeOrchestrator?.label || "unassigned"}
          </div>
          ${writeConflicts.length
            ? html`<div
                style="margin-top:var(--sp-s);padding:var(--sp-s);border:1px solid var(--accent);background:var(--accent-dim);font-size:var(--fs-s);color:var(--accent)"
              >
                ${writeConflicts
                  .map(
                    (conflict) =>
                      `${conflict.left?.title || conflict.left?.id} ↔ ${conflict.right?.title || conflict.right?.id} on ${(conflict.paths || []).join(", ")}`,
                  )
                  .join(" ; ")}
              </div>`
            : null}
          <div
            style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:var(--sp-xs);margin-top:var(--sp-s)"
          >
            ${participants.map((participant) => {
              const canWrite = activeWriters.some(
                (writer) => writer.participant_id === participant.id,
              );
              const isOrchestrator = activeOrchestratorId === participant.id;
              return html`<div
                style="padding:var(--sp-s);border:1px solid var(--border);background:var(--bg-soft)"
              >
                <div style="font-size:var(--fs-s);color:var(--text)">
                  ${participant.label}
                </div>
                <div
                  style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                >
                  ${participant.provider}${canWrite ? " · write enabled" : ""}
                </div>
                <div
                  style="display:flex;gap:var(--sp-xs);flex-wrap:wrap;margin-top:var(--sp-xs)"
                >
                  <button
                    class="action-btn"
                    disabled=${!!dualBusy.value || isOrchestrator}
                    onClick=${() => setOrchestrator(participant)}
                  >
                    ${isOrchestrator ? "orchestrator" : "make orchestrator"}
                  </button>
                  <button
                    class="action-btn"
                    disabled=${!!dualBusy.value}
                    onClick=${() => grantWriter(participant)}
                  >
                    grant
                  </button>
                  <button
                    class="action-btn"
                    disabled=${!!dualBusy.value}
                    onClick=${() => revokeWriter(participant)}
                  >
                    revoke
                  </button>
                </div>
              </div>`;
            })}
          </div>
        </details>
      </div>
      <div class="panel">
        <div
          style="font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
        >
          execution watch
        </div>
        <div
          style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:var(--sp-xs)"
        >
          <div
            style="padding:var(--sp-s);border:1px solid var(--border);background:var(--bg-soft)"
          >
            <div
              style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
            >
              write watch
            </div>
            <div
              style="margin-top:4px;font-size:var(--fs-s);color:${writeConflicts.length
                ? "var(--accent)"
                : activeLeases.length
                  ? "var(--yellow)"
                  : "var(--green)"};white-space:pre-wrap"
            >
              ${writeConflicts.length
                ? writeConflicts
                    .slice(0, 3)
                    .map(
                      (conflict) =>
                        `${conflict.left?.title || conflict.left?.id} vs ${conflict.right?.title || conflict.right?.id}`,
                    )
                    .join("\n")
                : activeLeases.length
                  ? activeLeases
                      .slice(0, 3)
                      .map(
                        (lease) =>
                          `${participantLabel(lease.participant_id)} -> ${(lease.paths || []).join(", ") || lease.work_item_id}`,
                      )
                      .join("\n")
                  : "No active write contention."}
            </div>
          </div>
          <div
            style="padding:var(--sp-s);border:1px solid var(--border);background:var(--bg-soft)"
          >
            <div
              style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
            >
              review watch
            </div>
            <div
              style="display:flex;flex-direction:column;gap:6px;margin-top:var(--sp-xs)"
            >
              ${(reviewFailures.length
                ? reviewFailures
                : reviewWarnings.length
                  ? reviewWarnings
                  : reviewApprovals
              )
                .slice(0, 3)
                .map(
                  (item) =>
                    html`<div
                      style="padding:6px;border:1px solid var(--border);background:var(--bg)"
                    >
                      <div style="font-size:var(--fs-s);color:var(--text)">
                        ${compactText(item.title || item.id, 72)}
                      </div>
                      <div
                        style="font-size:var(--fs-s);color:${reviewTone(
                          item.review_verdict?.status,
                        )};font-family:var(--font-mono)"
                      >
                        ${item.review_verdict?.status || item.status}
                      </div>
                      ${item.review_verdict?.summary
                        ? html`<div
                            style="font-size:var(--fs-s);color:var(--t3);margin-top:4px"
                          >
                            ${compactText(item.review_verdict.summary, 96)}
                          </div>`
                        : null}
                    </div>`,
                )}
              ${!reviewFailures.length &&
              !reviewWarnings.length &&
              !reviewApprovals.length
                ? html`<div style="font-size:var(--fs-s);color:var(--t3)">
                    No review outcomes yet.
                  </div>`
                : null}
            </div>
          </div>
          <div
            style="padding:var(--sp-s);border:1px solid var(--border);background:var(--bg-soft)"
          >
            <div
              style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
            >
              attention
            </div>
            <div
              style="display:flex;flex-direction:column;gap:6px;margin-top:var(--sp-xs)"
            >
              ${executionHighlights.length
                ? executionHighlights.slice(0, 4).map(
                    (entry) =>
                      html`<div
                        style="padding:6px;border:1px solid var(--border);background:var(--bg)"
                      >
                        <div
                          style="display:flex;justify-content:space-between;gap:var(--sp-xs)"
                        >
                          <div
                            style="font-size:var(--fs-s);color:${entry.tone};font-family:var(--font-mono)"
                          >
                            ${entry.actor} | ${entry.label}
                          </div>
                          <div
                            style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                          >
                            ${shortTs(entry.ts)}
                          </div>
                        </div>
                        <div
                          style="margin-top:4px;font-size:var(--fs-s);color:var(--text)"
                        >
                          ${entry.summary}
                        </div>
                        ${entry.detail
                          ? html`<div
                              style="margin-top:4px;font-size:var(--fs-s);color:var(--t3)"
                            >
                              ${entry.detail}
                            </div>`
                          : null}
                      </div>`,
                  )
                : html`<div style="font-size:var(--fs-s);color:var(--t3)">
                    No execution signals yet.
                  </div>`}
            </div>
          </div>
        </div>
      </div>
      <details class="panel">
        <summary
          style="cursor:pointer;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
        >
          create task
        </summary>
        <div
          style="display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:var(--sp-xs)"
        >
          <input
            value=${todoProject}
            onInput=${(e) => setTodoProject(e.currentTarget.value)}
            placeholder=${session?.project ||
            currentProject.value ||
            "_orchestrator"}
            style="background:var(--sf);border:1px solid var(--border);color:var(--text);padding:10px;font-family:var(--font-mono)"
          />
          <input
            value=${todoTitle}
            onInput=${(e) => setTodoTitle(e.currentTarget.value)}
            placeholder="title"
            style="background:var(--sf);border:1px solid var(--border);color:var(--text);padding:10px"
          />
          <select
            value=${todoAssignee}
            onInput=${(e) => setTodoAssignee(e.currentTarget.value)}
            style="background:var(--sf);border:1px solid var(--border);color:var(--text);padding:10px"
          >
            <option value="agent">agent</option>
            <option value="user">user</option>
          </select>
          <select
            value=${todoWriteIntent}
            onInput=${(e) => setTodoWriteIntent(e.currentTarget.value)}
            style="background:var(--sf);border:1px solid var(--border);color:var(--text);padding:10px"
          >
            <option value="read_only">read only</option>
            <option value="propose_write">propose write</option>
            <option value="exclusive_write">exclusive write</option>
          </select>
          <select
            value=${todoExecutorProvider}
            onInput=${(e) => setTodoExecutorProvider(e.currentTarget.value)}
            style="background:var(--sf);border:1px solid var(--border);color:var(--text);padding:10px"
          >
            <option value="">executor</option>
            <option value="claude">claude</option>
            <option value="codex">codex</option>
          </select>
          <select
            value=${todoReviewerProvider}
            onInput=${(e) => setTodoReviewerProvider(e.currentTarget.value)}
            style="background:var(--sf);border:1px solid var(--border);color:var(--text);padding:10px"
          >
            <option value="">reviewer</option>
            <option value="claude">claude</option>
            <option value="codex">codex</option>
          </select>
        </div>
        <textarea
          rows="3"
          value=${todoTask}
          onInput=${(e) => setTodoTask(e.currentTarget.value)}
          placeholder="task"
          style="width:100%;margin-top:var(--sp-xs);background:var(--sf);border:1px solid var(--border);color:var(--text);padding:10px;font-family:var(--font-mono);resize:vertical"
        />
        <textarea
          rows="2"
          value=${todoDeclaredPaths}
          onInput=${(e) => setTodoDeclaredPaths(e.currentTarget.value)}
          placeholder="declared paths: src/app.js, src/api.js"
          style="width:100%;margin-top:var(--sp-xs);background:var(--sf);border:1px solid var(--border);color:var(--text);padding:10px;font-family:var(--font-mono);resize:vertical"
        />
        <div
          style="display:flex;gap:var(--sp-xs);flex-wrap:wrap;margin-top:var(--sp-xs)"
        >
          <button
            class="action-btn"
            disabled=${!!dualBusy.value}
            onClick=${createTodo}
          >
            create task
          </button>
          <button
            class="action-btn"
            disabled=${!!dualBusy.value}
            onClick=${clearTodoComposer}
          >
            reset
          </button>
        </div>
      </details>
      ${activeLeases.length
        ? html`<details class="panel">
            <summary
              style="cursor:pointer;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
            >
              advanced: active leases (${activeLeases.length})
            </summary>
            <div
              style="display:flex;flex-direction:column;gap:var(--sp-xs);margin-top:var(--sp-s)"
            >
              ${activeLeases.map(
                (lease) =>
                  html`<div
                    style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                  >
                    <div style="font-size:var(--fs-s);color:var(--text)">
                      ${participantLabel(lease.participant_id)} ·
                      ${lease.work_item_id}
                    </div>
                    <div
                      style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                    >
                      ${(lease.paths || []).join(", ") || "no paths"}
                    </div>
                    <div
                      style="display:flex;gap:var(--sp-xs);flex-wrap:wrap;margin-top:var(--sp-xs)"
                    >
                      <button
                        class="action-btn"
                        disabled=${!!dualBusy.value}
                        onClick=${() => releaseLease(lease, false)}
                      >
                        release
                      </button>
                      <button
                        class="action-btn"
                        disabled=${!!dualBusy.value}
                        onClick=${() => releaseLease(lease, true)}
                      >
                        force
                      </button>
                    </div>
                  </div>`,
              )}
            </div>
          </details>`
        : null}
      <div class="panel">
        <div
          style="font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
        >
          tasks
        </div>
        <div
          style="display:flex;flex-direction:column;gap:var(--sp-xs);max-height:460px;overflow:auto"
        >
          ${linkedWorkItems.length
            ? linkedWorkItems.map((item) => {
                const canQueue =
                  item.assignee === "agent" &&
                  !item.delegation_id &&
                  item.status === "ready";
                const lease = activeLeases.find(
                  (entry) => entry.work_item_id === item.id,
                );
                const selected = selectedParallelItems.includes(item.id);
                return html`<div
                  style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                >
                  <div
                    style="display:flex;justify-content:space-between;gap:var(--sp-xs);align-items:flex-start"
                  >
                    <div>
                      <div style="font-size:var(--fs-s);color:var(--text)">
                        ${item.title || item.task}
                      </div>
                      <div
                        style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                      >
                        ${item.project} | ${item.assignee} |
                        ${item.status}${item.executor_provider
                          ? " | exec " + item.executor_provider
                          : ""}${item.reviewer_provider
                          ? " | review " + item.reviewer_provider
                          : ""}${item.updated_at
                          ? " | " + shortTs(item.updated_at)
                          : ""}
                      </div>
                      <div
                        style="display:flex;gap:6px;flex-wrap:wrap;margin-top:4px"
                      >
                        ${[
                          `scope ${String(item.write_intent || "read_only").replaceAll("_", " ")}`,
                          item.declared_paths?.length
                            ? `${item.declared_paths.length} path${item.declared_paths.length === 1 ? "" : "s"}`
                            : "",
                          lease
                            ? `lease ${(lease.paths || []).length || 0} path${(lease.paths || []).length === 1 ? "" : "s"}`
                            : "",
                          selected ? "parallel selected" : "",
                        ]
                          .filter(Boolean)
                          .map(
                            (label) =>
                              html`<span
                                style="padding:3px 8px;border:1px solid var(--border);background:var(--bg);font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                              >
                                ${label}
                              </span>`,
                          )}
                      </div>
                      ${item.declared_paths?.length ||
                      item.write_intent !== "read_only"
                        ? html`<details style="margin-top:4px">
                            <summary
                              style="cursor:pointer;font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                            >
                              scope detail
                            </summary>
                            <div
                              style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono);margin-top:4px"
                            >
                              ${item.write_intent}${item.declared_paths?.length
                                ? ": " + item.declared_paths.join(", ")
                                : ""}
                            </div>
                          </details>`
                        : null}
                      ${item.review_verdict?.status
                        ? html`<div
                            style="font-size:var(--fs-s);color:var(--accent)"
                          >
                            review:
                            ${item.review_verdict.status}${item.review_verdict
                              .summary
                              ? " · " + item.review_verdict.summary
                              : ""}
                          </div>`
                        : null}
                    </div>
                  </div>
                  <div
                    style="display:flex;gap:var(--sp-xs);flex-wrap:wrap;margin-top:var(--sp-xs)"
                  >
                    ${canQueue
                      ? html`<button
                          class="action-btn"
                          disabled=${!!dualBusy.value}
                          onClick=${() => queueExistingWorkItem(item)}
                        >
                          run task
                        </button>`
                      : null}
                    ${item.assignee === "user" && item.status !== "done"
                      ? html`<button
                          class="action-btn"
                          disabled=${!!dualBusy.value}
                          onClick=${() => completeExistingUserWorkItem(item)}
                        >
                          mark done
                        </button>`
                      : null}
                  </div>
                  ${selected
                    ? html`<div
                        style="margin-top:var(--sp-xs);font-size:var(--fs-s);color:var(--cyan);font-family:var(--font-mono)"
                      >
                        selected for safe parallel
                      </div>`
                    : null}
                  ${canQueue || (item.write_intent !== "read_only" && !lease)
                    ? html`<details style="margin-top:var(--sp-xs)">
                        <summary
                          style="cursor:pointer;font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                        >
                          advanced
                        </summary>
                        <div
                          style="display:flex;gap:var(--sp-xs);flex-wrap:wrap;margin-top:var(--sp-xs)"
                        >
                          ${canQueue
                            ? html`<label
                                style="display:flex;align-items:center;gap:6px;font-size:var(--fs-s);color:var(--t3)"
                              >
                                <input
                                  type="checkbox"
                                  checked=${selected}
                                  onInput=${() => toggleParallelItem(item.id)}
                                />
                                select for safe parallel
                              </label>`
                            : null}
                          ${item.write_intent !== "read_only" && !lease
                            ? html`<button
                                class="action-btn"
                                disabled=${!!dualBusy.value}
                                onClick=${() => acquireLease(item)}
                              >
                                acquire lease
                              </button>`
                            : null}
                        </div>
                      </details>`
                    : null}
                </div>`;
              })
            : html`<div style="font-size:var(--fs-s);color:var(--t3)">
                No work items yet.
              </div>`}
        </div>
      </div>
      <div class="panel">
        <div
          style="font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
        >
          review + results
        </div>
        <div
          style="display:grid;grid-template-columns:repeat(auto-fit,minmax(120px,1fr));gap:var(--sp-xs);margin-bottom:var(--sp-s)"
        >
          ${[
            ["approved", reviewApprovals.length, "var(--green)"],
            ["pending", pendingReviews.length, "var(--yellow)"],
            ["warn", reviewWarnings.length, "var(--yellow)"],
            ["fail", reviewFailures.length, "var(--accent)"],
            ["approvals", pendingApprovalDelegations.length, "var(--cyan)"],
            [
              "alerts",
              attentionSignals,
              attentionSignals ? "var(--accent)" : "var(--t3)",
            ],
          ].map(
            ([label, count, color]) =>
              html`<div
                style="padding:var(--sp-s);border:1px solid var(--border);background:var(--bg-soft)"
              >
                <div
                  style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                >
                  ${label}
                </div>
                <div
                  style="font-size:var(--fs-l);color:${color};font-family:var(--font-mono)"
                >
                  ${count}
                </div>
              </div>`,
          )}
        </div>
        <div
          style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:var(--sp-xs);margin-bottom:var(--sp-s)"
        >
          <div
            style="padding:var(--sp-s);border:1px solid var(--border);background:var(--bg-soft)"
          >
            <div
              style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
            >
              latest review decisions
            </div>
            <div
              style="display:flex;flex-direction:column;gap:6px;margin-top:var(--sp-xs)"
            >
              ${[...reviewFailures, ...reviewWarnings, ...reviewApprovals]
                .slice(0, 4)
                .map(
                  (item) =>
                    html`<div
                      style="padding:6px;border:1px solid var(--border);background:var(--bg)"
                    >
                      <div style="font-size:var(--fs-s);color:var(--text)">
                        ${compactText(item.title || item.id, 72)}
                      </div>
                      <div
                        style="font-size:var(--fs-s);color:${reviewTone(
                          item.review_verdict?.status,
                        )};font-family:var(--font-mono)"
                      >
                        ${item.review_verdict?.status || item.status}
                      </div>
                      ${item.review_verdict?.summary
                        ? html`<div
                            style="font-size:var(--fs-s);color:var(--t3);margin-top:4px"
                          >
                            ${compactText(item.review_verdict.summary, 96)}
                          </div>`
                        : null}
                    </div>`,
                )}
              ${!reviewFailures.length &&
              !reviewWarnings.length &&
              !reviewApprovals.length
                ? html`<div style="font-size:var(--fs-s);color:var(--t3)">
                    No review decisions yet.
                  </div>`
                : null}
            </div>
          </div>
          <div
            style="padding:var(--sp-s);border:1px solid var(--border);background:var(--bg-soft)"
          >
            <div
              style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
            >
              live results + alerts
            </div>
            <div
              style="display:flex;flex-direction:column;gap:6px;margin-top:var(--sp-xs)"
            >
              ${executionHighlights.slice(0, 4).map(
                (entry) =>
                  html`<div
                    style="padding:6px;border:1px solid var(--border);background:var(--bg)"
                  >
                    <div
                      style="display:flex;justify-content:space-between;gap:var(--sp-xs)"
                    >
                      <div
                        style="font-size:var(--fs-s);color:${entry.tone};font-family:var(--font-mono)"
                      >
                        ${entry.actor} | ${entry.label}
                      </div>
                      <div
                        style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                      >
                        ${shortTs(entry.ts)}
                      </div>
                    </div>
                    <div
                      style="margin-top:4px;font-size:var(--fs-s);color:var(--text)"
                    >
                      ${entry.summary}
                    </div>
                    ${entry.detail
                      ? html`<div
                          style="margin-top:4px;font-size:var(--fs-s);color:var(--t3)"
                        >
                          ${entry.detail}
                        </div>`
                      : null}
                  </div>`,
              )}
              ${!executionHighlights.length
                ? html`<div style="font-size:var(--fs-s);color:var(--t3)">
                    No execution highlights yet.
                  </div>`
                : null}
            </div>
          </div>
        </div>
        <details>
          <summary
            style="cursor:pointer;font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
          >
            details
          </summary>
          <div
            style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:var(--sp-s)"
          >
            <div>
              <div
                style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono);margin-bottom:var(--sp-xs)"
              >
                auto reviews
              </div>
              <div style="display:flex;flex-direction:column;gap:var(--sp-xs)">
                ${reviewerWorkItems.length
                  ? reviewerWorkItems.slice(0, 6).map(
                      (item) =>
                        html`<div
                          style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                        >
                          <div style="font-size:var(--fs-s);color:var(--text)">
                            ${item.title || item.id}
                          </div>
                          <div
                            style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                          >
                            ${item.executor_provider || "reviewer"} В·
                            ${item.status}
                          </div>
                          ${item.review_verdict?.status
                            ? html`<div
                                style="font-size:var(--fs-s);color:var(--accent)"
                              >
                                ${item.review_verdict.status}${item
                                  .review_verdict.summary
                                  ? " В· " + item.review_verdict.summary
                                  : ""}
                              </div>`
                            : null}
                        </div>`,
                    )
                  : html`<div style="font-size:var(--fs-s);color:var(--t3)">
                      No auto reviews yet.
                    </div>`}
              </div>
            </div>
            <div>
              <div
                style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono);margin-bottom:var(--sp-xs)"
              >
                execution feedback
              </div>
              <div style="display:flex;flex-direction:column;gap:var(--sp-xs)">
                ${linkedDelegations.length
                  ? linkedDelegations.slice(0, 6).map(
                      (delegation) =>
                        html`<div
                          style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                        >
                          <div style="font-size:var(--fs-s);color:var(--text)">
                            ${delegation.project} В· ${delegation.status}
                          </div>
                          <div
                            style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                          >
                            ${delegation.gate_result
                              ? "gate " + delegation.gate_result
                              : "no gate"}${delegation.review_verdict?.status
                              ? " В· review " + delegation.review_verdict.status
                              : ""}
                          </div>
                        </div>`,
                    )
                  : html`<div style="font-size:var(--fs-s);color:var(--t3)">
                      No linked delegations yet.
                    </div>`}
              </div>
            </div>
          </div>
        </details>
        ${linkedSignals.length || linkedInboxItems.length
          ? html`<details style="margin-top:var(--sp-s)">
              <summary
                style="cursor:pointer;font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
              >
                alerts
              </summary>
              <div
                style="display:flex;flex-direction:column;gap:var(--sp-xs);margin-top:var(--sp-s)"
              >
                ${linkedSignals.map(
                  (signal) =>
                    html`<div
                      style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                    >
                      <div style="font-size:var(--fs-s);color:var(--text)">
                        ${signal.level || "info"} В·
                        ${signal.source || "signal"}
                      </div>
                      <div style="font-size:var(--fs-s);color:var(--t3)">
                        ${signal.message ||
                        signal.title ||
                        JSON.stringify(signal)}
                      </div>
                    </div>`,
                )}
                ${linkedInboxItems.map(
                  (item) =>
                    html`<div
                      style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                    >
                      <div style="font-size:var(--fs-s);color:var(--text)">
                        ${item.project}
                      </div>
                      <div style="font-size:var(--fs-s);color:var(--t3)">
                        ${item.message}
                      </div>
                    </div>`,
                )}
              </div>
            </details>`
          : null}
      </div>
      ${parallelBatches.length
        ? html`<details class="panel">
            <summary
              style="cursor:pointer;font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
            >
              advanced: safe parallel runs (${parallelBatches.length})
            </summary>
            ${parallelBatches.map(
              (batch) =>
                html`<div
                  style="margin-top:var(--sp-xs);padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                >
                  <div style="font-size:var(--fs-s);color:var(--text)">
                    ${batch.batch_id} · ${batch.status}
                  </div>
                  <div
                    style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                  >
                    queued ${batch.pending} · running ${batch.running} · done
                    ${batch.done} · failed ${batch.failed + batch.rejected}
                  </div>
                </div>`,
            )}
          </details>`
        : null}
    </div>`;
  }
  return html`<div style="display:flex;flex-direction:column;gap:var(--sp-s)">
    <div class="panel">
      <div
        style="font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
      >
        auto reviews
      </div>
      <div style="display:flex;flex-direction:column;gap:var(--sp-xs)">
        ${reviewerWorkItems.length
          ? reviewerWorkItems.map(
              (item) =>
                html`<div
                  style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                >
                  <div style="font-size:var(--fs-s);color:var(--text)">
                    ${item.title || item.id}
                  </div>
                  <div
                    style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                  >
                    ${item.executor_provider || "reviewer"} ·
                    ${item.status}${item.source_id
                      ? " · source " + item.source_id
                      : ""}
                  </div>
                  ${item.review_verdict?.status
                    ? html`<div
                        style="font-size:var(--fs-s);color:var(--accent)"
                      >
                        ${item.review_verdict.status}${item.review_verdict
                          .summary
                          ? " · " + item.review_verdict.summary
                          : ""}
                      </div>`
                    : null}
                </div>`,
            )
          : html`<div style="font-size:var(--fs-s);color:var(--t3)">
              No auto reviews yet.
            </div>`}
      </div>
    </div>
    <div class="panel">
      <div
        style="font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
      >
        execution feedback
      </div>
      <div
        style="display:flex;flex-direction:column;gap:var(--sp-xs);max-height:420px;overflow:auto"
      >
        ${linkedDelegations.length
          ? linkedDelegations.map(
              (delegation) =>
                html`<div
                  style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft)"
                >
                  <div style="font-size:var(--fs-s);color:var(--text)">
                    ${delegation.project} · ${delegation.status}
                  </div>
                  <div
                    style="font-size:var(--fs-s);color:var(--t3);font-family:var(--font-mono)"
                  >
                    ${delegation.id}${delegation.gate_result
                      ? " · gate " + delegation.gate_result
                      : ""}${delegation.review_verdict?.status
                      ? " · review " + delegation.review_verdict.status
                      : ""}
                  </div>
                  ${delegation.summary
                    ? html`<div
                        style="font-size:var(--fs-s);color:var(--t2);margin-top:4px"
                      >
                        ${delegation.summary}
                      </div>`
                    : null}
                  ${delegation.status === "pending"
                    ? html`<div style="margin-top:var(--sp-xs)">
                        <button
                          class="action-btn"
                          disabled=${!!dualBusy.value}
                          onClick=${() => approveDel(delegation.id)}
                        >
                          approve
                        </button>
                      </div>`
                    : null}
                </div>`,
            )
          : html`<div style="font-size:var(--fs-s);color:var(--t3)">
              No linked delegations yet.
            </div>`}
      </div>
    </div>
    ${linkedSignals.length || linkedInboxItems.length
      ? html`<div class="panel">
          <div
            style="font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
          >
            signals + inbox
          </div>
          ${linkedSignals.map(
            (signal) =>
              html`<div
                style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft);margin-top:var(--sp-xs)"
              >
                <div style="font-size:var(--fs-s);color:var(--text)">
                  ${signal.level || "info"} · ${signal.source || "signal"}
                </div>
                <div style="font-size:var(--fs-s);color:var(--t3)">
                  ${signal.message || signal.title || JSON.stringify(signal)}
                </div>
              </div>`,
          )}
          ${linkedInboxItems.map(
            (item) =>
              html`<div
                style="padding:var(--sp-xs);border:1px solid var(--border);background:var(--bg-soft);margin-top:var(--sp-xs)"
              >
                <div style="font-size:var(--fs-s);color:var(--text)">
                  ${item.project}
                </div>
                <div style="font-size:var(--fs-s);color:var(--t3)">
                  ${item.message}
                </div>
              </div>`,
          )}
        </div>`
      : null}
  </div>`;
}

function DualAgentsView() {
  return html`<div class="content" style="padding:var(--sp-l)">
    <div class="panel">
      <div
        style="font-family:var(--font-mono);font-size:var(--fs-s);color:var(--yellow);margin-bottom:var(--sp-xs)"
      >
        legacy duo view
      </div>
      <div
        style="font-size:var(--fs-s);color:var(--t3);margin-bottom:var(--sp-s)"
      >
        This compatibility view is deprecated. Use the main chat with the
        Solo/Duo toggle instead.
      </div>
      <${EmbeddedDualAgentsPanel} tab="collaborate" />
    </div>
  </div>`;
}

export {
  SettingsPage,
  PlansView,
  StrategyView,
  DualAgentsView,
  EmbeddedDualAgentsPanel,
};
