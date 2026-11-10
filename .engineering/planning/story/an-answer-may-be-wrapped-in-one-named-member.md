---
format: aep.planning-md/3
id: story:an-answer-may-be-wrapped-in-one-named-member
kind: story
status: active
title: An answer may be wrapped in one named member
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#755
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/bare_field_and_errors.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/route_refusals_and_view_answers.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: cited
  path: website/docs/guides/http-routes.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:38:41Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:38:42Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Many services answer a read with the row, or the rows, under one named member:
`GET /apps/{uuid} -> {"app": {"uuid": "...", "name": "..."}}`, `GET /apps -> {"items": [...]}`.
`ess-http/1` answers a view's row bare (`single`), its rows bare (`array`) or as ESS's own
`{"rows": [...]}` (`rows`), and a command's response bare (`response`, `response.<field>`) or in the
envelope; it has no form for a value under a member of the author's choosing. A body member holds an
error's wire name or a literal only (`crates/specify/ess-http/src/lib.rs:500`).

Minimal reproduction (view `demo.apps.AppById(uuid, name)` selected by identity; view
`demo.apps.Apps`; ess 0.92.0):

```console
$ ess specify http validate --path spec --binding single-wrapped.http.yaml   # body: {app: single}
error: reading single-wrapped.http.yaml as HTTP binding YAML: routes[0].body.app: `single` is not what a body member holds: `error`, or `{literal: <text>}` at line 4 column 73
$ ess specify http validate --path spec --binding array-wrapped.http.yaml    # body: {items: array}
error: reading array-wrapped.http.yaml as HTTP binding YAML: routes[0].body.items: `array` is not what a body member holds: `error`, or `{literal: <text>}` at line 4 column 68
```

Design: `{<member>: <form>}` holds the value `<form>` answers bare, under `<member>`, for every bare
value form:

| body | for | wire |
|---|---|---|
| `{<member>: single}` | a view `single` admits | `{"<member>": <the row>}`; `empty` as for `single` |
| `{<member>: array}` | a view `array` admits | `{"<member>": [<rows>]}` |
| `{<member>: response}` | an outcome `response` admits | `{"<member>": <the response object>}` |
| `{<member>: response.<field>}` | an outcome `response.<field>` admits | `{"<member>": <the field's value>}` |

`{<member>: error}` keeps its meaning: the member holds the error's value, its wire name.
`{rows: array}` on a view that is not paged is `rows`, ESS's own answer, and compiles to the same IR.

## Acceptance

- `ess specify http validate` accepts `body: {app: single}` with `empty: {status: 404}` and
  `body: {items: array}` on the views above, and `body: {data: response}` and
  `body: {secret: response.secret}` on a `returns: true` outcome (exit 0); `ess specify http compile`
  writes the member and the form; `body: {rows: array}` compiles to the bytes `body: rows` does.
- Each wrapped form is refused wherever its bare form is (`ESS-HTTP-024`, `-030`, `-031`: a paged
  view, `single` without identity selection or without `empty`, `response` without `returns: true`
  or under `errors`), and an empty `<member>`, an object of two members, and `encoding: text`
  beside a wrapped form are refused with `ESS-HTTP-024`; a refinement changing it is `ESS-HTTP-012`.
- A binding without a wrapped form compiles to the bytes it compiled to before.
- `docs/design/http-presentation-binding.md` § "Response body per outcome" gains the four rows and
  the keys table their values; `website/docs/guides/http-routes.md` shows one.

## Scope

- cited: crates/specify/ess-http/src/lib.rs
- inferred: crates/specify/ess-http/tests/route_refusals_and_view_answers.rs
- inferred: crates/specify/ess-http/tests/bare_field_and_errors.rs
- cited: docs/design/http-presentation-binding.md
- cited: website/docs/guides/http-routes.md
confidence: medium

## Fit review

1. **Need.** A read or a command answers its value under one named member, and the specification
   cannot state it. Reproduction above. The requester's proposal (theirs): `body: {<member>:
   single}`, `{<member>: rows}`, `{<member>: array}` on views, an object schema with one required
   member in OpenAPI, conformance unwrapping the member; and the claim that, with the literal
   members a body already takes, it also covers `{"success": true, "app": {...}}`.
2. **Class: gap.** The body forms are a closed, documented set (`docs/design/http-presentation-binding.md`
   § "Response body per outcome"; `lib.rs:613-621`, `ViewBody`), not a defect.
3. **Expressible today? No.** `rows` writes the member `rows` only; a struct-typed view field
   cannot wrap a row (the view's fields are the row). No idiom.
4. **Fit.** The language grids do not apply. Vocabulary: the `{<member>: …}` shape already exists
   for errors; giving it "the member holds what the form answers" for every bare form keeps one
   reading (`{<member>: error}`'s member holds the error's wire name). Siblings: the requester asked
   for views only; command outcomes answer `response` and `response.<field>` bare by the same rule
   and gain the same wrapper, so one construct does not leave its sibling unable. `{<member>:
   rows}` was not taken: `rows` is already an object, so wrapping it nests `{"items": {"rows":
   …}}`, which is not what was asked; `{<member>: array}` is the list wrapper. Targets: OpenAPI and
   the Rust and Go servers in the dependent stories; UI refuses `--http`; no HTTP conformance target
   exists yet (§ "Stage 3"), so "conformance unwraps the member" lands with it.
5. **Second adopter.** A payments API answers `POST /refunds` with `{"data": {"refund_id": …,
   "amount": …}}` and `GET /refunds` with `{"data": [...]}`, the convention of its whole surface.
6. **Cost.** Additive values of existing keys inside `ess-http/1` / `ess-http-ir/1`, omitted when
   absent; no format version; no new code (`ESS-HTTP-024` widens).
7. **Alternatives.** (a) Change nothing. (b) Views only, as requested: leaves the command sibling
   unable to say the same (assessing skill, red flags). (c) A general object body with several
   members, literals of every type and one wrapped value: covers `{"success": true, "app": …}`, but
   no request needs it alone, and it is an addition later. Chosen: one member, every bare form.

## Decisions

**Accept, redesigned.** Kept: `{<member>: single}` and `{<member>: array}` on views, the one-member
object schema. Changed: `{<member>: rows}` is not admitted (it would nest `rows`); command outcomes
gain `{<member>: response}` and `{<member>: response.<field>}`. Not covered: fixed side members
beside the wrapped value (`{"success": true, "app": {...}}`); a body is one form, the existing
literal member is a refusal's whole body, and the requester's claim that it combines does not hold.
Such a body stays refused until a request needs it on its own.

This is also, for a view that is not paged, the "renaming the member that carries the rows" that
`story:a-view-answer-takes-its-protocols-list-shape` split out for a story of its own (beyond10x/ess#750
triage). A paged view keeps refusing a wrapped form until its page facts have a place beside the
rows; `story:a-view-route-carries-its-paging-facts-in-headers` gives them headers.

## Source

github:beyond10x/ess#755
