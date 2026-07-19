---
description: "Review product, graphic, and game design decisions against evidence-based design practices. Detects dark patterns, feature bloat, accessibility issues."
---
# Domain Design Review

## How to Use
When reviewing product, UX, graphic, or game design decisions, check against these evidence-based practices.
Covers three disciplines: product/UX design, graphic/visual design, and game design.

**Evidence Scale:** A = systematic review/large-scale empirical study. B = seminal work/major study. C = expert consensus/case studies.

## Anti-Patterns (NEVER recommend)

### Product & UX Anti-Patterns

1. **Feature Factory Mindset** — Measuring success by features shipped, not outcomes. WHY: teams become output machines disconnected from user value. INSTEAD: outcome-driven development with empowered product teams. EVIDENCE: B

2. **The Build Trap** — Locking into plans before evidence, measuring shipping velocity. WHY: teams ship features no one uses. INSTEAD: measure outcomes (retention, activation, revenue impact). EVIDENCE: B

3. **HiPPO-Driven Decisions** — Highest Paid Person's Opinion overrides data and research. WHY: ignores user needs, demoralizes product teams. INSTEAD: expose stakeholders to user research sessions. EVIDENCE: C

4. **Vanity Metrics Obsession** — Page views, downloads, follower counts as primary KPIs. WHY: not actionable, mask engagement problems, mislead strategy. INSTEAD: actionable metrics tied to behavior (activation, retention cohorts, NPS). EVIDENCE: B

5. **Dark Patterns / Deceptive Design** — UI tricks manipulating users into unintended actions. WHY: 97% of top EU sites use at least one (EC 2022); 76%+ in FTC 2024; carries legal liability under EU DSA. INSTEAD: transparent opt-ins, clear cancellation, honest defaults. EVIDENCE: A

6. **Accessibility as Afterthought** — WCAG compliance as end-of-development bolt-on. WHY: 96.3% of top 1M pages fail WCAG (WebAIM Million); retrofitting is 10x costlier than building inclusively. INSTEAD: integrate from first design sprint. EVIDENCE: A

7. **Accessibility Overlay Reliance** — JavaScript widgets (accessiBe, UserWay) as substitute for real work. WHY: 72% of disabled users say overlays are not effective; FTC fined accessiBe $1M for false claims. INSTEAD: semantic HTML, ARIA, keyboard navigation. EVIDENCE: A

8. **A/B Testing Cargo Cult** — Tests without proper methodology: peeking, no power calculation. WHY: peeking inflates false positive rate from 5% to 26.1%; 1 in 4 "winners" is noise. INSTEAD: pre-register hypothesis, calculate sample size, sequential testing. EVIDENCE: A

9. **Ignoring Cognitive Load** — Too many choices, too much information simultaneously. WHY: Miller's Law (7+/-2 chunks); Hick's Law (more options = slower decisions = abandonment). INSTEAD: chunk information, progressive disclosure, limit choices. EVIDENCE: A

10. **Ignoring Fitts's Law** — Interactive targets too small or too far from cursor/finger. WHY: small targets increase error rate; Apple HIG minimum 44x44px touch targets. INSTEAD: size proportional to importance, reduce sequential distances. EVIDENCE: A

11. **Skipping User Research Entirely** — Designing based on assumptions or competitor copying. WHY: $1 UX investment returns $100 (Forrester); 5 users catch 85% of major issues (Nielsen). INSTEAD: even minimal testing with target users. EVIDENCE: B

12. **Feature Bloat / Creep** — Continuously adding without removing or simplifying. WHY: cognitive load increases; maintenance cost exponential; onboarding harder. INSTEAD: audit usage, sunset unused, "one in, one out." EVIDENCE: B

13. **MVP as Minimum Product** — Stripping MVP until unusable, confusing "minimum" with "low quality." WHY: users reject products without core value; 74% of fast-growing startups fail by scaling pre-validation. INSTEAD: smallest thing delivering real value AND enabling learning. EVIDENCE: B

14. **Onboarding Information Dump** — Front-loading all education before value experience. WHY: 90% churn if no value in first week; context-free info forgotten instantly. INSTEAD: progressive onboarding tied to user actions; help at point of need. EVIDENCE: B

15. **Mandatory Account Before Value** — Requiring signup before any product experience. WHY: each friction point reduces completion 3-8%; users abandon before understanding value. INSTEAD: value-first, account-second. EVIDENCE: B

16. **Launching Without Analytics** — No instrumentation to measure feature success. WHY: flying blind; product debates become HiPPO battles without data. INSTEAD: define success metrics, instrument key events before launch. EVIDENCE: C


17. **Design Theater** — Creating polished mockups without validating with real users. WHY: pretty deliverables create false confidence. INSTEAD: test low-fidelity prototypes early. EVIDENCE: C

18. **Building What Users Literally Request** — Taking feature requests at face value. WHY: users cannot articulate solutions. INSTEAD: Jobs-to-be-Done framework. EVIDENCE: B

19. **Say-Do Gap Blindness** — Relying solely on surveys without observing behavior. WHY: users rate products highly while struggling. INSTEAD: behavioral analytics over attitudinal data. EVIDENCE: A

20. **Confirmation Bias in Research** — Designing studies to confirm existing beliefs. WHY: cherry-picks favorable data. INSTEAD: pre-register hypotheses; include disconfirming questions. EVIDENCE: B

21. **Notification Spam** — Over-notifying to boost engagement metrics. WHY: users disable notifications entirely or uninstall. INSTEAD: notify only for relevant triggers; granular controls. EVIDENCE: C

22. **Infinite Scroll Without Context** — Infinite scroll for goal-directed search. WHY: users lose place; footer inaccessible. INSTEAD: pagination for search; infinite scroll for casual browsing. EVIDENCE: B

### Graphic & Visual Design Anti-Patterns

17. **Using More Than 3 Fonts** — 4+ typefaces creating visual chaos. WHY: each new font resets cognitive processing, increasing load ~20%. INSTEAD: 2-3 fonts max (display, body, optional accent). EVIDENCE: B

18. **Canva Template Thinking** — Treating templates as finished design without understanding structure. WHY: ignoring grid, hierarchy, and spacing logic produces generic, misaligned work. INSTEAD: use templates to learn principles, customize structure. EVIDENCE: B

19. **Ignoring Whitespace** — Filling every pixel with content, icons, or decorative elements. WHY: whitespace increases comprehension by up to 20% (Wichita State study). INSTEAD: generous margins, padding, breathing room. EVIDENCE: A

20. **Poor Contrast / Readability** — Insufficient text/background contrast. WHY: WCAG requires 4.5:1 minimum; ~300M people with color vision deficiency globally. INSTEAD: test contrast ratios, comply with WCAG AA. EVIDENCE: A

21. **Decorative Over Functional** — Prioritizing visual beauty at expense of usability. WHY: users cannot find the checkout button if it is behind gradients and clever layouts. INSTEAD: functional clarity first, aesthetics layered on top. EVIDENCE: C

22. **Inconsistent Visual Language** — Different styles, colors, iconography across same product. WHY: users cannot build reliable mental models; undermines trust. INSTEAD: design tokens, consistent systems. EVIDENCE: B


23. **Ignoring Grid Systems** — No underlying grid structure. WHY: inconsistent alignment creates visual tension. INSTEAD: baseline grid + column grid. EVIDENCE: B

24. **Pure Black on Pure White** — Maximum contrast for body text. WHY: excessive luminance causes halation and eye strain. INSTEAD: near-black on off-white. EVIDENCE: B

25. **Ignoring Color Blindness** — Relying solely on color to convey info. WHY: 8% of men have color vision deficiency. INSTEAD: pair color with icons, patterns, text. EVIDENCE: A

26. **Stock Photo Overuse** — Generic stock photos as primary visuals. WHY: 39% of marketers report stock photos are worst-performing assets. INSTEAD: custom photography. EVIDENCE: A

27. **Complex Logo Myth** — Believing detailed logos are more professional. WHY: research shows simple logos enhance competence perception. INSTEAD: design for scalability. EVIDENCE: A

### Game Design Anti-Patterns

23. **Loot Box Gambling** — Randomized paid rewards with hidden odds exploiting variable-ratio reinforcement. WHY: 1-in-10 young players borrow money they cannot repay (UK Gambling Commission); illegal in Belgium/Netherlands. INSTEAD: transparent cosmetic shops. EVIDENCE: A

24. **FOMO Time-Limited Events** — Artificial urgency through time-gated content. WHY: exploits loss aversion; players feel punished for having a life outside the game, leading to resentment. INSTEAD: rotating availability with eventual permanent access. EVIDENCE: A

25. **Pay-to-Win Mechanics** — Selling gameplay advantages for competitive edge. WHY: destroys competitive integrity; creates compulsive spending cycles (Korean mobile game research). INSTEAD: cosmetics-only monetization (Fortnite, Path of Exile model). EVIDENCE: A

26. **Artificial Difficulty (Stat Inflation)** — Increasing difficulty by inflating enemy HP/damage only. WHY: negates strategic approaches; devolves into tedious grind without skill requirement. INSTEAD: design challenges requiring new skills or strategies. EVIDENCE: B

27. **Tutorial Hell (Information Dump)** — Front-loading all mechanics before gameplay begins. WHY: violates cognitive load principles (Hodent); working memory is limited. INSTEAD: progressive disclosure; one mechanic at a time through gameplay; Nintendo World 1-1 philosophy. EVIDENCE: A

28. **HUD Overload** — Displaying all possible information simultaneously. WHY: obstructs gameplay view; overwhelms attention. INSTEAD: prioritize essential information; progressive disclosure; allow customization. EVIDENCE: B

29. **Ignoring Game Accessibility** — No colorblind modes, remappable controls, subtitle options, text scaling. WHY: 31% of US gamers have a disability; The Last of Us Part II shipped 60+ accessibility settings to universal praise. INSTEAD: follow Game Accessibility Guidelines. EVIDENCE: A

30. **"Realistic = Fun" Misconception** — Simulation fidelity over enjoyment. WHY: Sid Meier cut scurvy from Pirates! because he "couldn't make it fun." Nintendo's stylized Mario outsells most realism. INSTEAD: realism serves immersion selectively; sacrifice when it conflicts with fun. EVIDENCE: B

31. **Copy-Paste Mechanics** — Borrowing mechanics without understanding why they work in context. WHY: a mechanic works within a specific design ecosystem; transplanting without context produces incoherence. INSTEAD: study WHY it works, then design your own solution. EVIDENCE: B

32. **Ludonarrative Dissonance (Unintentional)** — Gameplay contradicting the narrative. WHY: Uncharted's Nathan Drake is a "lovable rogue" who murders thousands; breaks immersion. INSTEAD: align mechanics with narrative themes. EVIDENCE: B

33. **Unchecked Scope Creep** — Scope expanding without resource/timeline evaluation. WHY: 70%+ of indie games exceeding scope failed deadlines (Game Developer Magazine). INSTEAD: MVP-first; "cut early, cut often." EVIDENCE: A

34. **Crunch Culture as "Passion"** — Extended mandatory overtime. WHY: "Crunch does not work" -- productivity declines after 4 days; more bugs, lower Metacritic. INSTEAD: sustainable pace, scope management. EVIDENCE: A

35. **No Game Juice** — Mechanically sound but no visual/audio/haptic feedback. WHY: correct actions feel empty without feedback; "juice" is the tactile connection sustaining engagement. INSTEAD: layer VFX, audio, haptics on every meaningful interaction. EVIDENCE: B

36. **Shipping Without Polish** — Feature-complete but unpolished release. WHY: first impressions at launch determine commercial success; unpolished feels janky regardless of content. INSTEAD: cut features before cutting polish time; schedule polish sprints. EVIDENCE: C

37. **Artificial Scarcity** — Restricting digital content for false exclusivity. WHY: digital goods have zero marginal cost; inherently deceptive. INSTEAD: earned exclusivity through skill/time. EVIDENCE: A

38. **Monetization-First Design** — Systems built around monetization funnels. WHY: extractive designs kill long-term revenue. INSTEAD: player-first design. EVIDENCE: B

39. **Intentional Grind Walls** — Free gameplay so tedious players pay to skip. WHY: deliberately degraded experience. INSTEAD: earn-or-buy parity. EVIDENCE: A

40. **Rubber-Band AI Without Transparency** — Hidden dynamic difficulty. WHY: detected assistance feels patronizing. INSTEAD: transparent difficulty adjustment. EVIDENCE: C

41. **Punishing Exploration** — Penalizing curiosity with traps/dead ends. WHY: exploration is intrinsically motivating. INSTEAD: reward curiosity; signal danger. EVIDENCE: C

42. **Inconsistent Rules** — Systems breaking their own logic. WHY: destroys mental models and trust. INSTEAD: establish and follow rules. EVIDENCE: B

43. **Balance by Spreadsheet Only** — Relying exclusively on math for balance. WHY: cannot predict real player behavior. INSTEAD: spreadsheets + playtesting. EVIDENCE: B

44. **Burden of Knowledge (Zileas)** — Mechanics requiring outside knowledge. WHY: nothing communicates mechanics in-game. INSTEAD: visual/audio language for all mechanics. EVIDENCE: B

45. **Cutscene-Gameplay Segregation** — Narrative only in non-interactive cutscenes. WHY: two disconnected experiences. INSTEAD: environmental storytelling. EVIDENCE: B

46. **Save System Hostage-Taking** — Restrictive saves forcing long replays. WHY: disrespects player time. INSTEAD: generous autosave + checkpoints. EVIDENCE: B


## Best Practices (ALWAYS consider)

### Product & UX Best Practices

1. **Outcome-Driven Development** — Success = measurable user/business outcomes, not features shipped. WHY: 70% of OKR-adopting orgs report improved alignment (McKinsey 2022). EVIDENCE: B

2. **Continuous Discovery** — Weekly customer interviews + assumption testing. WHY: prevents stale assumptions; builds compounding customer understanding. EVIDENCE: B

3. **Jobs-to-be-Done Framework** — Focus on what users are trying to accomplish, not demographics. WHY: reveals underlying motivations; prevents "faster horse" problem. EVIDENCE: B

4. **Nielsen's 10 Usability Heuristics** — Broad rules from 249 usability problems. WHY: confirmed across 2,048 users in 14 countries; unchanged and valid since 1994. EVIDENCE: A

5. **Progressive Disclosure** — Essential info first; reveal complexity as needed. WHY: improves learnability, efficiency, and error rate (NNG). EVIDENCE: B

6. **5-User Iterative Testing** — 5 users per round, fix, repeat. WHY: 5 users find ~85% of major usability problems per round; maximizes learning per dollar. EVIDENCE: B

7. **Accessible-First Design** — WCAG compliance from first sprint. WHY: prevents 10x retrofit cost; expands market by 15-20% (disabled users). EVIDENCE: A

8. **Cognitive Load Management** — Chunk to 5-9 items; limit choices per screen. WHY: aligns with Miller's and Hick's Laws; reduces abandonment. EVIDENCE: A

9. **Value-First, Account-Second** — Experience core value before requiring signup. WHY: doubles free-to-paid conversion; each removed friction point improves completion 3-8%. EVIDENCE: B

10. **Performance as UX Metric** — Load time and responsiveness as core UX metrics. WHY: 53% abandon at >3s (Google); each second costs ~7% conversions. EVIDENCE: A

11. **Pre-Registered Experiments** — Hypothesis, metric, sample size defined before A/B tests. WHY: prevents peeking bias (26% vs 5% false positive rate). EVIDENCE: A

12. **Error-State-First Design** — Design errors, empty states, edge cases before happy path. WHY: users encounter errors constantly; recovery actions prevent abandonment. EVIDENCE: C

13. **Baseline Before Redesign** — Quantitative UX metrics before changing anything. WHY: without baseline, cannot prove improvement; prevents expensive regressions. EVIDENCE: B

14. **Friction Auditing** — Map every step in critical flows; identify each friction point. WHY: each removed friction point improves completion 3-8%; makes optimization systematic. EVIDENCE: B

### Graphic & Visual Best Practices

15. **Visual Hierarchy** — Size, weight, color, position guide attention deliberately. WHY: users scan in F-pattern (NNG); hierarchy determines what gets seen first. EVIDENCE: A

16. **Whitespace as Design Element** — Intentional negative space. WHY: increases comprehension up to 20% (Wichita State); guides the eye. EVIDENCE: A

17. **Contrast Ratio Compliance** — 4.5:1 normal text, 3:1 large text (WCAG AA). WHY: benefits everyone in poor lighting; legally required; excludes ~300M globally if missed. EVIDENCE: A

18. **Design Tokens** — Semantic variables for colors, spacing, typography. WHY: enables systematic theme changes and consistency at scale. EVIDENCE: C

19. **Typography Hierarchy** — Deliberate size/weight/spacing system. WHY: creates scannable, readable content structure. EVIDENCE: B

20. **Consistent Visual Language** — Uniform color coding, iconography, UI patterns. WHY: reduces cognitive load via pattern recognition; users learn the language once. EVIDENCE: B


21. **Typography with Type Scale** — Mathematical ratio for heading/body sizes. WHY: creates natural proportional harmony. EVIDENCE: B

22. **60-30-10 Color Rule** — 60% dominant, 30% secondary, 10% accent. WHY: natural visual balance. EVIDENCE: B

23. **Grid System Foundation** — 12-column web, modular editorial. WHY: alignment, consistency, proportion. EVIDENCE: A

24. **Limit Line Length (50-75 chars)** — Constrain body text width. WHY: fastest reading speed; 27% improvement for dyslexic readers. EVIDENCE: A

25. **Generous Line Spacing (1.4-1.6x)** — Body text line-height. WHY: improves reading accuracy. EVIDENCE: A

26. **Pair Fonts by Contrast** — Combine distinct typefaces. WHY: contrast creates hierarchy; similar fonts look like mistakes. EVIDENCE: B

27. **Apply Gestalt Principles** — Proximity, similarity, continuity, closure. WHY: how humans perceive visual grouping. EVIDENCE: A

28. **Optimize Images for Web** — Compress, use WebP/AVIF. WHY: images 50-80% of page weight. EVIDENCE: A

### Game Design Best Practices

21. **Flow Channel Design** — Balance challenge to player skill (Csikszentmihalyi). WHY: decades of research confirm flow produces optimal engagement, focus, satisfaction. EVIDENCE: A

22. **Core Loop First** — Validate fundamental gameplay loop before adding systems. WHY: if the 30-second loop is not fun, no content, story, or polish saves it. EVIDENCE: A

23. **Subtractive Design** — Remove until only essential mechanics remain. WHY: prevents overdesign; Portal succeeded with one mechanic explored deeply. EVIDENCE: B

24. **Player Agency and Meaningful Choice** — Decisions with visible consequences, multiple valid strategies. WHY: Sid Meier: "A game is a series of interesting decisions." EVIDENCE: A

25. **Show Don't Tell** — Teach through interactive demonstration, not text walls. WHY: players learn by doing; interactive tutorials produce faster mastery. EVIDENCE: B

26. **Reward Curiosity** — Consistently reward players who explore and experiment. WHY: intrinsic motivation (curiosity, mastery) produces deeper engagement than extrinsic (SDT). EVIDENCE: B

27. **MVP-First Game Development** — Core features playable and fun before expanding scope. WHY: real-world feedback guides feature additions; Stardew Valley model. EVIDENCE: A

28. **Early and Frequent Playtesting** — Real players from earliest prototype. WHY: one playtest reveals more than weeks of design discussion. EVIDENCE: A

29. **Accessibility from Day One** — Build accessibility in from start, not as post-launch patch. WHY: retrofitting 10x more expensive; 31% of US gamers have a disability. EVIDENCE: A

30. **Remappable Controls** — Full button remapping on all platforms. WHY: single most impactful motor accessibility feature; low cost, massive inclusion. EVIDENCE: A

31. **Game Juice (Layered Feedback)** — Visual + audio + haptic on every meaningful action. WHY: creates satisfying "game feel"; audio alone is cheapest high-impact improvement. EVIDENCE: B

32. **Cosmetics-Only Monetization** — Visual customization not affecting gameplay balance. WHY: Fortnite proved at billions; preserves competitive integrity; players feel purchases are optional. EVIDENCE: A

33. **Ethical Engagement Design** — Opt-in features, no exploitation of psychological vulnerabilities. WHY: regulatory scrutiny increasing; ethical design builds long-term brand trust. EVIDENCE: A

34. **Community-Informed Iteration** — Player feedback + telemetry guide post-launch development. WHY: No Man's Sky and FFXIV recovered from poor launches through community-responsive iteration. EVIDENCE: A

35. **Elegant Design** — Simple rules creating emergent complexity. WHY: Chess has 6 piece types but infinite depth. EVIDENCE: B

36. **Failing Forward** — Failure teaches and motivates retry. WHY: Dark Souls succeeds because every death is a lesson. EVIDENCE: B

37. **Intrinsic Motivation Design (SDT)** — Autonomy, competence, relatedness. WHY: sustained engagement without manipulation. EVIDENCE: A

38. **Transparent Pricing** — Clear purchase info; no obfuscated currencies. WHY: builds trust; reduces refunds. EVIDENCE: B

39. **Respect Non-Paying Players** — Full core experience without payment in F2P. WHY: non-payers drive word-of-mouth. EVIDENCE: B

40. **Negative Feedback Loops** — Help losing players stay competitive. WHY: prevents snowball abandonment. EVIDENCE: B

41. **Environmental Storytelling** — Narrative through the world itself. WHY: respects agency; rewards observation. EVIDENCE: B

42. **Audio-First Design** — Sound as first-class citizen. WHY: processed faster than visual; most cost-effective improvement. EVIDENCE: B

43. **Procgen Within Authored Constraints** — Procedural generation with frameworks. WHY: authorial vision + replayability. EVIDENCE: B

44. **Modular Design for Expansion** — Extensible systems. WHY: reduces DLC/update cost. EVIDENCE: B


## Common LLM Mistakes in This Domain

1. **"Always use mobile-first design"** — Mobile-first is a strategy, not a law. Analytics dashboards need desktop-first. The correct approach is context-first: analyze where users perform the task.

2. **"5 users is always enough"** — Only for qualitative formative studies. Quantitative needs ~40. Random sets of 5 find 55-99% of problems -- enormous variability.

3. **"A/B test everything"** — Most orgs lack traffic for statistical power. Many decisions are better made with qualitative research or heuristic evaluation.

4. **"More features = more value"** — Feature bloat increases cognitive load and maintenance cost. Best products do LESS, better. Hick's Law: more options = more abandonment.

5. **"More features = better game"** — Subtractive design produces elegance. Portal has one mechanic and is a masterpiece.

6. **"Realistic graphics are always better"** — Art style coherence matters more than polygon count. Stardew Valley, Undertale, Celeste prove stylized art is commercially superior.

7. **"Add AI/personalization to improve UX"** — Requires sufficient behavioral data; thin-data recommendations are worse than none. Biggest LLM startup bottleneck IS UX.

8. **"Accessibility can be solved with an overlay"** — Only 2.4% effective (WebAIM survey); FTC fined accessiBe $1M. Requires structural code changes.

9. **"Difficulty should be uniform"** — Flow theory requires dynamic matching of challenge to skill. Difficulty should ebb and flow with safe spaces between challenges.

10. **"Build a design system early"** — Premature for small teams or pre-PMF startups. Start with style guide; extract patterns after 3+ repeated use cases.

## Key Sources
- Nielsen Norman Group (NNG) research corpus
- Teresa Torres. *Continuous Discovery Habits*
- Marty Cagan, SVPG. *Inspired*, *Empowered*
- Melissa Perri. *Escaping the Build Trap*
- Jesse Schell. *The Art of Game Design* (3rd ed.)
- Celia Hodent. *The Gamer's Brain*
- Csikszentmihalyi. *Flow* (1990)
- WebAIM Million report; WCAG 2.1
- Game Accessibility Guidelines (gameaccessibilityguidelines.com)
- Zileas (Tom Cadwell). Game Design Anti-Patterns (Riot, 2010)
- FTC 2024 deceptive patterns report; EU DSA
- Wichita State University readability research
- Binet & Field. *The Long and the Short of It*
