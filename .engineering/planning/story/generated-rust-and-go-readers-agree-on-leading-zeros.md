---
format: aep.planning-md/3
id: story:generated-rust-and-go-readers-agree-on-leading-zeros
kind: story
status: draft
title: Generated Rust and Go readers agree on leading zeros
summary: The generated Rust JSON reader accepts 01 (and 01.0 since U15); Go refuses it; found by the batch-3 adversary
tags:
- defect
- ess-next
- severity-medium
- zero-open-issues
relations:
- informed_by: story:an-integer-literal-is-written-into-the-suite-as-an-integer
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/go.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/shared.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding_rust.rs
revision: 8
---
## Outcome

A client gets the same answer from the generated Rust server and the generated Go server for every
spelling of a JSON number in a command body, a leading zero included.

## Problem

Found by the batch-3 adversary pass over the integer decoders (U15). Two servers generated from one
model answer the same request differently:

- The generated Rust server reads every body with `json::parse`
  (`crates/generate/ess-synth/src/rust/http.rs`, `json::parse(text)`). Its `number` reader
  (`crates/generate/ess-synth/src/rust/json.rs`, `fn number`) takes any run of digits, so `01`,
  `-01`, `01.0` and `007.0e0` are numbers, and `integer_at` reads `01` as `1`.
- The generated Go server without an `ess-http/1` binding reads a body with `encoding/json`
  (`readJSON` in `SURFACE_HELPERS`, `crates/generate/ess-synth/src/go/http.rs`), which refuses
  `01` as not JSON: `400 the body is not JSON: ...`.
- The generated Go server with a binding already reads as Rust does: `parseJSON` in
  `crates/generate/ess-synth/src/go/http/bound.rs`, pinned by
  `http_binding::go::a_json_body_is_read_as_the_rust_server_reads_it` (`place-leading-zero` is
  `201` in both servers).

Minimal reproduction: a command `Count {n: Integer}` with one outcome, served by
`ess generate synthesize --target rust` and `--target go` with no `--http`; `POST` the body
`{"n":01}` to the command's route. Rust answers the outcome as for `{"n":1}`; Go answers `400`.

JSON (RFC 8259 §6) forbids a leading zero, and the served OpenAPI document is JSON, so either both
servers refuse or both read; today the unbound pair disagrees and the bound pair reads.

## Decisions

Settled under D135 (`approval-record:d135-a-breaking-unit-leaves-the-batch`: a change that breaks
a consumer or a documented contract takes a non-breaking form first):

- **Chosen: (b)** the unbound Go server reads as Rust does, through the reader the bound server
  already uses (`parseJSON`), because it is the non-breaking form D135 asks for: no client that
  gets an answer today loses it, and the pinned `place-leading-zero` answer stays `201`.
- Not taken: (a) both refuse a leading zero (other than `0`, `0.x`, `0e..`) as malformed JSON,
  bound and unbound. It breaks a client that sends `01` to a Rust server today and moves the pinned
  `place-leading-zero` answer from `201` to `400`, so under D135 it would leave the batch for an
  operator decision.
- Cost of (b): the generated servers keep reading number spellings RFC 8259 does not admit.

## Acceptance

All tests are in `cargo test -p ess-synth --locked --test http_binding_rust`.

- A new test `generated_servers_agree_on_number_spellings` sends each spelling of the table `0`,
  `-0`, `0.5`, `0e1`, `01`, `-01`, `01.0`, `007.0e0`, `1` as an `Integer` input's value to the
  unbound Rust server, the unbound Go server and the bound pair, and asserts that every server
  answers each spelling with the same status and body.
- In the same test, the unbound Go server answers `{"n":01}`, `{"n":-01}`, `{"n":01.0}` and
  `{"n":007.0e0}` with the outcome for `1`, `-1`, `1` and `7`, as the Rust server does.
- `http_binding::go::a_json_body_is_read_as_the_rust_server_reads_it` still passes, with
  `place-leading-zero` answered `201`.

## Scope

- cited: crates/generate/ess-synth/src/go/http.rs
- cited: crates/generate/ess-synth/src/go/http/bound.rs
- cited: crates/generate/ess-synth/tests/http_binding/shared.rs
- cited: crates/generate/ess-synth/tests/http_binding/go.rs
- cited: crates/generate/ess-synth/tests/http_binding_rust.rs
confidence: medium

Shared surfaces: proposes its `CHANGELOG.md` sentence (the unbound Go server now reads a leading
zero as the Rust server does) and re-pins the generated-Go bytes the `ess-synth` `go` pin target
holds where the unbound `readJSON` helper moves.
