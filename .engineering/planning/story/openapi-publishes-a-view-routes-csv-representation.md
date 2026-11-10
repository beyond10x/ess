---
format: aep.planning-md/3
id: story:openapi-publishes-a-view-routes-csv-representation
kind: story
status: active
title: OpenAPI publishes a view route's CSV representation
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#753
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-view-route-answers-a-csv-representation-its-query-value-selects
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/fixtures/openapi-http/
- confidence: inferred
  path: crates/generate/ess-gen/tests/openapi_http_presentation.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:26Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:27Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A view route with `representations:`
(`story:a-view-route-answers-a-csv-representation-its-query-value-selects`) must be published in
the OpenAPI document `ess generate --kind openapi --http` writes, so a client generated from it
sends the selector and reads `text/csv`. Today a bound view operation answers `application/json`
only (`crates/generate/ess-gen/src/openapi.rs:1449` and the view responses after it), and the
document has no query parameter that is not a view parameter.

Neutral example: view `demo.reports.Rows(id, label)` on `get /reports/rows`, `body: array`, one
representation `query: {format: csv}`, `encoding: {csv: {separator: ";", header: true}}`, header
`Content-Disposition: {literal: attachment}`.

## Acceptance

- The operation `get /reports/rows` has one `in: query` parameter `format`, `required: false`,
  `schema: {type: string, enum: [csv]}`, its description naming what each value selects and that
  its absence selects the JSON answer.
- Its `200` response holds two media types: `application/json` with the array of the row schema, as
  today, and `text/csv` with `{type: string}`; the response carries `x-ess-representations`, one
  entry per representation in the order written, each with the selector, the media type, the
  separator, `header`, and the columns as wire names in field order. A header a representation
  declares is a response `headers` entry whose description names the selector value it applies to.
- A binding without `representations` keeps its OpenAPI bytes (`crates/generate/ess-gen/tests/openapi_http_presentation.rs`
  and `openapi_http.rs` pass unmodified); a new test pins the document above.

## Scope

- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/tests/openapi_http_presentation.rs
- inferred: crates/generate/ess-gen/tests/fixtures/openapi-http/
confidence: medium

## Fit review

1. **Need.** The published contract must show the second representation and its selector; without
   it a generated client cannot ask for it. Reproduction: the binding story's (the key is refused
   today, so no document can show it). Requester's proposal (theirs): "one path item whose response
   holds both media types, with `x-ess-operations` naming which query value selects which".
2. **Class: gap**, the projection half of beyond10x/ess#753.
3. **Expressible today? No**, the binding key does not exist.
4. **Fit.** One OpenAPI operation per method and path (OpenAPI 3.1 Path Item), several media types
   under one response: the standard place. The extension follows the `x-ess-*` naming of
   `x-ess-outcomes` and `x-ess-route-source` (`openapi.rs:219-231`). `x-ess-operations` is not
   used because there is one operation.
5. **Second adopter.** The ledger export of the binding story.
6. **Cost.** OpenAPI bytes move only for bindings that use the key. No format.
7. **Alternatives.** (a) Two operations under one path: not valid OpenAPI. (b) A `oneOf` under one
   media type: hides that the answer's media type changes. Chosen: two media types, one extension.

## Decisions

**Accept, redesigned**, as decided in
`story:a-view-route-answers-a-csv-representation-its-query-value-selects`; the extension is named
`x-ess-representations` instead of the requester's `x-ess-operations`.

## Source

github:beyond10x/ess#753
