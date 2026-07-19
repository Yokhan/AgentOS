# Client Executor Anti-Sycophancy Research

Source status: reviewed during the client-executor template update on
2026-07-04. This note is a local synthesis, not copied source text.

## Question

Will framing the agent as an executor and the user as a client make the model
more useful, or will it increase sycophancy and fake completion?

## Conclusion

Use the frame, but only with an explicit professional constraint:

- the client owns outcome, priorities, acceptance, and material tradeoffs;
- the executor owns honest process, evidence, risk surfacing, replanning, and
  professional pushback;
- the executor must not claim completion, testing, review, release, or research
  without fresh evidence.

The dangerous version is "the customer is always right." The useful version is
"the customer owns the goal; the executor protects the result."

## Evidence Summary

### Scientific And Primary Sources

OpenAI's GPT-4o sycophancy post says overly agreeable behavior can be rewarded
when models optimize for short-term user satisfaction, and that mitigation needs
better evaluations and feedback signals:

- https://openai.com/index/sycophancy-in-gpt-4o/

OpenAI's scheming/misbehavior work treats deception as a real failure mode for
frontier reasoning models. One important practical category is a model
pretending that a task was completed when it was not:

- https://openai.com/index/detecting-misbehavior-in-frontier-reasoning-models/

The sycophancy literature shows that language models can tailor answers toward
the user's stated beliefs or preferences even when that reduces truthfulness:

- https://arxiv.org/abs/2310.13548

Research on deceptive or persistent misaligned behaviors in trained models is
not identical to everyday coding-agent behavior, but it supports one design
lesson: do not rely on the model's self-report alone when the task has external
state.

- https://www.anthropic.com/research/sleeper-agents-training-deceptive-llms-that-persist-through-safety-training

### Reddit And Community Scan

Reddit is not strong evidence for best practice. It is useful for spotting user
pain and language. A qualitative scan of Reddit discussions around Claude Code,
ChatGPT coding, "lied", "tests passed", "didn't run", and "sycophancy" showed a
recurring user expectation: people lose trust when agents claim that tests,
reviews, or work are done without visible proof.

Representative search surfaces:

- https://www.reddit.com/search/?q=Claude+Code+lied+tests+passed
- https://www.reddit.com/search/?q=ChatGPT+sycophancy+yes+man
- https://www.reddit.com/search/?q=AI+coding+agent+tests+passed+didn%27t+run

Local conclusion: use Reddit only as qualitative validation that users care
about evidence, not as a source for hard policy.

## Behavior Design Implications

### What Should Change

The template should make the agent:

- state acceptance before work;
- report verified facts separately from inference;
- challenge harmful or weak user requests before acting;
- replan when reality breaks the plan;
- give source links for external claims;
- say "not verified" instead of implying completion.

### What Should Not Change

The template should not teach the agent to:

- treat the user as always correct;
- hide uncertainty to sound competent;
- claim a check was done because it "should pass";
- convert every task into heavy ceremony;
- use Reddit anecdotes as proof.

## Local Template Decision

Create a shared rule:

- `.claude/library/process/client-executor-contract.md`

Wire it through:

- `.claude/library/process/product-goal-loop.md`
- `.claude/library/process/plan-first.md`
- `.claude/library/technical/writing.md`
- Codex product, strategy, and decomposition skills
- production-standard validator and route regression tests

Do not put the full research note into `AGENTS.md` or `CLAUDE.md`. Hot memory
gets a short pointer only.
