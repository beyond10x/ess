---
format: aep.planning-md/3
id: story:a-view-route-carries-its-paging-facts-in-headers
kind: story
status: draft
title: A view route carries its paging facts in headers, and OpenAPI declares them
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#750
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-view-answer-takes-its-protocols-list-shape
revision: 2
---
## Need

A paged read answers its rows as a bare array and its continuation in a response header
(`X-Next-Cursor`, absent on the last page). An `ess-http/1` view route declares no header, and a
paged view cannot take `body: array`, because the answer must carry the page facts somewhere.

Minimal reproduction (0.92.0): the paged view of `story:a-view-answer-takes-its-protocols-list-shape`
served by a `reached_by: network` component, and this binding:

```yaml
routes:
  - view: example.log.Recent
    method: get
    path: /entries
    body: array
    headers:
      X-Next-Cursor: {source: page.next}
```

```text
error: reading view-header.http.yaml as HTTP binding YAML: routes[0]: unknown field `headers`, expected one of `command`, `view`, `operation_id`, `method`, `path`, `parameters`, `request`, `outcomes`, `errors`, `alike`, `body`, `empty`, `refusals`
```

Without `headers:`, `body: array` on a paged view is `ESS-HTTP-024` ("view `…` is paged: its
answer carries a page, which … cannot", `crates/specify/ess-http/src/lib.rs:6692-6702`).

## Acceptance

- `ess specify http validate` exits 0 on the binding above once the view declares
  `paging: {cursor: before, size: limit}`. A view route takes `headers:` with the command route's
  value shapes: `{literal: <text>}` and `{source: page.next}` / `{source: page.total}`; a challenge
  on a view route, `page.next` on a view without `cursor:`, `page.total` without `total: true`, and
  every other source are refused as `ESS-HTTP-026` naming the header and the source, with the
  existing token, owned-header and duplicate rules.
- `body: array` on a paged view is admitted when every paging fact the view answers (`next` under
  `cursor:`, `total` under `total: true`) is carried by a header, and stays `ESS-HTTP-024`
  otherwise, the message naming the fact no header carries.
- `ess specify http compile` writes the view route's headers ordered by name, omitted when empty;
  a binding without view headers keeps its `ess-http-ir/1` bytes.
- `ess generate --kind openapi` (with and without the binding) declares the `cursor:` and `size`
  parameters as optional query parameters under their wire names, the size with the bounds its
  newtype declares; without a binding the `rows` response gains an optional `next` string member;
  with the binding above the response is a bare array and declares the `X-Next-Cursor` header,
  optional, with the description "absent on the last page".

## Scope

- cited: crates/specify/ess-http/src/lib.rs
- cited: docs/design/http-presentation-binding.md
- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/src/docs.rs
- inferred: crates/specify/ess-http/tests/view_headers.rs
- inferred: crates/generate/ess-gen/tests/openapi_http.rs
- inferred: website/docs/reference/formats.md
confidence: medium

## Fit review

1. **Need.** A paged answer's continuation travels in a response header, and the body is the bare
   row array. Requester's syntax (theirs): "declared response headers on a view". Reproduction
   above, `run.txt` step 4.
2. **Class.** Gap: the view route keys have no header
   (`docs/design/http-presentation-binding.md`, key table: `headers` exists only under
   `routes[].outcomes.<outcome>` and `refusals.<key>`).
3. **Already expressible?** No. The requester declares the header by hand in a served OpenAPI
   document, so no suite reads it.
4. **Fit.** It reuses the command route's `headers:` key and value shapes
   (`http-presentation-binding.md`, "Response headers", `{literal}` / `{source}`), adding the one
   source family a view answer has (`page.<fact>`), so a view and a command spell a header the same
   way. It composes with `body: array` and `body: rows` (the facts not in a header stay in the
   envelope). Targets: generated Rust and Go servers refuse every paged view by name
   (`crates/generate/ess-synth/src/paging.rs:18-38`), so they never write a header they cannot
   compute; OpenAPI declares it; a suite reads the facts semantically, never by header name.
5. **Second adopter.** A list API that answers `X-Total-Count` beside a bare array (`total: true`,
   `{source: page.total}`), common in REST list endpoints.
6. **Cost.** One key on a view route in `ess-http/1`, additive, as `body: array` and
   `empty.body` were in 0.77.0 (`http-presentation-binding.md`, key table, unit column); no new
   diagnostic code. No source format.
7. **Alternatives.** (a) Change nothing: the header stays hand-written. (b) A view-only key,
   `next: {header: X-Next-Cursor}`: refused, a second spelling of a header. (c) `headers:` on a
   view route with `page.<fact>` sources: chosen.

## Decisions

accept, redesigned. The request's "declared response headers on a view" lands as the command
route's existing `headers:` key on a view route, with page facts as its sources; the header is the
binding's, not the model's.

## Source

github:beyond10x/ess#750
