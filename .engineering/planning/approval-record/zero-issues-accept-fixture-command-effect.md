---
format: aep.planning-md/3
id: approval-record:zero-issues-accept-fixture-command-effect
kind: approval-record
status: draft
title: A reproduced fixture-mode defect accepted into the running plan with no operator present
tags:
- non-interactive
relations:
- decides: story:fixture-mode-applies-a-confirmed-commands-effect
revision: 1
---
## Stop

Accepting `story:fixture-mode-applies-a-confirmed-commands-effect` into the running plan of
`epic:zero-open-issues-2026-11` (`draft -> proposed -> active`) and placing it in the earliest turn
its collisions allow. A story accepted after the plan was written would wait for the next plan.

## Reason: non-interactive

No operator turn exists. Operator goal, verbatim (2026-11-09): "zero remaining issues on github, all
bugs fixed from planning store", narrowed to "for repo ess". The run's coordinator relayed the
intake (2026-11-10): reproduce on main, file it as a story under the epic (defect), and place it in
the earliest turn its collisions allow, without disturbing a turn in flight.

## Decided in the operator's absence

- The defect reproduces on the 0.92.0 binary and on main 4ab2ea1449 (no `crates/ui` change between
  them): in fixture mode a confirmed command answers done and the filtered view still lists the
  row, in the React runtime (headless run) and in the terminal UI (`ess ui test` fails at the step
  that expects one row). The reproduction is `.engineering/repro/fixture-command-effect/`.
- It is a defect in shipped behaviour, so it is in the epic's scope; it joins the plan in the
  earliest turn after turn 1 whose stories it collides with on no file.

## What the operator would have been asked

"This defect reproduces; accept it into the running plan?"
