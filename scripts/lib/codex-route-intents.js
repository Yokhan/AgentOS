const INTENT_THRESHOLD = 2;
const CHANGE_STRATEGY_EXACT_PATTERN =
  /compare\s+(?:repair|rewrite).*?(?:replacement|migration)|repair\s*(?:vs|or|and|\/|,|->)\s*(?:replace|replacement|rewrite|migrate|migration)|replace\s*(?:vs|or|and|\/|,|->)\s*(?:repair|migrate)|fix\s+or\s+rewrite|patch\s+loop|architecture\s+mismatch.*(?:reading|first\s+patch)|second\s+failed\s+repair.*(?:same\s+acceptance|criterion)|compatibility\s+shim.*(?:old|legacy)|(?:dead|legacy|obsolete)\s+(?:code|path).*(?:rewrite|replace|migrate)|чинить[\p{L}\p{N}_-]*\s+или\s+(?:замен|перепис|мигрир)|(?:сравни|сопостав)[\p{L}\p{N}_-]*.*(?:ремонт|почин|замен|перепис|миграц)|(?:ремонт|почин|латан)[\p{L}\p{N}_-]*\s+(?:против|или|vs)\s+(?:замен|перепис|миграц)|(?:несоответств|конфликт)[\p{L}\p{N}_-]*\s+архитектур[\p{L}\p{N}_-]*.*(?:чтени|перв)[\p{L}\p{N}_-]*\s+(?:прав|патч)|втор[\p{L}\p{N}_-]*\s+неудачн[\p{L}\p{N}_-]*\s+(?:ремонт|почин|исправ)[\p{L}\p{N}_-]*.*(?:критери|приемк)|(?:снова|опять|трет|втор)[\p{L}\p{N}_-]*.*(?:лат|чин|патч)|совместимост[\p{L}\p{N}_-]*.*(?:стар|устар|legacy)/iu;
const READ_ONLY_PATTERN =
  /read[- ]only|do not (?:edit|modify|change)|analysis only|только\s+(?:анализ|провер|ревью)|не\s+(?:редактир|изменя|трогай)/i;

const INTENT_GROUPS = {
  api: [
    [/contract|schema|protocol|payload|request|response|integration/i, /контракт|схем|протокол|запрос|ответ|интеграц/i],
    [/consumer|client|backward compat|compatibility|versioned/i, /клиент|потребител|совместим|версир/i],
    [/status code|error shape|pagination|rate limit|field/i, /код.*статус|ошибк.*форм|пагинац|лимит|пол/i],
  ],
  bugfix: [
    [/stuck|hang|freeze|loop|blank|wrong|unexpected|does not/i, /завис|зацикл|бел\w*\s+экран|неверн|неожидан|не\s+может/i],
    [/after update|after deploy|after change|repro|steps|symptom/i, /после\s+обнов|после\s+релиз|после\s+депло|воспроиз|симптом/i],
    [/restore|recover|rollback|regress|worked before/i, /вернуть|откат|раньше\s+работ|регресс/i],
  ],
  "change-strategy": [
    [/repeated repair|patch loop|workaround|compatibility shim|compatibility-only|dead code|architecture drift|architecture mismatch|duplicate state|duplicate implementation|wrong ownership/i, /повторн\w*\s+ремонт|цикл\w*\s+патч|обход|костыл|мёртв\w*\s+код|мертв\w*\s+код|дрейф\w*\s+архитектур|несоответств\w*\s+архитектур|дублир\w*\s+(?:состояни|реализац)|неверн\w*\s+владен/i],
    [/repair|replace|rewrite|migrate|new architecture|start fresh/i, /чин|ремонт|замен|перепис|мигрир|нов\w*\s+архитектур|с\s+нуля/i],
    [/protected contract|consumer|live data|public api|compatibility|project posture|source of truth|ownership|accepted final path/i, /защищенн\w*\s+контракт|потребител|жив\w*\s+данн|публичн\w*\s+апи|совместимост|состояни\w*\s+проект|источник\w*\s+истин|владен|финальн\w*\s+пут/i],
    [/baseline|evidence|maintainability|performance|total cost|rollback/i, /базов\w*\s+уров|доказател|поддерживаем|производительност|полн\w*\s+стоимост|откат/i],
  ],
  design: [
    [/looks|visual|trust|polish|premium|cheap|clutter|hierarchy/i, /выгляд|довер|полиров|дешев|кустар|перегруж|иерарх/i],
    [/screen|page|surface|layout|state|empty|loading|error/i, /страниц|поверхност|состояни|пуст|загруз|ошибк/i],
    [/mobile|desktop|responsive|touch|viewport|overflow/i, /мобил|десктоп|адаптив|тап|вьюпорт|переполн/i],
  ],
  "design-system": [
    [/system|tokens|component|primitive|variant|storybook/i, /систем|токен|компонент|примитив|вариант|сторибук/i],
    [/spacing|typography|radius|motion|color|state/i, /отступ|типограф|радиус|скругл|движен|цвет|состояни/i],
    [/reuse|composition|contract|library|foundation/i, /переиспольз|композиц|контракт|библиотек|фундамент/i],
  ],
  "writing-literary": [
    [/novel|fiction|story|chapter|scene|dialogue|screenplay|narrative|poem/i, /роман|рассказ|повест|глав|сцен|диалог|сценари|стих/i],
    [/character arc|dramatic turn|point of view|worldbuilding|lore|reader experience/i, /арк\w*\s+персонаж|драматическ\w*\s+поворот|точк\w*\s+зрен|миростро|лор|читательск\w*\s+пережив/i],
    [/write|draft|rewrite|edit|continue/i, /напиш|черновик|перепиш|редакт|продолж/i],
  ],
  "writing-informational": [
    [/explain|guide|manual|article|tutorial|documentation|knowledge base/i, /объясн|инструкц|гайд|стат|руководств|документац|справк|баз\w*\s+знан/i],
    [/reader task|understand|learn|complete|procedure|prerequisite/i, /задач\w*\s+читател|поня|науч|выполн|процедур|предуслов|требован/i],
    [/write|draft|rewrite|publish|clarity|wording/i, /напиш|черновик|перепиш|опублик|ясност|формулиров/i],
  ],
  "writing-communication": [
    [/email|letter|message|notification|announcement|support reply|incident update/i, /письм|сообщен|уведомлен|объявлен|ответ\w*\s+поддерж|инцидент/i],
    [/recipient|sender|reply|action owner|deadline|next update/i, /получател|отправител|ответ|исполнител|срок|следующ\w*\s+обновлен/i],
    [/write|draft|rewrite|send|clarify|tone/i, /напиш|черновик|перепиш|отправ|уточн|тон/i],
  ],
  feature: [
    [/enable|allow|support|add ability|new flow|capability/i, /возможност|разреш|поддерж|нов\w*\s+флоу|функц/i],
    [/user can|should be able|workflow|use case/i, /пользователь\s+может|должен\s+уметь|сценар|кейс/i],
    [/module|service|handler|screen|component/i, /модул|сервис|обработчик|экран|компонент/i],
  ],
  figma: [
    [/figma|design file|frame|prototype|mock/i, /фигм|макет|прототип|фрейм/i],
    [/sync|capture|code connect|inspect/i, /синхр|захват|инспект/i],
  ],
  lessons: [
    [/repeat|again|same mistake|postmortem|retro|lesson|promote/i, /повтор|снова|та\s+же\s+ошибк|постмортем|ретро|урок|вывод/i],
    [/rule|validator|template behavior|prevent next time|guard/i, /правил|валидатор|поведен|предотврат|защит/i],
    [/downstream|project experience|recent work/i, /даунстрим|проектн\w*\s+опыт|последн\w*\s+работ/i],
  ],
  marketing: [
    [/positioning|offer|value prop|why buy|message|claim/i, /позиционир|оффер|ценностн|почему\s+покуп|сообщен|месседж|обещан/i],
    [/funnel|journey|lead|campaign|channel|distribution|icp|segment|customer|buyer|user/i, /воронк|путь|лид|кампан|канал|дистрибуц|сегмент|аудитор|пользовател|клиент|покупател/i],
    [/cac|ltv|roas|conversion|revenue|retention|activation|purchase/i, /cac|ltv|roas|конверс|выруч|удержан|активац|покуп|деньг/i],
    [/proof|trust|brand|awareness|demand|sales/i, /доказател|довер|бренд|узнаваем|спрос|продаж/i],
  ],
  mermaid: [
    [/diagram|map|flow|board|architecture picture|sequence/i, /диаграм|карта|схем|поток|борд|последовательн/i],
    [/mermaid|mmd|graph|control board/i, /мермейд|граф|контрольн\w*\s+борд/i],
  ],
  migration: [
    [/move data|schema change|backfill|rollback|dry run|cutover/i, /перенос\w*\s+данн|измен\w*\s+схем|бэкфилл|откат|dry.?run|переключ/i],
    [/database|table|column|storage|migration/i, /баз\w*\s+данн|таблиц|колонк|хранилищ|миграц/i],
    [/compatibility|downtime|backup|restore/i, /совместим|простой|бэкап|восстанов/i],
  ],
  openai: [
    [/model|responses api|reasoning effort|structured outputs|tool calling/i, /модел|responses api|reasoning|структурн\w*\s+вывод|tool/i],
    [/openai|codex|gpt|responses api/i, /openai|codex|gpt|опенаи|responses api/i],
    [/current|latest|docs|migration/i, /актуальн|последн|док|миграц/i],
  ],
  "product-goal": [
    [/outcome|goal|quality bar|done right|production ready|final/i, /исход|цель|планк\w*\s+кач|нормальн|продакшн|финал/i],
    [/revenue|retention|loyalty|activation|conversion|support load|kpi/i, /выруч|удержан|лояльн|активац|конверс|нагрузк\w*\s+поддерж|kpi|деньг/i],
    [/user value|customer value|business result|product owner|customer|user/i, /ценност\w*\s+польз|ценност\w*\s+клиент|бизнес.*результ|владелец\s+продукт|пользовател|клиент/i],
    [/end-state|future shape|future capability|accepted future|product model|final shape/i, /будущ\w*\s+(форм|функц|способност)|сразу\s+содерж|продуктов\w*\s+(модел|сущност|смысл)|финальн\w*\s+форм/i],
    [/callable|hook|slot|handler|stub|no-op|debug|harness|proof/i, /вызыва|хук|слот|обработчик|стаб|заглуш|debug|дебаг|харнесс|harness|доказ/i],
  ],
  "product-ux": [
    [/start|return|complete|continue|abandon|drop|stuck/i, /начина|возвращ|заверш|продолж|броса|отвалива|застрева/i],
    [/user path|journey|flow|entry|exit|dead end|next action/i, /путь\s+польз|сценар|флоу|вход|выход|тупик|следующ\w*\s+действ/i],
    [/session|account|login|dashboard|service|value/i, /сесси|аккаунт|логин|кабинет|сервис|ценност/i],
  ],
  release: [
    [/release|publish|tag|changelog|version bump|github release/i, /релиз|опубликов|выпуст|выкат|тег\b|подн\w*\s+верс|ченджлог/i],
    [/commit|push|remote|artifact|archive|asset/i, /коммит|пуш|remote|артефакт|архив|asset/i],
    [/verify release|workflow|tag points|latest release/i, /провер\w*\s+релиз|воркфлоу|tag.*указывает|latest\s+release/i],
  ],
  review: [
    [/evaluate|inspect|audit|critique|check|verify|assess/i, /провер|аудит|оцени|разбер|посмотр|инспект|верифиц/i],
    [/finding|risk|issue|gap|regression|evidence/i, /находк|риск|проблем|дыра|регресс|доказател/i],
  ],
  security: [
    [/unauthorized|credential|token|secret|private data|leak|exposure/i, /чуж\w*\s+данн|токен|секрет|приватн|утеч|экспоз/i],
    [/access|permission|impersonate|steal|bypass|inject/i, /доступ|прав|выдат|укра|обход|инъекц|получ/i],
    [/user data|account|session|auth|trust boundary/i, /данн\w*\s+польз|аккаунт|сесси|аутентиф|границ\w*\s+довер/i],
  ],
  strategy: [
    [/strategy|terrain|center of gravity|asymmetry|timing|constraint/i, /стратег|ландшафт|центр\s+тяжест|асимметр|тайминг|огранич/i],
    [/sun tzu|stratagem|art of war|competitive|alternative/i, /сунь|цзы|стратагем|конкурент|альтернатив/i],
    [/tradeoff|contradiction|triz|plan|roadmap|sequence/i, /компромисс|противореч|триз|план|роадмап|последовательн/i],
  ],
  template: [
    [/agent behavior|instruction|routing|context|source of truth|template/i, /поведен\w*\s+агент|инструкц|роутинг|контекст|источник\s+правд|шаблон/i],
    [/agents file|claude file|skill|subagent|hook|sync/i, /агентск\w*\s+файл|claude|скилл|сабагент|хук|синхрон/i],
    [/downstream|starter|release infrastructure|validator/i, /даунстрим|стартер|релизн\w*\s+инфраструктур|валидатор/i],
    [/write this into yourself|remember this rule|agent must|agent should/i, /пропиши\s+себе|запиши\s+себе|агент\s+долж|нейросет\w*\s+долж/i],
  ],
  testing: [
    [/coverage|assert|fixture|regression guard|test gap/i, /покрыт|ассерт|фикстур|регрессионн\w*\s+защит|тестов\w*\s+дыр/i],
    [/unit|integration|e2e|smoke|snapshot/i, /юнит|интеграц|e2e|смоук|снапшот/i],
    [/prove|verify behavior|quality gate/i, /доказ|провер\w*\s+поведен|гейт\s+кач/i],
  ],
};

function normalizeTask(task) {
  return task
    .toLowerCase()
    .replace(/ё/g, "е")
    .replace(/[^\p{L}\p{N}#+./-]+/gu, " ")
    .trim();
}

function doesGroupMatch(group, normalizedTask) {
  return group.some((pattern) => pattern.test(normalizedTask));
}

function getIntentMatch(mode, task) {
  const groups = INTENT_GROUPS[mode] || [];
  const normalizedTask = normalizeTask(task);
  const matchedGroups = groups.filter((group) => doesGroupMatch(group, normalizedTask));
  const threshold = Math.min(INTENT_THRESHOLD, groups.length || INTENT_THRESHOLD);
  const hasVendorAnchor = mode !== "openai" || /\b(?:openai|codex|gpt(?:-?\d(?:\.\d)?)?|responses api)\b|опенаи/i.test(normalizedTask);
  return {
    isMatch: hasVendorAnchor && matchedGroups.length >= threshold,
    score: matchedGroups.length,
    threshold,
  };
}

function getChangeStrategyActivation(task) {
  const exact = CHANGE_STRATEGY_EXACT_PATTERN.test(task);
  const intent = getIntentMatch("change-strategy", task);
  return {
    required: exact || intent.isMatch,
    exact,
    semantic: !exact && intent.isMatch,
    recordMode: READ_ONLY_PATTERN.test(task) ? "response-only" : "orchestrator-artifact",
    reasons: exact ? ["explicit-or-circuit-breaker-trigger"]
      : intent.isMatch ? ["semantic-change-strategy-intent"] : [],
  };
}

const REFERENCE_RESEARCH_PATTERN =
  /\b(?:analy[sz]e|compare|inspect|research|study)\b|изуч|исслед|посмотр|проанализ|сравн/iu;
const EXTERNAL_REFERENCE_PATTERN =
  /\b(?:announcement|chart|diagram|release|site|website)\b|график|диаграм|релиз|сайт|страниц/iu;
const OPERATION_ACTION_PATTERN =
  /\b(?:build|create|deploy|draw|fix|publish|repair|tag|update)\b|выпусти|исправ|нарис|обнов|опубликуй|почин|релизь|созда|тегир|выкат/iu;

function shouldSuppressRoute(mode, task) {
  if (!new Set(["bugfix", "mermaid", "release"]).has(mode)) return false;
  const isReferenceResearch =
    REFERENCE_RESEARCH_PATTERN.test(task) && EXTERNAL_REFERENCE_PATTERN.test(task);
  return isReferenceResearch && !OPERATION_ACTION_PATTERN.test(task);
}

module.exports = {
  getChangeStrategyActivation,
  getIntentMatch,
  normalizeTask,
  shouldSuppressRoute,
};
