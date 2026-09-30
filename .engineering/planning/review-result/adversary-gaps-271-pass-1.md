---
format: aep.planning-md/3
id: review-result:adversary-gaps-271-pass-1
kind: review-result
status: active
title: Adversary pass 1, downstream gaps unit feature-request-271
relations:
- reviews: story:feature-request-271
revision: 1
---
unit: story:feature-request-271, tree gaps-271
verdict: NEEDS-CHANGE, red 2 (introduced 1)
cases: executed 1928→1937

| # | file:line | severity | message |
|---|---|---|---|
| 1 | related_guard.rs:582 | blocker | where the subject is the owner the link input names, the success scenario sent the subject while the related row was arranged under another owner |
| 2 | related_guard.rs:342 | note | drive overwrote the owner created_owned arranged |

Tests: tests/adversary_271_pass1.rs. 56 specs synthesized byte-identically. Correction 1 pinned bound link inputs; 1938 passed.
