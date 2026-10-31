---
format: aep.planning-md/3
id: story:an-unguarded-branch-shadowing-a-guarded-sibling-is-reported
kind: story
status: draft
title: A guarded branch an unguarded sibling always shadows is reported
summary: Validate accepts a held state where an unguarded branch makes a guarded one unreachable
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

In one held state, a branch with no `when:` beside a guarded branch makes the guarded branch
unreachable (`selected_in_state` sees two selected), and `ess specify validate` accepts the command
anyway. Found by the wave D335 U2 implementor while reproducing an adopter finding on 0.77.0.

## Acceptance

- Validate refuses or warns about a guarded branch that an unguarded sibling in the same held state
  always shadows, naming both branches.

## Decisions

None yet.
