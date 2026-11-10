---
format: aep.planning-md/3
id: story:a-present-optional-element-is-appended
kind: story
status: draft
title: A suite appends a present Optional element and expects its value
tags:
- defect
- ess-d368-close
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/generate/ess-synth/tests/append_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/append_set.rs
revision: 8
---
## Finding

Found by both review passes of the unit that admitted an `Optional` element in `{append: …}`
(`review-result:adversary-d368-u1-pass-1`, `review-result:security-d368-u1-pass-1`), out of that
story's Acceptance. The suite for `{append: input.<field>}` over a `List<Optional<T>>` never sends a
present element, so a target that always appends `null` passes.

Reproduction on 0.92.0, the `LABELLED` model of `crates/verify/ess-conformance/tests/append_set.rs`
(`format: ess/31`): `Job { job_id, labels: List<Optional<String>> }`. `OpenJob(job_id, labels)`
creates it. `AddLabel(job_id, label: Optional<String>)` `updates` it with
`sets: {labels: {append: input.label}}`, and a read-your-writes view `Jobs` publishes `labels`.

```text
$ ess verify conform run --path . --target interpreted   → 2 scenarios: 2 passed
demo.jobs.AddLabel/outcome/labelled:
  execute_command OpenJob   {labels: ["labels.0"]}
  execute_command AddLabel  {job_id: instance job}            ← no `label`: absent
  expect_view Jobs contains {labels: ["labels.0", null]}
```

`AddLabel` is the only send and carries no `label`, so `null` is the only element any scenario
expects. The cause is `synthesize.rs` `appended_at` (:11199-11228): an absent or `null` element
over an Optional element type is pushed as `Node::Null`, and nothing picks a present witness for
the element.

## Decisions

- The present element is the input's ordinary `String` witness, the field's own name (`"label"`),
  as `OpenJob` already sends `"labels.0"` for its list element: no second witness rule is added.

## Acceptance

- `cargo test -p ess-conformance --locked --test append_set` passes a case asserting that the suite
  synthesized from `LABELLED` holds a scenario of `demo.jobs.AddLabel` outcome `labelled` whose
  `AddLabel` step sends `label: "label"` and whose `Jobs` expectation is
  `labels: ["labels.0", "label"]`, beside the existing scenario that leaves `label` out and expects
  `labels: ["labels.0", null]`.
- The same target runs that suite on the interpreted target for `LABELLED` and every scenario
  passes.
- The same target runs that suite on a mutant target (the interpreter with `label` dropped from
  every `AddLabel` input, so it appends `null` whatever is sent) and the present-element scenario
  fails.
- `cargo test -p ess-synth --locked --test append_set` passes a case over `LABELLED` that runs the
  synthesized suite against the generated Rust server and the generated Go server, and both pass
  the present-element scenario.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes, and every row
  the change moves is one whose model appends an `Optional` input element.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs (`appended_at` :11199, and the input
  choice that leaves `label` out)
- cited: crates/verify/ess-conformance/tests/append_set.rs (`LABELLED` :225,
  `the_synthesized_suite_appending_an_optional_element_passes_on_the_interpreted_target` :332)
- inferred: crates/generate/ess-synth/tests/append_set.rs (the generated Rust and Go run)

confidence: medium

Shared surfaces moved: re-pins the suite-bytes rows of models with an `Optional` append.
