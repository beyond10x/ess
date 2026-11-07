---
format: aep.planning-md/3
id: story:an-optional-list-view-parameter-is-read-as-published
kind: story
status: draft
title: An Optional<List<T>> view parameter is read as the repeated key OpenAPI publishes
summary: 'spindle wave 12: http::flat answers None for Optional<List<T>> (ess 0.88.0)'
tags:
- backlog
- from-consumer
- queue-0.91.0
- rank-6
relations:
- serves: vision:O2
revision: 1
---
## Need

Reported by spindle wave 12 against ess 0.88.0. `ess_gen::http::flat`
(`crates/generate/ess-gen/src/http.rs:729-736`) answers `None` for a view parameter of type
`Optional<List<T>>`: its `Optional` arm calls `scalar(of)`, and `scalar` answers `None` for a
list. `openapi::view_parameters` publishes the same parameter as a repeated query key (`style:
form`, `explode: true`), so a server built on `flat` cannot read what the contract publishes.
Spindle works around it in `crates/spindle-ingress/src/surface.rs`.

The same report notes, from source only and not run, a related mismatch. The synthesized Rust
server's `query_object` (`crates/generate/ess-synth/src/rust/http.rs` ~2206-2242) refuses a key
given twice, while the contract publishes a list parameter as that key repeated.

## Acceptance

- `flat` answers `Some((Shape::List, Flat::<T>))` for `Optional<List<T>>`, as it does for
  `List<T>`.
- A generated server reads `?tags=a&tags=b` for such a parameter as `["a", "b"]`, and an absent key
  as none.
- If the synthesized server is confirmed to refuse a repeated key, it reads the repeated form too.
  A served-HTTP test sends the published form.

## Scope

- inferred: `crates/generate/ess-gen/src/http.rs` (`flat`),
  `crates/generate/ess-synth/src/rust/http.rs` (`query_object`), and their HTTP tests.
