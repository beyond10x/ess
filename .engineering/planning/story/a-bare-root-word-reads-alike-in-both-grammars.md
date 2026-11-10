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
scope:
- confidence: cited
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/tests/checks.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/toggle_state.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/tests/toggle_state.rs
- confidence: cited
  path: crates/ui/ess-ui/src/filter.rs
revision: 8
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

- `ess ui check` refuses the reproduction's row action `te`, placed on the `row_actions_page`
  fixture of `crates/ui/ess-ui-check/tests/checks.rs`, with one finding of code
  `toggle_bind_bare_root` at `pages/p/sections/list/row_actions/te/bind/off`, whose message names
  `draft` and asks for it quoted (`'draft'`) or written as a path; a new case in
  `cargo test -p ess-ui-check --locked --test checks` asserts it with `trips_in`.
- In the same test, the bind `off: "not (row.stage == 'draft')"` (quoted) and the bind
  `on: not row.on` raise no `toggle_bind_bare_root` (`clean_at`).
- With the word quoted, `not (row.stage == 'draft')`, React sends `off: false` on row a and
  `off: true` on row b: a new case in `cargo test -p ess-ui-react --locked --test toggle_state`.
- With the word quoted, the terminal and React draw the mark on for row a and off for row b: a new
  case in `cargo test -p ess-ui-tui --locked --test toggle_state` and one in
  `cargo test -p ess-ui-react --locked --test toggle_state`. This already holds; the cases keep it.

## Decisions

- The check refuses the bare word instead of unifying the two grammars, because the refusal is the
  smaller change and unifying would change how every existing bind value reads a bare word.
- The terminal's dropped parenthesised bind value (below) is not fixed here, so no acceptance bullet
  reads the input the terminal sends.

## Scope

- cited: `crates/ui/ess-ui-check/src/rules.rs` (`toggle_state` :1324, beside `checked_unreadable`)
- cited: `crates/ui/ess-ui/src/filter.rs` (`tokenize` :234 tells `Token::Word` from
  `Token::String`, but `Predicate::Literal` records no quoting; the check needs the bare words)
- inferred: `crates/ui/ess-ui-check/tests/checks.rs` (the refusal)
- inferred: `crates/ui/ess-ui-react/tests/toggle_state.rs` (the sent input and the mark)
- inferred: `crates/ui/ess-ui-tui/tests/toggle_state.rs` (the mark)

confidence: medium — the readings are reproduced; the display roots the check compares against are
the terminal's `ROOTS` (`expr.rs:20`, crate-private), so the check needs its own copy or a list
moved into `ess-ui`.

Shared surfaces: proposes one `CHANGELOG.md` sentence for the new diagnostic; moves no byte pin.

The terminal drops any parenthesised bind value (`expr.rs:65`), quoted or not. On 0.92.0,
`off: "not (row.stage == 'draft')"` is sent as `{id: a}`, while `off: "not row.stage == 'draft'"`
is sent as `{id: a, off: false}` (`ess ui test`, `expect_command`). That is a separate defect found
here, and it needs its own story.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
