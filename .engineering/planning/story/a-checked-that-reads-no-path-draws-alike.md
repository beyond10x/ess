---
format: aep.planning-md/3
id: story:a-checked-that-reads-no-path-draws-alike
kind: story
status: archived
title: A checked that reads no path draws alike in both renderers
summary: 'checked_unreadable is silent on a checked: that reads no path, which React draws on and the terminal evaluates (UI-adopter wave 1 U2 pass-4 note, backlog)'
tags:
- already-fixed
- backlog
- from-finding
- review-finding
- zero-open-issues
relations:
- informed_by: review-result:w086-u2-adversary-pass-4
- decomposes: epic:zero-open-issues-2026-11
revision: 5
transitions:
- {from: "draft", to: "archived", at: "2026-11-09T23:06:47Z", actor: "human:timo", revision: 4}
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

Waved under epic:zero-open-issues-2026-11 (accepted 2026-11-10).


## Verification on 0.92.0 (2026-11-09)

Result: **already-fixed**. Checked against main at 4ab2ea1449 and the 0.92.0 release binary by the
zero-open-issues run (epic `epic:zero-open-issues-2026-11`); this draft leaves the plan as archived.

Proof: verify/a-checked-that-reads-no-path-draws-alike/repro/run.txt (toggles ta, tb, tl); templates/runtime/expr.ts.tmpl:266-277 (holds), :243 (filterForm admits a literal); commit ec2cb77cad (0.89.0)

The acceptance holds on 0.92.0: "a checked that reads no path ... draws the same mark in both renderers". Rows are [{off: true}, {off: false}]. For `checked: "not true"`, `"1 == 2"` and `"not -1"`, the generated React `holds` draws off, off, and the terminal's hint line draws `[ ]` on both rows for Alfa, Bravo and Lima (run.txt).
The draft's premise is that React evaluates a no-path `checked` as literal text (truthy, so on). It predates `holds`, added in 0.89.0 (ec2cb77cad; CHANGELOG 0.89.0: "the React runtime reads a toggle's `checked` state in the filter grammar"). `holds` parses in the filter grammar first, `filterForm` admits literals (:243), and the value is run there. It falls back to `evaluate` only for text the filter grammar refuses, and `checked_unreadable` warns on that text (rules.rs:1333-1345).
The pass-4 cases were never committed, and no test at TREE pins them. A small regression test would be cheap, but nothing is left to fix.

