---
format: aep.planning-md/3
id: story:the-one-row-check-decides-a-filtered-view-from-the-replayed-row
kind: story
status: active
title: The one-row check decides a filtered view from the replayed row
summary: 'Which views carry the one-row snapshot is decided from the steps'' bindings: a filtered view that would show the called row is left out, and a model with only a filtered identity view loses the reported shape''s scenario (third review pass, backlog)'
tags:
- backlog
- conformance
- defect
- from-finding
- review-finding
- severity-low
relations:
- serves: vision:O2
- informed_by: story:the-one-row-check-reads-the-instance-its-scenario-bound
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-13T11:02:42Z", actor: "agent:operator-delegate", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-13T11:02:42Z", actor: "agent:operator-delegate", revision: 3}
---
## Finding

The one-row check after an updating branch beside an `unknown_instance:` creation files a snapshot
of the row the scenario's own last call of the command names
(`story:the-one-row-check-reads-the-instance-its-scenario-bound`). Which views carry that snapshot
is decided without replaying the scenario's steps, and the third review pass of that unit found two
cases where the decision is narrower than it could be. The pass found no blocker: every case in
which the unit uses a filtered view files the snapshot 0.94.0 filed.

1. Where the called row and the row a second run of the branch names differ, and the model has
   identity views both with and without a filter, only the views without a filter carry the
   snapshot. A filtered view that would show the called row is left out.
2. For the reported shape in a model that has only a filtered identity view, the scenario is
   withdrawn where 0.94.0 filed a scenario that ended as an error. No scenario replaces it.

Both need the same thing: the row's stored fields after the scenario's steps, to decide whether a
filtered view shows it. The unit decides from the steps' bindings alone.

Where it lives: the one-row check of `crates/verify/ess-conformance/src/synthesize/` as changed by
`4d747041a0`, `6713653b9c` and `c99ace23be`.

## Acceptance (sketch, to be fixed by the fit review)

- A fixture for each case, red before: the filtered view carries the snapshot of the called row
  where the replayed row satisfies the view's filter, and the model with only a filtered identity
  view gets a passing scenario for the reported shape.
- No scenario 0.94.0 filed and passed is lost; the bytes table moves only in the fixtures' rows.

## Fit review

Owed. This draft records two observations of a review pass; no reproduction is in the repository
yet.
