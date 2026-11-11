---
format: aep.planning-md/3
id: story:every-read-back-of-a-generated-server-answer-ignores-the-intake-depth-limit
kind: story
status: active
title: Every read-back of a generated server's own answer ignores the intake depth limit
summary: Rust single() and the Go envelope read-backs read the server's own output at any depth, like entry::read.
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/json.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:44:00Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:44:01Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Found while landing `story:a-generated-rust-server-reads-its-own-answer-without-the-intake-depth-limit`
(0.69.0), which made the Rust `handle` read its own answer with `json::parse_written`. Other places
in the generated servers still read back output the server wrote itself with the depth-limited
intake reader (`DEPTH = 64`, `crates/generate/ess-synth/src/rust/json.rs:49`, `parse` :120; Go
`jsonDepth = 64`, `crates/generate/ess-synth/src/go/http/bound.rs:380`, `readJSONValue` :413).
Neither panics; both give a wrong answer for a value nested as deep as intake admits, because the
answer wraps it two or more levels deeper than the request held it:

- Rust `single` (`crates/generate/ess-synth/src/rust/http/bound.rs:618-626`) reads a view's
  `{"rows": [...]}` with `json::parse(rows).unwrap_or(Null)`, so a row with a deep `Json` column is
  answered as the empty view (`empty` status, no body).
- Rust bound array view answer (`rust/http/bound.rs:1285`): the same `json::parse(&body)
  .unwrap_or(Null)`, so the rows become `null`.
- Go `readEnvelope` (`go/http/bound.rs:992-999`) and the re-read in `pushJSON`'s default arm
  (`go/http/bound.rs:945`) read the server's own output with `parseJSON`, so a deep envelope becomes
  nil or `null`.

Reproduction is by code reading: the read-backs are in generated server source and need a
generated crate or module to be built and driven with a 64-level `Json` value.

## Acceptance

- A generated Rust server with a `single` view route whose row holds a `Json` column nested 64
  levels (the deepest intake admits) answers `200` with that row, not the `empty` answer; a test
  drives it.
- The same view bound with an array body answers the row, not `null`.
- The generated Go server answers the same deep command and view as the Rust one, byte for byte; a
  test drives both read-backs.
- A request nested 65 levels is still refused at intake with the unchanged status and body in both
  targets.

## Scope

- cited: crates/generate/ess-synth/src/rust/http/bound.rs
- cited: crates/generate/ess-synth/src/rust/json.rs
- cited: crates/generate/ess-synth/src/go/http/bound.rs
- inferred: crates/generate/ess-synth/tests/

confidence: high
