# Domain Guards — Evidence-Based Anti-Patterns (Condensed)

> Full details with sources: `.claude/docs/domain-full/domain-*.md`
> Evidence hierarchy: A (meta-analysis) > B (RCT) > C (consensus) > D (blog). D = INSUFFICIENT.

## Health & Fitness (SAFETY-CRITICAL — evidence enforcement mandatory)
**NEVER**: "10% brain" myth, learning styles, detox diets, spot reduction, "no pain no gain", megadose vitamins, stretching prevents injury, cardio-only fat loss, ice for all injuries
**ALWAYS**: sleep 7-9h, 150min/week exercise, progressive overload, protein 1.6-2.2g/kg, active recovery, Mediterranean diet, fiber 25-30g/day, creatine 5g/day
**RULE**: Every health recommendation MUST cite evidence level (A/B/C/D) + source. D-level = do NOT recommend.

## Software Development
**NEVER**: magic numbers, swallowing exceptions, god objects, premature optimization, copy-paste code, stringly-typed, cargo cult architecture, no tests before refactor, reinventing crypto
**ALWAYS**: YAGNI, strangler fig migration, immutability default, fail fast, vertical slices, trunk-based dev, dependency inversion, observability triad, property-based testing

## Business & Finance
**NEVER**: scaling before PMF, ignoring unit economics, timing the market, hiring too fast, single channel dependency, building before validating
**ALWAYS**: validate before building, retain > acquire, index fund core, 18+ months runway, fire fast hire slow, pricing on value

## Design (UX/UI/Game)
**NEVER**: feature factory, dark patterns, accessibility overlays, A/B cargo cult, loot boxes, tutorial info dump, 4+ fonts, pixel-perfect obsession
**ALWAYS**: continuous discovery, 5-user testing, accessible-first, flow channel, core loop first, performance as UX, subtractive design

## Marketing & Sales
**NEVER**: vanity metrics, content without distribution, SEO gaming, spray-and-pray outreach, discounting to close, fake urgency
**ALWAYS**: mental availability + physical availability, 60/40 brand/performance, SPIN selling, multi-touch attribution, retention first

## Psychology & Behavior
**NEVER**: NLP, learning styles VAK, 10000-hour rule, MBTI, left/right brain, brainstorming > individuals, ego depletion, catharsis theory
**ALWAYS**: spaced repetition, retrieval practice, implementation intentions, CBT principles, psychological safety, habit loop design

## Productivity & AI
**NEVER**: "you are an expert" prompts, CoT everywhere, context window stuffing, LLM as calculator, multitasking, higher temp = creative
**ALWAYS**: engineer context not prompts, few-shot examples, deep work blocks, model routing by task, verify then trust, defense-in-depth

## Science & Reasoning
**NEVER**: p-hacking, HARKing, publication bias, correlation=causation, cherry-picking, small sample overgeneralization
**ALWAYS**: pre-registration, effect sizes over p-values, replication, Bayesian reasoning, steelmanning, falsifiability
