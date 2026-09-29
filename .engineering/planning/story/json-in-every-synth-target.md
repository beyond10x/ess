---
format: aep.planning-md/3
id: story:json-in-every-synth-target
kind: story
status: implemented
title: Every code target represents Json
refs:
- provider: github
  reference: beyond10x/ess#224
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T19:00:14Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T19:00:14Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T19:29:04Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`ess generate synthesize` represents `Json` for the Go, web and clap targets as it does for Rust
since 0.44.0, so no code target refuses a model for using `Json` (closes beyond10x/ess#224).

## Acceptance

- Go: `Json` is a value that carries any JSON document unchanged (object member order and number
  spelling kept) at every position the Go target types; the generated package builds (`go vet`,
  `go build`) and round-trips an object, array, number, string, boolean and null.
- web: `Json` is a TypeScript type for any JSON value at every position the web target types; the
  generated package type-checks (`tsc --noEmit`) and round-trips the same six kinds.
- clap: a `Json` command input is accepted as a JSON document on the command line (and a `Json`
  response printed unchanged); the generated CLI builds and parses each of the six kinds.
- A model without `Json` synthesizes the same bytes for every target (`cargo xtask generate --check`).
- No target refuses `Json` any more; the `fn json` refusal in `failure.rs` is removed.

## Scope

- crates/generate/ess-synth/src/go/, web/, clap/
- crates/generate/ess-synth/src/failure.rs
- crates/generate/ess-synth/tests/
