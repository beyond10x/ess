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
revision: 3
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

## Acceptance

- Where a command appends an `Optional<T>` input element, synthesis sends one scenario with a
  present element (for the model above, `label: "label"` or the type's witness) and expects that
  value appended last in the view row, beside the absent case it sends today.
- A target that always appends `null` fails that scenario (mutation case in
  `tests/append_set.rs`); the interpreted target and generated Rust and Go pass it.
- Committed models whose suites move are named in the change, and their suite pins move with it.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize.rs` (`appended_at` :11199)
- cited: `crates/verify/ess-conformance/tests/append_set.rs` (`LABELLED` :225, `the_synthesized_suite_appending_an_optional_element_passes_on_the_interpreted_target` :332)
- inferred: `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` (rows of models with an Optional append)

confidence: medium
