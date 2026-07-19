# Writing Workflow - Purpose, Truth, And Four Semantic Modes

## Scope And Ownership

This is the single source of truth for user-facing writing: literary/narrative,
marketing/advertising, informational/explanatory, and communication text. It
also governs editing and client-facing work reports.

Claude uses `.claude/agents/writer.md` and `.claude/skills/writing-workflow/SKILL.md`.
Codex uses `$codex-writing-workflow`. Review skills remain independent critics;
they do not own the generation workflow.

Reference metadata, provenance, reusable properties, and editor role IDs live in
`.claude/library/technical/writing-reference-registry.json`. Project-owned
references live in `brain/03-knowledge/writing/reference-registry.json` and take
effect only through explicit `supersedes` entries, including project editor-role
replacements. Validate both registries before
using a profile; do not silently load an external example corpus.

Project-owned constitutions, terminology, brand voice, audience research,
approved claims, legal constraints, and style guides override template defaults.
If two plausible sources of truth conflict, use the SOT Conflict Protocol in
`docs/AGENT_CONTEXT_SOT.md` instead of choosing silently.

## Four Semantic Modes

| Mode | Production purpose | Typical outputs | Primary acceptance signal |
| --- | --- | --- | --- |
| Literary | Create a specific experience, dramatic change, image, or meaning | prose, scenes, dialogue, scripts, game lore | the intended reader experience and narrative function occur without breaking story truth |
| Marketing/advertising | Help a qualified audience make an informed commercial decision | landing copy, ads, product pages, campaigns, sales email | honest offer comprehension and a measurable next action tied to the application KPI |
| Informational | Help the reader understand, decide, or perform a task correctly | guides, articles, manuals, documentation, reports | the reader can find the answer or complete the job with supported claims |
| Communication | Change the state of an interaction between people or organizations | email, support reply, notification, announcement, PR response | the recipient knows what happened, what matters, what to do, who owns it, and when |

These are operating modes, not cosmetic tones. Voice, tone, formality, rhythm,
density, terminology, and channel are separate profile dimensions inside a mode.
When a text spans modes, choose one primary reader job and treat the others as
supporting sections. Do not average incompatible purposes into generic copy.

Technical writing is a specialization over informational or communication mode,
not a fifth reader job. When truth depends on code, schemas, interfaces, versions,
commands, operating systems, or runtime behavior, load
`technical-writing-profile.md` and require technical accuracy, procedure,
architecture, and language review lanes.

Target language and source function are independent decisions. The route must
return four authority groups separately: language/editorial, process, domain,
and technical profiles. A process source may shape planning and client service;
a domain or technical source may be authoritative about API semantics,
advertising claims, crisis response, or document architecture. None of them is
therefore qualified to shape the output language. Never flatten these groups
into an undifferentiated author list.

Each registry profile belongs to exactly one `authorityGroup`. Language profiles
may control syntax, lexical choice, line editing, and voice; process profiles
control planning and client communication; domain profiles control reader jobs,
claims, explanation, structure, and genre behavior; technical profiles control
product-specific accuracy and technical conventions. Split a mixed profile
instead of relying on first-match routing.

## Writing Contract Gate

Before drafting, determine:

1. Reader or recipient: situation, knowledge, objections, accessibility needs.
2. Production purpose: the one job the text must perform.
3. After-state: what the reader should understand, feel, decide, or do.
4. Product/business link: user value and the relevant KPI, trust, loyalty,
   conversion, retention, activation, support load, or operational outcome.
5. Mode, target language, channel, format, length, deadline, and publication constraints.
6. Sources of truth: approved facts, terminology, claims, evidence, links, lore,
   plan, previous messages, and superseded material.
7. Voice contract: desired qualities, concrete references, and forbidden traits.
8. Acceptance evidence: how the client can judge that the text did its job.

For a short low-risk request, infer obvious fields and write. For long-form,
commercial, public, sensitive, regulated, or ambiguous work, make the contract
explicit. If a missing fact would change the promise, advice, recipient action,
story logic, safety, or legal meaning, stop and ask with 2-3 concrete options.

## Language And Reference Boundary

The structured registry assigns every source a language, usage class, and allowed
effects. Every profile declares output languages and effects. The validator must
reject these cases:

- a domain or regulatory standard claims voice, syntax, line-editing, or
  example-method authority;
- a language-sensitive profile targets `all` languages;
- no source in the target language authorizes a language-sensitive effect;
- a project voice placeholder is treated as active before the project records
  real approved material.

For Russian output, load `russian-writing-profile.md`. It turns the Ilyakhov and
Sarycheva methods into operational review steps and original template examples.
For Russian communication, also load `russian-business-correspondence.md`. For
Russian informational, marketing, or communication text that explains or
persuades, load `russian-explanation-and-persuasion.md`.
For English or another language, do not apply that profile merely because the
request was written in Russian. Mixed-language artifacts require per-section
language resolution.

## External Tool Truth Gate

External services are capabilities, not authors or hidden review evidence. The
registry records their access state separately from source and profile records.
The template ships `glavred-api` as `not-configured`: it has no paid access,
credentials, HTTP adapter, or provider response. Information-style methods
selected by active profiles may still guide a manual edit, but the result must not be labeled a
Glavred check.

Configuration and execution are different states. A configured adapter still
routes as `execution:not-run` until an artifact-bound provider call succeeds.
Runtime routing validates the merged registry and fails closed before exposing a
project-configured tool or any profile.

An agent may claim an external check, score, warning, or provider result only
when all of these are true:

1. The project registry explicitly supersedes the template tool and marks it
   `project-configured` with configuration evidence.
2. The integration actually ran against the current artifact or its recorded
   content hash/version.
3. A successful provider response is available as evidence for this iteration.
4. The report distinguishes provider output from the agent's interpretation.

If any condition is missing, report `external check not run` and continue with
the applicable manual profiles. Never estimate, simulate, or backfill a vendor
score from model judgment or public methodology.

## LitAI-Derived Workflow

The workflow adapts LitAI's strongest mechanisms without importing its
book-specific entities or gates.

1. **Route** - select the semantic mode from the reader's job, not keywords alone.
2. **Research** - locate project SOTs, facts, prior text, audience evidence,
   references, constraints, and known gaps before drafting.
3. **Specify** - complete the writing contract and mark facts versus assumptions.
4. **Analyze references** - separate language/editorial authority from domain
   correctness, then extract usable properties such as structure, rhythm,
   density, evidence placement, vocabulary, and tone. Do not copy a source's
   structure blindly or mechanically imitate a named author.
   Select explicit registry profile IDs, record the resulting editor IDs, and
   reject stale, unverified-default, unlicensed, or hash-mismatched sources.
5. **Architect** - define thesis or dramatic function, reader path, section
   inventory, claims, evidence slots, transitions, and final action or resolution.
6. **Create the functional whole** - produce the smallest complete text that
   already performs the production purpose end to end.
7. **Sharpen** - deepen evidence, examples, scenes, objections, explanations,
   texture, and phrasing while preserving the accepted architecture.
8. **Review independently** - check purpose, source fidelity, logic, omissions,
   harm, manipulation, and mode-specific criteria before line polish.
9. **Edit** - run structural, line, terminology, grammar, and channel passes in
   that order. Preserve intentional voice and project terminology.
10. **Release and evolve** - verify final claims and links, publish in the right
    format, collect outcome evidence, update the active SOT, and replace
    superseded versions rather than leaving contradictory copies active.

## Progressive JPEG For Text

The progressive JPEG shape applies to writing. A low-detail text is not an
outline pretending to be a deliverable. At 1% it must already perform the text's
real production function honestly from beginning to end; later passes increase
resolution.

- Literary: a short complete scene has setup, pressure, turn, and resulting
  change, even if imagery and subtext are still rough.
- Marketing: the reader can identify the real offer, relevant outcome, honest
  proof boundary, and working next action. A placeholder testimonial is not proof.
- Informational: the core answer, required caveat, and executable next step are
  present, even if examples and edge cases are sparse.
- Communication: the recipient can act correctly from the message now; owner,
  timing, consequence, and reply path are not deferred.

Plans, headings, evidence slots, TODOs, and draft fragments are preparation, not
product evidence. Mark unresolved slots explicitly. Never invent a fact, quote,
testimonial, citation, statistic, customer detail, deadline, price, capability,
or narrative canon to make the text look complete.

Before each sharper pass:

1. Verify the accepted final section/function inventory still exists.
2. Check that the current whole still performs the production purpose.
3. Replace obsolete passages and stale placeholders; do not preserve them as
   disabled, commented, hidden, or release-excluded alternatives.
4. Report current depth honestly: contract, functional 1%, structured draft,
   reviewed draft, channel-ready, or outcome-validated.

## Mode-Specific Gates

### Literary

- Treat the author-approved story bible, plan, character/location facts, and
  chronology as active SOTs; do not fill canon gaps silently.
- Load relevant past state, current scene function, intended future consequence,
  character knowledge, location, and reference profile before drafting.
- Check causality, point of view, motivation, information access, continuity,
  pacing, sensory specificity, dialogue function, and tonal consistency.
- Use editorial lenses as diagnostic perspectives, then synthesize one coherent
  voice. Do not write one paragraph per lens.

### Marketing And Advertising

- Identify audience/ICP, category situation, journey stage, offer, objection,
  channel, distribution, proof, desired action, and measurement before copy.
- Connect claims to verifiable evidence. Distinguish features, outcomes,
  guarantees, estimates, and hypotheses.
- Optimize for the product's real KPI and long-term trust, not clicks, vanity
  metrics, fake urgency, pressure tactics, hidden conditions, or dark patterns.
- A CTA must lead to a real available path. If fulfillment is manual or limited,
  state that boundary instead of implying automation or availability.

### Informational

- Lead with the reader's answer or task, then provide context in dependency order.
- Separate facts, interpretations, assumptions, procedures, warnings, and examples.
- Cite or link sources when freshness, precision, disputed claims, or high stakes
  matter. Do not use citations as decoration.
- Test procedures against the stated environment. Include prerequisites, expected
  result, failure states, recovery, and version boundaries when relevant.

### Communication

- Identify sender, recipient, relationship, prior state, desired state change,
  action owner, timing, consequence, and reply/escalation path.
- Put urgent decisions and actions where the recipient will see them; provide
  context only to support correct interpretation.
- Match authority and emotional weight. Be direct without manufacturing certainty,
  blame, intimacy, apology, or urgency.
- For support and incident messages, separate observed facts, impact, current
  action, workaround, next update, and owner.

## Human Voice And Truth Boundary

Human voice comes from specific observation, meaningful choices, project language,
rhythm appropriate to the reader, and an accountable point of view. It does not
come from deliberately adding errors or random quirks.

Never require or claim:

- deliberate typos, grammar mistakes, incomplete thoughts, fake digressions, or
  arbitrary paragraph-length patterns;
- fake precise numbers, fabricated self-doubt, invented anecdotes, or unsupported
  personal experience;
- evasion of AI detectors or proof that a text is "undetectable";
- a universal ban list applied without regard to meaning, quotation, genre, or
  project voice;
- one stereotyped voice for every audience on a platform.

Use word and phrase lists as diagnostic signals. Remove filler, corporate
abstractions, cliches, and repeated structures when they reduce meaning; keep a
word when it is the clearest accurate choice.

## Review And Editing Order

1. Purpose: does the text perform the writing contract?
2. Truth: are facts, claims, canon, sources, and uncertainty represented honestly?
3. Structure: can the reader follow the intended path and find the next action?
4. Mode: does it pass the selected profile's gates?
5. Voice: is the language specific, coherent, natural, and project-appropriate?
6. Line edit: remove repetition, nominalizations, weak abstractions, accidental
   passive voice, buried actors, and inconsistent terminology.
7. Grammar and format: correct errors without flattening intentional voice.
8. Channel: verify length, hierarchy, links, accessibility, rendering, and CTA.

Generation and acceptance should be separate passes. A writer may self-check,
but must not present its own unverified judgment as independent review evidence.

## Client-Facing Work Reports

A closeout is a communication-mode text. It should let the client understand the
result and make the next decision without reading an internal build log.

Default order:

1. `Что было` - the user-visible problem, gap, or risk.
2. `Что стало` - the verified state now.
3. `Что это даёт` - the practical user or business effect.
4. `Чего ожидать дальше` - remaining dependency, next evidence, or operating note.

Lead with outcome, separate done from not verified, and mention files or commands
only when they change a decision or provide useful evidence. For M+, product,
template, release, or long work, include what is sharp now, what remains rough,
the next sharpened layer, and the replan trigger. Follow
`.claude/library/process/client-executor-contract.md`; never claim research,
tests, review, release, or completion without fresh evidence.

## Final Check

- The chosen mode matches the reader's real job.
- The text performs its production purpose at the declared depth.
- Facts, proof, citations, canon, and uncertainty are honest.
- The project voice and terminology are preserved.
- The next action or resolution is available and clear.
- Obsolete versions and placeholders are replaced or explicitly unresolved.
- UTF-8, no-BOM, line-ending, and mojibake policy passes for tracked text.
