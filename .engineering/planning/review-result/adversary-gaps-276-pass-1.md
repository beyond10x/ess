---
format: aep.planning-md/3
id: review-result:adversary-gaps-276-pass-1
kind: review-result
status: active
title: Adversary pass 1, downstream gaps unit feature-request-276
relations:
- reviews: story:feature-request-276
revision: 1
---
unit: story:feature-request-276, tree gaps-276
verdict: nothing found, red 0
cases: executed 255→266

- note (tests/one_sided_declarations.rs:162): only the actor test could fail on base; bindings had no can-fail case. Added on request: a one-sided binding with refs, shown red on base and green after.

Tests: tests/adversary_276_pass1.rs (11 cases). Coordinator rerun: ess-diff 267 passed, 0 failed; clippy exit 0.
