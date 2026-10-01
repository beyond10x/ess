---
format: aep.planning-md/3
id: review-result:adversary-gaps-270-pass-1
kind: review-result
status: active
title: Adversary pass 1, downstream gaps unit feature-request-270
relations:
- reviews: story:feature-request-270
revision: 1
---
unit: story:feature-request-270, tree gaps-270
verdict: CONFIRMED, red 2 (introduced 1)
cases: executed 2009→2024

- warning (synthesize/related_guard.rs:841): every row beside the named one was a decoy selecting another branch, so a target copying from any row the guard accepts passed (also under the ess/20 state guard).
- held: copies from a decoy, first or last row, or no row killed in 9 variants; ess/20 state guard and #271 owner links correct; success expectations pass a hand-written correct target.

Correction 1: a companion row the guard also accepts, with other copied values.
Tests: tests/adversary_270_pass1.rs (15).
