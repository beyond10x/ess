---
format: aep.planning-md/3
id: story:a-toggle-shows-the-row-state-it-flips
kind: story
status: active
title: A toggle shows the row state it flips
summary: 'A command toggle derives its state from the not-bind it flips or a checked: expression, and a toggle column renders a switch (D16, B1, D3; beyond10x/ess#723)'
tags:
- ess-ui-adopter-wave-1
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/ui/ess-ui-check/src/lib.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: cited
  path: crates/ui/ess-ui/src/filter.rs
- confidence: cited
  path: crates/ui/ess-ui/src/lib.rs
- confidence: cited
  path: crates/ui/ess-ui/src/model.rs
- confidence: cited
  path: crates/ui/ess-ui/src/toggle.rs
- confidence: cited
  path: schemas/ui/ess-ui.schema.yaml
- confidence: cited
  path: website/docs/reference/ess-ui.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T16:24:11Z", actor: "human:timo", revision: 11}
- {from: "proposed", to: "active", at: "2026-11-03T16:24:11Z", actor: "human:timo", revision: 12}
---
# Story: a toggle shows the row state it flips

## Outcome

A person looking at a list of rows with an on/off toggle sees each toggle on or off as the row
is, whether the toggle sets UI state or runs a command, and a boolean column shown `as: toggle`
reads as a switch instead of the words `true` and `false`.

## Context

A downstream adopter's review of ESS 0.84.0 generated pages (beyond10x/ess#723, items D16, G16, B1,
D3): every command-driven row toggle rendered off, including rows whose field is true. Wave 1 of
the ESS UI-adopter waves (D418, D420), release 0.86.0.

## Acceptance

Named scenarios; each is a test function, and the story is done when each runs and passes.

- `crates/ui/ess-ui/tests/toggle_state.rs`
  - `an_action_takes_a_checked_expression` — `{as: toggle, does: C, bind: {...}, checked: row.enabled}` loads and the typed action carries `checked`.
  - `a_command_toggle_derives_its_state_from_the_bind_it_flips` — with no `checked:`, a toggle whose `bind` has exactly one entry written `not <expr>` has the state `<expr>` (`enabled: not row.enabled` gives `row.enabled`), through one library function both renderers call.
  - `a_sets_toggle_keeps_its_state` — a toggle with `sets:` keeps today's state, the key it sets.
- `crates/ui/ess-ui-check/tests/toggle_state.rs`
  - `checked_on_an_action_that_is_not_a_toggle_is_an_error` — named rule, at the action's path.
  - `a_command_toggle_with_no_known_state_warns` — a `does:` toggle with no `checked:` and no `not` bind entry warns by a named rule (`toggle_state_unknown`) and still loads.
- `crates/ui/ess-ui-react/tests/toggle_state.rs`
  - `a_command_toggle_is_checked_when_the_row_field_is_true` — rows `enabled: true, true, false` render checked, checked, unchecked.
  - `an_explicit_checked_expression_wins` — `checked:` is used over the derived state.
  - `a_toggle_column_renders_a_read_only_switch` — a column `{field: enabled, as: toggle}` renders a disabled checkbox reflecting the value, labelled by the column, not the text `true`.
- `crates/ui/ess-ui-tui/tests/toggle_state.rs`
  - `a_command_toggle_shows_the_row_state` — where the terminal renders a row toggle, it shows on for a true row and off for a false one.
  - `a_toggle_column_shows_a_switch_mark` — a column `as: toggle` renders an on/off mark, not `true`/`false`.
- The reference page (`website/docs/reference/ess-ui.md`) is a fresh render of the schema.

## Out of Scope

- The value a header toggle `loads` (C17) and dropping a read param while a toggle is off (C19);
  the next UI-adopter wave.
- Deriving a toggle's state from a row field named like the action (the adopter's reference fix's
  fallback): a name match is a guess, refused here.

## Problem

`ActionControl` computes a toggle's state only from the first key of `sets`
(`crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl:286-287`); a toggle with `does:` and
`bind:` gets the literal `false`. A column `as: toggle` falls to the default branch of
`FieldValue` (`fields.tsx.tmpl:76-100`), which prints the value as text.

## Fit review

1. **Need.** A toggle that runs a command must show the current state of what it toggles.
   Requester's syntax: "a `checked:`/`state:` expression on a toggle action (`checked: row.enabled`),
   or derive it from the bound field".
2. **Class.** Defect: `as: toggle` is documented as a toggle (`crates/ui/ess-ui/src/model.rs:2417-2418`, `ActionAs::Toggle`), and a toggle
   that never shows on contradicts it.
3. **Already expressible?** Only for UI-state toggles: `sets:` carries the key, and the `toggle`
   shorthand expands to `not <key>` (`crates/ui/ess-ui/src/model.rs:2383-2385`). A command toggle
   has no way to name its state.
4. **Fit.** Both, decided from the grammar: the derivation reuses the `not <key>` rule the `toggle`
   shorthand already writes, so the common case needs no new key; `checked:` covers a command whose
   bind does not flip a row value (two commands, enable and disable, or a value computed elsewhere).
   One Rust function computes the state so the React and terminal renderers agree.
5. **Second adopter.** A feature-flag list whose row toggle runs `SetFlag {id, on: not row.on}`.
6. **Cost.** One optional key on an action in `ess-ui/1` (additive), two check rules, runtime and
   terminal rendering. No format change elsewhere.
7. **Considered.** (a) change nothing: rejected, a defect. (b) `checked:` only: rejected as the
   only form, every existing command toggle would need editing to stop rendering off. (c) derive
   only: rejected as the only form, a non-flipping bind has no state to derive. (d) chosen: derive
   from a single `not` bind entry, `checked:` wins when written, a warning when neither applies.

## Decisions

Accept, redesigned (2026-11-03, wave lead under D420): both of the requester's alternatives, with the
derivation restricted to one `not <expr>` bind entry and no name-based fallback.

## Delivered

UI-adopter wave 1 (D418, D420), release 0.86.0.

- Unit `b8b53b81da` merged at `d29dd12cc3`. Fix round 1 `dd38413e71` merged at `f8b12559cf`, round 2 `a4cef75e13` at `efbf3380d0`, round 3 `6d819f3177` at `976e4e7b07`.
- An action takes `checked:` (only with `as: toggle`, else `checked_needs_toggle`).
- Without `checked:`, a `does:` toggle shows the `<expr>` of its one bind entry written `not <expr>`. `ess_ui::toggle_state` reads that `<expr>` from the parse tree; every string literal is quoted, and every path must start at an admitted root (row, params, state, shell, args). Otherwise `toggle_state_unknown` warns and the toggle renders off.
- The React runtime and the terminal read the same state. The terminal evaluates it in the filter grammar with that grammar's truthiness. React reads negative number literals.
- `checked_unreadable` warns on a `checked:` the filter grammar does not read. A column `as: toggle` renders a switch.
- Every named acceptance scenario passes. Adversary passes 1 to 3 needed revision and were fixed. Pass 4 approved with two INFEASIBLE notes, filed as `story:a-checked-that-reads-no-path-draws-alike` and `story:both-renderers-tokenize-whitespace-alike`.
- Out of scope: a bare root word read alike in both grammars is `story:a-bare-root-word-reads-alike-in-both-grammars`, and terminal header toggles are `story:terminal-header-toggles-show-their-state` (all backlog).
- Scope: confirmed by the diff. `filter.rs`, `toggle.rs`, `lib.rs`, `expr.ts.tmpl` and the check `lib.rs` were added.
