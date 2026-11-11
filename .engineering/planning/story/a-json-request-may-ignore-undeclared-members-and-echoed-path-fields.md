---
format: aep.planning-md/3
id: story:a-json-request-may-ignore-undeclared-members-and-echoed-path-fields
kind: story
status: active
title: A JSON request may ignore undeclared members and echoed path fields
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#752
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/openapi_http.rs
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: cited
  path: crates/specify/ess-http/tests/request_encoding.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: cited
  path: website/docs/guides/http-routes.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:38:16Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:38:17Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A served route cannot accept a JSON request from an existing client that sends members the
command's input does not declare, or that repeats in the body a field the route already carries in
its path. A common client shape is "read the resource, change one member, send the whole object
back": the body then holds the identity the path carries and members the server computes.

`ess-http/1` already has the word for "a request parameter no field names is ignored",
`request.unknown: ignore`, but admits it beside `form` and `query` only; beside `json` it is refused
(`docs/design/http-presentation-binding.md` § "Decided for stage 2", answer 14). Minimal
reproduction (one command `depot.shelf.RelabelShelf(shelf_id, label)`, `shelf_id` in the path):

```yaml
type: ess-http/1
specification: {system: depot, version: v1, source_digest: "sha256:<digest>"}
routes:
  - command: depot.shelf.RelabelShelf
    method: put
    path: /shelves/{shelf_id}/label
    parameters: {shelf_id: input.shelf_id}
    request: {encoding: json, unknown: ignore}
```

```console
$ ess specify http validate --path spec --binding lenient.http.yaml
error[ESS-HTTP-020]: routes[0].request.unknown: `ignore` admits a request parameter no field names, which only a `form` or `query` request has; a `json` body stays closed, so `unknown` is `refuse` beside `json`
```

Without `request:`, the projected `depot.shelf.RelabelShelf.Input` is `additionalProperties: false`
and holds `label` only (ess 0.92.0).

This story is the binding and its projection: the rule, its validation, the IR reader and the
OpenAPI document. The generated servers follow in
`story:generated-servers-ignore-undeclared-json-members-and-check-echoed-path-fields`.

## Acceptance

- `ess specify http validate` accepts `request: {encoding: json, unknown: ignore}` and
  `request: {unknown: ignore}` on a command route (exit 0), and `ess specify http compile` writes
  `"request": {"unknown": "ignore"}` for both; `HttpIr::from_json` reads that IR back to the same
  bytes. The `ESS-HTTP-020` case "`ignore` with a `json` body" is removed from the code, its
  `CODES` text and its reference table; every other `ESS-HTTP-020` case stays.
- `request: {unknown: ignore}` beside `body: input.<field>` stays refused (`ESS-HTTP-020` at
  `routes[i].request.unknown`): a bare body is one field's value, which has no member to ignore.
- `ess generate --kind openapi --http` with an `unknown: ignore` route (encoding `json` or `form`)
  writes `additionalProperties: true` on that command's `{command}.Input`, and its request body
  description says that a member no field names is ignored and a member the path carries must
  equal the path's value; a binding without `unknown: ignore` keeps its OpenAPI bytes (the existing
  `crates/generate/ess-gen/tests` OpenAPI fixtures pass unmodified).
- `docs/design/http-presentation-binding.md` answer 14 and § "Precedence" step 6 state the rule for
  all three encodings: under `refuse` a key or member the path carries is malformed, as today; under
  `ignore` a member no field names is ignored silently, and a member the path carries is admitted
  when its value decodes to the path's value and malformed otherwise (two values for one field, the
  rule a scalar key given twice already follows). `website/docs/guides/http-routes.md` shows it.

## Scope

- cited: crates/specify/ess-http/src/lib.rs
- cited: crates/specify/ess-http/tests/request_encoding.rs
- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/tests/openapi_http.rs
- cited: docs/design/http-presentation-binding.md
- cited: website/docs/guides/http-routes.md
- inferred: CHANGELOG.md
confidence: high

## Fit review

1. **Need, apart from the syntax.** A served route can accept a JSON body that carries members the
   command does not declare and repeats a path-carried field, while still refusing a repeated field
   whose value contradicts the path. Reproduction above
   (ess 0.92.0: `ESS-HTTP-020` on `json` + `ignore`; `Input` closed with `additionalProperties:
   false`, `openapi/shelves.yaml:72-82`). The requester's proposal (theirs): "an opt-in, per-binding
   lenient body decoding", with an open question whether it lives binding-level, per route or both.
2. **Class: gap.** Not a defect: closing a JSON body is decided and documented
   (`docs/design/http-presentation-binding.md:845-847`, answer 14: "A JSON body stays closed").
   The domain fact "this endpoint ignores what it does not read" cannot be stated for a JSON body at
   all.
3. **Can it already be expressed? No.** `unknown: ignore` is refused beside `json`
   (`crates/specify/ess-http/src/lib.rs:4777-4785`, IR reader `lib.rs:2960-2963`). Declaring every
   echoed member as an input field misdescribes the command and would put the path field in the body
   twice; the generated servers then refuse it anyway (`crates/generate/ess-synth/src/rust/http/bound.rs:388-410`,
   `go/http/bound.rs:679-692`, "the path carries this field, so the request does not").
   `undeclared_fields: ignored` (`ess/24`) is a response and struct key; its fit review left input
   out on purpose (`.engineering/planning/story/response-and-struct-admit-undeclared-fields-when-declared-ignored.md`,
   question 4). No idiom keeps a checked figure in the specification.
4. **Fit.** The language grids (`docs/design/language-grids.md`) do not apply: no expression
   position or operand is touched. The binding's own axis is the encoding column of
   `request.unknown`; this opens it whole (`json` joins `form` and `query`), rather than adding a
   JSON-only option.
   - Vocabulary: reuses `request.unknown: ignore` (`lib.rs:224-231`), so no new key, and answers the
     requester's "where it lives" question with the existing per-route place.
   - Composition: under `refuse` nothing changes. The echoed path field follows the existing rule
     that one field given two values is malformed (`http-presentation-binding.md` § "Request
     encoding": "A key a scalar field receives twice is a malformed request"), extended from
     "always malformed" to "malformed when the values differ" under `ignore`, the same for all three
     encodings.
   - Siblings: `body: input.<field>` keeps refusing `unknown` (no members to ignore). Views take no
     `request:` (`ESS-HTTP-020`) and their query keys already follow view-parameter rules.
   - Targets: OpenAPI states it (`openapi.rs:1189-1214` today writes only a sentence for `form`);
     the Rust and Go servers follow in the dependent story; the UI targets refuse `--http` by name
     (design page § "Still refused"); emitted runners, interpreter and web bridge speak no HTTP
     (§ "What generated artifacts change (stage 2)"); `ess verify diff` does not compare bindings
     (answer 5).
5. **Second adopter.** A booking service's mobile client fetches `GET /bookings/{booking_id}`, edits
   the time, and sends the whole object back with `PUT /bookings/{booking_id}`, including
   `booking_id`, `created_at` and `status`, which the command `example.booking.MoveBooking(booking_id,
   starts_at)` does not take.
6. **Cost.** No source format and no `ess-http` format version: answer 14 was taken as the narrower
   option "so a later release can relax it as an addition without a format change"
   (`http-presentation-binding.md:818-820`). One `ESS-HTTP-020` case removed, no code added. An older
   reader refuses the new IR by name rather than misreading it (`lib.rs:2960`). OpenAPI bytes move
   only for bindings that use `ignore` (a `form` route with `ignore` gains `additionalProperties:
   true`, which its servers already behave as). Changed runtime rule for `form` and `query` under
   `ignore`: an echoed path key with a differing value becomes malformed instead of silently
   ignored (named in the CHANGELOG `Changed` entry).
7. **Alternatives.** (a) Change nothing: an existing client stays unservable from a specification.
   (b) The requester's binding-level switch: a second spelling of `unknown`, at a level that would
   make a protocol endpoint's closed body open by accident. (c) An input-side `undeclared_fields:
   ignored` in `ess/N`: puts a wire tolerance into the model and moves a source format for a binding
   fact (`architecture-decision-record:ess-evolution-02-semantics-before-bindings`). (d) Under
   `ignore`, ignore an echoed path field whatever its value (today's form/query behaviour): hides a
   request that names two shelves. Chosen: extend `unknown: ignore` to `json`, one echo rule for all
   encodings.

## Decisions

**Accept, redesigned.** The need is real (gap). Changed from the request: no new option and no
binding-level switch; the existing per-route `request.unknown: ignore` is admitted beside `json`,
reversing answer 14 as an addition. Kept from the request: a repeated path field with an equal value
is accepted and with a different value refused; undeclared members are ignored. Ignored members are
dropped silently, as `form` and `query` already drop them (`rust/http/bound.rs:279-282`); no header
or log line, because neither is part of the HTTP contract a specification checks. The form/query
echo rule changes with it so one rule holds for all encodings.

## Source

github:beyond10x/ess#752
