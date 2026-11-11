---
format: aep.planning-md/3
id: story:a-subject-route-refusal-names-every-creator-whatever-its-cause
kind: story
status: draft
title: A subject-route refusal names every creator whatever its cause
summary: On the when_subject route, creators failing for different kinds of cause fall back to naming the first creator only (turn-2 adversary, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:zi-t2-lane3-adv-1
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:an-arrangement-refusal-names-the-cause-of-every-creator-tried` made an ESS-SYNTH-004 refusal
name every creator tried with its own cause. On the `when_subject` route
(`crates/verify/ess-conformance/src/synthesize/subject_fact.rs`, `search_under`) the list is built
only when every creator failed with `InstanceRequired`. When one creator fails with
`InstanceRequired` and another with `GuardUnsatisfiable` (a `Record` that can create a `Decision`
only of kind `Erratum` beside a `Confirm` no input reaches), the refusal falls back to the first
creator's cause and names only `Confirm`, hiding the repair on `Record`: the misdirection of #728
again. Found by the turn-2 adversary (`review-result:zi-t2-lane3-adv-1`); the base keeps only the
first cause too, so it is pre-existing.

A note from the same review: `across_targets` in `synthesize.rs` merges per-target lists by
(creator, cause), so a creator blocked differently in two target states is named twice and the
text does not say which target state each entry is for, against its doc comment.

## Acceptance

- `adv728_a_creator_whose_rows_miss_the_subject_guard_is_named_too` in
  `crates/verify/ess-conformance/tests/arrangement_refusal_names_every_creator.rs` runs (no
  `#[ignore]`) and passes: the refusal names `Record` with its guard cause and `Confirm` with its
  instance cause.
- A refusal with one creator, or where every creator fails for the same cause, keeps its text.
- A creator named for two target states says which state each cause is for, or the doc comment of
  `across_targets` says what the text does.

## Scope

- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (`search_under`)
- `crates/verify/ess-conformance/src/synthesize.rs` (`Unreachable::of_creators`, `across_targets`)
- `crates/verify/ess-conformance/tests/arrangement_refusal_names_every_creator.rs`
