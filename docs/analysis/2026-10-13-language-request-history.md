# ESS feature requests: are we building them one-to-one?

Analysis of 2026-10-13. Read-only. Sources: every issue in `beyond10x/ess` (#72–#569, 353 issues,
2026-09-25 → 2026-10-13, fetched 2026-10-13 ~21:00 UTC), 367 issue and PR comments, 172 merged PRs, 71
releases, `CHANGELOG.md` and `docs/design/` at `origin/main` `260a3d1e3`, `ess specify formats`
from the installed `ess 0.59.0`, the specifying skill's references
(`~/.claude/plugins/cache/b10x/ess/0.22.8/skills/specifying/references/`), and adopter A's gap
table (`docs/ess-gaps.md`) and research note (`research/schemas.md` §6).

## Verdict

**Yes, strongly.** ESS builds language requests one-to-one. Of 81 language features it shipped, 59
(73%) are a construct shaped like the request: a new key, a named operator, or an existing operand
admitted in one more position. 18 (22%) are general mechanisms, and 7 of those 18 are relaxations or
literals. The median time from issue to release is 1 day. 23 source formats shipped in 39 days.

The projects are not esoteric. 90 of the 132 language requests (68%) are instances of four generic
capabilities ESS has chosen not to build: a typed expression language, selections and navigation
over rows, aggregates over expressions, and one branch-selection plan. Of the 35 language requests
that are open or parked in the planning store today, those four would absorb 27 (77%), and the first
three alone 23 (66%). Every adopter whose specification is a data model hits them: 80–93% of the
language requests from adopters A, D, H and J are generic. Only the adopters whose gaps are protocol
or wire shapes (F, I) bring mostly domain-specific needs.

The fit review ESS introduced on 2026-09-30 did not change the split. The share of shipped features
that are general stayed at 22% before and after. What changed is that requests began to be declined
with an idiom (15) or moved to the planning store (7). 6 of those 15 declines came back within 7
days, 4 of them explicitly reopened by another adopter for the same reason: the idiom moves the
figure out of the specification, so conformance never checks it.

Caveats. Classification is my judgement from issue text and changelog entries; Appendix A lists
every call so it can be checked. Adopter attribution is inferred: issues are filed by a bot in
brand-free words, and ESS's own review lists "how many distinct adopters stand behind the bot-filed
issues" as not established (`docs/design/review-external-requests-2026-09.md:119-120`). A bot
comment "Shipped in ESS 0.54.0" also sits on issues that were declined with an idiom (#439–#447), so
`CLOSED/COMPLETED` does not mean built.

## 1. What was counted

| kind | n | meaning |
|---|---|---|
| L | 132 | language request: a specification cannot say something (validate refuses it, or there is no syntax) |
| T | 87 | tooling request: synthesis, conformance, mutation, explorer, generators, CLI, diff, ess-ui |
| D | 124 | defect: an existing feature contradicts its own documentation, crashes or misbehaves |
| G | 5 | guidance or documentation request |
| X | 5 | duplicate refiled by the bot (#213–#215, #388) or moved to another repository (#232) |

Adopters (letters are labels, assigned from issue text): **A** incident ledger (#553–#569, all filed
2026-10-13) · **B** retrofits of existing backend services on 0.32–0.36 · **C** identity and sign-in
service · **D** software factory and its operator console · **E** a component lowered to Entity
Runtime · **F** metering event producer · **G** secrets custody service · **H** metrics read API ·
**I** models of published OAuth and OpenID standards · **J** queue and matching service · **K** rules
held in a condition language · **M** messaging and order service · **N** an adopter repository anonymised in this copy · **T** `agentplugins` trials. Public
repositories are named as such: `uilab`, `cortex`, `commission`, `connectors`, `loom`,
`epistemic-knowledge-runtime`, `llm-gateway`, `eventlog`. **U** is unattributed, **X** is ESS's own
reviews.

Adopter A's 17 issues match the request that prompted this analysis: #553 and #554 at 20:17 UTC, the
15 at 20:47 UTC (#555–#569), and 4 comments at 20:46–20:47 on #547, #553, #473 and #509.

## 2. (a) Families

| family | L | named adopters (L) | open or moved | T | what it covers |
|---|---|---|---|---|---|
| expressions / derived values | 28 | 9 (A B D E H I J K M) | 8 | – | operators, arithmetic, string functions, value sources (`{subject:}`, `{caller:}`, `else:`), Optional narrowing |
| references / relations | 20 | 6 (A B C D I J) | 8 | 1 | reading, guarding and changing other entities' rows, row-set selectors, composition |
| guards / preconditions | 18 | 8 (A B D J M T N `cortex`) | 5 | – | what selects a branch, combining guard kinds, precedence, `unknown_instance`, preconditions |
| wire / format | 15 | 4 (B F I `commission`) | 3 | – | wire names, presence, `Json`, text shapes, unit variants, response codes, HTTP binding |
| lifecycle shape / effects | 13 | 7 (A B G I J M `cortex`) | 3 | – | create into a state, delete, rename, effects on many records, two lifecycles |
| aggregation | 11 | 4 (A B D H) | 5 | – | aggregate views and their measures |
| realization / bindings / infra | 11 | 7 (B C D F H M `connectors`) | 0 | 25 | bindings, transport, topology, CLI binding; T: generators, targets, toolchain |
| views / queries | 7 | 5 (A B D H J) | 2 | – | params, paging, joins, grants, computed fields |
| temporal / periods | 4 | 3 (A B H) | 1 | – | `now`, durations, calendar and sliding windows |
| other | 5 | 2 (H K) | 0 | 2 | units, rules held as data, concurrency, requirement traceability |
| synthesis / conformance | – | – | – | 45 | witnesses, arrangement, mutation, explorer, authoring |
| ess-ui | – | – | – | 14 | the UI document format and its renderers |

All 10 language families recur across at least 2 named adopters; 8 recur across 4 or more. Counting
tooling requests as well, all 12 families recur.

Per adopter, the share of its language requests that one of the four generic capabilities of
section 9 would absorb:

| adopter | L | T | generic | not generic |
|---|---|---|---|---|
| A incident ledger | 14 | 3 | 13 (93%) | #559 two lifecycles |
| H metrics read API | 12 | 0 | 11 (92%) | #444 units |
| J queue and matching | 12 | 3 | 10 (83%) | #533, #542 |
| D software factory | 10 | 22 | 8 (80%) | #269, #286 |
| B service retrofits | 37 | 7 | 22 (59%) | 15: mostly wire and effect shapes |
| I published standards | 9 | 8 | 3 (33%) | #493 #497 #500 #507 #508 #509 |
| F metering events | 3 | 7 | 0 (0%) | #390 #391 #392 transport and envelope |

## 3. (b) General mechanism or a construct shaped like the request

| resolution | n | share of shipped |
|---|---|---|
| GEN, a general mechanism | 18 | 22% |
| SPEC, a construct shaped like the request | 59 | 73% |
| PART, part shipped | 4 | 5% |
| declined with an idiom | 15 | – |
| already expressible | 1 | – (#456: `when_subject_state: [..]` shipped in ess/18 for #201) |
| moved to the planning store, unshipped | 7 | – (all `status: draft`) |
| open | 28 | – |

7 of the 18 GEN are relaxations or literals (#135 #141 #142 #143 #176 #418 #445). The substantive
general mechanisms are #136 (nested mappings), #138 (`Json`), #151 (`deletes:`), #194/#268 (a
predicate on a binding), #204 (`state` as an operand), #225 (bare facts compare), #228/#299 (row-set
selector), #286 (view grants through `may:`) and #363 (`where:` on a measure).

22 of the 59 SPEC constructs are genuinely distinct concepts where a dedicated key is right, for
example `into:` (#150), `one_time_response:` (#389), `undeclared_fields:` (#500) and `presence:`
(#139). The other 37 are instances of the four generic capabilities of section 9.

Before and after the fit-review skill of 2026-09-30
(`.agents/skills/assessing-external-requests/SKILL.md`): 12 GEN of 54 shipped before it, and 6 of 27
from it on, 22% both times. All 15 idiom declines and all 7 moves to the planning store came after.

### Ten representative examples

| # | request | shipped syntax (format, release) | shape | what followed |
|---|---|---|---|---|
| 1 | #95 prefix, suffix and substring tests | `{f: {starts_with: "SM-"}}`, `ends_with`, `contains`; literal operand only, map form only (ess/8, 0.34.0; `website/docs/reference/predicates.md:855-860`) | SPEC | #140 added `equals_ignore_case` and `in_ignore_case` as separate operators (ess/15; `predicates.md:944-948`); #200 a typed `{param: q}` operand (ess/22); #449 regex, declined; #532 trim, open. Case-insensitive prefix, suffix and substring are out of scope (`docs/design/expression-family-source22.md:29-31`) |
| 2 | #104 text length | `<text>.count` (ess/11, 0.34.0) | SPEC | #233 needed `<text>.utf8_bytes` (ess/22), "refused in every other position" (`ess specify formats`, ess/22) |
| 3 | #134 increment a stored number | `{increment: 1}`, `sets:` only, literal (ess/14, 0.36.0; `later-formats.md:18`) | SPEC | #233 asked for `upper - lower <= 4000` and got one constant offset, `upper <= lower + 5` (ess/22; `predicates.md:297-328`); #441 and #442 declined; #443 cannot add an input amount (`docs/design/read-api-view-idioms.md:117-118`); #560, #561, #563 open |
| 4 | #171 a timestamp input against the current time | `now`, `now - 60s`, in `when:` over the input only (ess/16, 0.38.0) | SPEC (one position) | #244: `now` in `when_subject:` and `when_related:`, plus `window: {at, days, from, to, offset}` (ess/22); #439: `now` in a view filter, declined; #567 reopens it |
| 5 | #133 a value read from the stored row | `{subject: title}` (ess/14, 0.36.0); the text `subject.title` is refused (`later-formats.md:18`) | SPEC (one root) | #166 `{related: {via, field}}` (ess/16); #285 `via: [a, b]` (ess/22); #299 `{related: {entity, where, field}}` (ess/22); #458 `{subject: state}` (ess/23); #555 and #557 open |
| 6 | #96 aggregates | `group_by:` plus `aggregate: {count \| count_distinct \| sum \| min \| max \| avg: <one top-level field>}` (ess/10, 0.34.0) | SPEC (closed list, one field) | #148 `skip_absent: true` (ess/15); #363 `where:` (ess/22); #441 and #446 declined; #561 #562 #563 #565 #566 open |
| 7 | #145, #164, #201 select by existence and held state | `unknown_instance: true` (ess/15), `existing_instance: true` (ess/16), `when_subject_state: [A, B]` (ess/18) | SPEC | held state "is selectable five ways" (`docs/design/review-external-requests-2026-09.md:67-72`); combining guard kinds came back as #282 #454 #461 #473 #534 #554 |
| 8 | #167, #175 change records other than the subject | `instances: {where: …}`, `affects: [{entity, where, sets}]` (ess/16, 0.38.0) | SPEC | #452 `deletes:` with `instances:` (ess/23); #459 `affects: each: {in, as}` (ess/23); #531 and #547 open |
| 9 | #228, #299 uniqueness in a scope; the row a filter selects | `when_related: {entity, where, exists \| count \| forall}` and `{related: {entity, where, field}}` (ess/22, 0.53.0) | GEN | it absorbed #237's "count across records" with no new key. It has no order, so #543, #549 and #551 now add ordered heads (an `ess/25` integration branch, #551) |
| 10 | #363 count or sum only some rows | `aggregate: {count: {}, where: state == Completed}` (ess/22, 0.53.0) | GEN | the design took the request's spelling as "evidence of the need, not the selected syntax" and reused the predicate grammar (`docs/design/conditional-aggregate-measures.md:7, 22-31`). That is the pattern the other nine lacked |

`state` (#204, ess/18) is general in kind and still arrived one position at a time: in
`when_subject` (ess/18), in `when_related` (ess/20, #229, a format of its own), as a value (ess/23,
#458), and as a related row's value (#557, open). A view field derived from it was declined (#457).

## 4. (c) Recurrence: the same capability asked again

Two counts. **Mechanical lower bound:** 44 of 132 language requests cite an earlier language request
in their own body. **By reading:** 69 of 132 (52%) ask for a capability an earlier shipped or
declined construct already had in another position, for another type, or as another operator. Of
the 69, 29 shipped (23 of them as SPEC again), 10 were declined, 5 moved to the planning store and
25 are open.

| capability | chain (earlier → later) | later issues | adopters |
|---|---|---|---|
| text operators | #95 → #140, #200, #449, #532; #103 → #146 | 5 | B J K M |
| arithmetic | #134, #104, #171 → #233 → #441, #442 → #560; #134 → #443 | 5 | A E H |
| value roots | #137 → #163; #133 → #166 → #285, #299; #133, #204 → #458 → #557; #133 → #555; `{response:}` in payloads only (ess/4; `sets:` refuses it, `later-formats.md:29`) → #498 → #506, #545; #169 → #556 | 11 | A B D I J |
| ordered or filtered pick | #299, #446 → #543 → #549; #228 → #551 | 3 | J |
| current time | #171 → #244 → #439 → #567 | 3 | A H |
| aggregates | #96 → #148, #363, #446, #561, #562, #565, #566; #441 → #563 | 8 | A B D H |
| branch selection and precedence | #145 → #201 → #204 → #229, #461; #201 → #456; #211, #145 → #282; #227 → #454 → #534; #283, #228 → #473 → #554; #164, #75 → #530; #145 → #535, #533 | 13 | A C D J M N `cortex` |
| related rows | #211 → #283, #304, #505, #558, #509; #75 → #228 → #462, #463; #175, #229 → #547; #447 → #492, #564; #162 → #191, #525 | 13 | A C D H I J |
| multi-record effects | #167 → #452, #531; #167, #175 → #459; #175 → #497 | 4 | I J `cortex` |
| bindings | #194 → #268; #165 → #269; #195 → #440, #391 | 4 | D F H |

Three pairs that show the shape:

- **#442 → #560.** A field difference (`End.at - Begin.at`) was declined on 2026-10-06: "arithmetic
  beyond one constant offset is not in the value language" (`docs/design/read-api-view-idioms.md:84-86`).
  Adopter A asked for `ended_at - started_at` 7 days later.
- **#171 → #244 → #439 → #567.** `now` was admitted in `when:` (ess/16), then in `when_subject` and
  `when_related` (ess/22), declined in view filters (2026-10-06, `read-api-view-idioms.md:221-224`),
  and asked for again in a view filter on 2026-10-13.
- **#133 → #166 → #285 → #299 → #458 → #557.** Each new place a value can be read from got its own
  mapping form: the subject, one related row, an Optional or two-hop related row, a filtered row, the
  held state. The related row's state is the sixth request, open.

A binding has its own predicate grammar: "a binding condition compares a String or enum leaf with a
text literal" (#492, refusal `ESS-BINDING-002`), so `flag == false` works in a guard and fails in a
binding. It was accepted for a later format on 2026-10-07 and has not shipped: `ess/24` carried only
#500.

## 5. (d) Declines and whether they came back

| declined | asked | idiom given | came back |
|---|---|---|---|
| #439 clock-relative view windows | H | caller resolves the range into two `Timestamp` params (`read-api-view-idioms.md:198-232`) | **#567** (A, 7 days, "reopens"); #568 and #569: the idiom's own views do not synthesize |
| #441 ratio, difference, scaling | H | return both operands, the consumer divides (`read-api-view-idioms.md:26-54`) | **#563** (A, 7 days, "reopens"); #561 |
| #442 field derived from an earlier row | H | hold the begin row; the consumer subtracts (`read-api-view-idioms.md:56-89`) | **#560** (A, 7 days, "reopens") |
| #447 joins and computed columns | H | copy at write, or two views joined by the consumer (`read-api-view-idioms.md:174-196`) | **#492** item 4 (H itself, 1 day, declined again); **#564** (A, 7 days, "reopens"); ess-ui #364 asks for the same join in the UI layer |
| #446 first or last per group | H | `max`/`min`, or a paged view of size 1 (`read-api-view-idioms.md:145-172`) | **#543, #549** (J, 7 days): an ordered head, now on an integration branch |
| #454 an input refusal after the not-found and wrong-state answers | U | guide "which branch answers" | **#534, #554** (J, A): declare a precedence |
| #443 a per-key total | H | a grouped `sum` view | not yet |
| #444 units | H | a newtype per unit | not yet |
| #449 regex | K | "text shapes without patterns" (`predicates.md:1008-1020`) | #532 trim (J), adjacent |
| #457 a view field derived from the state | U | guide | #557 (A), adjacent |
| #440 #432 #451 #453 #497 | various | guides | not yet |

6 of 15 idiom declines came back within 7 days (#439, #441, #442, #446, #447, #454), 4 of them
explicitly reopened by another adopter; 2 more drew adjacent requests (#449 → #532, #457 → #557).
The reason given for reopening was the same each time: "The idiom does not serve: it moves the figure out
of the specification, so conformance never checks the number the reader sees" (#560, #563, #564,
#567). The idioms page says as much about itself: "The suite checks both numbers exactly, and the
abandonment rate is `abandoned / queued` in the client" (`read-api-view-idioms.md:35-37`). Adopter A
lists the same four declines as blockers (`docs/ess-gaps.md:26-41`, rows 11, 14, 15, 18).

## 6. (e) Format churn

| format | release | date | constructs added | single construct | narrow |
|---|---|---|---|---|---|
| ess/1 | 0.1.0 | 09-01 | baseline | – | – |
| ess/2 | 0.20.0 | 09-06 | `Binary64` | yes | |
| ess/3 | 0.23.0 | 09-11 | `when_subject_state`; envelope accessors | | |
| ess/4 | 0.23.0 | 09-11 | error wire names; response to payload | | |
| ess/5 | 0.27.0 | 09-20 | enum variant `wire`/`display`/`summary`/`code` | yes | yes |
| ess/6 | 0.28.0 | 09-21 | guard beside external; enum `when_subject`; `preserves` | | |
| ess/7 | 0.29.0 | 09-22 | `replays`; error default | | |
| ess/8 | 0.34.0 | 09-26 | `starts_with`/`ends_with`/`contains` | yes | yes |
| ess/9 | 0.34.0 | 09-26 | `when_subject: {predicate}` | yes | |
| ess/10 | 0.34.0 | 09-26 | aggregate views | yes | |
| ess/11 | 0.34.0 | 09-26 | `alphabet:`, `example:`, `.count` | | |
| ess/12 | 0.34.0 | 09-26 | `outcome_groups:` | yes | yes |
| ess/13 | 0.35.0 | 09-26 | `fixture_inputs:` | yes | yes |
| ess/14 | 0.36.0 | 09-27 | value forms in `payload:`/`sets:` (4) | | |
| ess/15 | 0.37.0 | 09-27 | 8 constructs | | |
| ess/16 | 0.38.0 | 09-28 | 7 constructs | | |
| ess/17 | 0.39.0 | 09-28 | `returns: true` | yes | yes |
| ess/18 | 0.41.0 | 09-29 | 4 constructs | | |
| ess/19 | 0.46.0 | 09-30 | `payload:` sources for error fields | yes | yes |
| ess/20 | 0.49.0 | 10-01 | `state` in `when_related` | yes | yes |
| ess/21 | 0.53.0 | 10-05 | `one_time_response:` | yes | yes |
| ess/22 | 0.53.0 | 10-05 | 14 constructs | | |
| ess/23 | 0.54.0 | 10-06 | 8 constructs | | |
| ess/24 | 0.58.0 | 10-10 | `undeclared_fields:` | yes | yes |
| ess/25 | – | – | on an integration branch (#551) | | |

Source: `ess specify formats` (ess 0.59.0) and `CHANGELOG.md` release headers.

- 23 formats after `ess/1` in 39 days: 18 in September (from 09-06), 5 in October to date, a sixth
  on a branch. Peak: 11 formats in 4 days (ess/8–ess/18, 09-26 → 09-29), 5 of them in one release
  (0.34.0).
- 12 of 23 added exactly one construct; 9 of those 12 added one narrow key or operand for one request
  (ess/5, 8, 12, 13, 17, 19, 20, 21, 24). ess/20 is the clearest case: one existing operand, `state`,
  admitted in one more position.
- 71 releases in the same 41 days (60 in September, 11 in October).
- ESS's own review asked for "formats that move once per decision"
  (`.engineering/planning/epic/language-consistency-after-adoption.md:13`, `status: draft` since
  2026-09-30).

## 7. (f) Time from request to release

Language requests that shipped, by the first release whose changelog entry names the issue:

| family | shipped | median days | max days |
|---|---|---|---|
| expressions | 18 | 1 | 7 |
| references | 12 | 1.5 | 6 |
| guards | 10 dated | 1 | 1 |
| lifecycle and effects | 9 | 1 | 7 |
| realization and bindings | 9 | 1 | 7 |
| wire and format | 12 | 0.5 | 3 |
| aggregation | 3 | 1 | 3 |
| views | 3 | 1 | 4 |
| temporal | 2 | 3.5 | 6 |
| all | 79 dated | 1 (p90 5) | 7 |

Tooling requests: synthesis 34 shipped, median 1 day; realization 21, median 1. Defects: synthesis
52, median 1. Idiom declines were decided in 0–1 days (15 of 15). The 7 requests moved to the
planning store are all still `status: draft`. The oldest open language request is #493 (6 days).

A one-day cycle leaves no step in which a second adopter's case can arrive before the key is fixed in
a format.

## 8. The language as it stands: construct keywords, not one expression language

- **Position decides what a predicate may read.** Nine places each admit their own roots
  (`predicates.md:21-34`): `when:` reads the input; `when_subject` stored fields and, from ess/15,
  `input.`; `when_related` a related row, with `state` only from ess/20; invariants and view filters
  their own fields; a binding `where:` "a bounded fragment". `now` is admitted in three guard
  positions and refused in invariants, view filters, selections and set-effect filters
  (`later-formats.md:33`).
- **Arithmetic is cut to one constant offset by design.** "General infix arithmetic, a left
  arithmetic expression, field-minus-field, Duration ordering, calendar units, days, zones,
  case-insensitive prefix/suffix/substring operators … are out of scope. `a - b <= k` is written
  `a <= b + k`" (`docs/design/expression-family-source22.md:29-33`). The cost avoided is synthesis:
  "It composes the base and left candidates rather than adding a general equation solver"
  (`expression-family-source22.md:184-185`).
- **Values have no expressions.** "Not in this design: Arbitrary arithmetic, string functions or
  conditionals beyond E4's one fallback" (`docs/design/value-expressions.md:394`). A value is one of
  a list of mapping forms (`{subject:}`, `{related:}`, `{increment:}`, `{input, else}`, `{caller:}`,
  `{response:}`, `{generated: true}`, `{cleared: true}`, `{count: changed}`).
- **Aggregates are a closed list over one top-level field.** No ratios or differences, no time
  buckets, no joins, no windows or ranking (`docs/design/aggregate-views.md:17-26, 1028-1038`).
  `ESS-VIEW-009`: "an aggregate reads one top-level field" (adopter A, probe g12b).
- **ESS has already found the fragmentation in itself.** Its fit review of 2026-09-30 found held state
  "selectable five ways", `exists` meaning "four things", `via` "spelled two ways", a construct
  "adopted verbatim", and "both requester alternatives adopted"
  (`docs/design/review-external-requests-2026-09.md:25, 43, 67-77, 113-115`). Both repair epics are
  `status: draft`: `language-consistency-after-adoption` and `one-selection-plan`.

## 9. (g) What a generic design would look like, and what it would absorb

**G1. One typed expression language over facts.**
- **Operands:** literals and paths rooted at `input`, `subject` (with `state`), `caller`,
  `response`, `param`, `event`, `context`, `now` and bound names.
- **Operators:** `+ - * /` over `Integer`/`Decimal`, `Timestamp - Timestamp → Duration`,
  `Timestamp ± Duration`; comparison on every ordered type; `and`/`or`/`not`/`in`; functions `len`,
  `utf8_bytes`, `lower`, `upper`, `trim`, `starts_with`, `contains`, `coalesce`, `if … then … else`
  and `round(x, mode, digits)`; `defined(x)` narrows `x` in its branch.
- **One grammar** for guards, invariants, view filters, binding conditions, `sets:`, `payload:`,
  response fields and computed view fields.
- **One position table** says which roots a place admits. Today's mapping forms become sugar that
  lowers to it, so existing documents keep their IR.
- **Cost:** a bounded linear-arithmetic witness search in synthesis, and exact evaluation of the same
  operators in the three runners. The runners already do exact `i128` offsets
  (`expression-family-source22.md:158-164`).

**G2. Selections and navigation as values.** `rows(E where p order_by k desc ties identity)` with
`exists`, `count`, `forall`, `first` and `each`, plus navigation through declared references
(`subject.incident.state`, `all(id in input.ids: exists(Service[id]))`). Usable in guards, values,
effects (`affects:` over a selection or one identity), view sources (a join along a declared
relation) and bindings. #228/#299 is already the seed; this adds order, navigation, list references
and every position.

**G3. Aggregates as expressions over a selection.** Measures take an expression operand
(`sum(ended_at - started_at)`, `sum(duration * level.weight)`). `min`/`max` work on any ordered type,
including an enum with a declared order. Group keys are expressions, including one group per list
element. Arithmetic over measures (`sum(up) / period_seconds`) carries a declared rounding and a
zero-denominator rule. `arg_max(expr, by)` and an interval-union measure clipped to a period are
library functions, not new keys.

**G4. One branch-selection plan.** `wrong_state`, `when_subject_state`, `unknown_instance`,
`existing_instance` and `when_related {exists: false}` become sugar for G1 predicates over
`exists(subject)`, `subject.state` and `exists(rel)`, in one ordered list where declaration order is
the precedence. This is the drafted `one-selection-plan` epic.

Issue by issue (Appendix A has the column for all 132):

| request set | n | G1 | G2 | G3 | G4 | none | G1–G3 | G1–G4 |
|---|---|---|---|---|---|---|---|---|
| open + moved today | 35 | 8 | 10 | 5 | 4 | 8 | 23 (66%) | 27 (77%) |
| recent, created ≥ 2026-10-05 | 64 | 15 | 17 | 8 | 7 | 17 | 40 (62%) | 47 (73%) |
| declined with an idiom or already expressible | 16 | 3 | 3 | 3 | 2 | 5 | 9 (56%) | 11 (69%) |
| shipped as SPEC | 59 | 16 | 12 | 2 | 7 | 22 | 30 (51%) | 37 (63%) |
| all language requests | 132 | 38 | 27 | 11 | 14 | 42 | 76 (58%) | 90 (68%) |

Open and moved, by absorber:

- **G1 (8):** #498 #506 #532 #545 #555 #556 #560 #567
- **G2 (10):** #473 #505 #531 #543 #547 #549 #551 #557 #558 #564
- **G3 (5):** #561 #562 #563 #565 #566
- **G4 (4):** #530 #534 #535 #554
- **none (8):** #493 HTTP method and path · #507 encoded JWS claims · #508 two wire codes · #509
  reference integrity · #525 contract role · #533 a view's answer for an unknown id · #542
  system-issued commands · #559 two lifecycles

The three that absorb the most among open and moved requests: G2 29%, G1 23%, G3 14%. Across all
132: G1 29%, G2 20%, G4 11%, G3 8%. Adopter A's 14 language requests: 13 absorbed (G1 4, G2 3, G3 5,
G4 1). The exception is two lifecycles on one entity.

## 10. What to change in how ESS takes requests

1. **Triage onto a capability matrix, not into one story per issue.** The planning store holds 130
   `story/feature-request-NNN.md` files, one per issue. Each request should name the cells it fills
   (operand × position, operator × type, effect × target), and the decision should close the row or
   column rather than the cell. Question 4 of the fit review, "a capability added to one construct is
   added to the siblings where the same question arises" (`SKILL.md:47-48`), is the rule; the matrix
   makes it checkable. Every recurrence chain in section 4 is a row closed one cell at a time.
2. **Decide G1 once, with its synthesis cost on the table.** The one-constant-offset cut
   (`expression-family-source22.md:29-33`, shipped 2026-10-05) avoided a solver. Within 8 days it
   was followed by #441, #442 and #443 (declined) and #560, #561 and #563 (open), all arithmetic. A decision that a general expression language is out of scope should be
   explicit and dated, so that idioms stop being the default answer.
3. **Do not decline with an idiom that takes an observable out of conformance.** 4 of 15 such
   declines were reopened within 7 days for exactly that reason. Rule: an idiom is valid only if the
   suite still checks the figure the specification's reader sees.
4. **One format per decision, not per request.** The fit review asked for it; 9 of 23 formats were a
   single narrow key. Hold language requests for a batch window (for example weekly) so that a second
   adopter's case can arrive before the key is frozen.
5. **Hold the generality rate as a measured number.** Count GEN against SPEC per release, and
   recurrences per capability. Today GEN is 22% and the fit review did not move it; declines rose
   instead.

## Appendix A. Every language request (kind L), 132 rows

Columns: issue, created, resolution (GEN/SPEC/PART shipped; IDIOM declined with an idiom; DUP already expressible; MOVED to the planning store, unshipped; OPEN), family, adopter, first release naming it (from `CHANGELOG.md`), the generic feature of section 9 that would absorb it, title.

| # | created | res | family | adopter | released | absorbed by | title |
|---|---|---|---|---|---|---|---|
| #75 | 09-25 | PART | guard | T | 0.34.0 | G1 | No way to state a constraint across records or over stored state |
| #95 | 09-25 | SPEC | expr | K | 0.34.0 | G1 | Proposal: starts_with, ends_with, contains for String predicates |
| #96 | 09-25 | SPEC | agg | H | 0.34.0 | G3 | Proposal: aggregate views (count, sum, min, max, avg, grouped) |
| #103 | 09-25 | SPEC | wire | B | 0.34.0 | none | A String input's character set cannot be declared, so a generated success scenario can never... |
| #104 | 09-25 | SPEC | expr | B | 0.34.0 | G1 | No length predicate on String in when: (keys.length > 64 is unobservable_fact) |
| #105 | 09-25 | SPEC | guard | B | 0.34.0 | none | Proposal: declare one outcome for a group of commands (e.g. a rejected session credential) |
| #113 | 09-26 | SPEC | guard | B | 0.34.0 | G4 | Integer literals in sets: must be quoted, and an unknown instance has no declared outcome |
| #133 | 09-27 | SPEC | expr | B | 0.36.0 | G1 | No payload or sets: source for a value read from, or computed over, stored state; subject.<f... |
| #134 | 09-27 | SPEC | expr | B | 0.36.0 | G1 | sets: cannot increment a stored number, and refuses {generated: true} |
| #135 | 09-27 | GEN | expr | B | 0.36.0 | G1 | No Decimal literal in sets: |
| #136 | 09-27 | GEN | expr | B | 0.36.0 | G1 | A struct-typed payload field cannot be built from inputs |
| #137 | 09-27 | SPEC | expr | B | 0.36.0 | G1 | No payload source for 'the caller's value if given, otherwise minted' |
| #138 | 09-27 | GEN | wire | B | 0.37.0 | none | No type for 'any JSON value' |
| #139 | 09-27 | SPEC | wire | B | 0.37.0 | none | An Optional field cannot say 'always sent as null' versus 'left out when unset' |
| #140 | 09-27 | SPEC | expr | B | 0.37.0 | G1 | No case-insensitive comparison in a when: guard |
| #141 | 09-27 | GEN | wire | B | 0.37.0 | none | Field names cannot begin with an underscore |
| #142 | 09-27 | GEN | wire | B | 0.37.0 | none | No field-level wire name on command inputs and event fields |
| #143 | 09-27 | GEN | wire | B | 0.37.0 | none | A map key cannot be a newtype of a primitive |
| #144 | 09-27 | SPEC | life | B | 0.37.0 | none | An accepted request that changes nothing cannot be declared |
| #145 | 09-27 | SPEC | guard | B | 0.37.0 | G4 | An unknown identity is folded into the wrong_state branch |
| #146 | 09-27 | SPEC | wire | B | 0.37.0 | none | A String type cannot constrain a prefix or pattern, so synthesized witnesses break the imple... |
| #148 | 09-27 | SPEC | agg | B | 0.37.0 | G3 | Aggregate views cannot aggregate over, or group by, an Optional field |
| #150 | 09-27 | SPEC | life | B | 0.37.0 | none | An outcome cannot create an entity into a non-initial state |
| #151 | 09-27 | GEN | life | B | 0.37.0 | none | No outcome can delete its subject; terminal rows the implementation removes cannot be specified |
| #152 | 09-27 | SPEC | guard | B | 0.37.0 | none | No ambient precondition (e.g. an open session): the generated explorer disagrees at step 1 |
| #157 | 09-27 | SPEC | expr | B | 0.37.0 | G1 | A guard cannot compare a command input with a stored field of its subject |
| #162 | 09-27 | SPEC | ref | B | 0.38.0 | none | A consumer specification cannot reference another system's declared types |
| #163 | 09-27 | SPEC | expr | B | 0.38.0 | G1 | A payload or sets: fallback after else: cannot be a literal |
| #164 | 09-27 | SPEC | guard | B | 0.38.0 | G4 | No outcome can be selected by whether the addressed record exists (update or create) |
| #165 | 09-27 | SPEC | real | B | 0.38.0 | none | A binding cannot state an attempt bound or which failures are final |
| #166 | 09-27 | SPEC | ref | B | 0.38.0 | G2 | No payload or sets: source for a field of a record the subject references |
| #167 | 09-27 | SPEC | life | B | 0.38.0 | G2 | No outcome can change every record that matches a filter |
| #168 | 09-27 | SPEC | expr | B | 0.38.0 | G1 | No value source or guard operand for the authenticated caller (e.g. the caller's account) |
| #169 | 09-27 | SPEC | expr | B | 0.38.0 | G1 | An Optional input is not narrowed after an outcome refuses its absence (ESS-COMMAND-002) |
| #170 | 09-27 | SPEC | wire | B | 0.38.0 | none | No outcome for a command that arrives with no input at all (absent request body) |
| #171 | 09-27 | SPEC | time | B | 0.38.0 | G1 | A guard cannot compare a Timestamp input with the current time (e.g. more than 60 s in the p... |
| #174 | 09-27 | PART | view | B | 0.38.0 | G1 | A view cannot declare paging (page/size, total) or a caller-supplied filter expression |
| #175 | 09-27 | SPEC | life | B | 0.38.0 | G2 | An outcome cannot change instances other than its subject |
| #176 | 09-27 | GEN | expr | B | 0.38.0 | G1 | defined() does not admit an Optional struct field, so presence cannot be an invariant |
| #178 | 09-27 | SPEC | guard | B | 0.38.0 | G4 | A refusal guarded on the identity overlaps every accepting branch, and the overlap cannot be... |
| #191 | 09-28 | SPEC | ref | U | 0.40.0 | none | ess-composition/2 conformances refuse reader-side widening: newtype as primitive, enum as St... |
| #194 | 09-28 | GEN | real | M | 0.53.0 | G1 | Feature request: a binding cannot invoke only when an Optional path is present (ESS-BINDING-... |
| #195 | 09-28 | SPEC | real | M | 0.41.0 | none | An event-triggered binding cannot read the delivery context (the channel or subscription the... |
| #197 | 09-28 | SPEC | life | M | 0.53.0 | none | Feature request: a refused command cannot declare the compensating change the service makes ... |
| #200 | 09-28 | SPEC | expr | M | 0.53.0 | G1 | Feature request: a search parameter on a view (a view parameter as the operand of contains /... |
| #201 | 09-28 | SPEC | guard | M | 0.41.0 | G4 | A command cannot answer differently in different states its moves do not start from (wrong_s... |
| #204 | 09-28 | GEN | guard | M | 0.41.0 | G4 | A branch cannot be selected by the held state and a stored field together ("a subject fact h... |
| #211 | 09-28 | SPEC | ref | C | 0.41.0 | G2 | A creating command cannot be guarded by another entity (no configuration / no matching entry... |
| #225 | 09-29 | GEN | expr | D | 0.53.0 | G1 | Feature request: a guard comparing two identity-typed inputs (a self-edge check) |
| #228 | 09-29 | GEN | ref | C | 0.53.0 | G2 | No way to declare a multi-field key unique within a scope (equality, one arranged row) |
| #229 | 09-29 | SPEC | ref | C | 0.49.0 | G2 | No guard on another entity's state for non-creating commands, and no effect on related records |
| #233 | 09-29 | SPEC | expr | E | 0.53.0 | G1 | Value expressions: dotted input paths in sets:/payload:, field arithmetic, sibling-field com... |
| #237 | 09-29 | SPEC | expr | E | 0.53.0 | G1 | Distinct list members and a count across records |
| #244 | 09-29 | SPEC | time | U | 0.53.0 | G1 | Feature request: guards over elapsed time since a stored instant and over calendar windows |
| #268 | 09-30 | GEN | real | D | 0.53.0 | G1 | bindings: react to one outcome of a command, not only to an event |
| #269 | 09-30 | SPEC | real | D | 0.53.0 | none | bindings: per-refusal failure policy |
| #282 | 10-01 | SPEC | guard | D | - | G4 | ESS-COMMAND-004 refuses a when_related refusal beside a wrong_state outcome; no precedence i... |
| #283 | 10-01 | SPEC | ref | D | 0.53.0 | G2 | A command can guard on only one related row (a second exists: false branch is ESS-COMMAND-004) |
| #285 | 10-01 | SPEC | ref | D | 0.53.0 | G2 | {related:} follows only one required reference: no read through an Optional reference or acr... |
| #286 | 10-01 | GEN | view | D | 0.53.0 | none | No read grant on a view: an actor's may: covers commands only |
| #299 | 10-01 | GEN | ref | D | 0.53.0 | G2 | No read of a row selected by a filter (the instance with field == input.x) in a guard or in ... |
| #304 | 10-01 | SPEC | ref | D | 0.53.0 | G2 | when_related through an Optional input: an absent reference takes the not-found branch inste... |
| #363 | 10-02 | GEN | agg | D | 0.53.0 | G3 | Aggregates: no conditional count or sum (count where state == Done) |
| #389 | 10-02 | SPEC | wire | U | 0.53.0 | none | Feature request: one-time response values with non-disclosure conformance checks |
| #390 | 10-02 | SPEC | real | F | 0.52.0 | none | No transport construct: an event's channel cannot name its broker, stream or retention |
| #391 | 10-02 | SPEC | real | F | 0.53.0 | none | A channel address cannot be a template bound to payload or context fields |
| #392 | 10-02 | SPEC | wire | F | 0.52.0 | none | An event cannot carry a JSON array of items as its message |
| #418 | 10-04 | GEN | wire | commission | 0.53.0 | none | Unit (payload-less) variants in mixed unions |
| #429 | 10-05 | SPEC | life | G | 0.54.0 | none | No effect for an outcome that changes a record's identity (rename) |
| #432 | 10-05 | IDIOM | real | C | guide 0.54.0 | none | Topology: one central write path with a read replica in each environment |
| #438 | 10-05 | PART | view | H | 0.54.0 | G1 | List-typed view parameters and parameter defaults |
| #439 | 10-05 | IDIOM | time | H | guide 0.54.0 | G1 | Clock-relative view windows: named ranges in a time zone and sliding windows |
| #440 | 10-05 | IDIOM | real | H | guide 0.54.0 | G2 | Binding inputs read from stored state at consume time |
| #441 | 10-05 | IDIOM | agg | H | guide 0.54.0 | G3 | Derived values over aggregates: ratio, difference, scaling, rounding |
| #442 | 10-05 | IDIOM | expr | H | guide 0.54.0 | G2 | A field derived from a correlated earlier record |
| #443 | 10-05 | IDIOM | agg | H | guide 0.54.0 | G3 | A maintained per-key aggregate record (fold) |
| #444 | 10-05 | IDIOM | other | H | guide 0.54.0 | none | Units on numeric fields |
| #445 | 10-05 | GEN | real | H | 0.54.0 | G1 | A binding mapping cannot write a Boolean or number literal |
| #446 | 10-05 | IDIOM | agg | H | guide 0.54.0 | G3 | Order-dependent aggregates and nested grouped results |
| #447 | 10-05 | IDIOM | view | H | guide 0.54.0 | G2 | Multi-source views: joins and computed columns |
| #449 | 10-05 | IDIOM | expr | K | guide 0.54.0 | G1 | A regular-expression predicate |
| #450 | 10-05 | SPEC | other | K | 0.54.0 | none | Declared reference rows |
| #451 | 10-05 | IDIOM | other | K | guide 0.54.0 | none | Rules held as data cannot be given semantics |
| #452 | 10-05 | SPEC | life | U | 0.54.0 | G2 | An outcome that removes many records, and cascade removal |
| #453 | 10-05 | IDIOM | other | U | guide 0.54.0 | none | Concurrency: a process-local lock serialising every transition |
| #454 | 10-05 | IDIOM | guard | U | guide 0.54.0 | G4 | An input-selected refusal cannot be ordered after unknown-instance and wrong_state |
| #456 | 10-05 | DUP | guard | U | - | G4 | when_subject_state names one state, not a list |
| #457 | 10-05 | IDIOM | view | U | guide 0.54.0 | G1 | A view field derived from the lifecycle state |
| #458 | 10-05 | SPEC | expr | U | 0.54.0 | G1 | An error field naming the target state versus the current state |
| #459 | 10-05 | SPEC | life | cortex | 0.54.0 | G2 | An outcome cannot create or update one record per element of a collection |
| #461 | 10-05 | SPEC | guard | cortex | 0.54.0 | G4 | A refusal selected by a stored field loses its unchanged-subject check, and cannot sit besid... |
| #462 | 10-05 | SPEC | ref | U | 0.54.0 | G2 | A row-set guard on an upsert command (external create, updating default) synthesizes as Unkn... |
| #463 | 10-05 | SPEC | ref | U | 0.54.0 | G2 | A row-set selector cannot select rows by a struct identity |
| #466 | 10-06 | SPEC | real | connectors | 0.54.0 | none | ess-cli has no source for a trailing argument list (-- <args…>) |
| #473 | 10-06 | MOVED | guard | N | - | G2 | One command cannot combine identity-addressed and row-set when_related guards (ESS-COMMAND-0... |
| #492 | 10-07 | PART | other | H | - | G1 | Gaps left after the read-API idioms: Boolean binding conditions, store-only outcomes, list s... |
| #493 | 10-07 | OPEN | wire | I | - | none | No way to bind a command to the HTTP method and path a protocol fixes or a deployment chooses |
| #497 | 10-08 | IDIOM | life | I | guide  | none | An outcome cannot consume its subject and create a record of another entity |
| #498 | 10-08 | MOVED | expr | I | - | G1 | No way to store a generated value and return the same value |
| #500 | 10-08 | SPEC | wire | I | 0.58.0 | none | A response cannot admit members it does not declare |
| #505 | 10-08 | MOVED | ref | I | - | G2 | when_related via: cannot read the related row's identity in its predicate (ESS-COMMAND-003) |
| #506 | 10-08 | MOVED | expr | I | - | G1 | A response field cannot state its value source: a required echo of an input or a stored field |
| #507 | 10-08 | MOVED | wire | I | - | none | Encoded structured values: a JWS string whose claims are constrained can only be typed String |
| #508 | 10-08 | MOVED | wire | I | - | none | One refusal answered with either of two wire codes cannot be declared |
| #509 | 10-08 | MOVED | ref | I | - | none | ESS-COMMAND-005 demands an exists: false branch the relation makes unreachable |
| #525 | 10-10 | OPEN | ref | U | - | none | A component cannot be declared to play another specification's component (a contract role) |
| #530 | 10-12 | OPEN | guard | J | - | G4 | A create-or-update command cannot select its update branches by stored fields |
| #531 | 10-12 | OPEN | life | J | - | G2 | A selected-rows effect cannot emit one event per changed row |
| #532 | 10-12 | OPEN | expr | J | - | G1 | No guard comparison that ignores surrounding whitespace |
| #533 | 10-12 | OPEN | view | J | - | none | A view has no answer for an identity no row carries |
| #534 | 10-12 | OPEN | guard | J | - | G4 | An external: refusal cannot answer before the held state |
| #535 | 10-12 | OPEN | guard | J | - | G4 | unknown_instance is refused on a command that only preserves or refuses |
| #542 | 10-13 | OPEN | life | J | - | none | A command the system issues itself cannot be declared, so conformance drives it and cannot c... |
| #543 | 10-13 | OPEN | ref | J | - | G2 | No payload source reads the first row of an ordered selection (the head of a ranked side) |
| #545 | 10-13 | OPEN | expr | J | - | G1 | A minted value cannot be required present in an Optional stored field, nor tied to the value... |
| #547 | 10-13 | OPEN | ref | J | - | G2 | affects: cannot address a related record by its identity, and the Rust and web generators re... |
| #549 | 10-13 | OPEN | ref | J | - | G2 | A head of an ordered selection cannot be scoped by another head the same outcome reads |
| #551 | 10-13 | OPEN | ref | J | - | G2 | A row selector cannot test each candidate's related rows (pick the best slot whose sides are... |
| #554 | 10-13 | OPEN | guard | A | - | G4 | A related-row guard cannot sit beside the addressed record's own guards (ESS-COMMAND-004): d... |
| #555 | 10-13 | OPEN | expr | A | - | G1 | A refusal's payload cannot read the addressed record's stored fields ({subject: …} on a rela... |
| #556 | 10-13 | OPEN | expr | A | - | G1 | An Optional value narrowed by its own guard should read as the inner type in a payload (exis... |
| #557 | 10-13 | OPEN | expr | A | - | G2 | A related row's lifecycle state cannot be read as a payload value |
| #558 | 10-13 | OPEN | ref | A | - | G2 | An existence check on each element of a List reference input (when_related over List<Id>) |
| #559 | 10-13 | OPEN | life | A | - | none | Two independent lifecycles on one entity |
| #560 | 10-13 | OPEN | expr | A | - | G1 | Timestamp - Timestamp as a Duration value in sets:, payload: and view fields |
| #561 | 10-13 | OPEN | agg | A | - | G3 | An aggregate over an expression (avg of a field difference, sum of a product with an enum at... |
| #562 | 10-13 | OPEN | agg | A | - | G3 | An interval-union aggregate clipped to a period (downtime counted once across overlapping wi... |
| #563 | 10-13 | OPEN | agg | A | - | G3 | A ratio over aggregates with the period length from view parameters as an operand (uptime) |
| #564 | 10-13 | OPEN | view | A | - | G2 | A view reading a referenced row's current fields, one hop (filter and project) |
| #565 | 10-13 | OPEN | agg | A | - | G3 | Group a view by a List field, one group per element |
| #566 | 10-13 | OPEN | agg | A | - | G3 | max and min over an enum with a declared order |
| #567 | 10-13 | OPEN | time | A | - | G1 | A view filter period relative to now (sliding window resolved at read time) |

## Appendix B. Tooling requests (T) and defects (D) by family

**Tooling requests (T)**

- synth (45): #110 #114 #132 #153 #156 #172 #179 #193 #209 #210 #212 #220 #221 #222 #223 #226 #234 #236 #265 #273 #287 #293 #294 #296 #297 #312 #317 #347 #360 #361 #362 #413 #427 #434 #496 #499 #501 #512 #515 #536 #537 #538 #553 #568 #569
- real (25): #73 #106 #147 #224 #231 #290 #310 #314 #318 #319 #379 #393 #394 #395 #406 #407 #408 #409 #412 #460 #468 #476 #481 #482 #514
- ui (14): #281 #284 #311 #315 #322 #327 #328 #330 #351 #352 #354 #358 #364 #365
- other (2): #503 #504
- ref (1): #437

**Defects (D)**

- synth (66): #74 #94 #101 #111 #112 #154 #155 #160 #161 #173 #186 #188 #192 #196 #198 #199 #202 #203 #216 #218 #235 #239 #240 #242 #243 #251 #257 #266 #267 #270 #271 #272 #275 #278 #279 #280 #288 #291 #292 #295 #301 #307 #308 #309 #342 #416 #428 #430 #464 #465 #471 #472 #474 #479 #480 #510 #511 #513 #516 #517 #521 #539 #544 #546 #550 #552
- real (27): #72 #76 #77 #78 #102 #219 #253 #256 #260 #261 #274 #276 #306 #316 #400 #414 #415 #423 #424 #433 #467 #469 #483 #484 #485 #541 #548
- ui (17): #262 #300 #303 #305 #320 #321 #323 #324 #325 #326 #329 #346 #348 #353 #355 #356 #357
- guard (8): #177 #205 #217 #227 #298 #455 #486 #489
- other (3): #79 #385 #448
- expr (2): #93 #289
- ref (1): #230

**Guidance (G)**

- real (3): #426 #435 #436
- expr (1): #92
- other (1): #206

**Duplicates / moved elsewhere (X)**

- ref (2): #214 #215
- guard (1): #213
- real (1): #232
- wire (1): #388

