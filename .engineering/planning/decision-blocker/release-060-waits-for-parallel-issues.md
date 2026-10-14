---
format: aep.planning-md/3
id: decision-blocker:release-060-waits-for-parallel-issues
kind: decision-blocker
status: cleared
title: Release 0.60.0 waits for the issues fixed on a parallel branch?
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-14T17:54:49Z", actor: "human:timo", revision: 3}
---
## Question

Does release 0.60.0 wait for the fixes of #542-#547, #549, #551 and #552? Another branch of the operator's is fixing those, but at 2026-10-14T13:1xZ it has no pull request open, and all nine issues are open.

## Options

- A: release 0.60.0 from `main` once the 15-issue pass (#575) merges. The nine ship in 0.61.0.
- B: hold 0.60.0 until that branch's pull request merges, so one release closes every open issue.
- C: take the nine issues into this plan as units of 0.61.0, which needs their draft stories.

## Recommendation

A. Two releases are cheaper than holding 15 shipped fixes for an unknown time.

## Decision

Option A, decided by the operator on 2026-10-14 at 15:21Z ("tag + gh release"): release 0.60.0 now from `main`, at the merge of #575. #542-#547, #549, #551 and #552 ship in a later release.
