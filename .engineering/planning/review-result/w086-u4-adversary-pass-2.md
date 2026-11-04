---
format: aep.planning-md/3
id: review-result:w086-u4-adversary-pass-2
kind: review-result
status: active
title: 'UI-adopter wave 1: adversary pass 2 of unit U4 (load error path)'
relations:
- reviews: story:a-refused-node-is-reported-at-its-own-path
revision: 1
---
# UI-adopter wave 1: adversary pass 2 of unit U4 (load error path)

Unit: `story:a-refused-node-is-reported-at-its-own-path`. Pass 2 attacked fix commit `086515559c`, at integration `082647b2d2`, in tree `ess-w086-adv-a`.

## Verdict

Approve, with no blocker. Pass 2 ran 24 -> 36 cases; 5 are red:
- 3 are in scope, and all three are INFEASIBLE notes.
- 2 are out of scope and pre-existing.

Every committed case is green.

## In-scope notes (introduced; INFEASIBLE, no fix in this wave)

- `export_selection` treats an action as bulk when the second-to-last segment of its path is `bulk_actions`. An action under a node that an author named `bulk_actions` therefore passes; no input found reaches this.
- A title of whitespace only draws a blank line in the terminal and a blank heading in React. The two renderers agree; no input found reaches this.

## Pre-existing, out of scope

- A map key `action` that is author data with a map value is still read as an Action (`locate.rs:154`, as at base 83391ebaeb). Examples: a field named `action` in a state type's record, or a nested export param.
- Filed as a backlog draft story.

## Attacked, not broken

- `states` and `states.empty` placed anywhere but a page section: in overlays, widget bodies, children, item, header and row_actions.
- Document-map names that collide with schema keys.
- A section merged from a page kind.
- `bulk_actions` reached through page kinds and overlays.
- `ExportParams` given null, a list, a number, a nested map, a non-string key and a tagged value.

The red cases are archived in scratch and not committed.

```findings
[
{"file":"crates/ui/ess-ui-check/src/rules.rs","line":52,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"export_selection reads bulk-ness from the path's second-to-last segment, so an action under a node named bulk_actions may read selection.<field> unrefused"},
{"file":"crates/ui/ess-ui-tui/src/view.rs","line":565,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"a whitespace-only empty-state title draws a blank title line in the terminal and a blank h3 in React (core.tsx.tmpl:454); the two agree with each other"},
{"file":"crates/ui/ess-ui/src/locate.rs","line":154,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"out of scope: a map key `action` that is author data (a state type's record field, a nested export param) with a map value is read as an Action and hides or misnames the real refusal"}
]
```
