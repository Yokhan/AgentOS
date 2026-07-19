# Writing Workflow Architecture And LitAI Adaptation

This document records why the template writing workflow has its current shape.
Behavior belongs to `.claude/library/technical/writing.md`; this is an architecture
record, not a competing instruction source.

## Source

The adaptation was based on the local LitAI project's research, idea, outline,
write, review, evolve, reference-analysis, grammar, text-analysis, and draft-review
commands plus its context-loading and common-error rules.

## Adopted

| LitAI mechanism | Generalized behavior |
| --- | --- |
| Research before planning | Load SOTs, facts, references, constraints, and gaps before drafting |
| Structured idea interview | Build a writing contract with options for missing decisions |
| Plan hierarchy | Separate final architecture from section-level execution |
| Mandatory context reload | Build a task-specific source pack immediately before drafting |
| Gap gate | Stop on missing facts that would change meaning; do not use model defaults |
| Reference analysis | Extract concrete properties instead of requesting generic imitation |
| Lens synthesis | Apply diagnostic perspectives to the whole text, then produce one voice |
| Technical plus deep review | Validate deterministic contracts before semantic judgment |
| Grammar pass with author vocabulary | Correct language without erasing intentional terms or voice |
| Cascade evolution and cache | Update the active SOT and affected text; regenerate derived state |

## Adapted

| Literary-specific behavior | Template behavior |
| --- | --- |
| Story bible, character passports, lore, chronology | Mode-specific source pack and evidence hierarchy |
| Master plan, chapter plan, scene plan | Text architecture and function inventory sized to the task |
| Editorial board of authors and psychologists | Purpose, structural, domain, evidence, and voice lenses |
| Past/current/future chapter context | Prior state, current reader job, and intended after-state |
| Reference-level chapter gate | Mode-specific acceptance evidence and progressive readiness |
| Author as sole authority | User owns intent; authoritative project/data/legal sources own factual claims |

## Rejected

- Book-only phase gates for every text.
- Literary lenses loaded for informational or operational communication.
- Named authors used as direct style generators.
- Stale drafts preserved as active warnings instead of being replaced.
- Universal word bans, deliberate imperfections, forced digressions, or arbitrary
  sentence and paragraph ratios as evidence of human voice.
- Claims that a text can or should evade AI detection.
- Self-review presented as independent acceptance.

## Ownership

| Decision | Owner |
| --- | --- |
| Authority and SOT conflict order | `docs/AGENT_CONTEXT_SOT.md` |
| Workflow, truth rules, four modes | `.claude/library/technical/writing.md` |
| Detailed mode contracts | `.claude/library/technical/writing-mode-profiles.md` |
| Russian language/editorial method and original examples | `.claude/library/technical/russian-writing-profile.md` |
| Russian business correspondence | `.claude/library/technical/russian-business-correspondence.md` |
| Russian explanation and evidence-calibrated persuasion | `.claude/library/technical/russian-explanation-and-persuasion.md` |
| Technical specialization | `.claude/library/technical/technical-writing-profile.md` |
| Template source/profile/editor catalog | `.claude/library/technical/writing-reference-registry.json` |
| Project-owned sources and replacements | `brain/03-knowledge/writing/reference-registry.json` |
| Required editorial lenses | `.claude/library/technical/writing-editorial-board.md` |
| Writing intent classification | `scripts/lib/writing-intent.js` |
| Cross-platform writing consequences | `scripts/lib/writing-route-policy.js` |
| Codex procedure | `.agents/skills/codex-writing-workflow/SKILL.md` |
| Claude procedure | `.claude/skills/writing-workflow/SKILL.md` |
| Claude runtime model/tools | `.claude/agents/writer.md` |
| Independent acceptance | domain review adapters using the shared mode gates |
| External writing-tool state and evidence | `externalTools` in the template and project writing registries |

Mode and language profiles are shared children of the writing SOT. Platform
adapters may reference them but must not duplicate or redefine their contracts.
The executable route resolves target language first and keeps four authority
groups separate: language/editorial, process, domain, and technical profiles.
The combined list exists only for adapter compatibility. An inferred language is
not enough to run language editing during edit or review: the agent must inspect
the artifact or ask before applying that profile.

Profiles declare exactly one authority group. Correspondence therefore resolves
to separate Russian language, client-process, and domain profiles; explanation
remains a domain profile and cannot acquire syntax or voice authority. The route
validates the merged template/project registry before selecting any profile.

Sources, profiles, and tools remain separate. Books and public guides authorize
declared editorial properties. A paid API supplies evidence only when configured
and executed. The template does not include a paid Glavred subscription or API
adapter, so eligible routes expose
`glavred-api:not-configured:not-run:paid` and prohibit fabricated checks, scores,
and warnings. A configured downstream adapter still reports `not-run` until an
artifact-specific provider response exists.

The project registry is optional. A downstream project without
`brain/03-knowledge/writing/reference-registry.json` uses the template baseline.
A customized schema-v1 registry must be migrated to schema v2 before it can be
merged: sources need language, usage class, and allowed effects; profiles need
output languages, declared effects, and `propertiesByEffect`.
External tool replacement uses `supersedes.toolIds` and a `project-configured`
tool record with configuration evidence.
