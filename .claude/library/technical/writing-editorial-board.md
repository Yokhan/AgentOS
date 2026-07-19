# Writing Editorial Board

This board composes review responsibilities by text purpose. It does not ask one
model to impersonate named authors, and it does not treat a writer's self-review
as independent acceptance.

The structured role IDs and their Codex mappings live in
`writing-reference-registry.json`. Project-owned editors, subject-matter experts,
legal reviewers, and brand owners override template role suggestions.
`scripts/lib/writing-route-policy.js` is the executable selector for required
profiles and roles. One reviewer may cover compatible lenses, but execution,
security, legal, or API-contract evidence requires the matching specialist lane.

## Board Composition

| Mode or overlay | Required review lenses | Conditional lenses |
| --- | --- | --- |
| Literary | structure, continuity, prose | sensitivity/domain, canon owner, format editor |
| Marketing | claims, business, ethics | legal/jurisdiction, accessibility, channel specialist |
| Informational | subject, reader task, language | science/safety, data, localization |
| Communication | recipient action, risk, language | incident commander, legal/privacy, support owner |
| Technical | accuracy, procedure, architecture, technical language | security, API contract, migration, accessibility |

## Language Authors Versus Domain Standards

These are different authority classes:

- A language author/editor may influence syntax, line editing, examples, rhythm,
  and voice properties only for declared output languages.
- A domain standard may influence facts, claims, safety, terminology, document
  architecture, accessibility, or procedure. It is not an author and may not
  impose source-language sentence patterns.
- A named craft lens contributes explicit properties, never impersonation.

For Russian informational, marketing, communication, and technical text, the
default language method is the registered Ilyakhov/Sarycheva profile plus active
project-owned Russian samples. The operational rules and original examples live
in `russian-writing-profile.md`.

## Named Lenses

Named authors are optional analytical lenses with explicit properties, never
voice generators:

- LitAI literary set: Bunin for sensory precision; Rowling for clarity and
  discovery; Sapkowski/Witcher for dialogue, irony, and moral ambiguity; Perumov
  for rule-bound magic and physical consequence. Path of Exile, Deus Ex, Skyrim,
  and Suits contribute conflict, world, and relationship properties.
- Russian information and line editing: Maxim Ilyakhov and Lyudmila Sarycheva for
  reader purpose, actors and actions, useful detail, structure, examples, and
  large-to-small editing. Maxim Ilyakhov's planning and client-work material also
  supplies the service and replan method. These are registered Russian profiles,
  not voice-imitation prompts.
- Marketing measurement: Claude Hopkins is an opt-in historical lens for
  specificity and testing. Current evidence, ethics, channels, and law override
  historical advice.
- Domain correctness and architecture: FTC, CDC, GOV.UK, Digital.gov, Diataxis,
  Google, Microsoft, Red Hat, and IETF are standards or frameworks, not authors
  of Russian prose. Their allowed effects are enforced by the registry validator.

## Independent Review Rule

A review is independent only when a separate reviewer receives the writing
contract, active source IDs, selected property profiles, the text, and observed
product evidence. It must report findings before rewrites and identify which
editor role produced each blocking finding.

For technical procedures, a prose reviewer cannot substitute for execution. For
marketing claims, conversion expertise cannot substitute for substantiation. For
literary text, line polish cannot substitute for canon and structural review.
