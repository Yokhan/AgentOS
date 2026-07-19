---
description: "Review any plan, decision, or task against strategic thinking principles. Ensures agents optimize for user VICTORY, not narrow task completion. Detects tunnel vision, silo optimization, and symptom treatment."
---
# Strategic Review

## When to Use
- Before starting any non-trivial task (>3 files, >1 hour estimated)
- When multiple agents will work in parallel
- When conditions change mid-task
- When a plan feels wrong but you can't articulate why
- When you've been stuck on the same problem for 3+ attempts

## The OODA Review Protocol

### Phase 1: Observe
- What is the current state of the project/system?
- What has been tried before? What worked? What didn't?
- What are the hard constraints (time, resources, technical debt)?
- What does the user actually need (vs what they literally said)?

### Phase 2: Orient
- **Commander's Intent**: What is the user's ultimate objective? Not the task — the PURPOSE.
- **Product/Business Priority**: Which real product user, user experience outcome, and app-specific revenue, loyalty, retention, conversion, activation, or KPI does this improve?
- **Center of Gravity**: What is the ONE thing that, if addressed, makes everything else easier?
- **Bottleneck**: What is currently limiting throughput? (Theory of Constraints)
- **Phase Detection**: Is this genesis (explore), improvement (optimize), stabilization (harden), or retirement (remove)?
- **Landscape**: Who else is affected? What are the second-order effects?

### Phase 3: Decide
- **Highest Leverage**: Which intervention produces most result for least effort?
- **Outcome Before Technical Neatness**: Is technical work directly unlocking, protecting, or measurably improving the user/business outcome?
- **Reversibility**: Can this be undone? Prefer reversible actions.
- **Positive-Sum**: Does this create value without destroying value elsewhere?
- **Victory Conditions**: What does success look like? How will we measure it?
- **Retreat Conditions**: At what point do we change approach?

### Phase 4: Act
- Start small, validate fast (PDCA)
- Flow over batch — deliver value continuously
- Maintain shared context with other agents and user
- Feed results back to Observe → new cycle

---

## 30 Strategic Principles (from 15 world frameworks)

### Orientation & Awareness

1. **See the Whole Before Acting** — Survey the full landscape before committing. The swordsman who sees only the blade misses the body. SOURCE: Sun Tzu, Musashi. DETECT: Agent jumps to code without reading requirements/context.

2. **Fog of War Is Normal** — You will never have complete information. Act effectively under uncertainty rather than blocking for perfect data. SOURCE: Clausewitz. DETECT: Agent paralyzed by "I need more context."

3. **Read the Phase** — Genesis needs exploration. Product needs engineering. Commodity needs automation. Retirement needs courage. Wrong approach for the phase = wasted effort. SOURCE: Wardley Mapping. DETECT: Over-engineering a prototype or under-engineering production.

4. **Terrain Awareness** — Greenfield vs legacy, solo vs team, MVP vs production demand fundamentally different strategies. SOURCE: Sun Tzu. DETECT: Applying production patterns to a hackathon project.

5. **Pattern Recognition** — Most situations have been encountered before. Use prior experience to accelerate correct response. SOURCE: I Ching, Boyd. DETECT: Agent treats every problem as novel when a known solution exists.

### Goal & Constraint Identification

6. **Commander's Intent First** — Define WHY and WHAT before HOW. Express intent, not micro-steps. SOURCE: Auftragstaktik/Mission Command. DETECT: Task has no stated purpose, only mechanical steps.

7. **Find the Bottleneck** — System improves only when the constraint improves. Everything else is waste. SOURCE: Goldratt (TOC). DETECT: Agent optimizes fast path while slow path determines overall throughput.

8. **Highest Leverage Intervention** — Small effort, maximum effect. Change feedback loops, not parameters. SOURCE: Meadows (Systems Thinking). DETECT: Agent tweaks 50 lines when an architectural change fixes the root cause.

9. **Define Victory Conditions** — OKR format: qualitative objective + quantitative key results. Know what "done" looks like BEFORE starting. SOURCE: OKR methodology. DETECT: Task starts with no success criteria.

10. **Positive-Sum Framing** — Seek solutions that create value for ALL stakeholders. SOURCE: Game Theory (Nash). DETECT: Solution trades off one user need for another without exploring alternatives.

### Execution Strategy

11. **Wu Wei — Effortless Action** — Find the path of least resistance. The best solution feels obvious. If it feels like fighting, wrong approach. SOURCE: Lao Tzu. DETECT: Complex workaround when a simple solution exists.

12. **Flow Over Batch** — Deliver value continuously in small increments. Don't stockpile work. SOURCE: Toyota/Deming. DETECT: Agent creates 15 files before testing any of them.

13. **Water Strategy** — Persistent, adaptable, flows around obstacles. Try alternatives before escalating. SOURCE: Lao Tzu, Musashi. DETECT: Agent crashes into a wall repeatedly instead of routing around.

14. **Speed Through the Loop** — Rapid OODA cycling. Fast iteration WITH validation beats both slow perfectionism and fast recklessness. SOURCE: Boyd. DETECT: Either no iteration (waterfall) or no validation (cowboy).

15. **Small Experiments, Rapid Learning** — Smallest possible test of core hypothesis. Minimize blast radius. SOURCE: PDCA. DETECT: Agent refactors entire codebase instead of testing approach on one module.

### Coordination & Alignment

16. **Cooperative Game Design** — Structure agent interactions so local optimization aligns with global optimization. SOURCE: Game Theory (mechanism design). DETECT: Agent optimizes its metric while degrading system.

17. **Cross-Functional Unity** — All agents pull in one direction. Silo optimization destroys system performance. SOURCE: Kotler, Deming. DETECT: Reviewer blocks code that implementer correctly prioritized for speed.

18. **Shared Vision** — Without shared vision, agents optimize locally. North Star must be visible. SOURCE: Senge. DETECT: Agents working at cross-purposes on the same project.

19. **Subordinate to Constraint** — Pace all work to match bottleneck capacity. Don't create upstream pile-ups. SOURCE: TOC. DETECT: Generating more code than can be reviewed/tested.

20. **Transparent Information Flow** — Decisions, findings, and reasoning must be visible. Information hoarding kills coordination. SOURCE: Meadows. DETECT: Agent completes work without explaining reasoning.

### Adaptation & Resilience

21. **Expect Friction** — Real execution encounters unforeseen difficulties. Build margin into plans. SOURCE: Clausewitz. DETECT: Plans with no error handling, no buffers, no fallbacks.

22. **No Fixed Form** — Overreliance on any single approach is a vulnerability. Match tool to context. SOURCE: Musashi. DETECT: Agent always reaches for the same pattern regardless of context.

23. **Plans Change, Planning Stays** — Hold plans loosely. The thinking endures even when the plan changes. SOURCE: Moltke. DETECT: Agent abandons all analysis when requirements change, starting from scratch.

24. **Multiple Options** — Develop contingency plans. Have fallback approaches ready. SOURCE: Moltke. DETECT: No Plan B when Plan A fails.

25. **Change = Normal** — Every state is transitional. Build strategies that assume change. SOURCE: I Ching. DETECT: Architecture that breaks when requirements evolve.

### Continuous Improvement

26. **Kaizen** — Every cycle should leave the system slightly better. SOURCE: Deming/Toyota. DETECT: No improvement captured after completing a task.

27. **Study, Don't Just Check** — When results differ from expectations, understand WHY deeply. SOURCE: PDCA. DETECT: "Tests pass" without understanding what was actually tested.

28. **Build Knowledge, Not Just Output** — Increase understanding of the system with each cycle. SOURCE: Senge. DETECT: Agent produces code without building a mental model.

29. **Fix the Loop, Not the Symptom** — If bugs recur, the feedback loop needs fixing (tests? architecture? requirements?). SOURCE: Systems Thinking. DETECT: Same type of bug fixed 3+ times.

30. **Know When to Stop** — Culmination point: good enough IS the optimum. SOURCE: Clausewitz. DETECT: Agent keeps refactoring past the point of diminishing returns.

---

## Strategic Anti-Patterns Checklist

Before completing any significant task, verify NONE of these apply:

- [ ] **Tunnel Vision** — Am I completing the literal task while ignoring the obvious larger problem?
- [ ] **Local Optimization** — Am I making one thing better while making the whole worse?
- [ ] **Technical Perfection Drift** — Am I improving code/tooling/architecture without a clear user or business outcome?
- [ ] **Wrong Problem** — Am I solving what was asked or what actually needs solving?
- [ ] **Ignoring the Constraint** — Am I improving non-bottlenecks?
- [ ] **Rigid Plan Execution** — Am I following a plan that reality has invalidated?
- [ ] **Speed Without Orientation** — Did I understand WHY before starting HOW?
- [ ] **Silo Execution** — Am I coordinating with other agents/tasks?
- [ ] **Gold Plating** — Am I engineering beyond what creates value?
- [ ] **Symptom Treatment** — Am I fixing root causes or patches?
- [ ] **Past Culmination** — Should I stop now? Is "good enough" actually good enough?

---

## Goal Selection Quick Reference

| Step | Question | Framework |
|------|----------|-----------|
| 1 | What is the user's ACTUAL goal? | Commander's Intent |
| 2 | What is the center of gravity? | Clausewitz |
| 3 | What is the current bottleneck? | Theory of Constraints |
| 4 | What phase are we in? | Wardley Mapping |
| 5 | What is the highest-leverage action? | Systems Thinking |
| 6 | What does victory look like? | OKR |
| 7 | What does retreat look like? | Moltke |
| 8 | Is this positive-sum? | Game Theory |
| 9 | Can this be reversed? | Risk Management |
| 10 | What is the smallest experiment? | PDCA |

---

## Key Sources
- Sun Tzu — Art of War (strategy, terrain, clarity)
- Miyamoto Musashi — Book of Five Rings (adaptability, no fixed form)
- I Ching — Book of Changes (change as constant, phase reading)
- Lao Tzu — Tao Te Ching (wu wei, water strategy, minimum force)
- Carl von Clausewitz — On War (friction, fog, center of gravity, culmination)
- John Boyd — OODA Loop (decision speed, orientation primacy)
- John Nash / Von Neumann — Game Theory (cooperative strategies, positive-sum)
- Donella Meadows / Peter Senge — Systems Thinking (leverage points, feedback loops)
- Simon Wardley — Wardley Mapping (evolution stages, situational awareness)
- Philip Kotler — Marketing Warfare (cross-departmental alignment)
- W. Edwards Deming / Toyota — TPS/Lean (flow, kaizen, continuous improvement)
- Eliyahu Goldratt — Theory of Constraints (bottleneck focus)
- PDCA / Shewhart-Deming — Plan-Do-Check-Act (scientific method applied)
- OKR / Commander's Intent / Auftragstaktik — Goal clarity with execution autonomy
- Helmuth von Moltke — Adaptive planning ("no plan survives contact")
