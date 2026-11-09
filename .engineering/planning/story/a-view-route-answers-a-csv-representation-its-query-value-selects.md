---
format: aep.planning-md/3
id: story:a-view-route-answers-a-csv-representation-its-query-value-selects
kind: story
status: draft
title: A view route answers a CSV representation its query value selects
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#753
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/representations.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: cited
  path: website/docs/guides/http-routes.md
- confidence: inferred
  path: website/docs/reference/diagnostics.md
revision: 4
---
## Need

An existing service answers one list endpoint in two representations of the same rows, picked by a
query value:

```
GET /reports/rows               -> 200 application/json, the rows
GET /reports/rows?format=csv    -> 200 text/csv, the same rows: a header line, the fields in a fixed
                                   order, `;` as separator; Content-Disposition: attachment
```

`ess-http/1` binds one view to one route with one JSON answer (`rows`, `array` or `single`) and has
no non-JSON view answer and no way to pick an answer by a query value. Minimal reproduction (view
`demo.reports.Rows(id, label)` over entity `demo.reports.Row`, served by a `reached_by: network`
component; ess 0.92.0):

```console
$ ess specify http validate --path spec --binding two-views.http.yaml     # Rows and a copy RowsCsv on get /reports/rows
error[ESS-HTTP-010]: routes[1]: two operations claim the same method and path: view `demo.reports.RowsCsv` (`routes[1]`, `get /reports/rows`) and view `demo.reports.Rows` (`routes[0]`, `get /reports/rows`) are one route on component `reports`, since variable names do not tell two templates apart
$ ess specify http validate --path spec --binding view-text.http.yaml     # body: text on a view
error[ESS-HTTP-024]: routes[0].body: view `demo.reports.RowsCsv` answers `rows`, `array` or `single`, and `text` is not a view's body
$ ess specify http validate --path spec --binding view-query.http.yaml    # query: {format: csv}
error: reading view-query.http.yaml as HTTP binding YAML: routes[0]: unknown field `query`, expected one of `command`, `view`, `operation_id`, `method`, `path`, `parameters`, `request`, `outcomes`, `errors`, `alike`, `body`, `empty`, `refusals` at line 4 column 68
$ ess specify http validate --path spec --binding one-view-twice.http.yaml
error[ESS-HTTP-005]: routes[1]: view `demo.reports.Rows` is already routed by `routes[0]`; an operation has one route
```

The design: the CSV is a second **representation** of one view's answer on one route, not a second
view. A view route gains `representations:`, each selected by one literal value of one query
parameter and answering the rows as delimited text:

```yaml
routes:
  - view: demo.reports.Rows
    method: get
    path: /reports/rows
    body: array
    representations:
      - query: {format: csv}
        encoding: {csv: {separator: ";", header: true}}
        headers:
          Content-Disposition: {literal: attachment}
```

- Without the selector parameter the route answers its `body` as today. With it, the value names
  one representation; any other value is `malformed`. Every representation of a route uses the
  same parameter name, and the values are distinct.
- `encoding: {csv: {separator, header}}`: `separator` is one of `","`, `";"` and `"\t"` (default
  `","`), `header` a Boolean (default `true`). One line per row in the order the JSON answer gives,
  CRLF line ends, columns in the view's field order under their wire names, each value in the flat
  text spelling `form` and `query` already use, an absent `Optional` as an empty field, and RFC 4180
  quoting (a value holding the separator, `"`, CR or LF is quoted, `"` doubled).
  `Content-Type: text/csv; charset=utf-8`.
- `headers:` takes `{literal: <text>}` values only, under the checks `refusals.<key>.headers`
  already has (`ESS-HTTP-026`). It is the same key a view route gains in
  `story:a-view-route-carries-its-paging-facts-in-headers` (beyond10x/ess#750); whichever lands
  second reuses the other's parsing, and a representation's headers are added to the route's.

## Acceptance

- `ess specify http validate` accepts the route above (exit 0) and `ess specify http compile`
  writes `representations` into `ess-http-ir/1` in the order written, each default omitted; a
  binding without `representations` compiles to the bytes it compiled to before.
- Refused with one new code (the next free `ESS-HTTP-0NN`), at `routes[i].representations[j]` and
  the key: `representations` on a command route, on a `single` view, or on a paged view; an empty
  list; a `query` holding other than exactly one parameter; two representations on different
  parameter names or on one value; a parameter name that is a view parameter's wire name or a path
  variable, or that is not RFC 3986 `unreserved` characters; a value that is empty or not
  `unreserved` characters; a `separator` outside the set; a view field without a flat spelling
  (struct, `List`, `Map`, `Json`), naming the field.
- `headers` under a representation is refused with `ESS-HTTP-026` for a `source` or `challenge`
  value and for every name that code already refuses.
- A refinement that adds, drops or changes a refined route's `representations` is `ESS-HTTP-012`.
- `docs/design/http-presentation-binding.md` gains the section "Representations of a view's
  answer", and its "Still refused" names CSV beside `text`; the requester's two-route form is
  recorded as refused with the reason below. `website/docs/guides/http-routes.md` shows the route.

## Scope

- cited: crates/specify/ess-http/src/lib.rs
- inferred: crates/specify/ess-http/tests/representations.rs
- cited: docs/design/http-presentation-binding.md
- cited: website/docs/guides/http-routes.md
- inferred: website/docs/reference/diagnostics.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** One read answers the same rows as JSON or as delimited text, the caller choosing by a
   query value; the specification cannot state the second representation at all. Reproduction
   above. The requester's proposal (theirs): (1) two operations may share a method and path when
   each names a distinct literal of one query parameter, `query: {format: csv}`; (2) a view body
   `{csv: {separator: ";", header: true}}`, with a second view `RowsCsv` beside `Rows`.
2. **Class: gap.** `ess-http/1` refuses non-JSON bodies except a `String` field as `text`
   (`docs/design/http-presentation-binding.md` § "Still refused"); the refusals are documented, not
   defects (`crates/specify/ess-http/src/lib.rs:3096`, `ESS-HTTP-024`).
3. **Expressible today? No.** Two views on one path: `ESS-HTTP-010`; one view on two routes:
   `ESS-HTTP-005`; a separate path the service does not serve misstates the protocol. A second view
   would duplicate the model to carry a wire choice. No idiom.
4. **Fit.** The language grids do not apply (no expression position). The binding's axes:
   - Vocabulary: `encoding` is the word an outcome answer already uses for its body's media type
     (`json`, `text`, design page § "Response body per outcome"); `query: {format: csv}` keeps the
     requester's selector spelling; `headers` with literal values is the shape `refusals` headers
     already have.
   - One route, one operation: OpenAPI's Path Item holds one Operation per method, so two
     operations on one method and path cannot be published as two operations; one operation with
     two response media types can. `ESS-HTTP-005` and `-010` keep their meaning.
   - Siblings: command routes refuse `representations` by name in this cut (an outcome answer has
     its own `encoding`, and no request asks for a command answer as CSV); selection by `Accept` is
     the natural second selector and stays an addition for later, refused today as an unknown key;
     `single` and paged views are refused because a CSV of one row or of a page has no defined shape.
   - Targets: OpenAPI and the Rust and Go servers in the dependent stories; UI targets refuse
     `--http` by name; the interpreter and emitted runners speak no HTTP; no HTTP conformance target
     exists yet (design page § "Stage 3"), so the CSV-against-JSON comparison the requester proposes
     lands with stage 3.
5. **Second adopter.** A ledger service answers `GET /ledgers/{ledger_id}/entries` as JSON for its
   UI and, with `?export=csv`, as a comma-separated download for a spreadsheet, the same filters and
   rows.
6. **Cost.** Additive optional keys inside `ess-http/1` and `ess-http-ir/1`, omitted when absent,
   as `operation_id` and `encoding: text` were (`story:an-http-binding-names-operation-ids-and-servers`);
   no format version, no source format. One new diagnostic code. An older reader refuses the key by
   name.
7. **Alternatives.** (a) Change nothing: the endpoint stays unbound. (b) The requester's two
   operations on one path plus a second view: duplicates the view in the model, cannot be
   published as two OpenAPI operations, and needs `ESS-HTTP-005`/`-010` exceptions. (c) Content
   negotiation by `Accept` only: the service selects by a query value, so it cannot be bound.
   Chosen: one route, a list of representations, the selector a query literal.

## Decisions

**Accept, redesigned.** Kept: the selector spelling `query: {format: csv}` and the CSV options
`separator` and `header`. Changed: no second view and no second operation on the path; the CSV is a
representation of the one view's route, so `ESS-HTTP-005` and `-010` stay as they are. Requester's
item 1 as a general rule for two operations is refused: OpenAPI cannot publish it, and the case it
was asked for is covered here. A `Content-Disposition` filename holding a timestamp cannot be
stated, because a header is a literal or a source and no source reads the clock; the binding states
`attachment` or leaves the header out.

## Source

github:beyond10x/ess#753
