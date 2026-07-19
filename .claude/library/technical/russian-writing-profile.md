# Russian Writing Profile

This profile controls Russian language, syntax, line editing, and the practical
use of examples. It is an overlay on the semantic modes in `writing.md`, not a
replacement for their product purpose or truth requirements.

The operational method is derived from:

- Maxim Ilyakhov and Lyudmila Sarycheva, `Пиши, сокращай - 2025`;
- Maxim Ilyakhov and Lyudmila Sarycheva, `Новые правила деловой переписки`;
- Maxim Ilyakhov, `Ясно, понятно` and `Текст по полочкам`;
- the public Bureau guides `Как написать` and `Информационный стиль`;
- Maxim Ilyakhov's official materials on information style, structure, author
  style, text in a working system, and work with a client;
- the user-provided Ilyakhov pages summarized in
  `brain/03-knowledge/communication/ilyakhov-planning-principles.md`;
- project-owned approved Russian texts when present.

The examples below are original template examples. They demonstrate principles;
they are not quotations, reconstructions, or a substitute for the source works.

## Source Boundary

Russian output follows this priority:

1. Project-owned approved Russian samples, terminology, and voice rules.
2. The accepted reader job, product context, and facts.
3. Russian author/editor methods selected in the reference registry.
4. Domain standards for correctness, claims, safety, and information
   architecture only.

An English domain standard may change facts, required sections, evidence,
terminology, or document architecture. It may not determine Russian voice,
syntax, idiom, rhythm, or line editing. Translate the requirement into natural
Russian through this profile; do not translate the source sentence pattern.

Load `russian-business-correspondence.md` for Russian communication whose job is
a request, task, status, meeting summary, feedback, refusal, claim, incident,
proposal, or mass message. Load `russian-explanation-and-persuasion.md` for
Russian informational, marketing, and communication text that must change the
reader's model or support a decision. These child profiles specialize this
language profile; they do not compete with the four semantic modes.

## Working Method

### 1. Start With The Reader's Change

Define what the reader should understand, decide, feel, or do after the text.
The text is part of a working product system: interface, offer, process, support,
book, game, or relationship. If changing the text cannot solve the problem,
change or question the system instead of decorating the symptom.

### 2. Gather Substance Before Polishing

Collect facts, actors, events, examples, objections, limits, and evidence before
line editing. Do not use clean prose to hide an empty source pack. Missing facts
remain explicit questions or product risks; they do not become confident copy.

### 3. Build The Meaning Skeleton

State the thesis or required change, then arrange context, argument, evidence,
counterargument, example, action, and conclusion in dependency order. The
skeleton is not a mandatory formula. It is a way to see missing links before
word-level editing.

### 4. Show Actors, Actions, And Consequences

Prefer observable people, objects, events, and actions over nominalized
processes and abstract phenomena. Name the actor when responsibility matters.
Use passive or impersonal syntax deliberately when the actor is unknown,
irrelevant, unsafe to disclose, or the object is the true focus.

### 5. Replace Evaluation With Evidence

Do not call a product convenient, fast, reliable, innovative, or unique without
showing what creates that outcome and for whom. A specific fact is useful only
when it helps the reader make the intended decision.

### 6. Explain Through Examples And Counterexamples

For a complex or debatable claim, show a concrete case, then explain what the
reader should notice. Use a counterexample when it reveals the boundary of the
rule. Examples support the argument; they do not replace proof or permission.

### 7. Edit From Large To Small

Use this order:

1. Product system and reader purpose.
2. Facts and source hierarchy.
3. Text function and complete section inventory.
4. Structure and argument.
5. Paragraph and sentence dependency.
6. Words, rhythm, grammar, typography, and channel formatting.

Do not spend time removing stop words from a passage that should be deleted,
moved, or replaced. Public information-style principles may be applied manually,
but this is not a Glavred API check. Claim `checked by Glavred`, report a Glavred
score, or reproduce provider warnings only when the registry marks
`glavred-api` as `project-configured` and a successful provider response is tied
to this exact artifact. Otherwise state that the external check was not run.

### 8. Preserve Meaning And Natural Russian

Shorter is not automatically better. Remove verbal waste, false precision,
boilerplate, and duplicated signals, but keep necessary context, tone, rhythm,
legal limits, and emotional weight. Prefer natural Russian word order over a
calque from an English source.

### 9. Treat Style As Decisions

Style is not a bag of decorative words. It appears in what the author notices,
omits, compares, proves, and puts first. Do not imitate a named author's voice.
Select observable properties that serve the current reader and product.

### 10. Work As An Accountable Service

The client brings a product problem, not merely an order for a string of words.
The executor formulates the task, asks for missing materials, challenges a text
that cannot solve the problem, and reports evidence honestly. A plan is a
forecast and coordination tool, not a fabricated promise.

## Derived Russian Examples

Each example uses `weak -> diagnosis -> stronger`. The stronger version is only
valid when its facts are true.

### Product And System

**Weak:** `Мы значительно улучшили процесс восстановления доступа.`

**Diagnosis:** no actor, changed behavior, time, or next action.

**Stronger:** `Если пользователь забыл пароль, он получает ссылку на почту и
задаёт новый пароль без обращения в поддержку.`

### Actor And Action

**Weak:** `После проведения проверки будет осуществлено уведомление клиента.`

**Diagnosis:** actions are hidden inside nouns and responsibility is unclear.

**Stronger:** `Служба безопасности проверит заявку и напишет клиенту.`

### Evidence Instead Of Praise

**Weak:** `Наш сервис работает невероятно быстро.`

**Diagnosis:** evaluation without a decision-relevant fact.

**Stronger:** `Отчёт по 100 тысячам строк формируется за 8 секунд на тарифе Pro.`

### Reader Purpose First

**Weak:** `Добро пожаловать в раздел управления подпиской.`

**Diagnosis:** interface narration delays the likely task.

**Stronger:** `Тариф Pro продлится 24 июля. Здесь можно сменить карту или отменить
продление.`

### Structure Before Detail

**Weak:** a guide starts with implementation history, then lists flags, and only
near the end says what command the reader needs.

**Diagnosis:** information follows the author's discovery order, not the reader's
dependency order.

**Stronger:** start with the working command and expected result; then explain
prerequisites, options, failure recovery, and implementation history if useful.

### Honest Marketing

**Weak:** `Лучший помощник для бизнеса. Увеличьте продажи уже сегодня.`

**Diagnosis:** unprovable superiority, vague audience, and guaranteed outcome.

**Stronger:** `Сервис собирает заявки из сайта и почты в одной очереди. Менеджер
видит ответственного и срок; 14 дней бесплатно.`

### Useful Communication

**Weak:** `Уважаемые пользователи! Доводим до вашего сведения, что в настоящий
момент наблюдаются временные технические неполадки.`

**Diagnosis:** ceremonial opening, no affected function, action, or next update.

**Stronger:** `Оплата картой не проходит с 14:20. Уже оплаченные заказы сохранены.
Следующее обновление дадим в 15:00.`

### Plan Without False Certainty

**Weak:** `Скоро всё будет готово.`

**Diagnosis:** no checkable result, dependency, or time.

**Stronger:** `К 16:00 покажу воспроизводимую причину сбоя и варианты исправления.
Срок релиза назову после этой проверки.`

### Replanning

**Weak:** `Работа продолжается, но потребуется немного больше времени.`

**Diagnosis:** hidden drift; the client cannot decide what to trade.

**Stronger:** `План предполагал готовый экспорт. В тестовой среде API возвращает
неполные данные. Можно сохранить пятничный срок без экспорта или перенести релиз
на вторник. Рекомендую перенос: иначе пользователь потеряет часть отчёта.`

### Technical Requirement In Natural Russian

**Weak:** `Вы должны имплементировать ретраи перед тем, как инициировать новый
реквест.`

**Diagnosis:** English syntax and unnecessary borrowings obscure the action.

**Stronger:** `Если запрос завершился ошибкой 429, повторите его с увеличивающейся
задержкой. Не отправляйте новый запрос раньше времени из `Retry-After`.`

### Stop-Word Theater

**Weak:** the editor removes every highlighted introductory word but leaves an
unsupported claim and a missing CTA.

**Diagnosis:** a word-level metric replaced product and argument review.

**Stronger:** first prove or remove the claim and add the real next action; then
edit the remaining sentences for clarity and rhythm.

### Text That Should Not Exist

**Weak:** a form says `Введите адрес электронной почты в поле электронной почты`.

**Diagnosis:** the interface already carries the meaning.

**Stronger:** label the field `Почта`; use helper text only for a real constraint,
for example `Пришлём сюда чек и ссылку для входа`.

## Review Gate

Before accepting Russian text, answer:

- What reader change does it produce in the real product system?
- Are facts, actors, actions, responsibility, and uncertainty visible?
- Does the structure follow the reader's dependencies?
- Does every evaluation have useful evidence or an honest boundary?
- Does a complex claim have an example or counterexample where needed?
- Does the Russian syntax sound written in Russian rather than translated?
- Did line editing preserve necessary meaning, tone, and consequence?
- Did any English domain source leak into voice, idiom, or sentence shape?
- Is the final action, resolution, or deliberate after-state clear?
