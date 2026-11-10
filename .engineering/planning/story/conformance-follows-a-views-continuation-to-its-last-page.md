---
format: aep.planning-md/3
id: story:conformance-follows-a-views-continuation-to-its-last-page
kind: story
status: active
title: Conformance follows a view's continuation to its last page
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#750
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-view-answer-takes-its-protocols-list-shape
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner/page.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/paging.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/view_paging.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/view_paging.rs
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:43:23Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:43:24Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A view paged by continuation (`story:a-view-answer-takes-its-protocols-list-shape`) needs scenarios
that hold its paging: the first page, the next page read through the answered continuation, and a
last page that answers none. Today's page scenarios send `page` and `size` numbers
(`docs/design/view-paging.md`, "What the page reads claim"; `run.txt` step 5 shows the reads
`{limit: 1, page: 0}`, `{limit: 1, page: 1}`, ...), and `SemanticViewResult` carries rows and a
total only (`crates/verify/ess-conformance/src/target.rs:1056-1068`), so a continuation has nowhere
to travel.

## Acceptance

- `SemanticViewResult` gains `next: Option<String>`, omitted when `None`.
- `ess verify conform synthesize` on the reproduction model of the parent story (with
  `cursor: before, size: limit`) writes, after the ranked expectation, reads that: send `limit: 1`
  and expect one row and a continuation; send `before: <that continuation>, limit: 1` and expect one
  row ranked after the first and not equal to it; read on with `limit` equal to the rows arranged
  and expect no continuation once every arranged row was read; and send no `limit` on a view with
  `default_size: n` and expect at most `n` rows. The suite is written at a new
  `ess-conformance` ordinary/coverage pair; a model without a continuation-paged view keeps its
  suite bytes.
- `ess verify conform run --target interpreted` passes every scenario. The interpreted target
  mints a continuation per page and refuses one it did not mint.
- Each wrong implementation fails a named scenario: one that ignores the continuation (first page
  again), one that always answers a continuation, one that never answers one, one that ignores
  `default_size`. `crates/verify/ess-conformance/tests/view_paging.rs` gains one in-memory target per
  defect.
- Views read whole (`paging::read_whole`) follow continuations to the last page instead of
  assuming an unpaged read answers every row, where the view declares `default_size:`.

## Scope

- cited: crates/verify/ess-conformance/src/target.rs
- cited: crates/verify/ess-conformance/src/synthesize/paging.rs
- cited: crates/verify/ess-conformance/src/runner/page.rs
- cited: crates/verify/ess-conformance/src/view_paging.rs
- cited: crates/verify/ess-conformance/src/interpret/views.rs
- inferred: crates/verify/ess-conformance/src/scenario.rs
- cited: crates/verify/ess-conformance/tests/view_paging.rs
- inferred: website/docs/reference/formats.md
confidence: medium

## Fit review

1. **Need.** The suite checks continuation paging: first page, next page through the continuation,
   last page without one. Requester's syntax (theirs): "conformance synthesis for the above (first
   page, next page through the header, last page without the header ...)". The header is the
   binding's placement of `next` (`story:a-view-route-carries-its-paging-facts-in-headers`); the
   suite reads it semantically.
2. **Class.** Gap: no suite vocabulary carries a continuation (`target.rs:1056-1068`).
3. **Already expressible?** No. The requester's workaround leaves paging with no scenario.
4. **Fit.** It extends the existing `page` expectation and `follows` continuation check
   (`view-paging.md`, "The suite vocabulary") rather than adding a second paging family; the claims
   stay floors that rows another user made cannot break, as the page-number claims are
   (`view-paging.md`, claim table). Generated Go and TypeScript runtimes:
   `story:the-go-and-typescript-runtimes-follow-continuations-and-check-malformed-answers`.
5. **Second adopter.** The protocol list operation of the parent story (`pageToken` /
   `nextPageToken`).
6. **Cost.** A new suite format pair (after `ess-conformance/63`), `SemanticViewResult.next`
   (additive for a Rust test adapter: a target that answers no continuation fails the scenarios of
   a continuation-paged view only). No source format beyond the parent's.
7. **Alternatives.** (a) Authored scenarios only: leaves every adopter to write the same three
   reads. (b) A new expectation family for continuations: refused, the `page` expectation already
   has `follows`. (c) Extend `page` with a continuation and a last-page claim: chosen.

## Decisions

accept, redesigned, as the parent story: the suite reads the continuation as a semantic answer
fact, so the same scenarios hold a header placement and a body placement.

## Source

github:beyond10x/ess#750
