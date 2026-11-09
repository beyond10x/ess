---
format: aep.planning-md/3
id: story:a-bare-root-word-reads-alike-in-both-grammars
kind: story
status: draft
title: A bare root word reads alike in the filter and display grammars
summary: 'A bare draft/selection/section in a toggle bind or checked: is text to the filter grammar and a path to React (UI-adopter wave 1 U2 fix-round-2 note, pre-existing, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:w086-u2-adversary-pass-2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

A bare root word reads differently in the two expression grammars. UI-adopter wave 1 U2 found it
in fix round 2 (review-result:w086-u2-adversary-pass-2). It predates the wave, because the filter
grammar and the display grammar were already split.

The toggle's mark no longer differs. Since 0.89.0 both renderers read a toggle's state in the
filter grammar, where a bare word is text: React's `holds` and `filterLiterals`
(`crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl:266, 280`), and the terminal's `toggle_on`
(`crates/ui/ess-ui-tui/src/app.rs:1764`). The command input is still split. A bind value is read in
the display grammar, where `draft`, `selection`, `section` and the other roots are paths
(`expr.ts.tmpl:10` `ROOTS`, `crates/ui/ess-ui-tui/src/expr.rs:20`):

- `ess_ui::toggle_state` derives the mark from a bind `off: "not (row.stage == draft)"` as
  `row.stage == 'draft'`, the text reading.
- React sends `off` with `evaluate` (`actions.tsx.tmpl:136-141`): `draft` is the root `draft`,
  undefined on a list page, so `off` is `true` on every row.
- The terminal's display `eval` (`expr.rs:43-75`) returns nothing for a parenthesised expression, so
  the terminal drops `off` from the input.
- `ess ui check` is silent.

Reproduction on 0.92.0. Rows `[{id: a, stage: draft}, {id: b, stage: live}]`, toggle row action
`{name: te, as: toggle, label: Echo, does: blog.posts.SetT, bind: {id: row.id, off: "not (row.stage == draft)"}}`:

| | row a | row b |
|---|---|---|
| mark, terminal and React (`row.stage == 'draft'`) | on | off |
| `off` the text reading implies | `false` | `true` |
| `off` React sends | `true` | `true` |
| `off` the terminal sends | absent | absent |

## Acceptance

- `ess ui check` refuses, at the action's `bind/<key>` path, a bind value that `toggle_state` reads
  as a state and that holds a bare word spelled like a display root. The message names the word
  and asks for it quoted (`'draft'`) or written as a path. Or else both grammars read the bare
  word one way, and the table above shows `false`/`true` for both renderers.
- With the word quoted, `not (row.stage == 'draft')`, React sends `off: false` on row a and
  `off: true` on row b (a React test). The terminal sends the same once it evaluates a
  parenthesised bind value. Today it drops one even when quoted (probe below), which is a separate
  defect.
- The same state on the same rows renders the same mark in React and in the terminal. This already
  holds; a test keeps it.

## Scope

- cited: `crates/ui/ess-ui-check/src/rules.rs` (`toggle_state` :1324, beside `checked_unreadable`)
- cited: `crates/ui/ess-ui/src/filter.rs` (the parser records no quoting; the derived state)
- cited: `crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl` (`bound` :136)
- cited: `crates/ui/ess-ui-tui/src/expr.rs` (`eval_expr` :47, `None` on `(` at :65)
- inferred: `crates/ui/ess-ui-check/tests/checks.rs` (the refusal)
- inferred: `crates/ui/ess-ui-tui/tests/` and `crates/ui/ess-ui-react/tests/toggle_state.rs` (the sent input)

confidence: medium — the readings are reproduced; whether to refuse or to unify the grammars is a
decision. A refusal is the smaller change.

The terminal drops any parenthesised bind value (`expr.rs:65`), quoted or not. On 0.92.0,
`off: "not (row.stage == 'draft')"` is sent as `{id: a}`, while `off: "not row.stage == 'draft'"`
is sent as `{id: a, off: false}` (`ess ui test`, `expect_command`). That is a separate defect found
here, and it needs its own story.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
