# Writing Reference Provenance

This document records source selection and the LitAI audit. Runtime behavior lives
in `.claude/library/technical/writing.md`; structured source authority lives in
`.claude/library/technical/writing-reference-registry.json`.

That file is the template baseline, not the place for a downstream project's
brand voice or licensed examples. Projects own
`brain/03-knowledge/writing/reference-registry.json`. Its IDs must be unique;
replacing a template source, profile, or editor role requires an explicit
`supersedes` entry.
The validator merges template and project registries deterministically and fails
on collisions, unknown roles/skills, missing local hashes, stale mandatory
sources, language/effect mismatches, or domain standards that claim language
authority.

## Source Classes And Language Boundary

The registry does not use one flat list of "reference authors." It records:

| Class | May influence | Must not influence |
| --- | --- | --- |
| Project reference | declared project voice, syntax, terminology, facts, structure | anything outside recorded ownership, language, or active version |
| Russian language method | Russian syntax, line editing, examples, structure, observable voice properties | copied corpus, mechanical author imitation, other output languages |
| Editorial/process method | reader purpose, planning, argument, examples, client communication | target-language voice unless explicitly authorized |
| Domain/regulatory standard | claims, facts, safety, procedure, terminology, accessibility, information architecture | voice, syntax, idiom, rhythm, line editing |
| Craft/historical lens | explicitly selected narrative, offer, or measurement properties | default author imitation or current legal authority |
| External tool | only the recorded output of an actually configured and executed service | authorial authority, inferred scores, simulated warnings, or evidence without a provider response |

`scripts/lib/writing-route-policy.js` returns four authority groups separately:
language/editorial, process, domain, and technical profiles. The process group
may define planning and client-service behavior but cannot supply target-language
syntax or voice. `scripts/lib/writing-reference-policy.js` enforces allowed
effects and target language.

## Russian Writing Baseline

Russian informational, marketing, communication, and technical output uses:

- project-owned approved Russian samples when the project has registered them;
- Maxim Ilyakhov and Lyudmila Sarycheva's `Пиши, сокращай - 2025` for Russian
  information-style and line-editing principles;
- Ilyakhov and Sarycheva's `Новые правила деловой переписки` for recipient care,
  executable requests, tasks, status, feedback, refusal, claims, and conflict;
- Ilyakhov's `Ясно, понятно` for context, known-to-new explanation, examples,
  counterexamples, bounded analogies, objections, and visual explanation;
- Ilyakhov's `Текст по полочкам` for working messages, task setting, reports,
  meeting summaries, proposals, criticism, claims, and information structure;
- the public Bureau guides `Как написать` and `Информационный стиль` for open,
  scenario-level rules and terminology boundaries;
- Maxim Ilyakhov's official materials on working systems, structure, examples,
  author style as decisions, and text as a client service;
- the user-provided planning pages, stored only as a paraphrased internal note.

The operational synthesis and template-authored examples live in three profiles:

- `.claude/library/technical/russian-writing-profile.md` owns general Russian
  language, structure, line editing, and source boundaries;
- `.claude/library/technical/russian-business-correspondence.md` owns Russian
  business and operational correspondence;
- `.claude/library/technical/russian-explanation-and-persuasion.md` owns Russian
  explanation and evidence-calibrated persuasion.

They contain original project examples, not quotations or reconstructed book
passages.

## Glavred Boundary

The public Glavred pages document the service boundary and declared capabilities.
They do not authorize a writing profile and do not prove that the paid service
ran. Manual editing uses the active Russian book/Bureau profiles, not a simulated
provider result.

`glavred-api` is therefore an `externalTool`, not a source or profile. The
template status is `not-configured`: no subscription, credentials, HTTP adapter,
or provider response exists in this repository. The route exposes that state and
adds `external-tool-unavailable` plus `external-tool-evidence-required` gates.
It also reports `execution:not-run`; configuration alone never authorizes a
provider-result claim. The merged registry is validated at the route boundary.

A downstream project may change the state only with an explicit `toolIds`
supersession and a same-ID replacement. Configuration evidence records an
existing project adapter path, an `env:` or `secret-store:` reference, an owner,
and a checked date; it never stores the secret itself. Even then, a report may say
`checked by Glavred`, show a score, or list provider warnings only after a
successful response is tied to that exact artifact. A manual edit must be called
manual; an uploaded report must be labeled user-supplied.

## LitAI Audit

The local `LitAi/GiantVale_Necromancer_Book/references` library contains one
declared user-owned full source, derivative analysis, and seven external work
profiles. The useful architecture is preserved; external examples are not copied
into the template.

| Reference | Preserved properties | Provenance decision |
| --- | --- | --- |
| User's Necromancer introduction | author voice, sensory detail, pacing, character voice, atmosphere, world through detail | user-declared project authority; downstream projects must provide their own approved samples |
| Ivan Bunin lens | sensory precision, texture, atmosphere, observable emotion | opt-in properties only; LitAI says concrete excerpts still need sourcing |
| J. K. Rowling lens | clarity, discovery, distinct traits, understandable artifacts and rules | opt-in properties only; no verified edition or excerpt provenance |
| Sapkowski/Witcher lens | subtext, irony, moral ambiguity, grounded consequence | opt-in composite lens; game and author sources must not be conflated |
| Nick Perumov lens | rule-bound magic, combat anatomy, physical cost | opt-in lens only; no verified source-text corpus found |
| Path of Exile | tragic arcs, justified antagonists, moral ambiguity | opt-in properties only |
| Deus Ex | layered conflict, dilemmas, systemic antagonists | opt-in properties only |
| Skyrim | world detail, locations, systems, artifacts | opt-in properties only |
| Suits | sharp dialogue, power, relationship change, setup and payoff | opt-in properties only |

Every external LitAI `examples.md` lacks at least one of canonical URL, edition,
page or scene locator, licence, permission record, or acquisition trail. Treat it
as potentially verbatim or closely derivative. It may not enter prompts,
downstream template payloads, or acceptance evidence until provenance is fixed.

## Source Selection

Default profiles use project-owned approved samples when present, the explicitly
approved Russian editorial method for Russian output, and current official or
regulatory guidance for its declared domain effects. Literary imitation and
unverified craft lenses remain opt-in. A source cannot become default while its
provenance is `unverified-derived`.

| Area | Default or conditional sources | Why |
| --- | --- | --- |
| Marketing | Russian information style plus explanation/persuasion profile; FTC for claim substantiation; project evidence | language, commercial truth, and persuasion ethics are separate review lanes |
| Informational | Russian information style plus explanation profile; Digital.gov/GOV.UK for reader-task architecture | English standards do not supply Russian syntax or voice |
| Communication | Russian information style plus business-correspondence and explanation profiles; GOV.UK/CDC for service or crisis constraints | recipient action, timing, and safety remain domain checks |
| Technical | active language profile; Diataxis, Google, Microsoft, Red Hat, and IETF for declared technical effects | technical authority does not become prose authority |
| Literary | project-owned approved prose first; LitAI properties only when selected | voice and canon belong to the project, not a universal template author |

## Content Policy

- `project-owned-only`: full text is allowed only after ownership or permission is
  recorded by the project.
- `properties-only`: retain analysis, never source or near-source passages.
- `link-and-properties`: keep the canonical link and extracted properties; do not
  mirror the source text.
- A public-domain label is jurisdiction-sensitive. The registry records the
  verifying institution and still defaults to links and properties.
- Fresh legal, safety, platform, and vendor guidance must be rechecked before use.
