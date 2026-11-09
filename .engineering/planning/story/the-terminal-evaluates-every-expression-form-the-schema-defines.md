---
format: aep.planning-md/3
id: story:the-terminal-evaluates-every-expression-form-the-schema-defines
kind: story
status: draft
title: The terminal evaluates every expression form the schema defines
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#722
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/journey_visible.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/expr.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/tests/visible.rs
revision: 4
---
## Need

The terminal renderer evaluates `visible:` conditions and action binds in a grammar narrower than
the one `ess-ui/1` defines, and skips `visible:` in two places, so a journey test sees what the
browser hides and sends what the browser does not (beyond10x/ess#722 items 20, 21, 22, on 0.84.0;
reproduced on 0.92.0):

- a header action's `visible:` is never applied: the header draws every action
  (`crates/ui/ess-ui-tui/src/view.rs:440-460` draws `header.actions` with no visibility check),
  so both "Start editing" and "Stop editing" show (item 20);
- a field's `visible:` is not applied to the fields of a record overlay (item 21);
- the terminal's expression evaluator has no `and`, `or` or `in`
  (`crates/ui/ess-ui-tui/src/expr.rs:47-78`), so `row.status == closed and row.customer != nobody`
  is split at its first `==` and read false, and the row action it guards is hidden (item 22);
- found in the same reproduction: `selection` is not one of the evaluator's roots
  (`expr.rs:20-22`), so a bulk bind `{ids: selection.id}` is sent as the text `"selection.id"`.

The schema lists `and`, `or`, `in` and `not` among the expression operators and `selection.<field>`
among the roots (`website/docs/reference/ess-ui.md` § Expressions); the React runtime evaluates
`and` and `or` (`crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl:125-127`), and the terminal
already reads toggle state in the shared filter grammar `ess_ui::filter`
(`crates/ui/ess-ui/src/filter.rs:43`, `:116`), which has them.

Minimal reproduction on 0.92.0:

```yaml
header:
  actions:
    - {name: edit_on, sets: {state.mode: edit}, label: Start editing, visible: "state.mode == view"}
    - {name: edit_off, sets: {state.mode: view}, label: Stop editing, visible: "state.mode == edit"}
# list row_actions:
    - {name: reopen, does: example.order.Reopen, bind: {id: row.id}, label: Reopen,
       visible: "row.status == closed and row.customer != nobody"}
# drawer record overlay fields:
    [id, customer, {field: secret_note, label: Secret note, visible: "row.status == closed"}]
# bulk_actions:
    - {name: archive2, does: example.order.Archive2, bulk: true, bind: {ids: selection.id}}
```

```
failed  item 20 ... expected no "Stop editing" at pages/orders.list ... [Start editing] [Stop editing] ...
failed  item 21 ... expected no "hidden-1" at pages/orders.list/overlays/detail
failed  item 22 ... act .../rows/o-2/row_actions/reopen: ... the row does not offer Reopen
failed  found ... command example.order.Archive2 was sent with {ids="selection.id"}, not with {ids=["o-1"]}
```

## Acceptance

Journey tests in `crates/ui/ess-ui-test/tests/`, each red on 0.92.0, and a unit test per form in
`crates/ui/ess-ui-tui`:

- A header action whose `visible:` is false is not drawn and is not offered by the palette;
  `expect: {at: pages/<p>, not_text: Stop editing}` holds and turns after the toggling action runs.
- A field whose `visible:` is false is not drawn in a record overlay, a form overlay, a record
  section or a collection, wherever a field is drawn.
- `visible:`, `bind:` and `checked:` are read in one grammar with `and`, `or`, `not`, `in`, `==`,
  `!=` and every root the schema lists; `row.status == closed and row.customer != nobody` is true on
  a closed row, and its row action is offered.
- A bulk bind `selection.<field>` sends the list of that field over the selected rows
  (`{ids: [o-1]}`).
- A condition outside the grammar still means shown, as today (`crates/ui/ess-ui-tui/src/app.rs:921-926`).

## Scope

- cited: crates/ui/ess-ui-tui/src/expr.rs
- cited: crates/ui/ess-ui-tui/src/app.rs
- cited: crates/ui/ess-ui-tui/src/view.rs
- inferred: crates/ui/ess-ui-tui/tests/visible.rs
- inferred: crates/ui/ess-ui-test/tests/journey_visible.rs
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** The terminal must hide what a document declares hidden and send what its binds say, as
   the browser does; a test file has one verdict in both renderers
   (`website/docs/reference/ess-ui-test.md` § One verdict in both renderers). Reproduction in § Need.
   Requester's ask: a step or a statement per item; no syntax proposed.
2. **Class.** Defect: the schema defines `visible` on header actions (`ess-ui.md` § Action,
   `visible`), on fields (§ Field), the `and`/`or`/`in` operators and the `selection` root
   (§ Expressions); the terminal contradicts each.
3. **Already expressible?** A conjunction can be avoided by nesting guards only where two nodes
   nest; a header action's visibility cannot be tested at all. No idiom.
4. **Fit.** No authored surface. One evaluator for every expression position replaces the
   terminal's two (the display evaluator in `expr.rs` and the filter grammar used for toggle state,
   `app.rs:904` and the toggle-state comment at `app.rs:1753-1762`). Related drafts on grammar
   agreement (story:a-bare-root-word-reads-alike-in-both-grammars,
   story:both-renderers-tokenize-whitespace-alike) touch the same evaluator; this story does not
   take their cases. The language grids do not apply.
5. **Second adopter.** A ticket list whose "Escalate" row action shows only for
   `row.priority == urgent and row.assignee != null`, and whose header switches between "Show
   archived" and "Hide archived".
6. **Cost.** Terminal only; no format change. A document whose terminal run drew a hidden header
   action now hides it, which is the documented behaviour.
7. **Considered.** (a) Change nothing and document the terminal's narrower grammar: rejected, it
   breaks the one-verdict rule. (b) Add `and`/`or` to `expr.rs` alone: rejected, it keeps two
   grammars in one renderer. (c) Chosen: read every expression in the shared grammar and apply
   `visible:` everywhere the schema allows it.

## Decisions

Accept, redesigned (triage of beyond10x/ess#722, 2026-11-09): items 20, 21 and 22 and the
`selection.<field>` bind become one terminal fix, every expression position read in the grammar the
schema defines.

## Source

github:beyond10x/ess#722
