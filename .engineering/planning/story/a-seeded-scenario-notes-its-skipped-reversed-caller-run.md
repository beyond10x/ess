---
format: aep.planning-md/3
id: story:a-seeded-scenario-notes-its-skipped-reversed-caller-run
kind: story
status: draft
title: A seeded scenario notes its skipped reversed-caller run
summary: A suite note names a seeded scenario whose fixed caller skips the reversed-caller run
tags:
- ess-0.75.0
relations:
- informed_by: story:a-synthesis-seed-is-accepted-for-a-caller-model
revision: 1
---
## Problem

Raised by the 0.74.0 wave's U2 implementor.

0.74.0 admits a synthesis seed for a model whose actors declare caller attributes. The seeded row
implies the caller of the seeded run. A scenario that would normally also run with the caller
swapped skips that reversed-caller run when the seed fixes the caller. The suite does not say so:
a reader sees no note, and cannot tell that the cross-caller property went unchecked for that
scenario.

## Acceptance

- A seeded scenario that skips its reversed-caller run carries a suite note naming the scenario
  and the seed row that fixed the caller.
- An unseeded model's suite bytes are unchanged.
