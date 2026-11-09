---
format: aep.planning-md/3
id: story:a-view-read-checks-every-rows-shape
kind: story
status: draft
title: A view read checks every returned row against the view fields
summary: 'Row shape check on synthesized and authored view reads, split from #680 for 0.70.0; design page first.'
tags:
- adopter-report
- ess-0.70.0
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#680
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
revision: 12
---
## Need

A view read in a conformance suite checks only the fields the scenario names. Nothing checks that
every row the target returns carries the view's declared required fields with their declared types,
so a target that drops a stored field from a view, or returns it with the wrong type, passes every
view step that does not name that field. An event step already carries its declared shape
(`PayloadShape`) and fails such a payload; a view step carries none.

Reproduced on 0.92.0 with a minimal model: `example.order.Order` (`title: String`,
`placed_at: Timestamp`, `placed_at` read from a minted payload field), a view
`example.order.Orders` over it with `order_id`, `title`, `placed_at`.

```console
$ ess verify conform synthesize --path rows --out suite.json
1 scenario(s) (0 authored), 0 refusal(s), written to suite.json
```

The `expect_event` step carries `"shape": {"order_id": {...string}, "placed_at": {...timestamp}}`;
the first `eventually_view` step carries only `"expectation": {"expect": "contains", "fields":
{"order_id": …, "title": …}}` and no shape, so nothing checks `placed_at` on that row. An authored
`contains: {title: a}` compiles to a view step that names `title` alone.

The first half of the report, the `ESS-AUTHOR-042` hint claiming a presence and type check on a
view row, was fixed in 0.69.0 (`CHANGELOG.md:512`): the hint now names `satisfies: defined(<field>)`
and `{$observed: …}` for a view row. This story is the second half, and the vocabulary, format and
Rust runner part of it.

## Acceptance

- `docs/design/view-row-shape.md` exists and states: which view steps carry a shape
  (`expect_view`, `eventually_view`), that the shape is the view's declared fields flattened as
  `PayloadShape` flattens an event, that it is checked on every row the read returns, that a
  declared `Optional` leaf admits absent and `null`, that an undeclared row key is not refused, and
  the format consequence (ordinary `ess-conformance/64`, coverage `/65`, each implying every major
  below).
- `ScenarioStep::ExpectView` and `ScenarioStep::EventuallyView` carry `shape: PayloadShape`,
  serialized only when not empty; a suite carrying it below suite/64 (/65 coverage) is refused at
  admission, naming the major it requires; `ess-conformance/64` and `/65` are in
  `SUPPORTED_SUITE_FORMATS`.
- The Rust runner, given a hand-written suite/64 whose `expect_view` step carries the shape of
  `example.order.Orders`, fails the step with a diagnostic naming the row and the field when the
  target returns a row without `placed_at`, and when it returns `placed_at: 7`; it passes when every
  row carries `order_id`, `title` and a timestamp `placed_at`.
- A suite with no view shape serializes byte for byte as before (a committed suite under
  `suites/generated/` round-trips unchanged).

## Scope

- cited: crates/verify/ess-conformance/src/scenario.rs
- cited: crates/verify/ess-conformance/src/admission.rs
- cited: crates/verify/ess-conformance/src/runner.rs
- inferred: crates/verify/ess-conformance/src/view_row_shape.rs
- inferred: crates/verify/ess-conformance/src/lib.rs
- inferred: crates/verify/ess-conformance/tests/view_row_shape.rs
- inferred: docs/design/view-row-shape.md
- inferred: website/docs/reference/formats.md
- inferred: CHANGELOG.md
confidence: medium

Notes from the earlier scoping (2026-11-02, still true at 995e29579d): `ExpectView` at
`scenario.rs:2754` and `EventuallyView` at `:2782` carry no shape; event steps carry one with
`skip_serializing_if = "PayloadShape::is_empty"` (`scenario.rs:2773-2774`); `admission.rs:656-657`
list the view step keys; `SUPPORTED_SUITE_FORMATS` ends at 63 (`scenario.rs:558-562`), the newest
pair being `member_paths` 62/63 (`member_paths.rs:40-44`). About 100 struct-literal sites of the two
variants across the test files need `shape: PayloadShape::new()` (mechanical).
`story:view-and-error-assertions-carry-their-value-shape` wants the same field on the same two
variants: it builds on this one and joins the same format pair.

## Fit review

1. **Need.** A view declares the fields each of its rows carries; a suite should fail a target whose
   rows do not carry them. Today no view read checks it (reproduction above; `scenario.rs:2754`,
   `:2782` have no shape; `runner.rs:3529` `row_matches` compares only the wanted keys). The
   requester's proposal, theirs: "a view read checks every returned row against the view's declared
   fields … a new suite format member, selected the way earlier additive suite formats were".
2. **Class.** Gap in what a suite checks, bordering on a defect: the event payload check
   (`scenario.rs:1990-2002`, "only what the specification declares is required") is documented
   and the same declaration on a view is not held. The only per-row schema check is
   `SnapshotCompleteSubject` (`subject.rs:98-115`, `admit_row`), emitted only in preservation
   scenarios.
3. **Already expressible?** Partly. `satisfies: defined(<field>)` requires presence on every row
   and at least one row, says nothing of the type, and must be written per field by an author;
   synthesis writes none (`run.txt` § 2). `$observed` ties one value to an event. Neither is
   emitted by synthesis, so the declared figure goes unchecked in a synthesized suite: the idiom
   is no answer under `docs/design/generic-language-features.md` § Policy, policy 1.
4. **Fit.** No source-language cell: no authored key, nothing on grid R or grid O
   (`docs/design/language-grids.md`). It reuses the suite's existing vocabulary, `PayloadShape`
   and `LeafShape` (`scenario.rs:2004`, `:2032`), on the sibling read steps (the event steps carry
   it at `:2258`, `:2417`, `:2548`, `:2773`). Every runner handles it: Rust here, Go and TypeScript
   in `story:the-go-runtime-checks-a-view-rows-shape` and
   `story:the-typescript-runtime-checks-a-view-rows-shape`, synthesis and authoring in
   `story:synthesized-and-authored-view-reads-carry-the-rows-shape`; an older runner refuses the
   suite by major (`admission.rs`, the pattern of `member_paths.rs:19-24`).
5. **Second adopter.** A catalogue service whose view `example.catalog.Products` declares
   `price: Decimal`; its read model serializes `price` as a string after a refactor. Every
   scenario that names only `product_id` passes today.
6. **Cost.** One suite format pair, `ess-conformance/64` and `/65`, no source format, no new
   diagnostic code (the step fails as a mismatch). Synthesized suites with a view read move to /64
   when the synthesis story lands, so committed suites are regenerated in that story. No generated
   API changes.
7. **Alternatives.** (a) Change nothing and document `satisfies: defined(...)`: no type check, not
   synthesized, refused under policy 1. (b) Reuse `subject.rs` `admit_row` (full typed declaration
   tree) on every view step: stronger for nested values but a second shape vocabulary beside
   `PayloadShape` on read steps, and the Go and TypeScript runtimes would need the declaration
   tree. (c) Chosen: `PayloadShape` on view steps, the same flattening and the same `admits` rule
   as events, so the three runtimes reuse their existing `holds` check.

## Decisions

Accept, redesigned. The need is accepted as reported. Changed from the request: the row check is
the existing event `PayloadShape` carried on `expect_view` and `eventually_view` (no new shape
vocabulary), checked on every returned row; the hint half is already shipped (0.69.0); the work is
split into this story (design page, vocabulary, format pair, Rust runner), one story per runtime
and one for synthesis and authoring. The format pair is shared with
`story:view-and-error-assertions-carry-their-value-shape` (one decision, one pair).

## Source

github:beyond10x/ess#680
