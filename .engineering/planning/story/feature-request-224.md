---
format: aep.planning-md/3
id: story:feature-request-224
kind: story
status: active
title: generate synthesize represents Json fields and inputs
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#224
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T16:35:55Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-29T16:35:55Z", actor: "human:timo", revision: 4}
---
## Outcome

`ess generate synthesize --target rust` represents `Json` fields, command inputs, event payloads and
view rows as the generated crate's dependency-free `json::Value`, so a specification using `Json`
synthesizes instead of being refused (beyond10x/ess#224). The Go and web targets keep refusing
`Json`, naming the target, until they get their own representation.

## Acceptance

- A specification with a `Json` entity field, command input and view column synthesizes for
  `--target rust`; the generated workspace builds (`cargo check`) and its types crate names
  `json::Value` at each of those positions.
- `json::Value` is reachable from the types crate (the module lives in or is re-exported by it), and
  round-trips through the generated serialization of a command input and a view row with an object,
  an array, a number, a string, a boolean and null.
- `--target go` and `--target web` still refuse the same specification, naming the target.
- A specification without `Json` synthesizes byte-identical output (`cargo xtask generate --check`).

## Scope

- crates/generate/ess-synth/src/failure.rs (`fn json`)
- crates/generate/ess-synth/src/rust/ (type mapping, json.rs placement)
- crates/generate/ess-synth/tests/
