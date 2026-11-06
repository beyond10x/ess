---
format: aep.planning-md/3
id: story:a-column-may-show-a-plain-string-row
kind: story
status: active
title: 'A column or record field may show a plain-string row with field: .'
summary: 'Gap B4-40 (G-51): field: . names the row itself'
tags:
- queue-0.91.0
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:19:02Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-06T22:19:03Z", actor: "human:timo", revision: 3}
---
## Need

Gap item B4-40 (G-51): a view whose rows are plain strings. A choice already offers such a row as
both value and label (`Choice::option`, `crates/ui/ess-ui/src/model.rs`), and since 0.89.0 an
`as: tags` input's `suggests` takes it too. A column or record field still has no way to name the
row itself, so a list of names cannot be shown as a table.

## Decision (lead, under the 0.90.0 approval)

`field: .` names the row itself, in a collection column and in a record field. On a row that is
an object, it reads as an absent value. Choice options keep their current rule.

## Acceptance

- A collection whose read answers `["alpha", "beta"]` with one column `field: .` draws two rows,
  `alpha` and `beta`, in React and in the terminal.
- `ess ui check --model` accepts `field: .` on a read whose row type is a string.
- `ess ui check --model` refuses `field: .` by name on a read whose rows are entities.
- Sort by that column orders the strings. Row keys fall back to the row itself.
- The `ess-ui/1` reference page states the rule.

## Scope

- inferred: `crates/ui/ess-ui/src/model.rs` (Field), `crates/ui/ess-ui-check/src/model.rs`,
  `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (`display`),
  `crates/ui/ess-ui-tui/src/`, `website/docs/reference/ess-ui.md`.
