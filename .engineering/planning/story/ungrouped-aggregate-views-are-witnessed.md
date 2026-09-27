---
format: aep.planning-md/2
id: story:ungrouped-aggregate-views-are-witnessed
kind: story
status: draft
title: An ungrouped, unparameterised aggregate view gets a scenario
relations:
- decomposes: epic:retrofit-findings-20260927
revision: 1
---
## Scope

An aggregate view with no `group_by` and no parameter, such as #148's `DurationTotal`
(`sum(duration)` over every `Order`), gets no scenario: synthesis refuses it with `ESS-SYNTH-016`
because nothing ties its one row to the rows the scenario created
(`crates/verify/ess-conformance/src/synthesize/aggregate.rs` ~651). This holds for every
ungrouped, unparameterised aggregate view, with or without `skip_absent`. Found by adversary pass 1
of the aggregates unit (`review-result:adversary-retrofit-w2-aggregates-pass-1`).

## Acceptance

The `DurationTotal` repro from #148 synthesizes a scenario whose expectation holds on a target
that starts empty and fails on a target that miscounts, or synthesis states in the suite why it
cannot (for example: the scenario asserts the change in the total, not its absolute value).
