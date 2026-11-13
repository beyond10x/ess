---
format: aep.planning-md/3
id: story:a-creating-branch-beside-a-positive-distinct-under-a-count-bound-is-witnessed
kind: story
status: draft
title: A creating branch beside a positive distinct under a count bound is witnessed
summary: With a refusal guarded by defined(items), items.count < 2 and a positive distinct, synthesis refuses the creating branch as ChildUnwitnessed; so before the unit whose review found it
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

For a command whose refusal is guarded by `{all: [defined(items), items.count < 2, distinct items
by name]}` (a positive `distinct` beside a count bound below two), synthesis refuses the creating
branch as `ChildUnwitnessed`.

Found by the second review pass of `story:refusal-overlap-witnessed-for-distinct-guard`, on that
unit's base, its first commit and its fix alike, so the unit does not cause it. The overlap of the
pair is sent or noted, which the unit's pins hold. The review ran the case on the three commits
and left no test in the repository; the cause is not read.

## Acceptance (sketch, to be fixed by the fit review)

- A fixture, red before: the creating branch of such a command gets its scenario.
- The cause is named in the story before a change is made.

## Fit review

Owed. The model is the review's; no reproduction is in the repository yet.
