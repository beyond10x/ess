---
format: aep.planning-md/3
id: approval-record:zero-issues-larger-turns
kind: approval-record
status: draft
title: The rest of the turn plan regrouped into eight larger turns
tags:
- non-interactive
relations:
- decides: epic:zero-open-issues-2026-11
revision: 1
---
## Stop

Changing the turn plan of `epic:zero-open-issues-2026-11` from turn 3 on
(`.engineering/waves/zero-open-issues.md` § The turns), which revises
`approval-record:zero-issues-turn-plan` for every turn after turn 2. A turn plan is the
planner's; the change is put to the operator with the next turn's proposal.

## Reason: non-interactive

No operator turn exists until the goal is met. Operator goal, verbatim (2026-11-09): "zero
remaining issues on github, all bugs fixed from planning store", narrowed to "for repo ess". On
wave size (operator, 2026-11-10): "each processing a whole wave of up to 10 issues (the more the
better) - run no expensive gates i the middle, only at the end of the wave". The run's coordinator
relayed (2026-11-10): turn 1 ran 12 hours (01:55Z to 13:49Z and not yet closed), so 32 turns take
weeks; regroup the remaining stories into at most 9 turns with one gate each.

## Decided in the operator's absence

- Turns 1 and 2 stay as planned. Turns 3 to 32 of the 32-turn plan become 8 turns (3 to 10) of 29
  to 33 stories, each joining consecutive old turns in order (3-4, 5-6, 7-8, 9-11, 12-14, 15-17,
  18-22, 23-32), so the plan's order, dependencies and collisions are kept. That makes about 9
  gates for the remaining stories instead of 31.
- Inside a turn at most 10 implementors run at once and the rest queue; a story starts only when
  every story it depends on is merged and no running story shares a file of its scope.
- The synthesize.rs lane runs up to 3 stories at a time whose scoped symbols do not overlap, each
  in its own tree from the lane head, merged in the listed order after re-running its package
  tests on the lane head; overlapping stories stay serial, and a pair that conflicts twice falls
  back to serial.
- One adversary pass per unit touching guards, synthesis or emitted bytes; a second pass only
  after a blocker; a weak finding is fixed without a further pass; findings outside the story's
  acceptance become backlog drafts.
- The gate is one chain per turn: fmt, clippy and `cargo xtask docs` first, then crate test
  groups under 25 GB with a clean between, `ess-conformance` in test-name groups, then the xtask
  and projection steps. Each implementor runs clippy on its own package, and `cargo xtask docs`
  when it touches `website/docs`, before handing over.
- Turn k+1 implements while turn k's gate runs; only merging and the gate are serial.

## What the operator would have been asked

"Run the rest of the plan as 8 turns of about 30 stories, with a parallel lane and one gate per
turn, instead of 30 more turns of 15?"
