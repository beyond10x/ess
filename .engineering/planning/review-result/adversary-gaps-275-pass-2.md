---
format: aep.planning-md/3
id: review-result:adversary-gaps-275-pass-2
kind: review-result
status: active
title: Adversary pass 2, downstream gaps unit feature-request-275
relations:
- reviews: story:feature-request-275
revision: 1
---
unit: story:feature-request-275, tree gaps-275
verdict: NEEDS-CHANGE, red 4 (introduced 2, pre-existing 1)
cases: executed 1979→1986

- blocker (synthesize/caller.rs): an integer identity copied through a declared conversion into a plain Integer field kept the first run's value.
- warning (synthesize/caller.rs): the swapped draw ignored guards and sent 524417 against `when: record_id < 100`.
- warning, pre-existing (synthesize.rs fresh_identity, existence.rs identity_at): the first run also ignored guards.

Correction 2 (coordinator-verified): redraw by source path; both runs draw values that keep every step on its guarded branch; a guard leaving no fresh value refuses the scenario by name.
Tests: tests/adversary_275_pass2.rs (7 cases). Coordinator rerun in gaps-275: pass1 4/4, pass2 7/7, caller_fresh_identity 8/8, lib 88 passed. Merged tree: ess-conformance 1987 passed, 0 failed.
