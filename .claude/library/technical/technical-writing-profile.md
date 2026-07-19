# Technical Writing Profile

Technical documentation does not inherit the prose voice of its domain sources.
Google, Microsoft, Red Hat, IETF, and Diataxis may constrain terminology,
notation, procedure, accessibility, normative meaning, or information
architecture. The active target-language profile owns syntax, idiom, rhythm,
and line editing. Russian technical text therefore loads
`russian-writing-profile.md` in addition to this profile, plus
`russian-explanation-and-persuasion.md` for explanations and
`russian-business-correspondence.md` for incidents, requests, status, release,
or migration communication.

This file is a child of `.claude/library/technical/writing.md`. Technical writing
is an overlay on the informational mode, not a fifth semantic mode. Release notes,
incidents, and migration notices may use communication as the primary mode while
still loading this technical contract.

## Entry Contract

Before drafting, identify:

1. User and competence level: learner, practitioner, operator, integrator, or
   maintainer.
2. Product and version: code commit, API/schema version, platform, operating
   system, deployment shape, and compatibility window.
3. Document kind: tutorial, how-to, reference, explanation, runbook, migration,
   troubleshooting, release note, or architecture decision.
4. Authority: code, generated schema, tested behavior, product specification,
   decision record, and project terminology.
5. Evidence: executable commands, contract checks, screenshots when relevant,
   link validation, docs build, and reviewer ownership.

If the implementation, schema, and existing documentation disagree, use the SOT
Conflict Protocol. Do not silently choose the easiest source.

Select the active technical profile and editor IDs through
`scripts/lib/writing-route-policy.js`. Template defaults come from the shared
registry; project-owned additions and replacements come from the `brain/`
registry and must pass `node scripts/validate-writing-references.js`.

## Document Kinds

| Kind | User job | Required shape |
| --- | --- | --- |
| Tutorial | Gain initial competence through a successful guided experience | controlled prerequisites, concrete steps, expected feedback, safe completion |
| How-to | Complete a real task | goal, prerequisites, shortest reliable procedure, verification, recovery |
| Reference | Look up exact product facts | product-shaped hierarchy, complete fields, types, defaults, limits, errors |
| Explanation | Understand why the system behaves this way | question, context, constraints, mechanism, tradeoffs, implications |
| Runbook | Restore or operate a system under pressure | trigger, authority, checks, actions, stop conditions, rollback, escalation |
| Migration | Move safely between versions or states | compatibility, backup, dry run, ordered change, validation, rollback |
| Troubleshooting | Diagnose and recover from a symptom | symptom, scope, observations, causes, discriminating checks, fixes, prevention |

Do not combine kinds merely to make one long page. Link between them when the
reader changes from learning to working, or from action to understanding.

## Docs-As-Code Contract

- Treat documentation as part of the product interface and change it with the
  code, schema, UI, or operational behavior it describes.
- Prefer generated reference for machine-owned facts, but review generated output
  for navigation, terminology, examples, and missing semantics.
- Every command identifies the shell and operating system. Use project platform
  helpers; never present Linux commands as Windows instructions.
- Code samples must be minimal, runnable, secure by default, and version-bounded.
- Procedures include prerequisites, expected output, failure states, recovery,
  and destructive-action warnings before the action.
- Link targets, headings, code blocks, examples, and docs builds are verification
  surfaces, not cosmetic cleanup.

## Technical Progressive JPEG

At functional 1%, the user can complete or correctly understand the primary job
through the final product path. A page containing headings, generated endpoint
names, or unexecuted commands is preparation, not a technical-writing slice.

Later passes add alternative environments, edge cases, examples, diagrams,
cross-links, performance notes, and localization without changing verified facts.
Replace stale procedures and generated fragments instead of keeping contradictory
versions hidden or marked legacy inside the active path.

## Acceptance Board

Technical acceptance requires separate evidence for:

1. Technical accuracy: compare against code, schema, interfaces, versions, and
   observed behavior.
2. Procedure execution: run the documented path in the stated environment and
   record expected, failure, and recovery results.
3. Information architecture: verify document-kind boundaries, navigation,
   discoverability, and lifecycle ownership.
4. Technical language: check terminology, code formatting, accessibility,
   localization, and unambiguous normative force.

The writer may run a self-check, but it is not independent technical acceptance.

External writing services are optional diagnostics. Without configured access
and an artifact-specific successful response, report that the external check was
not run. Never infer or fabricate a provider score from manual editing.
