---
format: aep.planning-md/3
id: review-result:adversary-gaps-229-pass-2
kind: review-result
status: active
title: Adversary pass 2, downstream gaps unit feature-request-229
relations:
- reviews: story:feature-request-229
revision: 1
---
unit: story:feature-request-229, tree gaps-229
verdict: NEEDS-CHANGE, red 2 (introduced)
cases: executed 1981→1987

- warning (synthesize/related_guard.rs:343): nesting ran below ess/20, so ess/18 and ess/19 documents gained ArchiveNode/outcome/archived.
- warning (synthesize/related_guard.rs:326): moving the exists:false path ahead of the arranging check changed ess/18 suites on its own.
- held: mutual nesting through two entities terminates and refuses by id; NESTED=9 numbering does not double-bind; all 8 pass-1 no-refusal claims hold when filtered by refusal.scenario.

Correction 2 (coordinator-verified): EssIr carries the document format (serde-skipped); related_guard::nests gates nesting and the reordered path to ess/20.
Tests: tests/adversary_229_pass2.rs (6). Coordinator rerun in gaps-229: pass1 9/9, pass2 6/6, related_guard_state 4/4, domain 7/7 and 17/17, ess-compiler 231 passed. Merged tree: 3689 passed, 0 failed; xtask docs, schema and generate --check current.
