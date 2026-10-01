---
format: aep.planning-md/3
id: review-result:adversary-gaps-257-pass-2
kind: review-result
status: active
title: Adversary pass 2, downstream gaps unit feature-request-257
relations:
- reviews: story:feature-request-257
revision: 1
---
unit: story:feature-request-257, tree gaps-257 (after correction 1)
verdict: CONFIRMED, red 2 (introduced 2)
cases: executed 1937→1948

| # | file:line | severity | message |
|---|---|---|---|
| 1 | aggregate.rs:1115 | blocker | dropping the Bk tuple of a key read from the grouped owner let a target grouping by that key alone pass |
| 2 | aggregate.rs:1663 | warning | a refuted row was forced onto a shared owner and the view refused |

Tests: tests/adversary_257_pass2.rs (includes the downstream own-field via, Optional key case). 73 specs synthesized byte-identically to the base.

## Correction 2 (coordinator-verified)

Bk kept under a new owner; refuted rows get their own owner. Coordinator rerun: the three unit test files passed 4, 8 and 11 cases; cargo test -p ess-conformance 1948 passed, 0 failed; clippy exit 0. Merged into integrate/gaps-w1.
