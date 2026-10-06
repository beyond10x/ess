---
format: aep.planning-md/3
id: story:feature-request-442
kind: story
status: implemented
title: A field derived from a correlated earlier record
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#442
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-441
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/latest-correlated-record.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/tests/read_api_view_idioms.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/row_sets.rs
- confidence: cited
  path: docs/design/filtered-related-reads.md
- confidence: inferred
  path: docs/design/read-api-view-idioms.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:24Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:24Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:49:11Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome
Resolve beyond10x/ess#442: A field derived from a correlated earlier record.

## Origin
beyond10x/ess#442, filed 2026-10-05; a downstream specification whose end rows take their duration from the latest earlier matching begin row, where the source has three branches and the ESS outcome one. Reproduced minimally in `<fit-review scratch>/probe-442/`.

## Fit review
1. Need: an end record, keyed by a correlation key, records one of three things:
   - carried: the end carries its own duration;
   - derived: otherwise, the latest earlier begin of the same key supplies `end.at - begin.at`, clamped at 0;
   - dropped: with neither, no row is recorded.
   The requester's proposal (theirs): "an outcome reading the latest (or earliest) row a filter selects, ordered by a field". The reproduction keys on two fields, `(customer, session)`; four conjuncts have the same shape.
   - The issue's key includes a field named `state`. On an entity with a lifecycle that name is refused, `ESS-ENTITY-006 wire field "state" is already used at … lifecycle` (probe `v-statefield`), so it needs another name.
   - "Latest earlier" is unclear. It may mean the latest begin to arrive, or the latest begin whose instant is not after the end's when begins arrive out of order. Quoted: "the latest earlier Begin row matching (customer, name, session, state), clamped at 0". The clamp suggests begins later than the end do occur. Ask them.
2. Class: convenience for the arrival-order reading; the idiom in Q3 runs. Arrival order is also what an ESS command can see: every guard and value reads the pre-outcome store (`docs/design/filtered-related-reads.md:15-18`).
   - Choosing no latest row is a documented rule, not a defect: "There is no implicit latest row" (`filtered-related-reads.md:17-18`), and "no first or latest row is chosen" (`website/docs/reference/predicates.md:1235-1238`). A latest-row implementation is a named failing control (`filtered-related-reads.md:213`, `:221-222`).
   - The out-of-order instant reading would be a gap. No evidence yet that it is needed.
   - The subtraction `end.at - begin.at` is outside the value language: "Arbitrary arithmetic … not in this design" (`docs/design/value-expressions.md:394`). It is the same line #441 keeps for views.
3. Existing idiom: hold the latest begin as one row per key, and select it, so that exactly one row matches. All runs below are on the 0.53.0 debug build (`<private scratch path>`).
   - `Begin` is the create-or-update pair on `OpenSpan`, identity `span_key` (`website/docs/guides/specify/commands-and-outcomes.md:99-130`). A later begin overwrites `began_at`.
   - `End`, with `duration: Optional<Integer>`, has five branches:
     - `carried`: `when: defined(duration)`, sets `duration: input.duration`.
     - `ambiguous`: refusal, `when_related {entity: OpenSpan, where: all[customer == input.customer, session == input.session], count: {gt: 1}}`.
     - `unmatched`: refusal, the same selector with `count: {eq: 0}`. This is the dropped branch.
     - `clamped`: the same selector with `forall: began_at >= input.at`, sets `duration: 0`.
     - `derived`: the default; `began_at: {related: {entity, where, field: began_at}}` and `ended_at: input.at`.
     Each guarded branch other than `carried` also has `when: missing(duration)`.
   - Results:
     - `ess specify validate`: `demo v1 — 1 file(s), valid`.
     - `synthesize`: `7 scenario(s) (0 authored), 0 refusal(s)`.
     - `run --target interpreted`: `7 scenarios: 7 passed` (probe `clamp/`). The suite itself arranged and witnessed `ambiguous`.
   - The derived branch stores both instants. The consumer computes the difference, as #441 decided for views.
   - Things that do not work:
     - The identity-addressed form, `when_related {via: …, exists: false}`, does not fit: that branch "answers a missing row before any other" (`predicates.md:27`), so it would drop a carried end.
     - Ordering begins by instant inside the create-or-update pair is refused: `when_subject: {predicate: began_at <= input.at}` on the update branch gives `ESS-COMMAND-004 subject fact selection requires one existing subject identity` (probe `clamp-ordered2`; `crates/specify/ess-domain/src/command/subject_fact.rs:213-222`).
   - The requester's shape is refused, but by the wrong message. `{related: {entity, where, field, order_by: […]}}` (probe `requested/`) and `{…, latest: at}` (probe `requested-latest/`) each give `a list is read only as the via: of {related: {via: […]}}`. `RawRelatedSelection::selects` needs exactly three keys (`crates/specify/ess-domain/src/command.rs:1624-1638`), so a fourth key falls back to the nested-mapping reading (`values-and-views.md:128-129`), and the error then lands on `where`'s list. The design says extra keys are refused (`filtered-related-reads.md:104-106`); the message names neither the key nor the rule.
   - Side observation, not filed: writing the identity into the creating `sets:` (`span_key: input.span_key`) makes synthesis refuse `ESS-SYNTH-001 … no arrangement of rows`, with a hint about finite types (probe `v-plaincreate/`). Without that line: 0 refusals.
4. Fit, for the requester's latest/earliest-by-field read:
   - It reverses an approved rule (`filtered-related-reads.md:17-18`), and it reverses a decisive control: `reverse_row_order` must pass and a latest-row implementation must fail (`:213`, `:221-222`).
   - It needs a tie rule on every target: interpreter, history checker, generated Rust and Go (a row-set command there is still a hand-written obligation, `predicates.md:1247-1249`), and the Go and TypeScript runners. Synthesis would need tying rows and an earlier decoy.
   - Siblings: the guard `when_related`, `affects:` filters and the view arg-max would each need the same order. #446 declined the view arg-max (`first`/`last`) and kept ordering with windows (`docs/design/aggregate-views.md:1034`). Adding an order here only would split that line.
5. Second adopter: "the price in force for a product at an order's instant" and "a device's last reported location before an alarm". Both are a current-value row per key when events arrive in order, so the idiom serves them. A point-in-time lookup over out-of-order history would also serve them, and nobody has asked for it yet.
6. Cost of the idiom: none in the format; a note section, one fixture and tests. A clearer extra-key message would be a small parser change with no format bump; it is not in scope. Cost of accepting: an order key on the selector and its IR variant (`related_selection`), a tie rule, diff classification, synthesis arrangement, every target, and `ess/23`.
7. Alternatives:
   - (a) Change nothing: the adopter keeps one branch, and the refusal message stays misleading.
   - (b) The requester's ordered read: refused, as in Q4.
   - (c) `{related: {entity, where, field, latest_by: <field>}}` with ties broken by identity: the smallest form of (b). Record it together with #446's `arg_max`, as one order-dependent-pick design, for when a point-in-time need is confirmed.
   - (d) Chosen: decline with the held-row idiom.

## Decisions
decline, with the idiom — "the latest earlier row" is a row the specification holds:
- one row per correlation key, written by a create-or-update begin;
- read by a row-set selector with explicit `count` branches;
- carried, derived, clamped and dropped as separate branches.

The difference `ended_at - began_at` stays the consumer's (value-expressions.md:394, as #441). The story body is the decline record. The idiom is documented as the section `## The latest earlier row is a held row` in `docs/design/read-api-view-idioms.md`. Depends on #441 (edge recorded), which creates the note and its test. Its model is the fixture `crates/verify/ess-conformance/tests/fixtures/latest-correlated-record.yaml`, domain `idioms.correlated` (entities `idioms.correlated.OpenSpan` and `idioms.correlated.SpanEnd`, commands `idioms.correlated.Begin` and `idioms.correlated.End`); it adds no file to the example directory. No format bump; not part of the `ess/23` bump.

Noted, not in scope: a `related:` mapping with a fourth key (`order_by:`, `latest:`) is refused with "a list is read only as the via: of {related: {via: […]}}", which names neither the key nor the no-order rule (Q3; `command.rs:1624-1638`, `:1849-1852`). The issue did not ask for a diagnostic change.

Reply to the requester with the idiom, and ask:
- whether begins arrive out of instant order;
- what "earlier" means.

If they confirm a point-in-time need, reopen as alternative (c) jointly with #446.

## Acceptance
- latest_correlated_record_idiom_validates_and_synthesizes: the fixture validates; synthesis writes all seven branch scenarios with no refusal.
- latest_correlated_record_idiom_runs_interpreted: all seven pass against the interpreter. A target that keeps the first begin instead of the latest fails `Begin/outcome/superseded` or `End/outcome/derived`.
- latest_earlier_row_section_states_the_idiom: `docs/design/read-api-view-idioms.md` has the heading `## The latest earlier row is a held row`. The section contains "one row per correlation key", "create-or-update", "`count: {gt: 1}`", "`count: {eq: 0}`", "`forall`", "no row order is chosen", "`ended_at - began_at`" and the fixture path `crates/verify/ess-conformance/tests/fixtures/latest-correlated-record.yaml`. The case `latest_earlier_row_section_states_the_idiom` in `crates/verify/ess-conformance/tests/read_api_view_idioms.rs` reads the note, fails on a missing heading or phrase, and checks that the named fixture exists.

## Scope
- crates/verify/ess-conformance/tests/row_sets.rs  cited — idiom synthesis and interpreted cases
- crates/verify/ess-conformance/tests/fixtures/latest-correlated-record.yaml  inferred — the idiom fixture
- docs/design/read-api-view-idioms.md  inferred — adds `## The latest earlier row is a held row` (file created by #441)
- crates/verify/ess-conformance/tests/read_api_view_idioms.rs  inferred — adds the named section case
- docs/design/filtered-related-reads.md  cited — the rule kept (17-18, 213, 221-222)
