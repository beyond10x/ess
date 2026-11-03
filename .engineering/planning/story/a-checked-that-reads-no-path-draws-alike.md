---
format: aep.planning-md/3
id: story:a-checked-that-reads-no-path-draws-alike
kind: story
status: draft
title: A checked that reads no path draws alike in both renderers
summary: 'checked_unreadable is silent on a checked: that reads no path, which React draws on and the terminal evaluates (UI-adopter wave 1 U2 pass-4 note, backlog)'
tags:
- backlog
- from-finding
- review-finding
relations:
- informed_by: review-result:w086-u2-adversary-pass-4
revision: 1
---
## Need

`checked_unreadable` is silent on a `checked:` that reads no path. The terminal and React then draw different marks. UI-adopter wave 1 U2 adversary pass 4 found this (review-result:w086-u2-adversary-pass-4, INFEASIBLE note, introduced). The red cases are hand-built expressions; no input from a downstream adopter is known to reach them.

- `crates/ui/ess-ui-check/src/rules.rs:1264`: the rule warns only when the filter grammar refuses the text or reads a path from a root it does not admit. `not true`, `1 == 2` and `not -1` read no path, so it is silent.
- `crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl:380`: React's `evaluate` returns an expression that reads no path as its literal text, which is truthy, so it draws every row on. The terminal evaluates the same text in the filter grammar and draws it off.

Cases from pass 4, not committed (React; the terminal twin expects the terminal's marks):

```rust
const UNWARNED: &[&str] = &[
    r#"{name: ta, as: toggle, label: Alfa, does: posts.SetT, bind: {id: row.id}, checked: "not true"}"#,
    r#"{name: tb, as: toggle, label: Bravo, does: posts.SetT, bind: {id: row.id}, checked: "1 == 2"}"#,
];
// rows [{off: true}, {off: false}]: the terminal draws off, off; React draws on, on.

// negative_literals_mark_each_row_as_the_terminal_does, entry tl:
//   checked: "not -1"  -> the terminal off on both rows, React on on both rows
```

The check case `a_checked_the_renderers_disagree_on_warns` asserts that `checked_unreadable` warns on `"not true"` and `"1 == 2"`.

## Acceptance

- A `checked:` that reads no path either warns, or draws the same mark in both renderers.
- The cases above pass.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
