---
format: aep.planning-md/3
id: story:generated-servers-ignore-undeclared-json-members-and-check-echoed-path-fields
kind: story
status: draft
title: Generated servers ignore undeclared JSON members and check echoed path fields
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#752
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-json-request-may-ignore-undeclared-members-and-echoed-path-fields
- depends_on: story:the-go-server-refuses-an-undeclared-member-at-any-depth
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
  path: crates/generate/ess-synth/tests/fixtures/http-binding-shop.routes.yaml
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding/board_routes.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/go.rs
revision: 5
---
## Need

Once a binding may say `request: {encoding: json, unknown: ignore}`
(`story:a-json-request-may-ignore-undeclared-members-and-echoed-path-fields`), the Rust and Go
servers `ess generate synthesize --http` writes must answer it. Today both refuse a JSON member the
path carries whatever its value (`crates/generate/ess-synth/src/rust/http/bound.rs:388-410`,
`crates/generate/ess-synth/src/go/http/bound.rs:678-692`: "the path carries this field, so the
request does not"), and the generated input decoder refuses an undeclared member
(`crates/generate/ess-synth/src/rust/json.rs:927`, "no field is named so"). For `form` and `query`
under `ignore` they drop a key the path carries whatever its value (`rust/http/bound.rs:305-316`).

The neutral case, `depot.shelf.RelabelShelf(shelf_id, label)` on `PUT /shelves/{shelf_id}/label`
with `request: {encoding: json, unknown: ignore}`:

| body sent to `/shelves/s1/label` | answer |
|---|---|
| `{"label":"north"}` | the outcome, as today |
| `{"label":"north","shelf_id":"s1"}` | the same outcome as the first |
| `{"label":"north","shelf_id":"s2"}` | `malformed` (`400`, or the binding's `refusals.malformed`) |
| `{"label":"north","origin":"web"}` | the same outcome as the first, `origin` ignored |

## Acceptance

This story lands after `story:the-go-server-refuses-an-undeclared-member-at-any-depth`, which
closes the Go server's nested struct decoding (`closedInput`, `closedMembers` and the struct arm of
`type_decoder`) for every binding. This story owns the combination: under `unknown: ignore` it
opens the top level of the input object only and leaves that nested closedness as the earlier
story wrote it. Every bullet is asserted by `cargo test -p ess-synth --locked --test
http_binding_rust` (which holds `tests/http_binding/go.rs`).

- For a route with `request: {encoding: json, unknown: ignore}`, the generated Rust server answers
  the four requests of the table above as the table says. A member no input field names is dropped
  before decoding, at the top level of the input object only. A member the path carries is dropped
  when its decoded value equals the path's value, and answered `malformed` naming
  `body.<wire name>` otherwise. The status is the binding's `refusals.malformed` where it declares
  one, else `400`.
- On the same route, an undeclared member inside a nested struct (`body.<field>.extra`) is still
  answered `400` naming its path, by the Rust server and the Go server alike, as
  `story:the-go-server-refuses-an-undeclared-member-at-any-depth` makes them answer it.
- For `form` and `query` under `ignore`, a key the path carries follows the same rule (equal value
  admitted, differing value `malformed`).
- Under `unknown: refuse`, every encoding answers each request of the table with the same status
  and body as before this change.
- The generated Go server answers each of these requests with the Rust server's status, headers and
  body bytes (`the_go_and_rust_servers_answer_one_binding_alike` gains the cases).
- A binding without `unknown: ignore` generates server source byte-identical to the source after
  `story:the-go-server-refuses-an-undeclared-member-at-any-depth`: the existing `http_binding_rust`
  byte pins pass unmodified.

## Scope

- cited: crates/generate/ess-synth/src/rust/http/bound.rs
- cited: crates/generate/ess-synth/src/go/http/bound.rs
- inferred: crates/generate/ess-synth/src/rust/http.rs
- inferred: crates/generate/ess-synth/src/go/http.rs
- cited: crates/generate/ess-synth/tests/http_binding/go.rs
- inferred: crates/generate/ess-synth/tests/http_binding/board_routes.rs
- inferred: crates/generate/ess-synth/tests/fixtures/http-binding-shop.routes.yaml

confidence: medium

Moves no shared surface. Shares `go/http.rs`, `go/http/bound.rs` and `tests/http_binding/go.rs`
with `story:the-go-server-refuses-an-undeclared-member-at-any-depth`, which it depends on.

## Fit review

1. **Need.** The servers ESS generates must honour what the binding states; the rule is decided in
   the story this depends on. Reproduction: the table above against the code cited in Need (no
   server is built in the triage; the refusals are read from `rust/http/bound.rs:388-410` and
   `rust/json.rs:927`). Requester's syntax: see the binding story.
2. **Class: gap**, the server half of the same gap (beyond10x/ess#752).
3. **Expressible today? No.** The binding is refused (`ESS-HTTP-020`), so no server can be asked.
4. **Fit.** Same rule in both servers, byte for byte, as k6 requires
   (`docs/design/http-presentation-binding.md` § "What the servers write (k5, k6)"). It reuses the
   existing `ignore` path of `flat_object` (`rust/http/bound.rs:288-345`) and extends `with_path`
   rather than adding a decoder. Other targets: the interpreter, emitted runners and web bridge speak
   no HTTP; the UI refuses `--http` (§ "Still refused").
5. **Second adopter.** The booking client of the binding story, whose `PUT` echoes `booking_id`
   and server-set members.
6. **Cost.** Generated server source changes only for routes with `unknown: ignore`. Behaviour
   change for `form`/`query` under `ignore`: a differing echoed path key is now malformed.
7. **Alternatives.** (a) Change nothing and leave the binding without a server: a target silently
   ignoring a construct is a red flag of the assessing skill. (b) Drop every echoed path member
   whatever its value: hides a contradictory request. Chosen: the binding story's rule.

## Decisions

**Accept, redesigned**, as decided in
`story:a-json-request-may-ignore-undeclared-members-and-echoed-path-fields`: the existing
`request.unknown: ignore` opens to `json`; the servers drop undeclared members silently and admit an
echoed path member only when it equals the path's value.

- Ordered after `story:the-go-server-refuses-an-undeclared-member-at-any-depth`, which owns nested
  closedness in the Go server; this story owns how `unknown: ignore` combines with it (top level
  open, nested levels as that story closed them), so its byte pins are taken over that story's
  output.

## Source

github:beyond10x/ess#752
