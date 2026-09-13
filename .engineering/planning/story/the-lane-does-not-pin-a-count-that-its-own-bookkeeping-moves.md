---
format: aep.planning-md/1
id: story:the-lane-does-not-pin-a-count-that-its-own-bookkeeping-moves
kind: story
status: draft
title: The lane does not pin a count that its own bookkeeping moves
scope:
- confidence: cited
  path: crates/edge/ess-xtask/tests/host_paths.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/host_paths_lane/mod.rs
revision: 3
---
# The lane does not pin a count that its own bookkeeping moves

`crates/edge/ess-xtask/tests/host_paths.rs:56` states that `.engineering/` is unread and carries
`60 files, 32806 lines`, and the lane asserts both by exact equality against its own detector.

49 of those 60 files are `review-result` documents. Every wave records one review-result per
adversary pass, and those documents quote `file:line` citations and worktree paths, which is what
makes them carry the class.

Measured 2026-09-12:

| Tree | Carrying files |
|---|---|
| `wt-5cf844fd8563` at base `bd722fa9` | 60 |
| main checkout with wave 24's store writes | 64 |

40% of the 122 tracked review-results carry the class. The lane goes red on the next recorded pass,
and the only remedy is editing a number in `host_paths.rs`.

The two sides are separated by ownership. `AGENTS.md:117` reserves store mutation to
`aep plan artifact`, the wave protocol reserves those calls to the coordinator, and the same
protocol gives the source edit to an implementor. The agent that trips the assertion cannot fix it
and the agent that can fix it cannot see it trip.

## Acceptance

Recording a review-result does not turn the host-path lane red, and the lane still refuses a bullet
that understates what the tree carries.

The obvious candidates — a lower bound rather than equality, a count excluded from the bullet, or a
generated bullet — each give up something the exact count buys. Whoever takes this says which and
why.

## Scope

- `crates/edge/ess-xtask/tests/host_paths.rs` — `cited`
- `crates/edge/ess-xtask/tests/host_paths_lane/mod.rs` — `cited`

Related: `story:scrub-the-planning-store-or-say-why-not` asks whether the store should carry these
paths at all. If it is scrubbed, this story changes shape but does not go away — the counts still
move.
