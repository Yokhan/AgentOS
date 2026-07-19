# Template Releases

Use this document when publishing or consuming `agent-project-template` releases.

## Ownership Model

There are three separate surfaces:

| Surface | Owner | Update mechanism |
| --- | --- | --- |
| Template repository | Template maintainers | Git commits, release tags, release notes |
| Generated projects | Project teams | `scripts/sync-template.sh` from local path or git ref |
| AgentOS workspace | AgentOS | AgentOS orchestrates work, then consumes a template release like any other downstream project |

AgentOS is an orchestrator, not the template source of truth. If AgentOS metadata is present, Codex follows AgentOS Strategy/Tactic/Plan/Todo/Gate artifacts as the task graph. Template files still come from this repository and are updated through template sync.

## Version Rule

The template version is declared in:

- `AGENTS.md`
- `CLAUDE.md`
- README badge and release notes

Use semantic version tags:

```bash
git tag v4.9.0
git push origin v4.9.0
```

Pushing a `vX.Y.Z` tag triggers `.github/workflows/release-template.yml`. The workflow runs the release gate and publishes a GitHub release archive named `agent-project-template-<tag>.tar.gz`.

Patch releases are for compatible fixes to rules, skills, hooks, scripts, and docs. Minor releases can add new skills, agents, release flows, or routing behavior. Major releases can change sync contracts, project ownership boundaries, or the default agent operating contract.

## Canonical Agent Update Protocol

This section is the source of truth for "update your template", "update the
agent template", or a handoff containing only the repository URL. Do not invent
another update path from examples elsewhere.

### 1. Classify The Workspace

- **Template source:** `PROJECT_SPEC.md` says `Name: agent-project-template`.
  This is release work; never sync the template into itself.
- **Generated downstream:** `.template-manifest.json` exists. Its
  `template_version` is the installed-version SOT.
- **Legacy downstream:** no manifest exists. Bootstrap once with the target
  release's script, then continue as generated downstream.

If ownership is unclear, stop and ask instead of guessing.

### 2. Resolve One Explicit Target

1. Explicit user tag > AgentOS-approved tag > verified latest stable release.
2. Otherwise read the stable, non-draft, non-prerelease tag from
   <https://github.com/Yokhan/agent-project-template/releases/latest>. Current stable tag: `v4.9.0`.
3. Installed version is comparison data, never the target. Never infer the
   target from `main`, a badge, cached memory, or a stale local sync script.
4. If no target can be verified, ask for a tag. Do not substitute a branch.
5. Installed equals target is a no-op. Downgrades and major jumps require
   explicit confirmation. Branches/commits require explicit canary intent.

### 3. Verify Source And Preflight

- Read `git remote get-url template`. Add the approved repository only when the
  remote is absent. Present an existing conflicting remote to the user.
- Inspect `git status --short`, manifest ownership, and `project-*` overlays.
  Stop for dirty ownership ambiguity, downgrade/major jump, changed product
  boundary, or unresolved `*.template-new` conflicts.
- On Windows, do not substitute Linux commands in PowerShell. Run the Unix sync
  flow in a declared Linux CI/maintenance environment until a native updater is
  available; native project creation remains `setup.bat`.

### 4. Preview And Apply The Same Tag

```bash
git remote get-url template
bash scripts/sync-template.sh --from-git --ref <tag> --dry-run
bash scripts/sync-template.sh --from-git --ref <tag>
```

The apply command must use the exact tag from the accepted preview. Bare
`--from-git` is canary behavior and is forbidden for normal update requests.

If the local sync script is missing or fails before a trustworthy preview,
check out the exact release and use its script against the project:

```bash
git clone --branch <tag> --depth 1 <template-url> <release-checkout>
bash <release-checkout>/scripts/sync-template.sh <release-checkout> --project-dir <project> --dry-run
bash <release-checkout>/scripts/sync-template.sh <release-checkout> --project-dir <project>
```

Do not patch a stale local sync script ad hoc and call it the release.
Use the target release checkout's script with `--project-dir` for this fallback.

### 5. Verify Before Claiming Success

1. Confirm `.template-manifest.json.template_version` equals the target without
   the leading `v`.
2. Confirm actual diff, preserved project-owned overlays, and explicit handling
   of every `*.template-new` file.
3. Run route-selected downstream checks: at minimum text policy, Codex
   agent/skill validation, routing smoke, and project tests when present.
4. Report installed -> target, repository, tag, preview/apply evidence,
   conflicts, checks, and remaining doubt.
5. A local tag proves only tag resolution. Say "published/live" only after
   checking the authoritative GitHub Release/workflow state.

### Current Stable Flow

After verifying that `releases/latest` resolves to `v4.9.0`, create a new project with:

```bash
git clone --branch v4.9.0 --depth 1 https://github.com/Yokhan/agent-project-template.git agent-project-template
cd agent-project-template
bash setup.sh my-project
```

Existing generated project:

```bash
template_url="$(git remote get-url template 2>/dev/null || true)"
[ -n "$template_url" ] || git remote add template https://github.com/Yokhan/agent-project-template.git
[ -z "$template_url" ] || [ "$template_url" = "https://github.com/Yokhan/agent-project-template.git" ] || { echo "template remote conflict: $template_url"; exit 1; }
bash scripts/sync-template.sh --from-git --ref v4.9.0 --dry-run
bash scripts/sync-template.sh --from-git --ref v4.9.0
```

Use `main` only for template development, explicit canary rollout, or when the product owner accepts untagged changes. Release archives are for inspection or offline transfer; agent-managed projects should prefer git tag sync because the selected version is explicit and rollbackable.

## v4 Production Standard Notes

Version `4.0.0` changes the agent operating contract:

- Real product work is not treated as MVP/prototype work unless explicitly requested.
- `tasks/goal.md` carries the persistent final outcome, quality bar, current step, dependencies, and risks.
- Plans, audits, status updates, and final reports must use the language of the user's request.
- Codex routing now exposes `planContract`, `productionBar`, `languagePolicy`, and `qualityGates`.
- Design-system and product-UX work have dedicated skills and rendered verification gates.
- Cross-project lessons can be promoted into rules, skills, validators, or router behavior.

Version `4.0.1` is a compatible patch release that keeps `.github/workflows/release-template.yml` source-only, ships only `.github/workflows/validate-template.yml` to downstream projects, and validates this in setup/sync smoke tests.

Version `4.0.2` is a compatible patch release that keeps source-only starter and bootstrap files out of generated-project sync, specifically `templates/project-starter/*`, `setup.sh`, and `setup.bat`.

Version `4.0.3` is a compatible patch release that adds fail-hard text/platform policy enforcement, blocks mojibake and unsafe shell OS assumptions, generalizes the UI Subtraction Gate for all UI work, and optimizes sync regression smoke coverage.

Version `4.1.0` is a compatible minor release that makes product-user experience and app-specific business outcomes the first priority for plans and improvements. Agents must name the user/business outcome before technical work and treat refactors, tooling, and architecture cleanup as second-order unless they directly unlock or protect revenue, loyalty, retention, conversion, activation, support load, or another application KPI.

Version `4.1.1` is a compatible patch release that updates GitHub workflows and CI templates to Node 24-compatible actions before the GitHub Actions Node 20 runner migration.

Version `4.2.0` is a compatible minor release that adds production design QA infrastructure: root `DESIGN.md` starter context, project-owned `design-policy.ignore`, design workflow command modes, a hard design-policy validator with fixtures, default Codex design-policy hook notifications, and browser/visual hardening gates. Existing downstream `DESIGN.md` and `design-policy.ignore` files are project-owned and must not be overwritten by template sync.

Version `4.3.0` is a compatible minor release that upgrades the production design pipeline and Codex design skills. It adds a detailed design command-mode reference, product/brand/mixed register gates, critique ordering that treats deterministic validators as evidence rather than judgment, design-system register review, and routing regression coverage so conversion/KPI design work is not misrouted as a release task.

Version `4.3.1` is a compatible patch release that fixes release-facing README/CLAUDE documentation drift and adds a regression gate so shipped counts and command lists match the real template filesystem.

Version `4.3.2` is a compatible patch release that promotes the concrete screen anatomy/root-frame contract into the shared design pipeline and Codex design skills, adds regression coverage so future releases cannot drop it, and fixes `sync-template.sh --from-git --dry-run` so it shows the real sync preview from a git ref before modifying downstream projects.

Version `4.3.3` is a compatible patch release that includes design-policy test fixtures in template sync delivery so downstream `test-design-policy` works after normal sync without manual fixture copying.

Version `4.3.4` is a compatible patch release that makes production-standard validation downstream-aware so generated projects do not need source-only `templates/project-starter/*` files while the template source still checks them.

Version `4.4.0` is a compatible minor release that adds the client-executor accountability contract, anti-sycophancy rules, no-fake-completion evidence gates, research notes, Codex skill wiring, and route/validator regression coverage. Agents treat the user as the client/product owner and the agent as the accountable executor: they must not agree by default, must not claim unverified work is done, and must challenge requests that lower product outcome, safety, quality, or app-specific KPI.

Version `4.4.1` is a compatible patch release that makes the GitHub README/release entrypoint agent-safe. It adds latest-release lookup, pinned release install/sync commands, explicit `main` versus release-tag guidance, and smoke coverage so future releases cannot silently drift back to stale tag examples.

Version `4.4.2` is a compatible patch release that promotes the Ilyakhov/progressive JPEG planning principles from cold knowledge into hot agent behavior. Plans, statuses, replans, closeouts, and Codex planning skills must show the first useful view, next sharpened evidence layer, rough edges, and replan trigger instead of hiding internal work until a final answer.

Version `4.5.0` is a compatible minor release that adds semantic intent scoring to Codex routing. Routes now combine exact patterns with concept groups in `scripts/lib/codex-route-intents.js`, report exact versus semantic matches, and include regression coverage for marketing/GTM, Sun Tzu/stratagem strategy, TRIZ contradictions, SOT conflicts, systemic-error analysis, and meaning-based bugfix/security/design/product/API/migration/lesson routes.

Version `4.5.1` is a compatible patch release that makes progressive JPEG an implementation gate. Known final product capabilities should appear from the first useful slice as an end-state skeleton with 1% callable hooks, slots, contracts, flags, no-op stubs, or dev-only debug signals. Do not substitute legacy harness proof unless it protects the current product path, and do not fake user-visible readiness.

Version `4.5.2` is a compatible patch release that clarifies progressive JPEG as final-plan-gated object readiness. If the final product plan is missing, agents must gate implementation and create/propose the plan first. At 1% readiness the whole planned object exists at low detail, all accepted classes/components/interfaces/routes/sections/functions/contracts are present or callable, and the object performs its smallest honest production function with explicit debug or placeholder behavior.

Version `4.5.3` is a compatible patch release that adds the progressive layer replacement pipeline and `PROGRESSIVE_STATUS` project-slice reporting. When a later layer supersedes an earlier wrong iteration, agents must delete, replace, or time-box the old artifact instead of preserving disabled legacy code, stale placeholders, skipped tests, hidden UI layers, stale feature flags, or release-only exclusion harnesses. Iteration reports should include an aligned ASCII project slice from `node scripts/progressive-status.js`, and `node scripts/progressive-status.js --check` fails when a tagged working document changed but its status header did not. Temporary migration scaffolding is allowed only when it protects live data, rollback, compatibility, or user safety and has an explicit removal condition.

Version `4.6.0` is a compatible minor release that adds a single GPT-5.6 agent policy SOT, role-specific Sol/Terra profiles capped at `xhigh`, proactive beneficial fan-out, product and systems reviewers, observable fan-out decisions, and semantic suppression for external release pages and diagrams that are being studied rather than created. The parent model remains user-owned, user opt-out wins, recursive delegation stays disabled, and write-capable agents require exact non-overlapping ownership.

Version `4.6.1` is a compatible patch release that makes template updates deterministic for agents and closes the release blockers found during verification. It adds one canonical source/downstream update protocol, explicit target-tag precedence, pinned preview/apply and post-sync evidence, safe manifest/path handling without generated JavaScript or shell hashing, exact semver tag fetches, dry-run project-file immutability, and a release workflow that validates and publishes the same tag commit without replacing an existing release asset.

Version `4.6.2` is a compatible patch release that adds bounded Luna roles for discovery, log extraction, and summarization; limits automatic fan-out to one evidence-backed wave; replaces marker-based subagent proof with genuine child-thread trace validation; and adds a progressive JPEG planner whose every implementation slice must fulfill the real product purpose end to end without stub-dependent or fabricated evidence.

Version `4.7.0` is a compatible minor release that adds purpose-first literary, marketing, informational, communication, and technical-writing workflows; Russian business-correspondence and explanation profiles; source provenance with one authority group per profile; fail-closed route validation; and truthful external-tool states. Paid providers such as Glavred remain `not-configured` and `not-run` until a real project adapter, secret reference, owner, check date, artifact binding, and provider response exist. The release does not recreate or bundle Glavred.

Version `4.8.0` is a compatible minor release that adds an evidence-backed Change Strategy Gate. Agents evaluate architecture fitness while reading, before the first patch; compare repair, bounded replacement, and retirement across explicit transition models; and bind any decision that resumes edits to a separate structured trigger. Protected contracts derive required API, data, and security checks instead of trusting self-declared impacts. The release also rejects duplicate alternatives and malformed route CLI flags, keeps read-only comparison out of the bugfix pipeline, and runs routing regressions in generated and synced payloads.

Version `4.9.0` is a compatible minor release that turns the selected ten-tool arsenal into one delivered workflow. It installs and health-checks the full pinned profile on Linux and Windows, keeps only Engram and one parser-backed code graph permanently available, routes the other tools on demand, safely merges the Codex MCP block without overwriting project settings, verifies the config with Codex `0.125.0`, and keeps AgentOS as the owner of its own task graph.

Downstream projects should sync `v4.9.0` with a dry run first and review local `project-*` skills and agents, Codex MCP conflicts, protected-contract inventories, change envelopes, route decisions, writing voice and terminology overlays, external-tool adapters, auth flows, design systems, task files, CI workflows, client-facing report conventions, progressive plan/status artifacts, adaptive fan-out behavior, update protocol assumptions, and any project-specific routing assumptions before applying.

## Release Gate

Before tagging, run:

```bash
bash scripts/validate-template.sh
bash scripts/check-drift.sh
bash scripts/test-hooks.sh
bash scripts/test-template.sh
bash scripts/sync-agents.sh
node scripts/test-codex-routing.js
node scripts/test-codex-agent-policy.js
node scripts/test-progressive-plan.js
node scripts/test-subagent-trace.js
node scripts/validate-codex-skills.js
node scripts/validate-codex-agents.js
node scripts/validate-production-standard.js
node scripts/validate-design-policy.js
node scripts/test-design-policy.js
```

`scripts/test-codex-subagents-live.sh --yes` is a quota-consuming compatibility
probe, not a mandatory release gate. Report its result separately and never
convert a parent-authored marker into child/model evidence.

Also run a generated-project smoke when the payload changes:

```bash
bash setup.sh template-release-smoke
cd template-release-smoke
bash scripts/test-hooks.sh
bash scripts/bootstrap-mcp.sh --dry-run
bash scripts/sync-template.sh /path/to/agent-project-template --dry-run
```

## Downstream Update From A Release

Inside a generated project:

```bash
template_url="$(git remote get-url template 2>/dev/null || true)"
[ -n "$template_url" ] || git remote add template https://github.com/Yokhan/agent-project-template.git
[ -z "$template_url" ] || [ "$template_url" = "https://github.com/Yokhan/agent-project-template.git" ] || { echo "template remote conflict: $template_url"; exit 1; }
bash scripts/sync-template.sh --from-git --ref v4.9.0 --dry-run
bash scripts/sync-template.sh --from-git --ref v4.9.0
```

Use `--dry-run` first when a project has local changes. If both the project and template changed the same template-owned file, sync writes `*.template-new` instead of overwriting silently.

## Canary Update From Main

Use the branch path only for early rollout or canary projects:

```bash
bash scripts/sync-template.sh --from-git --canary --ref main --dry-run
bash scripts/sync-template.sh --from-git --canary --ref main
```

Release tags are preferred for normal projects because they make rollbacks and AgentOS rollout plans explicit.

## AgentOS Consumption

AgentOS should not copy template internals manually. Recommended flow:

1. AgentOS decides which project should update and which template tag is allowed.
2. The project runs `scripts/sync-template.sh --from-git --ref <tag> --dry-run`.
3. AgentOS reviews conflicts and project-owned overlays.
4. The project applies sync and runs the release gate subset relevant to that project.
5. AgentOS records the template tag in its own orchestration state.

If AgentOS is absent, the parent Codex thread is the orchestrator for the update and must record the chosen route from `scripts/codex-route-task.js`.

## Rollback

Rollback is a normal git operation in the downstream project:

```bash
git tag backup/pre-template-sync-$(date +%Y%m%d-%H%M%S)
git revert <sync-commit>
```

For uncommitted sync attempts, use the backup stash/tag created by `sync-template.sh` or discard only the files changed by the sync after reviewing `git diff`.
