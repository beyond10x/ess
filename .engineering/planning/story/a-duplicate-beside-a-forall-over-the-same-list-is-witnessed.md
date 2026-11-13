---
format: aep.planning-md/3
id: story:a-duplicate-beside-a-forall-over-the-same-list-is-witnessed
kind: story
status: draft
title: A duplicate beside a forall over the same list is witnessed
summary: 'A refusal guarded by not distinct over a list and a forall over a member of the same list is refused as GuardUnsatisfiable: the middle element of the three-element duplicate is built at its own path and breaks the forall'
tags:
- backlog
- conformance
- defect
- from-finding
- review-finding
- severity-low
relations:
- serves: vision:O2
- informed_by: story:refusal-overlap-witnessed-for-distinct-guard
revision: 1
---
## Defect

A refusal guarded by a duplicate in a list (`not distinct items by name`) that also holds a
quantifier over a member of the same list (`forall i in items: i.kind == "a"`) is refused by
synthesis as `GuardUnsatisfiable`. The duplicate synthesis builds has three elements, the last
repeating the first one's key, and its middle element is built at its own path
(`crates/verify/ess-conformance/src/witness.rs:4597`), so its `kind` is `items.1.kind` and the
`forall` does not hold.

Found by the second review pass of `story:refusal-overlap-witnessed-for-distinct-guard`. The
refusal is the same at that unit's base; the unit's first commit, which tried two whole copies,
witnessed the branch, and its fix returned to the three-element duplicate the repository's
`distinct` witnesses require. No committed model reaches it. The review ran the case on the three
commits and left no test in the repository.

## Acceptance (sketch, to be fixed by the fit review)

- A fixture, red before: a command with such a guard gets a scenario for the refusal whose send
  holds the duplicate between elements that are not neighbours and satisfies the `forall`.
- The three-element duplicate stays decisive: `expression_distinct` passes unedited.
- The bytes table moves only in the fixture's rows.

## Fit review

Owed.
