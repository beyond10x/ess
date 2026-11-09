---
format: aep.planning-md/3
id: story:both-renderers-read-one-binding-expression-grammar
kind: story
status: draft
title: 'Both renderers read one expression grammar: and/or in the terminal, a list in a bind, an action''s loaded row'
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- decomposes: epic:zero-open-issues-2026-11
revision: 2
---
## Need

The binding expressions of `ess-ui/1` say less than the schema promises, and the two renderers
read them differently. On ess 0.92.0 (documents in the triage's `repro/`):

- **`and` in the terminal.** The schema's operators are `==, !=, in [..], not, and, or, literals`
  (`schemas/ui/ess-ui.schema.yaml` `expressions.forms`), and the React runtime parses all of them
  (`crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl:116-127`). The terminal's display and
  `visible` evaluator knows `not`, `==` and `!=` only (`crates/ui/ess-ui-tui/src/expr.rs:47-75`), so
  `visible: "state.a == on and state.b == off"` with both true hides the section:
  `ess ui test` fails `section shown is not drawn`; the same test with `visible: "state.a == on"`
  passes.
- **A list in a bind.** `bind: {ids: [row.id]}` is refused,
  `an expression is a scalar, not Sequence [String("row.id")]` (`crates/ui/ess-ui/src/model.rs:37-50`),
  so a command whose input is a list cannot be sent one row's identity.
- **The row an action loads.** A settings toggle in a page header that sends
  `enabled: not <the stored value>` has no expression for the stored value: `Action.loads` declares
  the read, but "no renderer reads its rows for the action's value"
  (`schemas/ui/ess-ui.schema.yaml` `Action.loads`). `bind: {enabled: not loads.enabled}` loads, reads
  `loads.enabled` as literal text, and check warns `toggle_state_unknown`.

## Acceptance

- The terminal evaluates every display, `visible` and `checked` expression with the one parser
  `ess_ui::filter` already holds (`crates/ui/ess-ui/src/filter.rs`), so `and`, `or`, `not`, `in` and
  parentheses mean in the terminal what they mean in React; the `ess ui test` case of the
  reproduction passes (`crates/ui/ess-ui-tui/tests/expressions.rs`). Each position keeps its failure rule: a row filter fails closed (`filter.rs:1-5`), a `visible` outside the grammar still means shown (`crates/ui/ess-ui-tui/src/app.rs:922-926`). A string that reads no path
  stays literal text in both (`expr.rs:9-12`).
- A `bind` value may be a YAML sequence of expressions, sent as a JSON array of their values, as
  decided (G-12 answer a); a sequence anywhere else an expression is expected is refused by path,
  as today (`crates/ui/ess-ui/tests/bind_list.rs`, React and terminal send tests).
- Inside an action that declares `loads`, `loads.<field>` reads that field of the first row the
  read answers, in `bind`, `checked` and `visible`; anywhere else, or on an action without `loads`,
  `ess ui check` refuses `loads.` by path. A `does:` toggle bound `not loads.<field>` shows the
  loaded state, and `toggle_state_unknown` no longer fires for it. With `--model`, `<field>` is
  held to the view's fields.
- `schemas/ui/ess-ui.schema.yaml` `expressions.forms` lists `loads.<field>` and the bind list;
  the reference is a fresh render.

## Scope

- cited: `crates/ui/ess-ui-tui/src/expr.rs`
- cited: `crates/ui/ess-ui/src/filter.rs`
- cited: `crates/ui/ess-ui/src/model.rs`
- cited: `crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl`
- cited: `schemas/ui/ess-ui.schema.yaml`
- inferred: `crates/ui/ess-ui/src/toggle.rs`
- inferred: `crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl`
- inferred: `crates/ui/ess-ui-check/src/rules.rs`
- inferred: `crates/ui/ess-ui-tui/src/app.rs`
- inferred: `docs/design/ui-read-filter.md`
- inferred: `website/docs/reference/ess-ui.md`
- confidence: medium

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 items C17, C20 and C21.

1. **Need.** An author writes one condition and both renderers judge it alike; sends a list input
   from one row; and writes a toggle that flips the stored value its action reads. Reproductions
   above (`repro/run.txt`, `repro/c17.txt`, `repro/c21/`). The requester's proposals, theirs: "an
   expression naming the value a header action loads", "a list literal in binds", "`and` in the
   terminal evaluator".
2. **Class.** C21: *defect* (the schema lists `and`; the terminal ignores it). C20: *gap*, decided.
   C17: *gap*: `loads` exists and is checked against the model, but nothing can read it.
3. **Already expressible?** C21: nesting `visible` on a parent and a child joins two conditions
   only where the node tree allows it, so no. C20: no (refused above). C17: `checked:` can name a
   path, but no path reaches the loaded row; `section.<name>.selection` reads a selection, not a
   row a header action loads.
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). One parser
   for both renderers removes a second dialect rather than adding one; `ess_ui::filter` already
   serves `checked` in the terminal (CHANGELOG 0.86.0, "reads it in the filter grammar"). The list
   reuses YAML structure, as the type rule asks (`schemas/ui/ess-ui.schema.yaml` `type_rule`: "Types
   are YAML structure, never strings to be parsed"). `loads.<field>` is named after the key that
   declares the read, as `does.input_fields` names its command, and is scoped to that action, as
   `row.` is to its row.
5. **Second adopter.** A notification settings page with a header toggle "Paused" over a stored
   preference; a bulk-free "add to group" row action sending `[row.id]`; a section shown only for an
   signed-in reader on a paid plan (`actor.signed_in and state.plan == paid`).
6. **Cost.** Additive in `ess-ui/1`: one expression root and one bind form; the terminal evaluator
   is replaced by the shared parser (its literal-text rule kept); no migration.
7. **Considered.** For C17: (a) a root `section.<name>.row.<field>` reading a section's first row
   (title_from's shape): works only where a section reads the same view; (b) chosen: `loads.<field>`
   on the action that declares the read. For C21: (a) add `and` to `expr.rs`; (b) chosen: one parser,
   so `or`, `in` and parentheses do not drift next. For C20: as decided.

## Decisions

**Accept, redesigned.** C20 as decided (`approval-record:ess-0870-wave`, G-12 answer a). C21 is a
renderer fix by sharing the parser. C17 is answered by a `loads.<field>` root scoped to the action
that declares `loads`, not by a general "stored value" expression. No new format version;
`docs/design/ui-read-filter.md` gains a section on the shared grammar and the `loads` root before
code.

## Source

`github:beyond10x/ess#723`
