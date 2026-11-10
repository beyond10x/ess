---
format: aep.planning-md/3
id: story:a-system-step-default-states-every-input-its-siblings-read
kind: story
status: draft
title: A system step's default branch states every input its siblings' guards read
summary: A default branch on a system-issued step is awaited without the inputs read inside a when_subject predicate, beside a held-state guard, or by when_related's via input (turn 1 stored-guard adversary, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:zi-w1-guard-adv-1
revision: 1
---
## Need

Turn 1 made `guard` in `crates/verify/ess-conformance/src/synthesize/system_issued.rs` return the
input half of `when_subject:`, `when_related:` and row-set guards
(story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes). Its adversary
(review-result:zi-w1-guard-adv-1) showed three shapes where the default branch's await still
states nothing, so a correct target can take another branch and fail the scenario:

1. a `when_subject:` predicate that itself reads an input (`paused == input.hold`), refused with the
   story's own reason but not stated for the default;
2. an input `when:` beside `when_subject_state:`, `SubjectField` or `StateChange` (no arm in
   `guard`);
3. the via input of `when_related:` (`input.owner_id`): the `exists: false` sibling is avoided only
   when the identity names a held row. The interpreter tries only held rows and masks it.

The three red cases are kept in the turn scratch as `backlog-red-cases.rs`.

## Acceptance

- `guarded_input` collects every input path a sibling branch's condition reads: the via input,
  inputs read by a row-set `where:` or a `when_subject` predicate, and the input guard beside a
  held-state condition.
- Each of the three shapes above synthesizes a default await stating those inputs, and passes
  against the interpreted target.

## Scope

- `crates/verify/ess-conformance/src/synthesize/system_issued.rs` (cited by the review)
