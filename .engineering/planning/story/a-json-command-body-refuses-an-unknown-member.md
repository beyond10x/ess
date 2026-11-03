---
format: aep.planning-md/3
id: story:a-json-command-body-refuses-an-unknown-member
kind: story
status: implemented
title: A JSON command body refuses a member no input field names
summary: 'Generated servers answer 201 to an unknown JSON member under the default unknown: refuse; the design says malformed.'
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding/shared.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:48:50Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-02T20:48:50Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-11-03T02:29:23Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Need

Found while landing `story:an-outcome-may-answer-a-response-field-bare-and-errors-share-a-default-status`
(0.69.0, adversary pass 1, note N1). On an `ess-http/1` route with the default JSON encoding and the
default `unknown: refuse`, both generated servers answer `201` to a command body that carries a
member no input field names, such as `{"item_id":"i1","count":1,"x":1}`. The design says such a
request is malformed: step 6 of the answer order lists "an unknown key under `refuse`" among the
inputs that cannot be read (`docs/design/http-presentation-binding.md:555`), and decision 14 says
"A JSON body stays closed, as ESS's bodies are today" (`:691-692`).

## Where it lives

- Observed by the adversary's runtime probe against both generated servers (Rust and Go), on the
  integration branch at `d3f407440a`; not reduced to a line of the generators yet.
- Existing HTTP binding tests send extra members on JSON routes and expect success, so a fix moves
  their expectations; which of the two the design means is the first question.

## Acceptance

- A JSON command body with a member no input field names, on a route whose `unknown` is `refuse`,
  is answered with `refusals.malformed`, else `400`, by the generated Rust and Go servers.
- The OpenAPI request schema for such a route says `additionalProperties: false`, or the design
  records why it does not.
- A conformance or HTTP binding scenario holds both servers to it.

## Format

None expected; the binding already declares `unknown: refuse` as the default.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** the bound-route JSON read in `crates/generate/ess-synth`, in both server emitters — cited.
- **Files:** `crates/generate/ess-synth/src/rust/http/bound.rs`: `command()` builds the JSON read at :846-850 as `http::json_body`, and `json_body` (:354-360) only parses; it checks no member name — cited.
- **Files:** `crates/generate/ess-synth/src/go/http/bound.rs`: `jsonBody` (:295-307) parses and checks no names; called at :1243; the form and query paths already check in `flatObject` (:196) — cited.
- **Also likely:** `crates/generate/ess-synth/tests/http_binding/shared.rs`, the request table both servers are held to; the scenario runs in `tests/http_binding_rust.rs`, which also compiles the Go module — inferred.
- **Documents:** none to edit; `docs/design/http-presentation-binding.md` already says malformed (step 6 at :597, decision 14 at :789; the story's :555 and :691-692 are stale) — cited.
- **OpenAPI:** no edit; `{command}.Input` is already closed (`crates/generate/ess-gen/src/openapi.rs:2222-2230` via `types::message`, closed at `types.rs:43`; `tests/openapi.rs:1094` asserts `additionalProperties: false`) — cited.
- **Format:** no `ess-http` edit; `HTTP_FORMAT` is `ess-http/1` (`crates/specify/ess-http/src/lib.rs:29`), and `unknown: ignore` on `json` is already refused (`lib.rs:2911`, `:4637`) — cited.
- **Confidence:** high — the JSON read checks no member name; the 201 is the story's own probe, not re-run.
- **Would collide with:** any unit editing the bound-route emitters (`rust/http/bound.rs`, `go/http/bound.rs`) or the shared request table `tests/http_binding/shared.rs`; `story:the-generated-rust-server-refuses-what-its-openapi-refuses` is the same unit (GEN1) — inferred.
- **Safety fact:** the JSON command body is read only at `rust/http/bound.rs:848` and `go/http/bound.rs:1243` — cited.
- **Not established:** depth — a top-level check leaves nested struct members open in Go; the Rust nested case is the sibling story's; a `Json`-typed field must stay open (`go.rs:1351-1373` sends extra members inside one) — inferred.
- **Size:** medium, about 120 lines; no format version — inferred.
- **Targeted run:** `cargo test -p ess-synth --test http_binding_rust` (covers the Go module) and `cargo test -p ess-gen --test openapi_http` — inferred.

## Delivered

W1 (D394), 2026-11-02: a JSON command body refuses an undeclared member in generated Rust and Go servers, and Go derived routes refuse it too. Unit 1af2b1a6c, with 93c9d4155 (review round 1) and 9471e396a (review cases); merged 12ca705bc and e1aa6bc04.
