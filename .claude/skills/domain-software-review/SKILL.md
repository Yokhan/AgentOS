---
description: "Review code and architecture decisions against evidence-based software engineering practices. Detects cargo cult patterns, overengineering, and common anti-patterns."
---
# Domain Software Review

## How to Use
When reviewing code, architecture, or engineering decisions, check against these evidence-based practices.
Flag any anti-pattern found. Suggest the corresponding best practice. Cite evidence level.

**Evidence Scale:** A = systematic review/large-scale empirical study. B = seminal work/major industry study. C = expert consensus/widely accepted practice.

## Anti-Patterns (NEVER recommend)

### Code Level

1. **Magic Numbers / Hardcoded Values** — Unexplained numeric literals in code (e.g., `if (status == 3)`). WHY: destroys readability, error-prone refactoring. DETECT: bare numeric literals in conditionals, function args, array indices. INSTEAD: named constants, enums, config files. EVIDENCE: C

2. **Swallowing Exceptions (Silent Catch)** — Empty catch blocks discarding errors. WHY: ~20% of errors never reach logs; creates "silent failures" extremely difficult to debug. INSTEAD: log with stack trace, catch specific exceptions, rethrow when appropriate. EVIDENCE: B

3. **God Object / God Class** — Single class accumulating all responsibilities. WHY: 0.55 Spearman correlation between anti-pattern count and bug count (ArXiv 2021). DETECT: classes >500 LOC, >10 public methods, >15 dependencies. INSTEAD: decompose into cohesive classes with single responsibilities. EVIDENCE: B

4. **Stringly-Typed Programming** — Strings where enums/types should be used (`role = "admin"`). WHY: no compile-time checking, typos become runtime bugs, no IDE autocomplete. INSTEAD: enums, union types, typed constants. EVIDENCE: C

5. **Copy-Paste Programming** — Copying code without understanding it (Stack Overflow, AI output). WHY: propagates bugs, introduces undebugable code; "generative cargo cult programming" with AI. INSTEAD: understand before integrating, write tests. EVIDENCE: B

6. **Premature Optimization** — Optimizing before profiling identifies bottlenecks. WHY: wastes time on the 97% that does not matter (Knuth 1974). INSTEAD: profile first, optimize the critical 3%; choose good algorithms upfront. EVIDENCE: B

7. **Deep Nesting (Arrow Code)** — Functions with >3 levels of nested conditionals. WHY: neuroscience research shows exponentially increased cognitive workload (PMC 2023). DETECT: SonarSource cognitive complexity metric; visual arrow shape in code. INSTEAD: early returns, guard clauses, extract functions. EVIDENCE: B

8. **Boolean Blindness** — Bare booleans with unclear meaning (`process(true, false, true)`). WHY: caller cannot understand parameter meaning; high bug risk during maintenance. INSTEAD: named parameters, enums, option objects. EVIDENCE: C

9. **Mutable Global State** — Globally accessible mutable variables shared across modules. WHY: IEEE study (2020) found increased defect proneness; unpredictable, hard to test. DETECT: module-level let/var, static mutable fields, singletons holding state. INSTEAD: pass dependencies explicitly, use immutable data, limit scope. EVIDENCE: B

10. **Long Parameter Lists** — Functions taking >4-5 parameters. WHY: increases cognitive load, callers frequently confuse parameter order. INSTEAD: parameter objects, builder pattern, decompose function. EVIDENCE: C

11. **Commented-Out Code** — Dead code left in comments "just in case." WHY: creates confusion, accumulates noise; version control preserves history. INSTEAD: delete it; retrieve from git if needed. EVIDENCE: C

12. **Inappropriate Comments (What, Not Why)** — Comments restating what code does. WHY: redundant comments rot faster than code; >30min/day wasted on bad documentation. INSTEAD: self-documenting code for WHAT; reserve comments for WHY. EVIDENCE: C

13. **Ignoring Return Values / Error Codes** — Discarding error-indicating return values. WHY: equivalent to swallowing exceptions; creates silent invalid state. INSTEAD: always check return values; use result types. EVIDENCE: C

14. **Type Coercion Abuse** — Relying on implicit type conversions (JS: `"" == false`). WHY: produces surprising behavior, source of subtle bugs. INSTEAD: strict equality, explicit conversions, TypeScript. EVIDENCE: C

15. **Primitive Obsession** — Primitives representing domain concepts (`string email`). WHY: no validation at type level, invalid states representable. INSTEAD: value objects, domain types. EVIDENCE: C

16. **Null Returns (Billion Dollar Mistake)** — Returning null for normal absence of value. WHY: Tony Hoare called null references his "billion-dollar mistake"; top cause of production errors. INSTEAD: Option/Maybe types, null object pattern. EVIDENCE: C

17. **Callback Hell / Pyramid of Doom** — Deeply nested asynchronous callbacks. WHY: unreadable, unmaintainable, error handling nearly impossible. INSTEAD: async/await, Promises, reactive patterns. EVIDENCE: C


18. **Reinventing Security/Crypto** — Custom implementations of auth, encryption, or hashing. WHY: custom security code has near-100% vulnerability rate. DETECT: custom password hashing, hand-rolled JWT, custom session management. INSTEAD: bcrypt/argon2, OAuth/OIDC, established crypto libraries. EVIDENCE: A

19. **Overly Clever Code** — Code prioritizing cleverness over readability. WHY: code is read 10x more than written. DETECT: ternary chains >2 deep, regex without comments, bitwise ops for non-perf logic. INSTEAD: prefer boring, obvious code. EVIDENCE: C

20. **Hardcoded Credentials in Source** — API keys, passwords, connection strings in version control. WHY: credential theft is top attack vector. DETECT: patterns matching API keys, tokens, connection strings in source. INSTEAD: environment variables, secrets management, .gitignore. EVIDENCE: A

### Design Level

24. **Premature Abstraction** — Abstractions after seeing only 1-2 instances. WHY: Sandi Metz: "duplication is far cheaper than the wrong abstraction." Wrong abstraction costs compound. INSTEAD: Rule of Three; AHA (Avoid Hasty Abstractions). EVIDENCE: C

22. **DRY Obsession (Over-DRYing)** — Eliminating ALL duplication regardless of whether pieces represent same concept. WHY: creates coupling between unrelated features; overly generic abstractions. INSTEAD: DRY applies to knowledge, not syntax; use WET/AHA. EVIDENCE: C

23. **Singleton Overuse** — Singleton as global access point throughout the application. WHY: hidden global state, breaks encapsulation, makes testing extremely difficult. INSTEAD: dependency injection; create instances explicitly. EVIDENCE: C

21. **Inheritance for Code Reuse** — Class inheritance primarily to share code, not model "is-a." WHY: fragile base class problem; tight coupling; study of 93 Java programs confirmed. INSTEAD: composition over inheritance; mixins/traits. EVIDENCE: B

25. **Feature Envy** — Method using more features of another class than its own. WHY: misplaced responsibility, increased coupling. INSTEAD: move method to the class whose data it uses most. EVIDENCE: C

26. **DI Container Overuse** — Injecting every dependency even with one implementation. WHY: excessive interfaces and boilerplate; DI container becomes "black box." INSTEAD: DI where you genuinely need testability or multiple implementations. EVIDENCE: C

27. **Anemic Domain Model** — Domain objects as just data containers; all logic in services. WHY: loses OOP power, scatters business logic, hard to enforce invariants. INSTEAD: rich domain models where complexity warrants. EVIDENCE: C

28. **Leaky Abstractions Ignored** — Building on abstractions without understanding what they hide. WHY: Spolsky's Law -- all non-trivial abstractions leak; mysterious production failures. INSTEAD: understand at least one level below your abstraction. EVIDENCE: C

29. **Shotgun Surgery** — One change requiring modifications across many files. WHY: high risk of missing a location; expensive and error-prone. INSTEAD: consolidate related logic; co-locate behavior. EVIDENCE: C

30. **Golden Hammer** — Same familiar technology for every problem. WHY: architectural mismatch; 82% of devs choose tech for resume, not product (ICSE 2021). INSTEAD: evaluate tools against requirements; prefer boring technology. EVIDENCE: B

31. **Lava Flow** — Dead code and abandoned experiments lingering. WHY: cognitive load; developers afraid to remove. INSTEAD: delete aggressively; git preserves history; add tests if uncertain. EVIDENCE: C

32. **Accidental Complexity in Testing** — Tests coupled to implementation details. WHY: break on every refactor even when behavior preserved; false confidence. INSTEAD: test behavior through public interfaces; mock at boundaries. EVIDENCE: C

33. **Circular Dependencies** — Module A depends on B depends on A. WHY: impossible to understand, test, or deploy independently. INSTEAD: extract shared logic; dependency inversion. EVIDENCE: C

34. **Configuration Over Convention Excess** — Extensive XML/YAML config for every behavior. WHY: config files become their own unmaintainable codebase. INSTEAD: sensible defaults with convention-over-configuration. EVIDENCE: C

35. **Temporal Coupling** — Methods that must be called in specific order without enforcement. WHY: silent bugs when callers do not follow implicit protocol. INSTEAD: builder pattern, state machines, type-level enforcement. EVIDENCE: C

36. **Interface Bloat** — Interfaces with many methods forcing implementors to stub unused ones. WHY: violates ISP, creates coupling. INSTEAD: split into focused role interfaces (but avoid over-splitting). EVIDENCE: C


37. **No Tests Before Refactoring** — Refactoring without test coverage. WHY: no safety net means any change could break anything. DETECT: refactoring PRs with no test additions. INSTEAD: write characterization tests first. EVIDENCE: B

38. **Over-Mocking** — Mocking every dependency, testing mock behavior. WHY: tests pass but production fails. DETECT: tests with >3 mocks; mock setup longer than test. INSTEAD: mock at boundaries only; use fakes/in-memory implementations. EVIDENCE: C

### Architecture Level

44. **Premature Microservices** — Starting with microservices before understanding domain boundaries. WHY: Fowler: "almost all successful microservice stories started with a monolith." INSTEAD: well-modularized monolith; extract when needed. EVIDENCE: B

40. **Distributed Monolith** — Tightly coupled microservices requiring coordinated deployments. WHY: all distributed complexity, none of the benefits (no independent deployment). INSTEAD: if services cannot deploy independently, merge them. EVIDENCE: C

41. **Big Ball of Mud** — No discernible architecture; spaghetti dependencies. WHY: the most common anti-pattern; impossible to maintain. INSTEAD: enforce module boundaries; architecture fitness functions. EVIDENCE: B

42. **Big Bang Rewrite** — Rewriting entire system from scratch. WHY: Netscape lost the browser war this way; takes far longer than estimated. INSTEAD: Strangler Fig pattern -- incrementally replace. EVIDENCE: B

43. **Ignoring Fallacies of Distributed Computing** — Assuming reliable network, zero latency, infinite bandwidth. WHY: Deutsch's 8 fallacies (1994) consistently validated in production. INSTEAD: circuit breakers, retries with backoff, idempotency, timeouts. EVIDENCE: B

39. **Database as Integration Layer** — Multiple services sharing same database schema. WHY: invisible coupling; schema changes break all consumers. INSTEAD: each service owns its data; APIs/events for communication. EVIDENCE: C

45. **No Observability** — Production without logging, metrics, or tracing. WHY: diagnosis becomes guesswork; MTTR skyrockets. INSTEAD: three pillars -- logs, metrics, traces. EVIDENCE: C

46. **Environment-Specific Code** — Hardcoded URLs, credentials, feature flags in app code. WHY: deployment risks; "works on my machine" syndrome. INSTEAD: environment variables, config files, secrets management. EVIDENCE: C

47. **Shared Mutable State in Distributed Systems** — Multiple services directly mutating shared state. WHY: race conditions, lost updates, data corruption; CAP theorem proves fundamental limitation. INSTEAD: event sourcing, CQRS, consensus protocols. EVIDENCE: B

48. **Ignoring Technical Debt** — Letting debt accumulate without tracking or addressing. WHY: 42% of dev time wasted on debt (Stripe); $1.52T annual US cost (CISQ); 25-50% slower delivery (McKinsey). INSTEAD: make visible, budget 15-20% capacity for reduction. EVIDENCE: A

49. **No Feature Flags for Risky Deployments** — Deploying risky changes without toggle capability. WHY: forces rollback of entire deployments. INSTEAD: feature flags for gradual rollout and dark deployment. EVIDENCE: C

50. **Inverted Testing Pyramid** — More E2E/UI tests than unit tests (ice cream cone). WHY: E2E tests are slow, flaky, 10x more expensive; bugs caught late cost 100x more. INSTEAD: many unit, fewer integration, minimal E2E. EVIDENCE: B

51. **Cargo Cult Architecture** — Adopting microservices/CQRS/K8s because "Netflix does it." WHY: what works for 1000+ engineers creates pure overhead for 5. INSTEAD: simplest architecture that solves actual problems. EVIDENCE: C

52. **Tightly Coupled Deployment** — All components deployed together despite different change rates. WHY: low-risk changes blocked by high-risk ones; deployment frequency drops. INSTEAD: independent deployment with contract testing. EVIDENCE: B

53. **No Automated Deployment Pipeline** — Manual deployment steps. WHY: "Accelerate" (n=23,000) showed elite performers deploy on demand with <1hr lead time; manual deployment is the #1 bottleneck. INSTEAD: CI/CD. EVIDENCE: A

54. **Security as Afterthought** — Adding security only late or after incidents. WHY: architectural flaws in production cost 100x more to fix than during design (NIST). INSTEAD: shift-left; threat modeling; SAST/DAST in CI. EVIDENCE: A

55. **Vendor Lock-In Without Exit Strategy** — Core business logic on proprietary APIs without abstraction. WHY: migration costs grow exponentially; vendor price changes threaten business. INSTEAD: adapter/anti-corruption layers for vendor integrations. EVIDENCE: C

## Best Practices (ALWAYS consider)

### Code Level

1. **Meaningful Naming** — Intention-revealing names, 2-4 natural language words. WHY: significantly improves comprehension (Relf naming study); enables self-documenting code. EVIDENCE: B

2. **Small Focused Functions** — <20 lines, single level of abstraction. WHY: easier to name, test, review; reviewers catch defects best in 200-400 LOC chunks. EVIDENCE: B

3. **Fail Fast and Loud** — Detect invalid conditions early, throw clear errors immediately. WHY: bugs caught closer to source; reduces debugging time up to 50%. EVIDENCE: C

4. **Use the Type System** — Leverage static types, generics, value objects to make illegal states unrepresentable. WHY: shifts error detection from runtime to compile time; eliminates entire bug categories. EVIDENCE: C

5. **Immutability by Default** — Prefer const/readonly/final; return new copies. WHY: IEEE research found mutable state increases defect proneness; eliminates race conditions. EVIDENCE: B

6. **Guard Clauses (Early Return)** — Handle edge cases and errors at function top, then happy path. WHY: reduces nesting depth; neuroscience shows cognitive complexity correlates with comprehension difficulty. EVIDENCE: B

7. **Explicit Error Handling** — Result/Either types, checked exceptions, explicit error returns. WHY: makes error paths visible in type signatures; forces callers to handle failures. EVIDENCE: C

8. **Consistent Code Formatting** — Automated formatters (Prettier, Black, gofmt) enforced by CI. WHY: eliminates style debates in review; reduces cognitive load. EVIDENCE: C

9. **Comments That Explain Why** — Business decisions, non-obvious trade-offs, workarounds. WHY: well-documented code reduces debugging time up to 50%; explains reasoning code cannot. EVIDENCE: C

10. **Avoid Deep Nesting** — Max 2-3 levels using extraction, early returns, decomposition. WHY: neuroscience research demonstrates exponential cognitive difficulty increase with nesting. EVIDENCE: B

11. **Descriptive Test Names** — Scenario + expected outcome (`shouldRejectOrderWhenInventoryIsZero`). WHY: failing test names become documentation; immediately clear what broke. EVIDENCE: C

12. **Limit Function Parameters** — 3 or fewer; group related params into objects. WHY: eliminates parameter ordering bugs; makes calls self-documenting. EVIDENCE: C

13. **Defensive Copying at Boundaries** — Pass copies of mutable data across boundaries, not references. WHY: prevents action-at-a-distance bugs; maintains encapsulation. EVIDENCE: C

14. **Dead Code Deletion** — Remove unused code aggressively rather than commenting out. WHY: less cognitive load; git preserves history; less maintenance surface. EVIDENCE: C

15. **Property-Based Testing** — For pure functions with wide input space, generate random inputs. WHY: catches edge cases unit tests miss; reveals hidden assumptions. EVIDENCE: B

### Design Level

16. **Composition Over Inheritance** — Build behavior by composing objects. WHY: more flexible; avoids fragile base class problem; 93-program study confirmed overuse of inheritance. EVIDENCE: B

17. **Dependency Inversion (Pragmatic)** — Abstractions at architectural boundaries, not everywhere. WHY: decouples high-level policy from low-level details where it matters. KEY: only create interfaces with genuine variation. EVIDENCE: C

18. **Separation of Concerns** — Distinct sections per concern (logic/data/presentation). WHY: enables independent development, testing, modification. EVIDENCE: C

19. **Rule of Three** — Wait for 3 instances of genuine duplication before abstracting. WHY: two instances rarely provide enough information for the right abstraction (Metz). EVIDENCE: C

20. **Design for Testability** — Structure code for isolated component testing. WHY: meta-analysis of 27 TDD studies found positive quality effect; RCTs showed improved coverage and maintainability. EVIDENCE: A

21. **Encapsulate What Varies** — Separate varying aspects from stable ones. WHY: limits blast radius of change; fundamental principle behind most design patterns. EVIDENCE: C

22. **Rich Domain Model (Where Warranted)** — Business logic on data-owning objects. WHY: enforces invariants at source; self-documenting domain. KEY: CRUD apps can use anemic models fine. EVIDENCE: C

23. **API First Design** — Design API contract before implementation. WHY: catches mismatches early; enables parallel client/server development. EVIDENCE: C

24. **Repository Pattern** — Abstract data access behind interface. WHY: swappable persistence; simplified testing. KEY: don't over-abstract for simple CRUD. EVIDENCE: C

25. **Behavior-Driven Testing** — Test public behavior/API, not implementation details. WHY: tests survive refactoring; reduces maintenance burden. EVIDENCE: C

26. **Event-Driven Decoupling** — Events/messages for async communication where appropriate. WHY: reduces temporal coupling; enables independent scaling. KEY: adds complexity -- use judiciously. EVIDENCE: C

27. **Strategic Design Patterns** — Patterns to solve identified problems, not preemptively. WHY: GoF warned against overuse; pattern without matching problem is anti-pattern. EVIDENCE: C

28. **Feature Flags** — Wrap risky/incomplete features behind toggles. WHY: enables trunk-based dev, gradual rollout, instant rollback. EVIDENCE: B

29. **Bounded Contexts** — Clear boundaries where domain models apply, with explicit translation at edges. WHY: prevents model pollution; prerequisite for service extraction. EVIDENCE: C

30. **Anti-Corruption Layer** — Adapter between external/legacy systems and your domain model. WHY: protects domain from external model pollution; isolates vendor change. EVIDENCE: C

31. **Strangler Fig Pattern** — Incrementally replace legacy components via routing facade. WHY: avoids big-bang rewrite risk; old system runs while new proves itself. EVIDENCE: C

32. **Code Review (200-400 LOC)** — Focused review batches for maximum defect detection. WHY: individual inspections catch ~60% of defects; combined catch 70-85%. Drops sharply beyond 400 LOC. EVIDENCE: B

33. **Contract Testing** — Verify service interfaces meet agreed contracts. WHY: catches integration issues early; faster and more reliable than E2E tests. EVIDENCE: C

### Architecture Level

34. **Monolith First** — Start well-structured monolith; extract services only when scaling demands. WHY: Fowler: "almost all successes started with a monolith." GitHub and Shopify prove monoliths scale. EVIDENCE: B

35. **Testing Pyramid** — Many fast unit tests, fewer integration, minimal E2E covering critical paths. WHY: bug in unit test costs ~$1, in E2E ~$100, in production ~$1000+. EVIDENCE: B

36. **CI/CD Pipeline Automation** — Automate build, test, deploy in continuous pipeline. WHY: strongest predictor of delivery performance and organizational outcomes (Accelerate, n=23,000). EVIDENCE: A

37. **Design for Failure** — Circuit breakers, retries with backoff, timeouts, graceful degradation. WHY: Deutsch's 8 Fallacies consistently validated; systems that ignore this suffer cascading outages. EVIDENCE: B

38. **Observability (Logs, Metrics, Traces)** — Structured logging, metrics, distributed tracing from the start. WHY: dramatically reduces MTTR; enables understanding production behavior. EVIDENCE: C

39. **12-Factor App Principles** — Battle-tested methodology for portable, scalable apps. WHY: covers config, dependencies, ports, logs; creates deployment-environment independence. EVIDENCE: B

40. **Choose Boring Technology** — Proven tech over newest shiny option. WHY: each new tech is an "innovation token" with limited budget; spend on what differentiates your business. EVIDENCE: C

41. **Evolutionary Architecture** — Support incremental change via fitness functions. WHY: requirements evolve; big-upfront architecture becomes stale. EVIDENCE: C

42. **Infrastructure as Code** — Version-controlled infrastructure definitions. WHY: reproducible environments, auditable changes, disaster recovery. EVIDENCE: B

43. **Shift-Left Security** — SAST/DAST in CI, threat modeling during design. WHY: production security flaws cost 100x more to fix (NIST); IBM Data Breach report validates annually. EVIDENCE: A

44. **Trunk-Based Development** — Short-lived branches (<1 day) merging frequently to main. WHY: strong predictor of high delivery performance (Accelerate); reduces merge conflicts. EVIDENCE: A

45. **Data Ownership Per Service** — Each service owns its data store; no shared databases. WHY: enables independent evolution, deployment; each team chooses appropriate storage. EVIDENCE: C

46. **Incremental Database Migrations** — Versioned, reversible migration scripts. WHY: repeatable deployments; supports rollback; zero-downtime with feature flags. EVIDENCE: B

47. **Load Testing Before Launch** — Performance test under realistic load pre-production. WHY: performance problems found in production are most expensive to fix. EVIDENCE: C

48. **Graceful Degradation** — Partial functionality over total failure when dependencies fail. WHY: users prefer reduced service over complete outage; improves perceived reliability. EVIDENCE: C

49. **Technical Debt Budget** — 15-20% of development capacity for debt reduction. WHY: high-debt orgs deliver 25-50% slower (McKinsey); 42% dev time wasted (Stripe). EVIDENCE: A

50. **Architecture Decision Records** — Document decisions with context, options, and rationale. WHY: prevents re-litigating; helps new members understand the "why." EVIDENCE: C

51. **Measure Before Optimizing** — Profile and measure before optimizing; algorithmic improvements first. WHY: Knuth's actual position supports algorithmic efficiency but opposes blind micro-optimization. EVIDENCE: B

52. **Vertical Slice Architecture** — Organize by feature, not by layer; each slice independently deployable/testable. WHY: reduces cross-cutting changes; aligns team ownership with feature boundaries. EVIDENCE: C

53. **Chaos Engineering** — Intentionally inject failures to build confidence in system resilience. WHY: Netflix pioneered; proactive failure testing prevents unexpected production outages. EVIDENCE: B

## Architectural Decision Heuristics

When evaluating architectural choices, apply these decision rules:

- **Monolith vs Microservices**: If <10 engineers OR unclear domain boundaries OR pre-PMF -> monolith. Extract only at proven bottlenecks.
- **SQL vs NoSQL**: If relationships matter OR need transactions -> SQL. If schema flexibility OR massive write throughput -> NoSQL. When in doubt, PostgreSQL.
- **Sync vs Async**: If caller needs immediate result -> sync. If fire-and-forget OR long-running -> async. If order matters -> ordered queue.
- **Build vs Buy**: If core differentiator -> build. If commodity (auth, email, logging) -> buy/use existing.
- **Framework vs Library**: If you want control -> library. If you want convention -> framework. For teams >5, convention usually wins.
- **Rewrite vs Refactor**: Almost always refactor (Strangler Fig). Rewrite only if codebase is <6 months old OR no users depend on current behavior.
- **Strong vs Eventual Consistency**: If money/safety -> strong. If social feeds/analytics -> eventual. Mixed is common.

## Common LLM Mistakes in This Domain

1. **Over-recommending design patterns** — Suggesting Factory/Strategy/Observer for simple problems. The GoF book itself warns against overuse. Most code does not need formal patterns; a simple function often suffices.

2. **Defaulting to microservices** — Training data heavily biased toward microservices content. Most projects should start as monoliths and extract services only when scaling demands it.

3. **Excessive abstraction and interface creation** — Creating interfaces for every class. An interface with one implementation is just indirection, not abstraction.

4. **Over-applying SOLID as rigid rules** — Splitting simple code into excessive classes. SOLID principles are trade-offs, not commandments. A 50-line class with two reasons to change is often better than five 10-line classes.

5. **DRY before understanding duplication** — Aggressively extracting shared code from unrelated domains. Sandi Metz: "Duplication is far cheaper than the wrong abstraction." Wait for three instances.

6. **Over-commented code** — Generating `// Set value to 5` comments. Research shows bad comments are worse than none. Comments should explain WHY, not WHAT.

7. **Overly complex error handling** — Elaborate try-catch hierarchies with custom exception classes for prototypes. Match error handling complexity to application needs.

8. **Popular frameworks over simple solutions** — React for static pages, Kubernetes for single-server apps, Redis when a HashMap suffices. Evaluate whether a dependency is justified.

9. **Misunderstanding premature optimization** — Either over-optimizing trivially (avoiding string concatenation in one-off code) or missing genuine algorithmic inefficiencies (O(n^2) where O(n log n) is straightforward). Choose good algorithms upfront; profile before micro-optimizing.

10. **Confidently generating subtly wrong code** — MIT research (2025) found LLMs leverage grammatical patterns over domain knowledge. When making early mistakes, they elaborate rather than self-correct. 40%+ of syntactic errors involve missing/incorrect code blocks. Always verify.

## Key Sources
- Forsgren, Humble, Kim. *Accelerate* (2018) -- DORA metrics, n=23,000
- Fowler. *Refactoring* (2018); MonolithFirst, StranglerFig, Practical Test Pyramid
- Martin. *Clean Architecture* (2017); *Clean Code* (2008) -- apply critically
- Evans. *Domain-Driven Design* (2003)
- McConnell. *Code Complete* (2004)
- Sandi Metz. "The Wrong Abstraction" (2016)
- Knuth. "Structured Programming with go to Statements" (1974)
- IEEE mutable global state study (2020)
- PMC neuroscience-based code complexity study (2023)
- ArXiv design anti-pattern mutations study (2021)
- ACM ICSE-SEIS Resume-Driven Development paper (2021)
- Springer TDD meta-analysis (27 studies)
- Stripe Developer Coefficient report; McKinsey Digital tech debt research
- CISQ Cost of Poor Software Quality report
- NIST; IBM Cost of a Data Breach annual report
- MIT research on LLM reliability (2025)
