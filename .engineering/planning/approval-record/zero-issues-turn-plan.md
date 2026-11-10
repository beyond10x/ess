---
format: aep.planning-md/3
id: approval-record:zero-issues-turn-plan
kind: approval-record
status: draft
title: The wave plan reordered into turns with one serial synthesize.rs lane
tags:
- non-interactive
relations:
- decides: epic:zero-open-issues-2026-11
revision: 1
---
## Stop

Reordering the accepted 54-wave plan of `epic:zero-open-issues-2026-11` into turns
(`.engineering/waves/zero-open-issues.md` § The turns). A wave plan is the planner's; the
reordering is put to the operator with the next wave's proposal.

## Reason: non-interactive

No operator turn exists until the goal is met. Operator goal, verbatim (2026-11-09): "zero
remaining issues on github, all bugs fixed from planning store", narrowed to "for repo ess". On
wave size (operator, 2026-11-10): "each processing a whole wave of up to 10 issues (the more the
better) - run no expensive gates i the middle, only at the end of the wave". The run's coordinator
relayed the operator-side planner's instruction (2026-11-10): 54 waves with 54 full gates is too
slow; keep the plan's order and collisions and run fewer, wider turns, with the stories that edit
`synthesize.rs` as one serial lane.

## Decided in the operator's absence

- The turns are the operator-side planner's reordering of the 54-wave plan (2026-11-10): the
  stories that edit `crates/verify/ess-conformance/src/synthesize.rs` run serially in one lane,
  which goes against no printed collision, because colliding stories stay serial; every other story
  keeps its collisions apart and its dependencies earlier.
- Per turn: a lane of up to 5 stories (one `aep:implementor` on sonnet, one tree, one commit and
  one red/green proof per story, then one `aep:adversary` per story on the lane branch), up to 10
  parallel units, one gate on the integration branch after everything of the turn is merged.
- At those caps the plan is 32 turns (the planner estimated about 11; the dependency chains of the
  218 stories outside the lane set the count). Turn 1 takes the reproduced bugs first: #765, #766,
  the #705-#740 group and `story:metric-error-state-on-failed-read`.
- At most three trees hold a built target at once, the lane's counting as one (goal rule 13).

## What the operator would have been asked

"Run the plan as 32 turns of one serial synthesize.rs lane plus ten parallel units, instead of 54
waves?"
