---
format: aep.planning-md/3
id: decision-blocker:ess-059-wave-scope
kind: decision-blocker
status: cleared
title: Units of wave 2026-10-10i for 0.59.0
relations:
- blocks: release-plan:ess-059
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-10T18:36:35Z", actor: "human:timo", revision: 3}
---
## Question

Which units make wave 2026-10-10i, released as 0.59.0? Open issues on 2026-10-10: https://github.com/beyond10x/ess/issues/493, https://github.com/beyond10x/ess/issues/496, https://github.com/beyond10x/ess/issues/515, https://github.com/beyond10x/ess/issues/521, https://github.com/beyond10x/ess/issues/525.

## Options

- **A.** Three units, run one after another (one build at a time): i1 `story:related-guard-on-a-captured-struct-input-member` (Fixes #521; `synthesize/related_guard.rs`); i2 `story:authored-act-states-caller-attributes` (`authored.rs`, the scenario schema and its reference page; no issue); i3 `story:synthesize-stored-field-guards-without-a-view` (Fixes #496; `synthesize/subject_fact.rs`, `synthesize.rs` note). #525 is triaged without a build: a comment, a draft story naming it, closed as planned. Issues after release: 2 (#493, #515).
- **B.** i1 and i2 only, as release-plan:ess-059 lists them; #496 and #525 stay open. Issues after release: 4.
- **C.** A plus `story:mutate-covers-subject-related-guards-sets-and-authored` (#515), three more serial units in `mutate.rs`. Issues after release: 1; the wave about doubles in length.

## Recommendation

A: it brings the count under the cap of 3 with one extra unit whose fit review is written and whose arrangement already exists in synthesis.

## Decided

A, with https://github.com/beyond10x/ess/issues/525 kept open. Wave 2026-10-10i is three serial units, one build at a time: i1 `story:related-guard-on-a-captured-struct-input-member` (Fixes #521), i2 `story:authored-act-states-caller-attributes`, i3 `story:synthesize-stored-field-guards-without-a-view` (Fixes #496), one integration branch and one pull request, then 0.59.0 after a green full pre-release suite. #525 gets a draft story naming it and stays open until that story is delivered: a consumer's request is not closed before it ships.
