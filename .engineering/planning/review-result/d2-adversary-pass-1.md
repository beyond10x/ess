---
format: aep.planning-md/2
id: review-result:d2-adversary-pass-1
kind: review-result
status: active
title: D2 adversary pass 1 (aggregate views)
relations:
- reviews: story:aggregate-views
revision: 1
---
Adversary pass 1 on wave unit D2 (#96 aggregate views, ess/10). Tree `ess-wave-20260925-d2` on
`358247192` plus the implementor's uncommitted diff.

Header as returned: verdict NEEDS-CHANGE; cases executed 555→560, red 5; origin introduced 4,
pre-existing 0, undecided 0. Added `crates/verify/ess-conformance/tests/aggregate_views_adversary.rs`.

Coordinator routing: all four back to the implementor with the adversary's named fixes (F1: arrange
means whose 7th decimal separates rounding from truncation, else refuse 017; F2, F3: refuse 017;
F4: Bₖ for every non-scoped key).

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize/aggregate.rs","line":745,"category":"mutant","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The avg adjustment only makes the mean non-terminating, so the avg-truncates mutant survives for every avg input except input 0 at m=3 or m=6."},{"file":"crates/verify/ess-conformance/src/synthesize/aggregate.rs","line":950,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"min/max over a String identity asserts the synthetic placeholder row-0, so the suite fails every correct target instead of refusing ESS-SYNTH-017."},{"file":"crates/verify/ess-conformance/src/synthesize/aggregate.rs","line":1013,"category":"property","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"A declared move that rewrites a group key leaves the asserted groups keyed by unscoped witness values, voiding the shared-target scoping guarantee."},{"file":"crates/verify/ess-conformance/src/synthesize/aggregate.rs","line":700,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"Bk rows are arranged only for keys after the first, so ignoring a non-scoped first group key changes no asserted row."}]
```
