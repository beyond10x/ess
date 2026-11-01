---
format: aep.planning-md/3
id: story:a-boundary-row-of-a-deleting-branch-is-sent-for-a-row-that-exists
kind: story
status: draft
title: A boundary row of a deleting branch is sent for a row that exists
summary: Synthesis sends a deleting branch's boundary row for the row its own act deleted; the scenario fails on a conforming target
relations:
- serves: vision:O2
- informed_by: story:an-accepting-branch-below-unknown-instance-is-sent-for-a-stored-row
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
revision: 2
---
## Problem

Found on 2026-10-29 while preparing
`story:an-accepting-branch-below-unknown-instance-is-sent-for-a-stored-row`, and not part of
beyond10x/ess#645. A deleting default branch whose sibling has an orderable input guard is further
witnessed at that guard's boundary. Synthesis sends the boundary row in the same scenario, for the
row the scenario's own act has just deleted, so the target cannot answer the deleting branch.

Minimal model (neutral names): `CloseNote(note_id, count: Integer)` with
`dropped: {when: 'count > 3', accepts: nothing}` and a default `closed` that
`deletes: demo.notes.Note` on `instance: note_id`. `OpenNote` creates a note.

Checked on `ess 0.77.0` (a repro outside the repository), without any `unknown_instance:` branch:

```text
demo v1 against interpreted 0.77.0 — failed
  failed demo.notes.CloseNote/outcome/closed
  passed demo.notes.CloseNote/outcome/dropped
  passed demo.notes.OpenNote/outcome/opened
```

The `closed` scenario opens a note, closes it with `count = 1`, then sends `count = 3` for the same
captured note and expects `closed`. The target reports "no declared outcome was reached". With an
`unknown_instance:` branch declared, it observes `unknown`.

Read in the tree (not confirmed by a build): `boundaries` (`crates/verify/ess-conformance/src/
synthesize.rs:17008`) arranges a fresh row for each boundary only for a `moves:` branch
(`subject.effect.transition().is_some()`, `:17117-17120`; `ResolvedEffect::transition` is `None`
for `deletes:` in `crates/specify/ess-compiler/src/ir.rs:783-787`). Its doc names `moves:` and
`updates:` only.

## Acceptance

- In the model above, the `closed` scenario's boundary send (`count = 3`) goes to a note that
  exists when it is sent, and the scenario passes on the interpreted target.
- A `moves:` or `updates:` branch keeps its boundary arrangement byte for byte, and the existing
  rows of the synthesis bytes tables do not move except where a repository model has a deleting
  branch with a boundary row. Any such row is listed in the fit review.
