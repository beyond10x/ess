---
format: aep.planning-md/3
id: review-result:adversary-gaps-275-pass-1
kind: review-result
status: active
title: Adversary pass 1, downstream gaps unit feature-request-275
relations:
- reviews: story:feature-request-275
revision: 1
---
unit: story:feature-request-275, tree gaps-275
verdict: NEEDS-CHANGE, red 3 (introduced 2)
cases: executed 1973→1977

- blocker (synthesize/caller.rs): an identity the model repeats under another name (event field echo, stored origin) kept the first run's value in the swapped run, so a correct target failed 2 scenarios.
- Go runner gave the reference verdicts on the echoed suite (green).

Correction 1: replace the identity value in every copy, not only under its own field name.
Tests: tests/adversary_275_pass1.rs (4 cases).
