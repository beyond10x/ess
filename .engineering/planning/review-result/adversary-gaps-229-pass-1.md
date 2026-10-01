---
format: aep.planning-md/3
id: review-result:adversary-gaps-229-pass-1
kind: review-result
status: active
title: Adversary pass 1, downstream gaps unit feature-request-229
relations:
- reviews: story:feature-request-229
revision: 1
---
unit: story:feature-request-229, tree gaps-229
verdict: NEEDS-CHANGE, red 2 (introduced)

- warning (synthesize/related_guard.rs:312): `arranging.contains(&entity)` stopped a nested same-entity arrangement, so AcceptCandidate/outcome/parent-not-accepted and PublishRelease/outcome/published got no scenario.
- held: state with stored fields, `in` vs `==`, ess/18 and ess/19 refuse `state` naming ess/20, partition gaps and overlaps, when_subject unchanged, every runtime target refuses the new guard by name.

Correction 1: arrange a related row of the same entity one level deep, fresh in its initial state; refuse deeper with a named bound.
Tests: tests/adversary_229_pass1.rs (conformance 9, domain 7).
