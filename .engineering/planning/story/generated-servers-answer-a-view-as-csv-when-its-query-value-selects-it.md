---
format: aep.planning-md/3
id: story:generated-servers-answer-a-view-as-csv-when-its-query-value-selects-it
kind: story
status: draft
title: Generated servers answer a view as CSV when its query value selects it
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#753
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-view-route-answers-a-csv-representation-its-query-value-selects
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding/answer_forms.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/go.rs
revision: 4
---
## Need

The Rust and Go servers `ess generate synthesize --http` writes must answer a view route's
`representations:` (`story:a-view-route-answers-a-csv-representation-its-query-value-selects`).
Today a bound view route answers JSON only (`crates/generate/ess-synth/src/rust/http/bound.rs:1224-1295`),
and a view without parameters never reads the query string (`bound.rs:1232-1236`, `_request`).

Neutral example: view `demo.reports.Rows(id, label)` on `get /reports/rows`, `body: array`, one
representation `query: {format: csv}`, `encoding: {csv: {separator: ";", header: true}}`, header
`Content-Disposition: {literal: attachment}`, holding rows `(r-1, north)` and `(r-2, "a;b")`.

## Acceptance

- `GET /reports/rows` answers `200`, `application/json`, `[{"id":"r-1","label":"north"},{"id":"r-2","label":"a;b"}]` as today.
- `GET /reports/rows?format=csv` answers `200`, `Content-Type: text/csv; charset=utf-8`,
  `Content-Disposition: attachment`, body `id;label\r\nr-1;north\r\nr-2;"a;b"\r\n`; with
  `header: false` the first line is absent; an absent `Optional` value is an empty field; a value
  holding `"`, CR or LF is quoted with `"` doubled.
- `GET /reports/rows?format=xml` answers `malformed` (`400`, or the binding's `refusals.malformed`);
  `format` given twice is `malformed`.
- The view's own query parameters keep their reading beside the selector, and the CSV rows are the
  rows the JSON answer gives for the same request, in the same order.
- The generated Go server answers each request above with the Rust server's status, headers and
  body bytes (`the_go_and_rust_servers_answer_one_binding_alike` gains the cases); a binding without
  `representations` generates byte-identical server source.

## Scope

- cited: crates/generate/ess-synth/src/rust/http/bound.rs
- cited: crates/generate/ess-synth/src/go/http/bound.rs
- inferred: crates/generate/ess-synth/src/rust/http.rs
- inferred: crates/generate/ess-synth/src/go/http.rs
- cited: crates/generate/ess-synth/tests/http_binding/go.rs
- inferred: crates/generate/ess-synth/tests/http_binding/answer_forms.rs
confidence: medium

## Fit review

1. **Need.** A server generated from the binding must serve the representation it states.
   Reproduction: the binding story's; the code that answers JSON only is cited above.
2. **Class: gap**, the server half of beyond10x/ess#753.
3. **Expressible today? No.**
4. **Fit.** The CSV values reuse the flat text spelling the form and query decoders read
   (`rust/http/bound.rs:275-345`), so one value has one text form in both directions; Rust and Go
   answer byte for byte as k6 requires (`docs/design/http-presentation-binding.md` § "What the
   servers write (k5, k6)").
5. **Second adopter.** The ledger export of the binding story.
6. **Cost.** Server source changes only for routes with the key.
7. **Alternatives.** (a) No server support: a target silently ignoring a construct (red flag).
   (b) A CSV library dependency in generated code: the generated servers carry their own JSON
   reader and writer; a dozen lines of RFC 4180 writing do not justify one.

## Decisions

**Accept, redesigned**, as decided in
`story:a-view-route-answers-a-csv-representation-its-query-value-selects`.

## Source

github:beyond10x/ess#753
