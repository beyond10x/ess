---
format: aep.planning-md/3
id: story:the-go-http-server-answers-an-unauthenticated-call-with-its-challenge
kind: story
status: active
title: The Go HTTP server answers an unauthenticated call with its challenge
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#681
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:the-rust-http-server-answers-an-unauthenticated-call-with-its-challenge
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/served_authentication_go.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:47:34Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:35Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

The Go server generated with `ess generate synthesize --target go --http` takes
`authenticate func(*http.Request) *Caller` and answers a request authenticated as no actor with the
standard `403` not-granted refusal (`crates/generate/ess-synth/src/go/http.rs:363-441`,
`:1302-1314`). The binding design keeps the Go server's answers byte for byte those of the Rust
server (`docs/design/http-presentation-binding.md`, unit k6). When a route requires a scheme
(`story:an-http-binding-declares-how-a-caller-authenticates`), an unauthenticated request there
must be answered with the `unauthenticated` refusal, as
`story:the-rust-http-server-answers-an-unauthenticated-call-with-its-challenge` makes the Rust
server answer it.

## Acceptance

All tests are in `crates/generate/ess-synth/tests/served_authentication_go.rs` (new), run with
`cargo test -p ess-synth --locked --test served_authentication_go`, on the model and binding of
`story:an-http-binding-declares-how-a-caller-authenticates`.

- `an_unauthenticated_call_gets_the_challenge`: the generated Go server, with an `authenticate`
  returning `nil`, answers `POST /orders` with `401`, `WWW-Authenticate: Bearer` and the same body
  bytes as the Rust server, and runs no command.
- `an_open_route_and_an_ungranted_caller_answer_as_today`: an open route answers as before, and an
  authenticated caller without the grant gets the standard `403` not-granted refusal.
- `a_binding_without_authentication_is_unchanged`: the Go server generated from the same binding
  without `authentication` is byte for byte the one 0.92.0 generates, and
  `cargo test -p ess-synth --locked --test go` passes with no pinned byte moved.

## Scope

- cited: crates/generate/ess-synth/src/go/http.rs
- cited: crates/generate/ess-synth/src/go/http/bound.rs
- inferred: crates/generate/ess-synth/tests/served_authentication_go.rs

confidence: medium

`go/http.rs` routes a `nil` caller on a route that requires a scheme to the new refusal;
`go/http/bound.rs` writes it beside `refusalNotGranted` in `refusal_set` and spells its
`WWW-Authenticate` value with `challenge_text`. Shared surfaces: moves none.

## Fit review

1. **Need.** As for the Rust server: a declared requirement the Go server ignores is a binding
   that lies. Proposal (theirs, #681): OpenAPI members only.
2. **Class.** Gap (see the Rust story).
3. **Already expressible?** No: `nil` is fixed to `403` (`go/http.rs:429`).
4. **Fit.** Reuses the existing hook and the Rust server's answers (unit k6 parity rule).
5. **Second adopter.** Any Go realization of a served component whose binding requires a scheme.
6. **Cost.** No format; generated Go API unchanged in signature.
7. **Alternatives.** Leave Go at `403`: breaks k6 parity. Chosen: parity.

## Decisions

Accept, redesigned (Go target of `story:an-http-binding-declares-how-a-caller-authenticates`).

## Source

github:beyond10x/ess#681
