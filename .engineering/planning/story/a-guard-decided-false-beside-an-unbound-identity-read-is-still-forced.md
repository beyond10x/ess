---
format: aep.planning-md/3
id: story:a-guard-decided-false-beside-an-unbound-identity-read-is-still-forced
kind: story
status: draft
title: A guard decided false beside an unbound identity read is still forced
summary: Synthesis skips forcing a guard that reads an unbound identity even when another conjunct decides it false (turn 1 lane 3 adversary pass 2, origin undecided, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:zi-w1-lane3-adv-2
revision: 1
---
## Need

Turn 1 made synthesis force a refusal whose external guard relates an input list to a stored list
(story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed, beyond10x/ess#727).
Its second adversary pass (review-result:zi-w1-lane3-adv-2) found that a guard which also reads an
unbound identity is never forced, even when another conjunct already decides it false. With the
narrowed guard `all: [requires == input.missing, origin == input.id]` and an admitted row whose
`requires` differs from `missing`, the guard is false whatever the id is, yet no scenario forces the
refusal, and a target that ignores the narrowing passes all 13 scenarios. The skip in
`crates/verify/ess-conformance/src/synthesize.rs` (force path, around line 4392) drops any guard
reading an unbound identity before three-valued evaluation can decide it. The root cause is
`unknown_by_absence` (`crates/verify/ess-conformance/src/synthesize/subject_fact.rs`), which reads a
field `flatten_partial` left unbound as absent. Whether this predates the turn was not run, so the
origin is undecided.

The red case is kept in the turn scratch as `lane3-backlog-red-case.rs`.

## Acceptance

- A guard that reads an unbound identity beside a conjunct that decides it false is evaluated, not
  skipped, and its refusal is forced.
- A target that ignores the narrowing in the model above fails at least one synthesized scenario,
  and the interpreted target passes them all.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs` (cited by the review: the force skip)
- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (cited by the review: `unknown_by_absence`)
