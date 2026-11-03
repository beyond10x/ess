---
format: aep.planning-md/3
id: story:terminal-header-toggles-show-their-state
kind: story
status: draft
title: Terminal header toggles show their state, and toggle row paths are checked
summary: The terminal draws header toggles without a mark, and checked:/bind row paths are not checked against the view (UI-adopter wave 1 U2 review note, pre-existing, backlog)
tags:
- backlog
- from-finding
- review-finding
relations:
- informed_by: review-result:w086-u2-adversary-pass-1
revision: 1
---
## Need

Two gaps around toggles, pre-existing at 83391ebaeb, found by the UI-adopter wave 1 adversary pass
on the toggle unit (review-result:w086-u2-adversary-pass-1, note):

- The terminal draws a header toggle with no on/off mark (`crates/ui/ess-ui-tui/src/view.rs:425-452`),
  while a row toggle shows `[x]`/`[ ]`.
- A toggle's `checked:` expression and its bind entries may name a `row.<field>` the view does not
  return; `ess ui check` does not compare them with the view's fields.

## Acceptance

- The terminal shows a header toggle's state the way it shows a row toggle's.
- `ess ui check` reports a `checked:` or bind `row.<field>` path the read view does not return,
  at the action's path.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
