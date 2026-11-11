---
format: aep.planning-md/3
id: story:a-generated-ui-reads-the-answers-its-http-binding-declares
kind: story
status: active
title: A generated UI reads the answers its HTTP binding declares, and renews its credential on a 401
tags:
- defect
- severity-medium
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-answer-may-be-wrapped-in-one-named-member
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/generate_ui_http.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/http.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/tests/binding.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/project/README.md.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/answer.ts.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/bound_answers.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/http.rs
- confidence: cited
  path: crates/ui/ess-ui/src/binding.rs
- confidence: inferred
  path: crates/ui/ess-ui/tests/answers.rs
- confidence: cited
  path: crates/ui/ess-ui/tests/vectors/answers.json
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:38:07Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-10T00:38:08Z", actor: "human:timo", revision: 5}
---
## Outcome

A user of a UI generated with `--http` sees a command the server accepted as accepted, a read's rows
however the binding answers them, and a refusal as the binding declares it; and a host whose access
token expires renews it on a 401 without the user seeing a failure.

## Evidence

Reproduced on ess 0.92.0 with the items model of `crates/edge/ess-cli/tests/fixtures/ui-http/items`
and four bindings over it (`i723-gap/b16/run.sh`, output in `b16/run.txt`).

- `endpoint:` is traceability, not a route: `website/docs/reference/ess-ui.md:2805-2809`
  ("`endpoint` ... the HTTP call of a retrofitted app"), `schemas/ui/ess-ui.schema.yaml:661`,
  `:773` (`note: traceability`). A document whose read and row action carry
  `endpoint: "GET /api/v2/catalog/items"` and `"PATCH /api/v2/catalog/items/{id}"` generates a
  `binding.ts` and `runtime/data.ts` byte-identical to the one without them; no `api/v2` appears in
  the generated `src/`.
- Paths, methods, path parameters, query keys and a bare request body already come from the
  `ess-http/1` binding (`--http`, beyond10x/ess#713, `story:a-generated-ui-calls-the-routes-its-http-binding-serves`):
  `binding.ts` carries `"path": "/items/{item_id}"`, `"method": "PUT"`, `"path_params"`.
- The answer form of a route does not reach the UI. A binding whose view answers `body: array` and
  whose `RenameItem` outcome answers `{status: 204, body: none}` validates (`3 route(s), 0 open,
  valid`) and generates a `binding.ts` byte-identical to the default binding's. The route tables
  have no member for it (`crates/ui/ess-ui/src/binding.rs:143-160` `ViewRoute`, `:181-215`
  `CommandRoute`; `crates/ui/ess-ui-check/src/model.rs:1801-1851` `unbound` checks method, path
  and request encoding only).
- Both runtimes guess the answer from its shape. React reads `{rows}`, else a bare array, else one
  row (`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:453-465`); the terminal does the same
  (`crates/ui/ess-ui-tui/src/http.rs:415-437`). A command's answer is read only as the default
  envelope (`templates/runtime/answer.ts.tmpl:30-60`, `crates/ui/ess-ui/src/binding.rs:274-297`):
  the generated `classify` run under node answers
  `204, body none: {"answer":"transport"}`, so an accepted rename on that route is shown as
  "No answer from the server" (`data.ts.tmpl:361-362`, `:512-516`). The same holds for `response`,
  `response.<field>`, `{<member>: error}` and `encoding: text`, which
  `docs/design/http-presentation-binding.md:370-395` lists as declared answers. A `single` view
  answering its declared `empty: {status: 404}` throws `DataError` 404 (`data.ts.tmpl:457-459`),
  rendered as "Could not load." (`data.ts.tmpl:692`) instead of the section's empty state.
- A wrapped answer (`body: {items: array}`) cannot be declared yet:
  `routes[0].body.items: \`array\` is not what a body member holds`; that is
  `story:an-answer-may-be-wrapped-in-one-named-member` (beyond10x/ess#755).
- The credential is one opaque header the host sets: `setAuthorization`
  (`data.ts.tmpl:300-311`), `ESS_UI_AUTHORIZATION` for the terminal
  (`crates/ui/ess-ui-tui/src/lib.rs:55`). Nothing in the generated runtime reacts to a 401 (no
  `401` in `src/runtime/`), and `classify(401, "")` answers `{"answer":"transport"}`.

## Acceptance

- `ess generate ui --target react|tui --http <binding>` writes into the route table each route's
  declared answer where it is not the default: a view's `body` (`array`, `single`) and its `empty`
  status; per command outcome its status and body form (`none`, `response`, `response.<field>`,
  `{<member>: error}`) and `encoding: text`. Each member is omitted at its default. Observed by
  `cargo test -p ess-cli --test generate_ui_http`: a new case reads the members from `binding.ts`
  and `binding.rs` over a bare binding, and
  `without_http_the_generated_bytes_are_unchanged` plus the default-binding digests stay green.
- The React runtime reads each declared form. A `204` with no body on an outcome declared
  `body: none` resolves as accepted. A bare `response` resolves with that object. A
  `{<member>: error}` body on its declared status rejects with that error's display text. A
  `single` view's declared empty status reads zero rows, so the section draws its `states.empty`.
  Observed by `cargo test -p ess-ui-react --test bound_answers`, which compiles the generated
  runtime with `tsc` and runs it under node against a scripted `fetch`, as `tests/bare_body.rs`
  does.
- Once `story:an-answer-may-be-wrapped-in-one-named-member` has merged, `{<member>: single}`,
  `{<member>: array}`, `{<member>: response}` and `{<member>: response.<field>}` are read from
  under the member in both runtimes. Observed by the same `bound_answers` test and a terminal case
  in `crates/ui/ess-ui-tui/src/http.rs` tests.
- Both generated UIs reach the Rust server synthesized from a binding with `body: array` and a
  `204`/`none` outcome, and every call is accepted. Observed by
  `both_generated_uis_reach_the_server_generated_with_the_same_binding`
  (`crates/edge/ess-cli/tests/generate_ui_http.rs:644`), extended with that binding, after
  `task ui-http-servers`.
- No answer form is silently ignored. A test enumerates every view and outcome body form that
  `ess specify http compile` accepts and asserts that `ess generate ui --http` either carries it or
  refuses it, naming the route, the form and the binding's file. Observed by
  `cargo test -p ess-ui-check --test binding`.
- `setAuthorization(value, renew?)` takes an optional `renew: () => Promise<string | undefined>`.
  When a read, a download or a command is answered `401`, the adapter awaits `renew` once, sets
  the value it returns and repeats the request once. Concurrent requests share one `renew` call. A
  second `401`, an `undefined` value or a rejected `renew` surfaces the `401`. Observed by
  `bound_answers`: a scripted `fetch` answering `401` then `200` sees exactly one `renew` call and
  two requests, the second carrying the new header; two reads answered `401` at once share one
  `renew`. Without `renew`, today's behaviour is unchanged.
- A command answered `401` is classified as `not_granted` with no actor, in both ports, and shown
  as "Not signed in as anyone this may be done by" instead of "No answer from the server". It is
  held by a new vector in `crates/ui/ess-ui/tests/vectors/answers.json`. Observed by
  `cargo test -p ess-ui --test answers` and the React port's vector test
  (`crates/ui/ess-ui-react/tests/live_binding.rs`).
- The generated README's "Caller" paragraph names `renew` and the `Bearer <token>` value. A project
  generated without `--model` is byte-identical. Observed by `generate_ui_http` and the plain
  project digests.

## Scope

- cited: crates/ui/ess-ui/src/binding.rs
- cited: crates/ui/ess-ui-check/src/model.rs
- cited: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
- cited: crates/ui/ess-ui-react/templates/runtime/answer.ts.tmpl
- cited: crates/ui/ess-ui-tui/src/http.rs
- cited: crates/ui/ess-ui/tests/vectors/answers.json
- cited: crates/edge/ess-cli/tests/generate_ui_http.rs
- inferred: crates/ui/ess-ui-react/templates/project/README.md.tmpl
- inferred: crates/ui/ess-ui-react/tests/bound_answers.rs
- inferred: crates/ui/ess-ui-check/tests/binding.rs
- inferred: crates/ui/ess-ui/tests/answers.rs
- inferred: crates/generate/ess-gen/src/http.rs
confidence: medium (the route-table members are cited; whether `ess_gen::http` already exposes each route's declared answers to `ess-ui-check`, or needs an accessor, is inferred)

## Fit review

Tree `3bd1b6e9d2`, binary ess 0.92.0. Covers #723 item B16.

1. **Need.** A UI generated for a server that a binding describes must read that server's answers
   as the binding declares them. A host whose access token is short-lived must be able to renew it
   when the server answers 401, without replacing the adapter. Reproduction above. The requester's
   proposal, theirs: the HTTP adapter builds its routes from each read's and command's `endpoint:`,
   with path params, query or body, response unwrapping, and bearer auth with refresh on 401.
2. **Class.** The answer forms are a *defect*. `ess generate ui --http` accepts a binding and is
   documented as generating "for the server it binds" (`crates/ui/ess-ui-check/src/model.rs:1523-1531`),
   yet it misreads that server's declared answers (a `204` as no answer). Routes from `endpoint:`
   are a *convenience* already answered by `--http`. Renewal on 401 is a *gap* in the generated
   runtime's host API: there is no seam short of replacing the adapter.
3. **Already expressible?** Routes, methods, path parameters, query keys and a bare request body:
   yes, through the `ess-http/1` binding (`binding.ts` above), which the server generator reads
   too. Answer forms: no; the binding states them and the UI drops them. Renewal: only by wrapping
   `httpAdapter(binding)` in a host adapter that catches `DataError.status === 401` on `read`,
   `download` and `send`, calls `setAuthorization` and repeats, and passing that to
   `setDataAdapter`. This works, but every host writes the same retry, and an unshared one renews
   once per concurrent request.
4. **Fit.** The language grids do not apply: no source-language cell
   (`docs/design/language-grids.md`). `endpoint:` stays traceability. Reading routes from it would
   be a second route declaration beside the binding, with a free-text grammar (`UNMAPPED:` is a
   valid value) that the server generator does not read: two spellings for one fact. The answer
   forms reuse the binding's own vocabulary (`body`, `status`, `empty`, `encoding`), carried as
   `method` and `path_params` were for #713 and omitted at their defaults
   (`crates/ui/ess-ui/src/binding.rs:155-160`). Siblings: views and commands both, and both
   runtimes; `classify` changes in both ports, held by one vector file. Renewal is host API, not
   document surface. The terminal reads its header from `ESS_UI_AUTHORIZATION` and has no host
   code to renew with, so it keeps that and shows the 401 as not signed in. Which scheme the server
   requires is `story:an-http-binding-declares-how-a-caller-authenticates` (#681). This story
   neither reads nor precludes it.
5. **Second adopter.** A ticket desk generated against a service that answers `DELETE /tickets/{id}`
   with `204`, lists tickets as a bare array, and accepts 5-minute access tokens from an identity
   provider that the host renews with a refresh token.
6. **Cost.** No source format change: no `ess/37`, no `ess-ui/1` key, no `ess-http/1` key. The
   generated route table gains optional members, absent at defaults. The generated runtime API
   gains one optional parameter. One classification changes: a 401 becomes `not_granted` instead
   of `transport`. That is visible to a host that branches on `answer`, and it is the honest
   reading. No conformance suite change (`ess-conformance/64`, `65` untouched): no HTTP
   conformance target exists yet (`docs/design/http-presentation-binding.md` § "Stage 3").
7. **Considered.** (a) Change nothing: an accepted command shown as failed, which is the defect.
   (b) The requester's routes from `endpoint:`: refused as above. (c) Refuse every non-default
   answer form under `--http`: honest, but a REST service is exactly what `--http` is for.
   (d) Chosen: carry the declared answer and read it, refuse what is not read, and add a renewal
   seam to the host API. (e) Derive OAuth refresh from the binding: needs a token endpoint and a
   grant, which the #681 story leaves out by name (`oauth2` not yet admitted), and a refresh token is the host's to
   hold.

## Decisions

**Accept, redesigned.**

- Taken: response unwrapping, as reading the answer form the binding declares, including the
  wrapped forms of #755. Renewal on 401, as an optional `renew` beside `setAuthorization`.
- Changed: routes do not come from `endpoint:`. They come from the `ess-http/1` binding that
  `--http` reads since #713, and `endpoint:` stays traceability
  (`website/docs/reference/ess-ui.md:2805-2809`).
- Already shipped, no work: path params, query keys, a bare body, and a bearer token sent as
  `setAuthorization("Bearer <token>")`.
- Not covered: reads answered 401 keep rendering as failed after renewal; the terminal gets no
  renewal.
- `depends_on: story:an-answer-may-be-wrapped-in-one-named-member` for the wrapped-form bullet
  only. The other bullets need nothing unmerged.

## Source

github:beyond10x/ess#723, item B16.
