---
format: aep.planning-md/3
id: review-result:adversary-gaps-271-pass-2
kind: review-result
status: active
title: Adversary pass 2, downstream gaps unit feature-request-271
relations:
- reviews: story:feature-request-271
revision: 1
---
unit: story:feature-request-271, tree gaps-271 (after correction 1)
verdict: NEEDS-CHANGE, red 2 (pre-existing 1)
cases: executed 1938→1945

| # | file:line | severity | message |
|---|---|---|---|
| 1 | subject_fact.rs:2594 | warning | under a cardinality-one owns relation the pinned search refused to file the related row under a pinned owner that held none |

Tests: tests/adversary_271_pass2.rs. when_subject suites synthesized byte-identically.

## Correction 2 (coordinator-verified)

A pinned owner known to hold none gets one row (holds_none). Coordinator rerun: the five related-guard test files passed 9, 7, 7, 6 and 4 cases; the crate passed 1945 with 0 failed; clippy exit 0.
