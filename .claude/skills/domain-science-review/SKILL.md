---
description: "Review research methodology and decision-making against evidence-based reasoning. Detects p-hacking, cognitive biases, logical fallacies."
---
# Domain Science Review

## How to Use
When reviewing research claims, methodology, decision-making processes, or reasoning quality,
check against these evidence-based practices. This is a META-DOMAIN -- critical thinking and
decision quality underpin all other domains.

**Evidence Scale:** A = replicated experimental evidence / meta-analyses. B = well-supported by multiple studies. C = theoretical/expert consensus with limited direct experimental tests.

## Anti-Patterns (NEVER recommend)

### Statistical & Methodological Sins

1. **P-Hacking** — Running multiple analyses, excluding outliers, or changing variables until p<0.05. WHY: inflates false positive rate far above 5%; testing 20 metrics at alpha=0.05 gives ~64% chance of at least one false positive. INSTEAD: pre-register analyses; use Bonferroni or BH corrections. EVIDENCE: A

2. **HARKing (Hypothesizing After Results Known)** — Generating hypotheses after seeing data, presenting as a priori predictions. WHY: converts exploratory findings into false "confirmatory" results; inflates apparent predictive power. INSTEAD: clearly separate exploratory vs confirmatory; pre-registration. EVIDENCE: A

3. **Publication Bias (File Drawer Problem)** — Journals preferentially publish positive/significant results. WHY: creates systematically distorted literature with inflated effect sizes and accumulated false positives. INSTEAD: registered reports; publish null results; pre-print servers. EVIDENCE: A

4. **Underpowered Studies** — Sample too small to detect real effects reliably. WHY: inflates false negative rate AND inflates observed effect sizes of "significant" results (winner's curse). INSTEAD: power analysis before data collection; adequate sample size. EVIDENCE: A

5. **Multiple Comparisons Without Correction** — Testing many hypotheses without adjusting alpha threshold. WHY: 5% error rate per test compounds across many tests; "significant" findings are noise. INSTEAD: Bonferroni, FDR control, or pre-registration of specific tests. EVIDENCE: A

6. **Confusing Correlation with Causation** — Inferring X causes Y from co-occurrence alone. WHY: confounders, reverse causation, and common causes all produce correlations without causal links. INSTEAD: randomized experiments, natural experiments, causal inference frameworks (DAGs, IV, DiD). EVIDENCE: A

7. **Cherry-Picking Data** — Selecting data subsets that support desired conclusions. WHY: any large dataset contains spurious patterns that disappear in the full data. INSTEAD: pre-specify analysis plan; report ALL results including unfavorable ones. EVIDENCE: A

8. **Survivorship Bias** — Studying only successes, ignoring failures. WHY: creates false models of success. Classic example: WWII bomber armor analysis -- reinforcing where planes returned hit, not where destroyed planes were hit. INSTEAD: actively include failures in analysis. EVIDENCE: A

9. **Texas Sharpshooter Fallacy** — Finding patterns in random data and claiming prediction. WHY: with enough data points, clusters appear purely by chance. INSTEAD: out-of-sample validation; hold-out test sets. EVIDENCE: B

10. **Ecological Fallacy** — Applying group-level findings to individuals. WHY: group averages hide enormous individual variation; what is true of a group may not be true of any member. INSTEAD: multi-level modeling; state level of analysis explicitly. EVIDENCE: B

11. **Base Rate Neglect** — Ignoring prior probability when evaluating evidence. WHY: even a 99% accurate test produces mostly false positives for a 1% prevalence condition (Bayes' theorem). INSTEAD: always incorporate base rates; use Bayesian reasoning. EXAMPLE: 99% accurate test with 1% prevalence yields only 50% PPV. EVIDENCE: A

12. **Regression to the Mean Ignored** — Attributing normal statistical regression to interventions. WHY: extreme values naturally move toward mean on retesting, regardless of treatment. INSTEAD: use control groups to distinguish treatment from regression. EVIDENCE: A

13. **Ignoring Effect Size** — Focusing only on p-values (statistical significance). WHY: with large N, trivially small effects become "statistically significant" while remaining practically meaningless. INSTEAD: always report and interpret effect sizes (Cohen's d, r, odds ratio). EVIDENCE: A

14. **Goodhart's Law Violations** — Optimizing for a metric until it stops measuring what it was meant to. WHY: "When a measure becomes a target, it ceases to be a good measure." INSTEAD: multiple orthogonal metrics; qualitative checks; rotate metrics. EVIDENCE: B

15. **Simpson's Paradox Blindness** — Aggregate trends that reverse within subgroups. WHY: confounding variables create misleading aggregate patterns that reverse when data is properly stratified. INSTEAD: always check subgroup analyses; investigate confounders. EVIDENCE: A

### Cognitive Biases in Decision-Making

16. **Confirmation Bias** — Seeking, interpreting, and remembering evidence that confirms existing beliefs. WHY: most pervasive cognitive bias; affects experts equally. DEBIASING: consider-the-opposite; assign devil advocate; decision journal. INSTEAD: actively seek disconfirming evidence; ask "what would change my mind?" EVIDENCE: A

17. **Anchoring Effect** — Over-relying on first piece of information received. WHY: affects numerical estimates by 30-50% even with arbitrary, random anchors. DEBIASING: generate own estimate BEFORE seeing anchor; use multiple independent anchors. INSTEAD: generate independent estimate before seeing any anchor value. EVIDENCE: A

18. **Availability Heuristic** — Judging probability by ease of recall rather than actual frequency. WHY: vivid, recent, or emotional events feel more likely than base rates suggest. INSTEAD: use actual statistics and base rates, not gut feeling. EVIDENCE: A

19. **Dunning-Kruger Effect** — Least competent systematically overestimate ability; experts tend to underestimate. WHY: the metacognitive skills needed to judge competence ARE the competence itself. INSTEAD: calibration training; seek external feedback; track prediction accuracy. EVIDENCE: A

20. **Sunk Cost Fallacy** — Continuing investment because of past spending, not future value. WHY: leads to escalation of commitment; throwing good money after bad. INSTEAD: "If starting fresh today, would I choose this?" If no, cut losses now. EVIDENCE: A

21. **Status Quo Bias** — Preferring current state regardless of evidence for change. WHY: loss aversion makes potential losses loom ~2x larger than equivalent gains. INSTEAD: evaluate options from neutral starting point; consider opportunity cost of inaction. EVIDENCE: A

22. **Framing Effects** — Different conclusions from identical data presented differently. WHY: "90% survival" and "10% mortality" produce measurably different decisions despite being identical. INSTEAD: deliberately reframe problems multiple ways before deciding. EVIDENCE: A

23. **Overconfidence Bias** — Systematically overestimating accuracy of own beliefs. WHY: calibration studies show people saying "95% confident" are right only ~75% of the time. INSTEAD: calibration training; reference class forecasting; track prediction accuracy. DEBIASING: track accuracy over time; widen intervals; reference class. EVIDENCE: A

24. **Planning Fallacy** — Underestimating time, cost, and risk for future tasks. WHY: people focus on unique aspects of current task while ignoring base rates from similar past tasks. INSTEAD: reference class forecasting -- how long did similar projects actually take? EVIDENCE: A

25. **Groupthink** — Conformity pressure suppressing dissent in cohesive groups. WHY: historically linked to catastrophic failures (Bay of Pigs, Challenger, Boeing 737 MAX). INSTEAD: assign devil's advocate; collect anonymous input; ensure diverse perspectives. EVIDENCE: A

### Logical Fallacies

26. **Appeal to Authority** — Accepting claim because of who said it, not evidence quality. WHY: experts are wrong in proportion to how far they stray from their specific expertise. INSTEAD: evaluate the evidence itself, not the source's credentials. EVIDENCE: B

27. **Appeal to Nature** — "Natural" = good, "artificial" = bad. WHY: arsenic is natural, insulin is artificial; naturalness has zero correlation with safety or efficacy. INSTEAD: evaluate by evidence. EVIDENCE: B

28. **Ad Hominem** — Attacking the person instead of the argument. WHY: source quality is relevant context but does not determine argument validity. INSTEAD: evaluate argument independently of source. EVIDENCE: C

29. **False Dichotomy** — Presenting only two options when more exist. WHY: oversimplifies complex decisions; excludes creative solutions and middle ground. INSTEAD: brainstorm at least 3 additional options before deciding. EVIDENCE: C

30. **Slippery Slope (Without Mechanism)** — Claiming A inevitably leads to Z without demonstrating causal chain. WHY: ignores intermediate decision points and feedback mechanisms. INSTEAD: specify the concrete causal mechanism at each step. EVIDENCE: C

31. **Straw Man** — Misrepresenting opponent's position to make it easier to attack. WHY: creates false victory while leaving the real argument unaddressed. INSTEAD: steelman -- construct the strongest version of the opposing argument. EVIDENCE: C

32. **Appeal to Tradition** — "We have always done it this way." WHY: past practice may have been wrong, or context may have fundamentally changed. INSTEAD: evaluate current evidence for current conditions. EVIDENCE: C

33. **Burden of Proof Shifting** — Asking others to disprove unfalsifiable claims. WHY: impossible to prove a universal negative. INSTEAD: extraordinary claims require extraordinary evidence; burden is on the claimant. EVIDENCE: B

34. **Genetic Fallacy** — Judging a claim solely by its origin rather than its merit. WHY: even flawed or disreputable sources occasionally produce valid insights. INSTEAD: evaluate the argument itself, noting source as context. EVIDENCE: C

35. **Composition/Division Fallacy** — What is true of parts must be true of whole (or vice versa). WHY: emergent properties mean wholes behave differently from parts; ecological fallacy applies. INSTEAD: test claims at the appropriate level of analysis. EVIDENCE: C

### Research Red Flags

36. **No Pre-Registration** — Study not registered before data collection. WHY: enables post-hoc hypothesis generation, analysis flexibility, and outcome switching. INSTEAD: check OSF, ClinicalTrials.gov for registration. EVIDENCE: A

37. **No Effect Size Reported** — Only p-values without practical significance measures. WHY: p-values alone say nothing about magnitude or practical importance. INSTEAD: require Cohen's d, odds ratio, or other effect size measures. EVIDENCE: A

38. **No Replication** — Single study presented as definitive truth. WHY: replication crisis showed ~50% of psychology findings failed independent replication (OSC 2015). INSTEAD: weight replicated findings 3-5x higher than single studies. EVIDENCE: A

39. **Predatory Journal Publication** — Published in journal with no real peer review standards. WHY: pay-to-publish model incentivizes accepting everything; no quality filter. INSTEAD: check journal indexing, impact factor, editorial board. EVIDENCE: B

40. **No Control Group** — Before/after comparison without comparison group. WHY: regression to mean, placebo, Hawthorne effect, and natural course all confound results. INSTEAD: randomized controlled design with proper control. EVIDENCE: A

41. **Self-Report Only Data** — Relying exclusively on what participants say they do/feel. WHY: social desirability bias, memory errors, massive say-do gap in behavior. INSTEAD: combine with behavioral measures, observational data. EVIDENCE: A

42. **Ignoring Replication Crisis** — Treating all published findings as equally reliable. WHY: ~50% of psychology, ~70% of preclinical findings fail replication. INSTEAD: weight evidence by replication status and sample size. EVIDENCE: A

43. **Narrative Reviews Over Systematic** — Citing narrative reviews that can cherry-pick supporting studies. WHY: narrative reviews lack systematic search criteria and reproducibility. INSTEAD: prioritize Cochrane/systematic reviews with transparent methodology. EVIDENCE: A

44. **Ignoring Confidence Intervals** — Reporting point estimates without uncertainty ranges. WHY: creates false precision; a "23% improvement" means nothing without CI. INSTEAD: always report and interpret confidence intervals alongside point estimates. EVIDENCE: A

45. **Observer Bias Without Blinding** — Researchers knowing treatment allocation. WHY: introduces systematic measurement and interpretation bias. INSTEAD: double-blinding where possible; blinded outcome assessment. EVIDENCE: A

### Decision-Making Traps

46. **Analysis Paralysis** — Gathering more data indefinitely instead of deciding. WHY: diminishing returns on information; 70% information is often sufficient for reversible decisions (Bezos). INSTEAD: set decision deadlines; classify reversible (decide fast) vs irreversible (decide carefully). EVIDENCE: C

47. **Narrative Fallacy** — Constructing coherent stories from random events. WHY: humans need narrative; we see patterns and causes in noise. INSTEAD: require statistical evidence; a good story is not evidence. EVIDENCE: B

48. **Recency Bias** — Overweighting recent events in forecasts and judgments. WHY: memory accessibility distorts probability estimation. INSTEAD: use historical base rates and complete datasets, not just recent observations. EVIDENCE: A

49. **Zero-Risk Bias** — Preferring complete elimination of one small risk over larger overall risk reduction. WHY: certainty of elimination feels disproportionately better than partial reduction of bigger risk. INSTEAD: maximize expected value of risk reduction across all threats. EVIDENCE: B

50. **Neglect of Probability** — Focusing on outcome magnitude while ignoring likelihood. WHY: emotional reactions to dramatic outcomes override probabilistic thinking. INSTEAD: explicit probability estimation; expected value calculation; decision matrices. EVIDENCE: A

## Best Practices (ALWAYS consider)

### Evidence Evaluation

1. **Evidence Hierarchy** — Systematic review > RCT > cohort > case-control > case series > expert opinion > anecdote. WHY: higher levels systematically reduce bias. EVIDENCE: A

2. **Pre-Registration** — Register hypotheses and analysis plans before data collection. WHY: prevents p-hacking and HARKing; makes methods transparent. EVIDENCE: A

3. **Effect Size + Confidence Intervals** — Report magnitude and uncertainty, not just significance. WHY: p-values answer "is there an effect?" but not "how big?" or "how certain?" EVIDENCE: A

4. **Replication Weighting** — Weight replicated findings 3-5x over single studies. WHY: replication crisis demonstrates unreliability of any single study. EVIDENCE: A

5. **Bayesian Reasoning** — Update beliefs proportionally to evidence strength, incorporating priors. WHY: prevents base rate neglect; more intuitive than frequentist for practical decisions. EVIDENCE: A

6. **Power Analysis Before Study** — Calculate required sample size for target effect and error rates. WHY: prevents wasting resources on underpowered studies that cannot detect real effects. EVIDENCE: A

7. **Multiple Comparison Correction** — Adjust for family-wise error rate when testing many hypotheses. WHY: prevents false discovery inflation from multiple testing. EVIDENCE: A

8. **Systematic Over Narrative Reviews** — Prioritize reviews with transparent, reproducible methodology. WHY: eliminates cherry-picking; search strategy and inclusion criteria are explicit. EVIDENCE: A

9. **Blinding** — Double-blind designs where possible. WHY: eliminates observer bias and expectation effects simultaneously. EVIDENCE: A

10. **Premortem Analysis** — Before committing to plan, imagine it failed; work backward to identify causes. WHY: overcomes overconfidence and planning fallacy; surfaces hidden risks (Kahneman). EVIDENCE: B

### Critical Thinking Practices

11. **Red Team / Devil's Advocate** — Assign someone to argue against consensus. WHY: breaks groupthink, surfaces hidden assumptions, stress-tests plans. EVIDENCE: B

12. **Steelmanning** — Construct the strongest version of opposing arguments before rebutting. WHY: prevents straw man; leads to better decisions by engaging real objections at full strength. EVIDENCE: C

13. **Reference Class Forecasting** — Use base rates from similar past cases instead of inside view estimates. WHY: reduces planning fallacy by 30-50%; superforecasters use this consistently (Tetlock). EVIDENCE: A

14. **Falsification Thinking** — Ask "what evidence would disprove this?" WHY: Popper's demarcation criterion; non-falsifiable claims are outside science. EVIDENCE: B

15. **Seek Disconfirming Evidence** — Actively look for data against your hypothesis. WHY: directly counters confirmation bias, the most pervasive cognitive distortion. EVIDENCE: A

16. **Calibration Training** — Practice probabilistic predictions and track accuracy over time. WHY: superforecasters outperform domain experts through calibration (Tetlock). EVIDENCE: A

17. **Decision Journal** — Record decisions, reasoning, and expected outcomes; review periodically. WHY: enables learning from past decisions; reveals systematic errors invisible in real-time. EVIDENCE: C

18. **Independent Estimates Before Discussion** — Each person writes estimates before group sharing. WHY: prevents anchoring and conformity effects in group decisions. EVIDENCE: A

19. **Reversibility Check** — Classify decisions as Type 1 (irreversible, need care) or Type 2 (reversible, need speed). WHY: prevents analysis paralysis on reversible decisions and recklessness on irreversible ones. EVIDENCE: C

20. **Distinguish Noise from Signal** — Ask whether observed variation exceeds expected random variation. WHY: humans systematically over-interpret noise as meaningful signal; see patterns in randomness. EVIDENCE: A

### Research Best Practices

21. **Open Data and Methods** — Share datasets and analysis code publicly. WHY: enables independent replication and error detection by the community. EVIDENCE: A

22. **Registered Reports** — Journals accept papers based on methodology before results are known. WHY: eliminates publication bias at the source; methodology quality drives publication. EVIDENCE: A

23. **Multi-Site Replication** — Replicate across multiple labs/contexts. WHY: single-site effects may be artifacts of local conditions, equipment, or populations. EVIDENCE: A

24. **Mixed Methods Research** — Combine quantitative (what/how much) with qualitative (why/how). WHY: neither alone gives complete picture; quantitative identifies patterns, qualitative explains them. EVIDENCE: B

25. **Sensitivity Analysis** — Test whether conclusions change under different reasonable assumptions. WHY: reveals robustness -- or fragility -- of findings. EVIDENCE: A

26. **Specify Boundary Conditions** — State when a finding does and does NOT apply. WHY: prevents overgeneralization; all findings have contexts where they hold and contexts where they fail. EVIDENCE: B

27. **Report Null Results** — Publish findings regardless of significance direction. WHY: reduces publication bias; helps future researchers avoid dead ends. EVIDENCE: A

28. **Appropriate Statistical Tests** — Match test to data type, distribution, independence, and research question. WHY: wrong test = wrong conclusion; parametric tests on non-normal data mislead. EVIDENCE: A

29. **Exploratory vs Confirmatory Labeling** — Explicitly label post-hoc analyses as exploratory. WHY: prevents false confidence in discoveries that were not pre-planned. EVIDENCE: A

30. **Cross-Validation** — Test predictive models on held-out data not used in model fitting. WHY: training accuracy systematically overstates true predictive performance. EVIDENCE: A

### Decision Framework Best Practices

31. **Expected Value Calculation** — Probability x magnitude for each decision option. WHY: systematic approach to decisions under uncertainty; avoids neglect of probability. EVIDENCE: B

32. **Inversion Thinking** — Instead of "how to succeed," ask "how would I guarantee failure?" and avoid those paths. WHY: failure modes are often more identifiable and actionable than success paths. EVIDENCE: C

33. **Second-Order Thinking** — For every consequence, ask "and then what?" WHY: first-order thinking misses cascading downstream effects that often dominate outcomes. EVIDENCE: C

34. **Probabilistic Thinking** — Express beliefs as probabilities, not binary certainties. WHY: acknowledges uncertainty; enables calibration, updating, and rational betting. EVIDENCE: B

35. **Margin of Safety** — Build buffers into all plans for unexpected events. WHY: plans always encounter reality; buffers prevent catastrophic failure from inevitable surprises. EVIDENCE: C

36. **Fermi Estimation** — Break unknowns into estimable components. WHY: rough decomposition beats gut feeling. EVIDENCE: C

37. **Dialectical Thinking** — Hold thesis and antithesis; seek synthesis. WHY: prevents premature closure; surfaces hidden assumptions. EVIDENCE: C

38. **Distinguish Risk from Uncertainty** — Risk has known probabilities; uncertainty unknown. WHY: different tools needed -- insurance for risk, optionality for uncertainty. EVIDENCE: B

39. **Occam's Razor (With Nuance)** — Prefer simpler explanations that fit data. WHY: simpler models generalize better; but oversimplification misses real complexity. EVIDENCE: B

40. **Adversarial Collaboration** — Disagreeing researchers design studies together. WHY: prevents each side from biasing methodology. EVIDENCE: B

41. **Track Base Rates** — Maintain personal database of base rates for common claims. WHY: base rate neglect is most common reasoning error. EVIDENCE: B

42. **Absence of Evidence vs Evidence of Absence** — No studies found X differs from studies found no X. WHY: critical distinction for medical and policy decisions. EVIDENCE: B

43. **Multiple Competing Hypotheses** — Generate 3+ explanations before testing. WHY: prevents anchoring on first theory. EVIDENCE: B

44. **Specify Effect Size Benchmarks** — Small d=0.2, medium d=0.5, large d=0.8. WHY: without benchmarks, effect sizes are uninterpretable. EVIDENCE: A

45. **Demand Mechanism Not Just Correlation** — Ask HOW X causes Y, not just IF they co-occur. WHY: mechanism strengthens causal inference; implausible mechanism suggests confounding. EVIDENCE: B

46. **Weighted Evidence Aggregation** — Weight findings by sample size, replication status, and study quality. WHY: all findings are not created equal; a meta-analysis of 50 studies trumps one n=30. EVIDENCE: A

47. **Scope Conditions for Every Finding** — State population, context, and time when finding applies. WHY: all findings have boundaries; overgeneralization is default failure mode. EVIDENCE: B

48. **Quantify Uncertainty Explicitly** — State confidence level, interval width, or probability range. WHY: point estimates without uncertainty create false precision. EVIDENCE: A

49. **Update Incrementally** — Adjust beliefs in proportion to evidence strength. WHY: all-or-nothing updating ignores evidence quality; Bayesian updating is optimal. EVIDENCE: A

50. **Seek Natural Experiments** — When RCTs are impossible, find policy changes, geographic boundaries, or discontinuities. WHY: quasi-experimental designs provide causal evidence without randomization. EVIDENCE: B


## Cross-Domain Red Flags
Universal patterns appearing in 3+ domains with independent evidence:
- **Premature optimization** — Building complex systems before understanding actual needs
- **Cargo cult practices** — Copying without understanding WHY it works; do those conditions exist here?
- **Survivorship bias** — Learning only from successes while ignoring identical approaches that failed
- **One-size-fits-all** — Every practice has boundary conditions; state when it does NOT apply
- **Complexity bias** — Start simple; add complexity only when simplicity demonstrably fails
- **Sunk cost escalation** — "If starting fresh today, would I choose this?" If no, cut losses now
- **Authority over evidence** — Evaluate claims by data quality, not source prestige
- **Narrative fallacy** — Coherent stories from random events; stories are not evidence
- **Base rate neglect** — Always apply Bayes before interpreting evidence
- **Planning fallacy** — Use reference class forecasting; buffer 50-100%
- **Regression to mean confusion** — Do not attribute normal regression to interventions
- **False dichotomy default** — Brainstorm 3+ options before deciding
- **Anchoring on first data** — First information shapes all subsequent analysis
- **Composition fallacy** — What is true of parts may not be true of the whole
- **Feature bloat / scope creep** — For every addition, ask "what can I remove?"

## Common LLM Mistakes in This Domain

1. **Presenting single studies as definitive** — Citing one paper without meta-analytic context or replication status. Always check and report replication status.

2. **Fabricating citations** — Generating plausible but non-existent references with confident formatting. Never cite a source you cannot verify exists.

3. **False precision** — Stating "23.7% improvement" when evidence supports only "significant improvement." Prefer directional claims over fabricated precision.

4. **Conflating correlation and causation** — Using causal language ("X causes Y") for purely observational findings. State study design explicitly.

5. **Ignoring base rates** — Discussing disease risk or test accuracy without prevalence context. Always apply Bayes' theorem.

6. **Overconfidence in reasoning** — Presenting uncertain conclusions with high confidence tone. State uncertainty explicitly with calibrated confidence levels.

7. **Authority-based reasoning** — "Einstein said..." as evidence. Evaluate the argument on its merits, not the source's fame.

8. **Narrative over statistics** — Compelling anecdotes presented alongside (or instead of) weak quantitative data. Anecdotes illustrate; they do not prove.

9. **Missing boundary conditions** — Presenting findings as universal without stating when they do NOT apply. All findings have limits.

10. **Symmetry bias** — Treating both sides as equally valid ("some say X, others say Y") when evidence overwhelmingly favors one side. Report the weight of evidence.

## Key Sources
- Simmons, Nelson & Simonsohn. "False-Positive Psychology" (2011)
- Kahneman. *Thinking, Fast and Slow* (2011)
- Ioannidis. "Why Most Published Research Findings Are False" (2005)
- Open Science Collaboration. "Reproducibility of Psychological Science" (2015)
- Cochrane Collaboration methodology handbooks
- Tetlock. *Superforecasting* (2015)
- Gigerenzer. *Risk Savvy* (2014)
- Taleb. *The Black Swan* (2007); *Antifragile* (2012)
- Popper. *The Logic of Scientific Discovery* (1959)
- Tversky & Kahneman. Prospect Theory (1979)
- Kerr. "HARKing" (1998)
- Head et al. "Extent and Consequences of P-Hacking" (2015)
