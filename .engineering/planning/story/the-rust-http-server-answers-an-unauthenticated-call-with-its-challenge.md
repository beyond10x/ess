---
format: aep.planning-md/3
id: story:the-rust-http-server-answers-an-unauthenticated-call-with-its-challenge
kind: story
status: draft
title: The Rust HTTP server answers an unauthenticated call with its challenge
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#681
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-http-binding-declares-how-a-caller-authenticates
scope:
- confidence: inferred
  path: crates/generate/ess-gen/src/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/served_authentication.rs
revision: 4
---
## Need

The Rust server generated with `ess generate synthesize --target rust --http` already takes the
realization's `authenticate` hook and answers a request authenticated as no actor with the
standard `403` `{"refused": "not granted", "actor": null}`
(`crates/generate/ess-synth/src/rust/http.rs:733-758`, `:785-800`; 0.92.0 OpenAPI prose in
`run.txt` § 2). When the binding declares that a route requires a scheme
(`story:an-http-binding-declares-how-a-caller-authenticates`), a request with no authenticated
caller on that route is a missing credential, not a missing grant: it must be answered with the
`unauthenticated` refusal (`401` and the scheme's `WWW-Authenticate` challenge, or the declared
override) before the body is decoded, with no effect.

## Acceptance

- A Rust server generated from the model and binding of
  `story:an-http-binding-declares-how-a-caller-authenticates`, run with an `authenticate` that
  returns `None`, answers `POST /orders` with `401`, `WWW-Authenticate: Bearer` and the standard
  `unauthenticated` body, and the command is not run (no event published, no row stored).
- The same server answers an open route (`authentication: []`) for an unauthenticated caller as it
  does today, and a route requiring a scheme for an authenticated caller without the grant with
  today's `403`.
- A server generated from a binding without `authentication` is byte for byte unchanged.

## Scope

- cited: crates/generate/ess-synth/src/rust/http.rs
- cited: crates/generate/ess-synth/src/rust/http/bound.rs
- inferred: crates/generate/ess-gen/src/http.rs
- inferred: crates/generate/ess-synth/tests/served_authentication.rs
confidence: medium

## Fit review

1. **Need.** A declared authentication requirement must change the server's answer, or the
   binding claims something the server ignores. Proposal (theirs, #681): only the OpenAPI members;
   the server half comes from the ADR's effect ("Rust servers call `Authenticate` before decoding
   the body").
2. **Class.** Gap (no declaration today); without this story it would become a defect, the OpenAPI
   document promising a `401` the server answers as `403`.
3. **Already expressible?** No: the hook exists, the answer to `None` is fixed at `403`
   (`rust/http.rs:785-800`).
4. **Fit.** Reuses the existing `authenticate` hook and the standard-refusal machinery
   (`refusals:` overrides, `rust/http/bound.rs:651`); the challenge is written by the existing
   `http::challenge` (`rust/http/bound.rs:452-470`).
5. **Second adopter.** Any served component whose binding requires a scheme.
6. **Cost.** Generated Rust API: `serve` takes `authenticate` whenever the binding declares
   `authentication` (already so whenever the model declares an actor, which the binding story
   requires). No format.
7. **Alternatives.** Keep `403` for every unauthenticated call: contradicts the projected `401`.
   Have the server parse credentials itself: the ADR keeps verification in the realization's port.
   Chosen: the port says who, the binding says what a missing credential is answered with.

## Decisions

Accept, redesigned (Rust target of `story:an-http-binding-declares-how-a-caller-authenticates`).

## Source

github:beyond10x/ess#681
