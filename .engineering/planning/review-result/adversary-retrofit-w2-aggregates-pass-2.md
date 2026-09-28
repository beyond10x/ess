---
format: aep.planning-md/3
id: review-result:adversary-retrofit-w2-aggregates-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: absent rows do not move extremes'
relations:
- reviews: story:aggregates-over-optional-fields
revision: 1
---
Adversary pass 2 against story:aggregates-over-optional-fields (beyond10x/ess#148), aep:adversary, 2026-09-27, after correction round 1.

verdict: CONFIRMED (1 warning)
cases: executed 684→686, red 1
origin: introduced 1, pre-existing 0, undecided 0

New cases in `crates/verify/ess-conformance/tests/aggregate_optional_fields_adversary_pass2.rs`. Red:
an ungrouped, parameter-scoped view with only skipping `min`/`max` cannot tell a target that drops
rows lacking any skipping input. Green: skipping min over Decimal, max over Timestamp and sum over
Integer beside a required avg, a count and an optional key all match SQL. Held: the 9-row cap bound,
avg denominators, count_distinct of an absent value.

Coordinator routing: back to the implementor in a final correction round.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize/aggregate.rs", "line": 888, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "Each absent row holds the other inputs at their lowest A value, so in a view whose skipping aggregates are only min, max or count_distinct a target filtering on WHERE a IS NOT NULL AND b IS NOT NULL passes every assertion."}]
```
