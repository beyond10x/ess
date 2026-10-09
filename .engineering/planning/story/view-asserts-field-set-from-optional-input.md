---
format: aep.planning-md/3
id: story:view-asserts-field-set-from-optional-input
kind: story
status: draft
title: A synthesized view expectation asserts a field set from an Optional input
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A synthesized scenario asserts a view field whose value an outcome sets from an Optional input,
whenever the scenario sends a value for that input, as it already does for a required input.

## Evidence

An adopter on ess 0.56.0. Neutral fixture to write for the test:

- Command `demo.sites.Register`, input `site: Optional<String>`.
- An input-guarded refusal declared first answers when `site` is absent
  (`when: not defined(input.site)`), so the accepting outcome runs only with `site` present.
- The accepting outcome `sets: {site: input.site}` on an entity field `site: String`.
- An unfiltered view publishes `site`.

Observed: the synthesized scenario for the accepting outcome sends a literal for `site`, but its
`expect_view` asserts only identity and state, not `site`, so sets-retarget mutants on such fields
survive (12 in the adopter's model). With the input typed `String`, `expect_view` asserts
`"site": {"kind": "literal", "value": …}`.

On `main` (read, not run): `settled`
(`crates/verify/ess-conformance/src/synthesize.rs:9354-9422`) copies an input-sourced field when
the scenario sent it, and the compiler narrows the input after the `not defined` refusal
(`crates/specify/ess-compiler/src/resolve.rs:2755-2805`). The only skip matching the symptom is
`None => continue` at `synthesize.rs:9390`, taken when `site` is missing from the input `settled`
receives, which may differ from what the step sends (`supply`, `:5966`, or the update
re-witnessing). The first step is the fixture run on `main`.

Not `story:refusal-witness-sends-echoed-optional-input`: there the input is never sent; here it is
sent and not asserted.

## Acceptance

- A synthesis test over the fixture: the accepting scenario's `expect_view` row carries `site`
  equal to the value sent.
- A sets-retarget mutant on `site` is killed by a synthesized scenario.
