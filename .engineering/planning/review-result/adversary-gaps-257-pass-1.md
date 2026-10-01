---
format: aep.planning-md/3
id: review-result:adversary-gaps-257-pass-1
kind: review-result
status: active
title: Adversary pass 1, downstream gaps unit feature-request-257
relations:
- reviews: story:feature-request-257
revision: 1
---
unit: story:feature-request-257, tree gaps-257
verdict: NEEDS-CHANGE, red 4 (introduced 3, pre-existing 1)
cases: executed 1928→1936

| # | file:line | severity | message |
|---|---|---|---|
| 1 | aggregate.rs:1480 | blocker | a per-row related row replaced the shared owner; one-row groups let a target that never groups pass |
| 2 | aggregate.rs:285 | warning | creating branch chosen per field, so two keys from one entity were refused |
| 3 | aggregate.rs:905 | warning | aggregate-input refusal ended in "but " with no reason |
| 4 | aggregate.rs:814 | warning | optional related key refused with a false claim (pre-existing) |

Tests: tests/adversary_257_pass1.rs. Correction 1 fixed all four; 1937 passed.
