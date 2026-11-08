---
format: aep.planning-md/3
id: story:a-duration-total-and-mean-are-witnessed-by-synthesis
kind: story
status: draft
title: A Duration total and a Duration mean are witnessed by conformance synthesis
summary: Synthesis refuses a sum of Durations and an undecidable Duration mean as ESS-SYNTH-017; witness them
tags:
- backlog
relations:
- serves: vision:O2
- informed_by: story:aggregate-measures-over-expressions
revision: 1
---
## Need

Conformance synthesis refuses, by name as `ESS-SYNTH-017`, a view whose `sum` totals Durations
(ungrouped: "the change of an ungrouped view's total is asserted as a number"; grouped, as the
guide's own `downtime` example over `ended_at - started_at`: "has no exact value over the arranged
rows"), and an `avg` of a Duration field where no arrangement tells rounding from truncation. The
format admits these measures from `ess/36`, so a target computing them wrongly passes every suite.
Found by the adversary pass on aggregate measures over expressions
(`crates/verify/ess-conformance/tests/adversary_w091_u3_agg_2.rs`, which asserts the refusals today).

## Acceptance

- The guide's `downtime` view (`website/docs/guides/specify/aggregate-views.md`) synthesizes an
  aggregate scenario that passes on the interpreted target and fails a target that sums one end of
  the difference.
- An ungrouped `sum` of a Duration field is witnessed, its change asserted as a Duration.
- A grouped `avg` of a Duration field is witnessed with an arrangement whose mean tells rounding
  from truncation.
- The three cases in `adversary_w091_u3_agg_2.rs` move from asserting the refusal to
  `witnessed_and_passing`, and the guide drops the refusal sentence.
