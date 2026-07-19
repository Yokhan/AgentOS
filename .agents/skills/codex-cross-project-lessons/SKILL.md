---
name: codex-cross-project-lessons
description: "Extract reusable lessons from downstream projects and promote them into template rules, skills, validators, or docs without copying project-specific secrets or implementation details. Use for retrospectives, repeated failures, or template self-improvement."
---

# Codex Cross-Project Lessons

Read:

- `tasks/lessons.md`
- `tasks/current.md`
- Relevant downstream `tasks/lessons.md`, `PROJECT_SPEC.md`, and deployment notes when available

## Process

1. Separate reusable agent/process failure from project-specific facts.
2. Classify the lesson: product, design, auth, docs, deployment, security, testing, routing, or template sync.
3. Decide the right target:
   - Rule: reusable reasoning or invariant.
   - Skill: repeatable workflow.
   - Validator: machine-checkable failure.
   - Router: task selection failure.
   - Docs: rollout or human decision guidance.
4. Remove secrets, credentials, private customer data, and incidental domain noise.
5. Add verification so the lesson does not remain only prose.

## Promotion Gate

Promote a lesson only when it would have prevented a real repeated mistake or materially improves future product quality.

Do not promote one-off preferences unless the user explicitly turns them into a project standard.
