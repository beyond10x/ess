---
format: aep.planning-md/2
id: decision-blocker:wave24-unit2-leaves-after-two-passes
kind: decision-blocker
status: open
title: Unit 2 leaves wave 24 after two adversary passes
relations:
- blocks: story:planning-store-carries-workstation-paths
revision: 1
---
# Unit 2 leaves wave 24 after two adversary passes

`story:planning-store-carries-workstation-paths`, branch
`impl/planning-store-carries-workstation-paths`, worktree `wt-5cf844fd8563`, uncommitted over base
`bd722fa964bd225b9755f272e22b45fab334449f`. The work is not lost; the branch stays.

The wave protocol gives an adversary budget of two passes. Both returned NEEDS-CHANGE:

| Pass | Record | Outcome |
|---|---|---|
| 1 | `review-result:adversary-wave24-unit2-pass-1` | falsified the decision's load-bearing sentence |
| correction 1 | `review-result:wave24-unit2-correction-1` | sentence removed, parse widened, 175 cases green |
| 2 | `review-result:adversary-wave24-unit2-pass-2` | CONFIRMED blocker, 3 red cases |

The unit does not merge. It is red at exit 101 on the pinned profile
(`host_paths_adversary_6.rs`, 3 failed), and two of its findings are structural rather than
mechanical.

## Why it cannot be corrected inside this wave

**The blocker is the same defect pass 1 found, one layer in.** Pass 1: the parse read the bullet's
first backtick token and stopped, so every number was unread prose and one of them was false. The
correction widened the parse to the head counts. Pass 2, measured at `host_paths_adversary_6.rs:83`
exit 101: the parse stops at the bullet's **first colon**, and the falsified superlative and the
other five numbers sit after it. The corrected bullet and a bullet carrying the falsified claim
whose reasons contradict its own head both parse to `[(".engineering/", true, 60, 32806)]`.

The module doc at `host_paths.rs:1262` states the widening addressed exactly that — "every number in
the bullet was unread prose. One of those numbers was wrong." The number that was wrong is still
unread. That is a false claim about what the check does, in the check's own documentation.

**The design finding is live, not forecast.** `host_paths.rs:56` pins `60 files, 32806 lines` by
exact equality over the planning store. Measured by the coordinator:

| Tree | Carrying files |
|---|---|
| `wt-5cf844fd8563` at base `bd722fa9` | 60 |
| main checkout with this session's store writes | 64 |

49 of the 60 are `review-result` documents, and every wave records one per adversary pass. This
wave has written four. The lane goes red on its own bookkeeping, and the remedy is an edit to
`host_paths.rs` — which the protocol reserves store writes to the coordinator and forbids the
coordinator the source edit. The trigger and the remedy sit on opposite sides of an ownership
boundary.

A third correction round would have to change the design, not fix a bug, and the pass budget that
would check it is spent.

## What was carried out of it

- `story:the-unread-tree-bullet-is-read-whole` — F1, F2, F3, F4, F6
- `story:the-lane-does-not-pin-a-count-that-its-own-bookkeeping-moves` — F5
- `story:the-metadata-guard-rejects-every-build-but-one` — F7, pre-existing

## Coordinator error recorded against this escalation

I reported unit 2's `--bin ess-xtask` red as not reproducing and offered "never existed or
order-dependent". Pass 2 established it is environment-dependent and deterministic: plain
`cargo test -p ess-xtask --locked` fails on `DEBUG`, a partial pin fails on `NUM_JOBS`, the
Taskfile's full `env` passes. Both my runs used the pinned env. The unit's number was right and I
measured a different build.
