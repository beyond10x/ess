---
format: aep.planning-md/3
id: story:generated-servers-read-chosen-inputs-from-the-query-beside-the-body
kind: story
status: active
title: Generated servers read chosen inputs from the query beside the body
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#754
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-command-route-reads-chosen-inputs-from-the-query-beside-its-body
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding/board_routes.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/go.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:44:40Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:44:41Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

The Rust and Go servers `ess generate synthesize --http` writes must read the fields a route lists
under `request.query` (`story:a-command-route-reads-chosen-inputs-from-the-query-beside-its-body`)
from the query string and the rest from the body. Today a `json` or `form` command route reads the
body alone (`crates/generate/ess-synth/src/rust/http/bound.rs:883-925`), and only `encoding: query`
reads `request.query` (`bound.rs:917-921`).

Neutral example: `demo.reports.ImportRows(format, dry_run: Boolean)` on `post /reports/rows/import`,
`request: {encoding: json, query: [input.dry_run]}`, `dry_run` with wire name `dryRun`.

## Acceptance

- `POST /reports/rows/import?dryRun=true` with body `{"format":"csv"}` runs the command with
  `dry_run = true`; without the key, an `Optional` field is absent and a required one is
  `malformed`.
- `dryRun` in the body is `malformed` under `unknown: refuse` (the query carries it), and a query
  key no listed field names is `malformed` under `refuse` and ignored under `ignore`; `dryRun`
  given twice is `malformed`.
- The generated Go server answers each request above with the Rust server's status, headers and
  body bytes (`the_go_and_rust_servers_answer_one_binding_alike` gains the cases); a binding without
  `request.query` generates byte-identical server source.

## Scope

- cited: crates/generate/ess-synth/src/rust/http/bound.rs
- cited: crates/generate/ess-synth/src/go/http/bound.rs
- cited: crates/generate/ess-synth/tests/http_binding/go.rs
- inferred: crates/generate/ess-synth/tests/http_binding/board_routes.rs
confidence: medium

## Fit review

1. **Need.** The servers must honour the placement the binding states. Reproduction: the binding
   story's; the body-only reading is cited above.
2. **Class: gap**, the server half of the query placement of beyond10x/ess#754.
3. **Expressible today? No**, the key does not exist.
4. **Fit.** Reuses `flat_object` over `request.query` (`bound.rs:288-345`) and merges its members
   with the body's, as `with_path` merges the path's (`bound.rs:388-410`); one reader, one rule for
   a field given in two places (malformed). Rust and Go alike, as k6 requires.
5. **Second adopter.** The document store's `if_revision` of the binding story.
6. **Cost.** Server source changes only for routes with the key.
7. **Alternatives.** (a) No server support: a target silently ignoring a key (red flag).

## Decisions

**Accept, redesigned**, as decided in
`story:a-command-route-reads-chosen-inputs-from-the-query-beside-its-body`.

## Source

github:beyond10x/ess#754
