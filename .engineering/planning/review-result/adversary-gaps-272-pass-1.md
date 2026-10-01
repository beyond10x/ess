---
format: aep.planning-md/3
id: review-result:adversary-gaps-272-pass-1
kind: review-result
status: active
title: Adversary pass 1, downstream gaps unit feature-request-272
relations:
- reviews: story:feature-request-272
revision: 1
---
unit: story:feature-request-272, tree gaps-272
verdict: NEEDS-CHANGE, red 3 (reproduce on 0.49.0; kept in this unit by the coordinator)

- blocker: a predicate guard comparing an input with the guarded row's owner link refused every aggregate not keyed on that input.
- blocker: an assay owned by the study the predicate compares was refused as two owners.
- warning: the predicate guard beside a key copied from the owner link was refused.
- note: bare "row cannot be created" refusal text.

Correction 1: compared input found on its own; one owner when the row's owner is set from that input; per-row arrangement; refusal reasons.
Tests: tests/adversary_272_pass1.rs (5).
