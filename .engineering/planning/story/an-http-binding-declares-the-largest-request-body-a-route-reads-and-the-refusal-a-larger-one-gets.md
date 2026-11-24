---
format: aep.planning-md/3
id: story:an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
kind: story
status: active
title: An HTTP binding declares the largest request body a route reads and the refusal a larger one gets
tags:
- external-request
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-cli/tests/http.rs
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/openapi_http_presentation.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/go.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/shared.rs
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: cited
  path: crates/specify/ess-http/tests
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: cited
  path: website/docs/reference/diagnostics.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 20
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T04:37:11Z", actor: "human:timo", revision: 17}
- {from: "proposed", to: "active", at: "2026-11-23T04:37:12Z", actor: "human:timo", revision: 18}
---
## Outcome

An `ess-http/1` binding states, once for the binding and again per route where it differs, the
largest request body a route reads, in bytes, and the answer a caller gets when a body is larger
(`refusals.too_large`, `413` unless the binding says another `4xx`). The generated Rust and Go
servers enforce it from the announced length before they read a body byte past it, the OpenAPI
projection publishes the `413`, and a binding that says nothing keeps today's bytes, its
generated servers and their fixed 1 MiB cap included.

## Need

A downstream specification's backend answers `413` on 17 routes (1 MiB on most, 2 MiB on two
groups) and its binding cannot say so. The adopter's two probes, reproduced on a generic model
(`run.txt`; model `model/system.yaml`, binding `ok.http.yaml`; `ess 0.97.4`):

- A fourth standard refusal (`p1.http.yaml`, `refusals: {too_large: {status: 413, body: none}}`):
  `error[ESS-HTTP-029]: refusals.too_large: `too_large` is not a standard refusal; `refusals:` takes `malformed`, `not_granted` and `unfinished``
- A route cap (`p2.http.yaml`, `request: {max_bytes: 1048576}` on `items.catalog.AddItem`):
  `error: reading p2.http.yaml as HTTP binding YAML: routes[1].request: unknown field `max_bytes`, expected one of `encoding`, `unknown`, `body` at line 12 column 15`

Two facts the adopter did not report, both found while assessing:

1. The generated servers already cap every request body, in a way no binding can state. The Rust
   server carries `pub const MAX_BODY: usize = 1_048_576` and answers `413` when `Content-Length`
   exceeds it (`crates/generate/ess-synth/src/rust/http.rs:1788`, `:2037-2041`); the Go server
   carries `const maxBody = 1048576`, reads through `io.LimitReader(request.Body, maxBody+1)` and
   answers `413` (`crates/generate/ess-synth/src/go/http.rs:2677`, `:2773-2781`). Synthesizing the
   generic model with `--http ok.http.yaml` for both targets writes these lines
   (`out-rust/crates/items-server/src/http.rs:23,275-278`, `out-go/server/server.go:44,164`;
   `run.txt`). So a generated server for the adopter's two 2 MiB groups refuses at 1 MiB, and the
   adopter cannot raise it from the binding.
2. That `413` is written nowhere. `git grep -w 413` over `docs/design`, `website/docs`,
   `crates/generate/ess-gen/src` and `crates/verify` finds no page, no OpenAPI response and no
   conformance scenario for it, and no test under `crates/generate/ess-synth/tests` names `413` or the cap (inferred: none sends an
   oversize body). Its body is the generated servers' own `{"refused": "<detail>"}`
   (`rust/http.rs:1882-1889`), not overridable: the Rust reader returns it from `http::read`,
   before any route or binding is consulted (`rust/http.rs:864-867`).

What the binding can say about a request before the model sees it, today:

| fact | where declared | what the projections write |
|---|---|---|
| the input cannot be read (framing of the input, media type, UTF-8, a repeated scalar key, an unknown key under `refuse`, a missing or ill-typed field) | `refusals.malformed`, or a route's own (`ess-http` `RefusalKey`, `crates/specify/ess-http/src/lib.rs:822-860`; precedence step 6, `docs/design/http-presentation-binding.md:684-686`) | OpenAPI `400` for a command with input (`crates/generate/ess-gen/src/openapi.rs:1404`); generated `refusal_malformed` functions (`rust/http/bound.rs:1399`, `go/http/bound.rs:1909`) |
| no grant admits the caller | `refusals.not_granted` | OpenAPI `403` where grants are checked; `refusal_not_granted` |
| the realization is unfinished | `refusals.unfinished` (`5xx`) | OpenAPI `501`; `refusal_unfinished` |
| how the body is encoded and which members are allowed | `routes[].request: {encoding, unknown, body}` (`lib.rs:269-283`) | request body schema; servers decode accordingly |
| the size of a field | the model: a newtype with `value.count <= N` | `maxLength: N` (run on `model-cap`: `out-cap/openapi/items.yaml:370,431`) |
| the size of the whole body, a header count or size, a time limit | nothing | the servers' fixed constants: 1 MiB body (above), `MAX_HEADERS` and `MAX_HEAD` (`431`, `rust/http.rs:2019-2027`, Go `maxHeaders`, `go/http.rs:2748`) |

The conformance suite is not in this picture: `crates/verify` has no `ess_http` dependency and
the suite is transport-neutral (the interpreted target answers commands, not requests;
`crates/generate/ess-synth/src/web/bridge.rs:1-10` says the web bridge holds no route). No
synthesized scenario witnesses `malformed` either. Transport refusals are witnessed only by the
tests in `crates/generate/ess-synth/tests/http_binding/` that run a generated server, for
instance `go.rs:415-434` (a malformed body answers `refusals.malformed`).

The adopter's own spelling, labelled theirs: `refusals: {too_large: {status: 413}}` and a route's
`request: {max_bytes: 1048576}`.

## Fit review

1. **Need.** A domain-neutral transport fact cannot be stated: a binding cannot say how large a
   request body a route reads, or what a caller gets for a larger one, although every generated
   server already enforces a size and answers `413`. A caller of the OpenAPI document cannot learn
   the limit or the status.
2. **Class.** Gap. The convenience reading fails on question 3: a model-level field cap
   (`value.count <= N`) is expressible and projects `maxLength` (`run.txt`), but it bounds one
   field, answers the input refusal (`400`, `malformed`), and says nothing of the JSON that
   surrounds the field, a `List` of fields, a form body, or a body full of members the route will
   refuse. It cannot state a limit of the request. The figure the reader sees (the body limit) is
   not checked by anything, and an idiom that leaves it unchecked is not an answer (skill, Q3).
   Not local policy: a size limit on a request is a property of every HTTP surface; the
   generated servers already hold one.
3. **Idiom.** None that moves the figure into the specification. Keeping the cap in the adopter's
   handler leaves the OpenAPI silent and the generated server at 1 MiB.
4. **Fit.** Grid R (what can be read where): the cap is a fact about the request before decoding,
   so it sits with the three standard refusals, in the binding, never in the model. It reuses the
   binding's two-level shape (binding default, route override that stands in place; precedent
   `refusals:` at both levels, `routes[i].refusals`, 0.77.0, `docs/design/http-presentation-binding.md:540-543`)
   and the refusal's own vocabulary (`status`, `body`, `headers`, `ESS-HTTP-029`). It composes with
   refinement (the refining document restates it, `ESS-HTTP-012`, `lib.rs:3137`), with a route's
   `outcomes` (an outcome and an override on one status must be told apart, `ESS-HTTP-025`
   already covers a `refusals.<key>` override), and with views (a view route reads no body: the
   cap is refused there by name). Every target: Rust, Go, OpenAPI (below); the web bridge and the
   interpreter hold no route and are untouched; `ess verify diff` compares specifications, not
   bindings (inferred: no `ess_http` reader in `crates/verify`).
5. **Second adopter.** A service accepting file metadata uploads with `shop.order` attachments
   in `PlaceOrder`: 64 KiB for the order route, 8 MiB for `AttachInvoice`, `413` with
   `{"error": "too_large"}`. Any HTTP server states these per route.
6. **Cost.** No new format: `ess-http/1` and `ess-http-ir/1` gain optional keys that compile to
   nothing when absent, the way `routes[].refusals`, `alike`, `empty.body` and `encoding` did
   (0.69.0 and 0.77.0; table at `docs/design/http-presentation-binding.md:600-620`; CHANGELOG
   0.77.0: "existing bindings keep their bytes"). `AGENTS.md` § Determinism: a new format version
   is required when meaning, identity, references, canonicalization, names or the envelope change;
   an optional key absent from every existing document changes none. `ess/N` is untouched (the
   binding is its own document). One new diagnostic code (`ESS-HTTP-034`, last today `033`,
   `lib.rs:60-65`), one changed message (the `ESS-HTTP-029` key list), and the rows in
   `docs/design/http-presentation-binding.md`, `website/docs/reference/diagnostics.md` and
   `formats.md`. Generated Rust and Go bytes of a binding with no limit are unchanged.
7. **Considered.**
   - Change nothing, answer with the idiom: rejected above (Q3).
   - The adopter's `request: {max_bytes}` per route only: rejected as the whole answer. `request:`
     says how a body is encoded; a limit is a sibling fact beside it, and 17 routes sharing two
     values need a binding-level default (precedent `refusals:`).
   - A fourth key of `refusals:` with the limit written inside the refusal: rejected: a refusal
     declares what a caller gets, a limit declares when; `refusals.malformed` has no number inside
     it either.
   - A generic `limits:` map of name to number, or a refusal registry: forbidden by `AGENTS.md`
     § Boundaries (no generic facet registry, no property bags). The construct below is two closed
     keys with typed values; a sibling is a new key added when a projector needs it.
   - Reading the cap from the model (a `max_bytes` on a command input): rejected: the model has
     no wire, no framing and no bytes (`ess_domain::component` "No variant names a wire, a port, a
     path or a verb", cited by `lib.rs:3-4`); a JSON body's byte size depends on the encoding.
   - Rate limits (`429`): declined, as `story:a-view-route-answers-declared-refusals` declines
     throttling: no request-time fact decides it, and a conformance suite cannot witness it.
   - Taking the cap from the OpenAPI projection only (an `x-` key, servers unchanged): rejected: a
     target would silently ignore a construct, a red flag in the skill.

Findings covered: the unspellable body limit; the undocumented, untested, un-overridable `413` of
the generated servers; the two 2 MiB route groups the generated servers would refuse; the OpenAPI
that publishes no `413`.

Decision: **accept, redesigned.** What changed from the request and why: the size is a key of a
closed `limits:` object at binding and route level instead of a `request.max_bytes` on routes
only; the refusal is a fourth closed key of `refusals:` (`too_large`) instead of a free refusal;
the number is bounded; and the generated servers' existing fixed `413` becomes the stated default.

## Design

Authored (`ess-http/1`, optional keys, absent compiles to nothing):

```yaml
limits: {request_bytes: 1048576}
refusals:
  too_large: {status: 413, body: {error: {literal: too_large}}}
routes:
  - command: shop.order.PlaceOrder
    method: post
    path: /orders
  - command: shop.order.AttachInvoice
    method: post
    path: /orders/{order_id}/invoice
    parameters: {order_id: input.order_id}
    limits: {request_bytes: 2097152}
```

- **Family.** What the transport enforces before it decodes the input: body size now; header
  count and size, an unsupported media type, a time limit later. Each is a closed key of
  `limits:` when it has a number and a closed key of `refusals:` for its answer, added by its
  own story when a projector needs it, never by a registry. Today's fixed answers of the same
  family are `411` (chunked, Rust), `431` (headers), `400` (head unreadable); the precedence
  step 1 "never overridden" stays true for them. Rate (`429`) is not in the family (decided by
  the environment, not the request).
- **`limits.request_bytes`** (binding): an integer, bytes, `1..=16_777_216` (decision D4), the
  default for every route that reads a body. `routes[i].limits.request_bytes` stands in its
  place on that route. Absent at both levels: today's fixed 1 MiB, so existing bindings keep
  their bytes. Written in the IR only when present; a route's value equal to the binding's is
  still written (it is a route's own statement, like `refusals`), a value equal to 1 MiB is
  not omitted (the author said it).
- **`refusals.too_large`** (binding or route): `status` a `4xx` (default when absent: `413`),
  `body` `none` or `{<member>: {literal: <text>}}`, `headers` literals only: the checks of the
  other three keys (`ESS-HTTP-029`, `-026`, `-027`, `-025`). Not listed: today's `413` with the
  generated `{"refused": "<detail>"}` body. `RefusalKey` grows a fourth variant in name order
  (`malformed`, `not_granted`, `too_large`, `unfinished`); the IR map stays ordered.
- **Validation.** New `ESS-HTTP-034`, at `limits.request_bytes` or `routes[i].limits.request_bytes`:
  not an integer, below 1 or above the ceiling; `routes[i].limits` on a route that reads no body (a view, a
  `get`, a `query` encoding, a `delete` with a `query` request); an unknown key under
  `limits` is a parse error naming the keys (`request_bytes`), as everywhere in `ess-http/1`.
  A binding-level `limits` applies to body-reading routes and is not an error where a route reads
  none. `ESS-HTTP-029` lists `too_large` among the keys. `ESS-HTTP-012`: a refining document
  restates each route's `limits`, and the binding's, and `refusals.too_large` exactly.
- **Precedence.** The size check keeps today's place, before any body byte is read and before the
  404, 405 and grant steps, because it needs only the head: the cap of the route the request's
  method and path match, else the binding's, else 1 MiB, is compared with `Content-Length`
  (Rust) or bounded while reading, `LimitReader(cap+1)` (Go); answered `refusals.too_large` (the
  route's own, else the binding's), else today's `413`. A path no route matches uses the binding's
  cap or 1 MiB. `docs/design/http-presentation-binding.md` § Precedence names it as step 1a
  (today the page does not mention `413` at all).
- **Rust server.** `http::read` returns the `413` itself, before a route is known
  (`rust/http.rs:864-867`, `:2037`). It becomes head, cap lookup by the generated route table
  (the same matcher `dispatch_order` orders), then body read bounded by the cap, with the
  answer built by the generated `refusal_too_large{suffix}` beside `refusal_malformed{suffix}`
  (`rust/http/bound.rs:1399`). Chunked stays `411` (unchanged).
- **Go server.** `readBody` takes the cap and the route's refusal; the dispatch matches the route
  before reading (`go/http/bound.rs:1154` reads before matching today).
- **OpenAPI.** A route with a cap at either level, which reads a body, gets a `413` response
  (the override's status, body, headers, or the default `{refused: string}` shape), and
  `x-ess-request-max-bytes: <N>` on the operation. OpenAPI 3.1 has no standard keyword for a
  request body byte limit (the Request Body Object holds `description`, `content` and `required`;
  `maxLength` is a string keyword): none, stated from the specification text, not from a run. A
  binding with no `limits` keeps its OpenAPI bytes: no `413` is added to routes that declare
  nothing (decision D7).
- **Web client, UI, interpreter, conformance suite.** The generated UI and the web bridge read
  routes, not limits (`crates/ui/ess-ui/src/binding.rs:7`, `web/bridge.rs:1-10`): untouched, and
  a `413` is shown like every other non-success answer. The interpreted target and the suite are
  transport-neutral. The refusal is witnessed in `crates/generate/ess-synth/tests/http_binding`
  against both generated servers: one route with `request_bytes: 64`, a body of 64 bytes
  answers the command, a body of 65 bytes answers `too_large`. That costs two small requests on
  a server the file already builds; no megabytes are sent and no new server is built.
  `story:a-conformance-run-drives-a-live-http-target` (draft) would carry a suite scenario
  later; this story does not wait for it.
- **Format consequence.** None: `ess-http/1` and `ess-http-ir/1` stay (optional keys, omitted when
  absent, the compiled `Request` refuses an unknown key by name, `lib.rs:269-290`, inferred for the rest of the IR, and a
  document that does not use the keys is byte-identical). `ess/N` and every conformance format
  stay. An old `ess` reading a binding that uses `limits` refuses by name: `unknown field
  `limits``.
- **Cut into units (each one sitting, 300 to 600 changed lines):**
  1. `ess-http`: `limits` on spec and IR, `RefusalKey::TooLarge`, `ESS-HTTP-034`, `-029`,
     `-012`, `-025` coverage, canonical IR, tests both sides of every rule, design page table,
     `diagnostics.md`, `formats.md` (about 450 lines).
  2. Rust server: head and body split, route cap lookup, `refusal_too_large`, the 64/65-byte
     test on the existing `http_binding` fixture (about 450 lines).
  3. Go server: the same, with `LimitReader` bound by route, and the Rust-equals-Go parity case
     (about 400 lines).
  4. OpenAPI `413` and `x-ess-request-max-bytes`, `CHANGELOG.md`, `release-notes` line, the
     design page precedence section (about 350 lines).
  Units 2 and 3 depend on 1 and are independent of each other; 4 depends on 1.

## Acceptance

- `ess specify http validate` accepts `limits: {request_bytes: 1048576}`, a route's own, and
  `refusals: {too_large: {status: 413, body: none}}`, and `compile` writes them into the IR in
  key order; checkable by new cases in `crates/specify/ess-http/tests`.
- A binding without `limits` and `too_large` compiles, and generates Rust, Go and OpenAPI, to the
  same bytes as 0.97.4; checkable by the existing byte pins (`ess-gen` `openapi_http*`, the
  `ess-synth` `http_binding` tests) and one explicit regression that diffs generated output.
- `ESS-HTTP-034` is reported at `limits.request_bytes` for 0, a value above the ceiling and a
  non-integer, and at `routes[i].limits` on a view, a `get`, a `query` request; `ESS-HTTP-029`
  lists `too_large`; `ESS-HTTP-012` reports a refining document that drops or changes a cap or the
  override; each with an accepting twin.
- A generated Rust server and a generated Go server with `request_bytes: 64` on a route answer a
  64-byte body with the command's answer and a 65-byte body with `refusals.too_large` (route's
  own over the binding's, else `413` and the generated body), without reading the body past the
  cap where `Content-Length` is announced; checkable by the `http_binding` tests, each under 60 s
  and sharing the fixture's one server build.
- A route with a 2 MiB cap accepts a body the default 1 MiB refuses; checked by a unit test of the
  cap lookup in each server, no 2 MiB request sent.
- OpenAPI carries `413` with the declared status and body and `x-ess-request-max-bytes` on a
  route with a cap, and neither on one without; checkable in `openapi_http_presentation.rs`.
- The design page's precedence section names the size check, and `task test-xtask` passes.

## Decisions

Default for each is the recommended option; the question goes to the operator, and with no answer
the recommended option is taken.

- **D1. Spelling.**
  - A (recommended): closed `limits: {request_bytes: N}` at binding level and per route.
  - B: `request: {max_bytes: N}` per route plus a binding-level `request_max_bytes: N`.
  - C: the adopter's per-route `request: {max_bytes}` only, no binding default.
- **D2. The answer.**
  - A (recommended): `refusals.too_large`, a fourth closed key, any `4xx`, default `413`.
  - B: a fixed `413` whose body the binding cannot change.
  - C: fold into `refusals.malformed`.
- **D3. Unit of the size.**
  - A (recommended): an integer of bytes.
  - B: also a string with a binary suffix (`1MiB`, `2MiB`).
- **D4. Ceiling of a declared cap** (the Rust server allocates the announced length once it is
  within the cap).
  - A (recommended): `16_777_216` (16 MiB, as the conformance process line, `formats.md:419`).
  - B: 64 MiB.
  - C: none.
- **D5. Default without a declaration.**
  - A (recommended): the fixed 1 MiB, now named in the design page and the generated comments;
    bytes unchanged.
  - B: leave it unwritten, as today.
- **D6. Where the size is checked.**
  - A (recommended): at head time, with the matched route's cap, the body never read past it.
  - B: read up to the binding's largest cap and compare after matching (simpler server change; a
    small route reads up to the ceiling before it is refused).
- **D7. OpenAPI for a binding with no declaration.**
  - A (recommended): no `413` added (bytes unchanged).
  - B: add `413` with the fixed 1 MiB to every body-reading route (more honest, changes every
    existing document).

## Decisions taken

Taken for the wave of 2026-11-23 (decision D33: this story takes the place of the mutation audit in
that wave, because it blocks an accepted story of a consumer): the story's own recommendations
stand.

- D1 A: `limits.request_bytes` at binding level and per route.
- D2 A: `refusals.too_large` as a fourth closed key; status, body and headers declarable as the
  other three are.
- D3 A: an integer of bytes.
- D4 A: a ceiling of 16 MiB (`16_777_216`) on a declared cap.
- D5 A: the fixed 1 MiB stays the default without a declaration.
- D6 A: the check at head time, with the matched route's cap.
- D7 A: no `413` in OpenAPI for a binding that declares nothing.

The requester confirmed that a binding default with a per-route override and a declarable refusal
body fit its routes.

Decided on 2026-11-23, after round 1 of the plan critics, beside D1 to D7 above:

- D8. The new behaviour is tested against two servers built once by two harness tasks
  (`body-cap-rust-server`, `body-cap-go-server`, each read through an environment variable),
  because no test may build with cargo and the committed example servers keep their bytes. This
  replaces the sentence "no new server is built".
- D9. The guide page on HTTP routes gets one subsection, "Limit the request body".
- D10. In the OpenAPI projection an operation keeps the form it has: a route without bound
  outcomes gains one `413` response and `x-ess-request-max-bytes`; a route in the bound form
  gains the `413` among its bound responses. The precedence list gets a step 1a, with no
  renumbering.
- D11. The unit for the Go server owns both Go set-up steps of the workflows (the test-build job
  and the hotfix test job) and depends on the unit for the Rust server, because the parity case
  (both servers answer the same bytes for the same over-cap request) needs both.
