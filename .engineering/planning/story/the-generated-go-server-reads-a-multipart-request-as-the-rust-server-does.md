---
format: aep.planning-md/3
id: story:the-generated-go-server-reads-a-multipart-request-as-the-rust-server-does
kind: story
status: active
title: The generated Go server reads a multipart request as the Rust server does
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#754
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:the-generated-rust-server-reads-a-multipart-request
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/go.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:47:24Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:25Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

The Go server `ess generate synthesize --target go --http` writes must read a multipart route
(`story:a-command-route-takes-a-multipart-request-with-file-parts`) with the Rust server's answers
(`story:the-generated-rust-server-reads-a-multipart-request`), as the two servers already answer
one binding alike (`crates/generate/ess-synth/tests/http_binding/go.rs:751`,
`the_go_and_rust_servers_answer_one_binding_alike`). Today the Go server decodes `json`, `form` and
`query` only (`crates/generate/ess-synth/src/go/http/bound.rs:241`, `flatObject`, and the JSON
reader).

## Acceptance

All cases run in `cargo test -p ess-synth --locked --test http_binding_rust` (its `go` module,
`crates/generate/ess-synth/tests/http_binding/go.rs`).

- Every request of the Rust story's acceptance (the `file`/`format`/`dry_run` request and each
  `malformed` case), sent to the Go server generated from the same binding, gets the Rust
  server's status, headers and body bytes, with no exception; the cases join
  `the_go_and_rust_servers_answer_one_binding_alike`.
- `docs/design/http-presentation-binding.md` § "What the servers write (k5, k6)" still lists
  exactly the two differences it lists today (the malformed percent-escape and the member order of
  an answer the binding does not write), and says the Go server reads multipart as the Rust
  server does.
- A route without `multipart` generates byte-identical Go source: `cargo test -p ess-synth --locked
  --test go` passes with no pinned row changed.

## Scope

- cited: crates/generate/ess-synth/src/go/http/bound.rs
- inferred: crates/generate/ess-synth/src/go/http.rs
- cited: crates/generate/ess-synth/tests/http_binding/go.rs
- cited: docs/design/http-presentation-binding.md

confidence: medium

Shared surfaces moved: proposes its `CHANGELOG.md` sentence; no byte-pin row moves.

## Fit review

1. **Need.** Same as the Rust story, for the Go target.
2. **Class: gap**, the Go server half of beyond10x/ess#754.
3. **Expressible today? No.**
4. **Fit.** k6's rule: Go answers byte for byte as Rust wherever the binding writes the answer
   (design page § "What the servers write (k5, k6)"); the standard library reader keeps the
   generated module free of dependencies.
5. **Second adopter.** The claims photo upload of the binding story.
6. **Cost.** Generated Go source grows only for multipart routes.
7. **Alternatives.** (a) Port the Rust reader to Go: identical answers by construction, more code
   to keep in step; taken only if `mime/multipart` differs on a case the acceptance names.

## Decisions

**Accept, redesigned**, as decided in
`story:a-command-route-takes-a-multipart-request-with-file-parts`.

- No multipart difference between the servers is admitted: k6 requires Go to answer byte for byte
  as Rust wherever the binding writes the answer, so where `mime/multipart` answers a case of the
  acceptance differently, the Go server reads that case with a port of the Rust reader
  (alternative (a)), as it already does for `json`.

## Source

github:beyond10x/ess#754
