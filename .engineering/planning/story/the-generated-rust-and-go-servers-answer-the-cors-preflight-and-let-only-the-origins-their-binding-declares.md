---
format: aep.planning-md/3
id: story:the-generated-rust-and-go-servers-answer-the-cors-preflight-and-let-only-the-origins-their-binding-declares
kind: story
status: draft
title: The generated Rust and Go servers answer the CORS preflight and let only the origins their binding declares read an answer
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-http-binding-declares-which-origins-may-read-its-answers-and-the-openapi-projection-states-the-preflight
revision: 1
---
## Need

After `ae3a-a-binding-declares-which-origins-may-read-its-answers`, a binding states a CORS policy.
The generated servers (`crates/generate/ess-synth/src/rust/http.rs`, `rust/http/bound.rs`,
`go/http/bound.rs`) send no CORS header and answer `OPTIONS` as an unknown method. A browser page the
policy admits could not read an answer, and nothing would say so.

## Outcome

Both servers, for a route under a policy:

- **Preflight.** An `OPTIONS` request on a covered path with `Origin` and
  `Access-Control-Request-Method` is answered `204`, before authentication and before any body is
  read:
  - `Access-Control-Allow-Origin`: the request's origin when listed, or `*` under `any`;
  - `Access-Control-Allow-Methods`: the derived methods;
  - `Access-Control-Allow-Headers`: the declared headers;
  - `Access-Control-Max-Age` when declared;
  - `Access-Control-Allow-Credentials: true` when declared;
  - `Vary: Origin` when origins are listed.

  An origin not admitted, or a method the path does not route, is answered `204` with no CORS
  header. The browser then blocks the read: fail-closed.
- **Answers.** Every answer of a covered route to a request with an admitted `Origin` carries
  `Access-Control-Allow-Origin` (and `Vary`, `Expose-Headers`, `Allow-Credentials` as declared). A
  request without `Origin`, or with one not admitted, gets no CORS header. The answer is otherwise
  unchanged: CORS never refuses a request, it decides only what a page may read.
- `OPTIONS` on a path no covered route has keeps today's answer.

## Acceptance

1. Red today: a fixture binding with `cors: {origins: [https://a.example.test]}` in
   `crates/generate/ess-synth/tests/http_binding/`. The Rust and Go cases send a preflight from
   `https://a.example.test` and assert `204` with `Access-Control-Allow-Origin:
   https://a.example.test`. This fails.
2. After, both alike:
   - the admitted preflight;
   - a preflight from `https://b.example.test` gets `204` with no CORS header;
   - a `GET` from the admitted origin carries the header and `Vary: Origin`;
   - a `GET` without `Origin` carries none;
   - a route with `cors: none` carries none;
   - `origins: any` answers `*` and never echoes;
   - the preflight runs no command and reads no body.
3. Every existing byte pin and committed tree passes unmodified (`http_binding_rust`,
   `served_codec_names`, `go`). A binding without `cors` generates the same bytes.

## Scope

- inferred: `crates/generate/ess-synth/src/rust/http.rs`, `rust/http/bound.rs`, `go/http/bound.rs`
- cited: `crates/generate/ess-synth/tests/http_binding_rust.rs`

## Owned files

The three server files above, plus one fixture and cases in `crates/generate/ess-synth/tests/http_binding/`.

## Not yours

Validation and OpenAPI (`ae3a`). The authentication port (its story, active): the preflight runs
before it by the Fetch standard, so a preflight never carries credentials.

## Tests first

Acceptance 1, Rust then Go, on the shared fixture build (once per run). Name any case over 5 s.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes.

```console
task fmt-check
cargo clippy -p ess-synth --all-targets --locked -- -D warnings
cargo test -p ess-synth --locked --test http_binding_rust
cargo test -p ess-synth --locked --test served_codec_names
cargo test -p ess-synth --locked --test go
```

## Depends on

`ae3a-a-binding-declares-which-origins-may-read-its-answers`. Same server files as `ae1c`/`ae1e`:
one after another.

## Changelog

- The generated Rust and Go servers answer the CORS preflight and add the CORS headers their binding
  declares, only for an admitted origin.

## Fit review

As `ae3a`. Target note: no generated target may ignore `cors`. The Rust and Go servers implement it.
Every other reader of `ess-http-ir/1` either does not serve (OpenAPI, apps) or refuses `--http` by
name (`http-presentation-binding.md:872-873`).

## Decisions

**AE-D5** (SECURITY), default A.

## Size

M, about 420 lines with tests.
