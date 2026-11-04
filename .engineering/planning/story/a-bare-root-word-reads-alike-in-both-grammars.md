---
format: aep.planning-md/3
id: story:a-bare-root-word-reads-alike-in-both-grammars
kind: story
status: draft
title: A bare root word reads alike in the filter and display grammars
summary: 'A bare draft/selection/section in a toggle bind or checked: is text to the filter grammar and a path to React (UI-adopter wave 1 U2 fix-round-2 note, pre-existing, backlog)'
tags:
- backlog
- from-finding
- review-finding
relations:
- informed_by: review-result:w086-u2-adversary-pass-2
revision: 1
---
## Need

A bare root word reads differently in the two expression grammars. UI-adopter wave 1 U2 found it in fix round 2. It predates the wave, because the filter grammar and the display grammar were already split:

- In a bind written `not (row.stage == draft)` or a `checked: row.stage == draft`, the filter grammar (`crates/ui/ess-ui/src/filter.rs`) reads the bare `draft` as text.
- React's display grammar reads `draft` as the root `draft`. `selection` and `section` behave the same way.
- So a toggle state written back by `ess_ui::toggle_state` becomes `row.stage == 'draft'`. React evaluates the bind's own `draft` as a path, and the terminal compares it with the text.
- The filter parser does not record whether a word was quoted, so the toggle code cannot tell the two readings apart.

## Acceptance

- One reading of a bare root word holds in both grammars, or `ess ui check` refuses the ambiguous form at the action's path.
- The same state on the same rows renders the same mark in React and in the terminal.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
