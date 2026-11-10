---
format: aep.planning-md/3
id: story:an-optional-list-view-parameter-is-read-as-published
kind: story
status: active
title: An Optional<List<T>> view parameter is read as the repeated key OpenAPI publishes
summary: 'spindle wave 12: http::flat answers None for Optional<List<T>> (ess 0.88.0)'
tags:
- backlog
- defect
- from-consumer
- queue-0.91.0
- rank-6
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/src/view_query.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/adversary_view_params_pass1.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:40:26Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:40:28Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Reported by a downstream consumer against ess 0.88.0, re-checked on 0.92.0. The OpenAPI document
publishes a view parameter of type `List<T>` or `Optional<List<T>>` as a repeated query key
(`style: form`, `explode: true`, `openapi::view_parameters`), but nothing ESS generates reads that
form:

- `ess_gen::http::flat` (`crates/generate/ess-gen/src/http.rs:800-807`) answers `None` for
  `Optional<List<T>>`: its `Optional` arm calls `scalar(of)`, and `scalar` answers `None` for a list
  (`http.rs:829-831`). It answers `Some((Shape::List, _))` for `List<T>`. A server built on `flat`
  cannot read what the contract publishes; the consumer works around it by hand.
- The synthesized Rust and Go servers refuse a served view with any list parameter, by name, bound
  or not (`crates/generate/ess-synth/src/view_query.rs:210-237`, `refuse_unqueryable`).
  Their query reader `query_object` (`crates/generate/ess-synth/src/rust/http.rs:2206-2242`) takes
  scalars only and refuses a key given twice.

Reproduction (`spec.yaml`): view `desk.work.Probe` over `desk.work.Task` with params
`owners: Optional<List<String>>` and `required_owners: List<String>`, each read by
`exists: {in: param.<p>, as: x, that: owner == x}`, served by a `reached_by: network` component.

```
$ ess specify validate --path spec.yaml
desk v1 — 1 file(s), valid
$ ess generate --path spec.yaml --kind openapi --out oa
  # owners: in: query, required: false, style: form, explode: true, schema: {type: array, items: {type: string}}
$ ess generate synthesize --path spec.yaml --target rust --http http.yaml --out rs
rust target cannot emit this workspace
views.desk.work.Probe.params.owners: a served view's parameter is read from the query string, and a query value carries only a scalar
views.desk.work.Probe.params.required_owners: a served view's parameter is read from the query string, and a query value carries only a scalar
exit 1
```

The Go target and the unbound Rust target refuse the same two parameters.

## Acceptance

- `ess_gen::http::flat` answers `Some((Shape::List, flat))` for `Optional<List<T>>` of a scalar `T`,
  as it does for `List<T>`; a unit test in `crates/generate/ess-gen` pins both.
- `ess generate synthesize --target rust` and `--target go`, with and without `--http`, over the
  model above exit 0, and no longer report `views.*.params.*: … carries only a scalar` for a `List`
  or `Optional<List>` of a scalar.
- A served-HTTP test sends the published form: the generated Rust and Go servers read
  `?owners=a&owners=b` as `["a", "b"]`, an absent `owners` as absent (`Optional`) or as `[]`
  (`List`), and hand the port those values.
- A list parameter of a non-scalar element (a struct, `Json`) is still refused by name.

## Scope

- cited: crates/generate/ess-gen/src/http.rs
- cited: crates/generate/ess-synth/src/view_query.rs
- cited: crates/generate/ess-synth/src/rust/http.rs
- inferred: crates/generate/ess-synth/src/rust/http/bound.rs
- inferred: crates/generate/ess-synth/src/go/http/bound.rs
- inferred: crates/generate/ess-synth/tests/adversary_view_params_pass1.rs

confidence: medium
