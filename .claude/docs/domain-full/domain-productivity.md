# Domain: Productivity & AI Use — Evidence-Based Guard

## NEVER Recommend (Myths & Anti-patterns)
1. **"You Are an Expert" Prompt Prefix** — Study of 162 personas: no improvement vs control (Zheng 2024). Instead: provide domain context and examples.
2. **"Think Step by Step" Everywhere** — For simple tasks, CoT adds 39-79% more tokens with no accuracy gain. Instead: reserve CoT for multi-step reasoning only.
3. **Context Window Stuffing** — "Lost in the Middle" (Liu 2023): U-shaped attention, middle info lost. Instead: curate to 30-50% window, critical info at start/end.
4. **LLM as Calculator/Database** — LLMs fail at arithmetic and hallucinate facts confidently. Instead: use code execution for math, RAG for facts.
5. **Multitasking as Productivity** — Task-switching costs 40% productivity (APA). Only 2.5% are genuine supertaskers. Instead: deep work, time-blocking.
6. **Hustle Culture (More Hours = More Output)** — Productivity drops sharply after 50hrs/week; at 55hrs it's negative. Instead: protect recovery, sleep, boundaries.
7. **No Output Verification** — Hallucination rates 3-27% across models. Instead: verify all AI outputs; implement human-in-the-loop for high-stakes.
8. **Agent Loops Without Termination** — Missing max_turns causes runaway costs. Instead: layer max iterations + cost caps + no-progress detection.
9. **Higher Temperature = More Creative** — Weakly correlated with novelty, moderately with incoherence (Peeperkorn 2024). Instead: creativity from better prompts.
10. **Panic-Driven Crisis Decisions** — Cortisol narrows cognition; snap decisions under acute stress are systematically wrong. Instead: pre-established frameworks, 10-min cooling.
11. **RAG Without Retrieval Evaluation** — Standard RAG pipelines retrieve-and-generate without checking if retrieval worked. Irrelevant docs = confident wrong answers. Instead: evaluate retrieval quality separately; implement relevance scoring.
12. **Naive Fixed-Size Chunking** — Breaking documents at arbitrary character counts destroys semantic units and context. Instead: semantic chunking respecting document structure, with overlap.
13. **Scheduling Without Energy Awareness** — Placing deep work during energy troughs (typically post-lunch). Instead: match task cognitive demand to circadian energy peaks (typically 9-11am, 3-5pm for most chronotypes).
14. **Ignoring Chronotype in Routines** — "Wake at 5am" advice fails for ~25% of population who are evening chronotypes. Instead: align peak work to your biological peak, not an arbitrary schedule.
15. **Prompt Template Worship** — Minor formatting changes shift accuracy up to 76 percentage points (Sclar 2024). What works on GPT-4 may degrade Claude. Instead: understand the principle, test on your specific model/task.

## ALWAYS Apply (Evidence-Based Principles)
1. **Engineer Context, Not Just Prompts** — Design full information architecture: task, examples, RAG data, tools, state. 50% quality improvement reported.
2. **Few-Shot Examples** — 2-5 examples show 10-50% improvement over zero-shot for most tasks. Format and distribution matter more than label accuracy.
3. **Deep Work Blocks** — Newport: 2-4 hours uninterrupted focused work produces more than 8 hours of fragmented time. Protect ruthlessly.
4. **Spaced Repetition for Learning** — 50-200% retention improvement vs massed practice. Use expanding intervals (Anki, etc.).
5. **Implementation Intentions** — "When X, I will Y" doubles follow-through (Gollwitzer). Convert vague goals to specific triggers.
6. **Model Routing by Task** — Small models for simple tasks (classification, extraction); frontier models for complex reasoning. 20-60x cost savings.
7. **Verify Then Trust** — All AI outputs need validation. Fabricated citations, stats, and code are generated with equal confidence to accurate ones.
8. **Defense-in-Depth for AI Security** — Prompt injection is OWASP #1 for LLMs. Layer: input sanitization, output filtering, privilege separation.
9. **Pre-Mortem for Decisions** — Imagine the decision failed; list what went wrong. Surfaces risks optimism bias hides.
10. **Blameless Post-Mortems** — After failures, analyze root cause without blame. Organizations that blame first repeat crises (Elliott 2009).
11. **Energy Management Over Time Management** — Manage energy (physical, emotional, mental, spiritual) not just hours. Strategic breaks every 90-120 min restore focus (Schwartz, The Energy Project).
12. **Ultradian Rhythm Alignment** — Work in 90-minute focused blocks followed by 15-20 min breaks. Matches natural attention cycles (Kleitman's Basic Rest-Activity Cycle).
13. **Hybrid Retrieval (Keyword + Semantic)** — Neither BM25 nor vector search alone covers all query types. Combining both improves precision and recall for RAG systems.
14. **Query Rewriting Before Retrieval** — Transform complex user queries before hitting the retriever. Decompose multi-part questions; use HyDE for better semantic matching.
15. **Sycophancy-Aware Interaction** — LLMs systematically agree with users regardless of accuracy. Never ask "are you sure?" (causes correct-to-wrong flips). Instead: use structured disagreement protocols and external validation.

## For Details
See `brain/03-knowledge/domains/crisis-fitness-productivity.md` and `ai-orchestration-prompting.md` for full practices.
