---
format: aep.planning-md/3
id: story:the-ui-test-runner-acts-on-an-action-inside-an-overlay
kind: story
status: active
title: A journey step acts on every action the terminal draws, by the key recorded at its path
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#722
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- depends_on: story:the-terminal-draws-a-host-slot-as-a-labelled-region
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/ui/ess-ui-test/src/parity.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/src/playwright.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/runner.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/journey_actions.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: website/docs/reference/ess-ui-test.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-07T21:42:29Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-07T21:42:29Z", actor: "human:timo", revision: 3}
---
## Need

`ess ui test` cannot run an action the terminal draws outside a section's or a row's own hint
line. The runner finds an action by scraping its key hint from screen lines
(`crates/ui/ess-ui-test/src/runner.rs:544-650`, `hint_key` at `:1415-1442`) and refuses or silently
misses every other place an `ess-ui/1` document may put an action:

- an action of an overlay of any kind (dialog, drawer, fullscreen, popover) and either component
  (record, form): every overlay is drawn in a bordered block, so the hint line starts with the
  border and never matches (`the overlay does not offer <label>`); found as B11-1 on 0.90.0 and as
  beyond10x/ess#722 item 13 on 0.84.0;
- an action in a row's `item` node (#722 item 14): `<path> is not an action`;
- a section child button's `action` (#722 item 15): `<path> is not an action`;
- the `actions` of a collection drawn as an overlay's body (#722 item 16): the terminal does not
  draw them, so the overlay offers nothing to press;
- an action of a row's `expand` detail (#722 item 25): `act` passes and nothing is sent
  (`sent: []`), so the state bind it carries cannot be pinned.

`ess ui check` reports no error for any of them. Minimal reproduction on 0.92.0 (a page whose list
has `row_actions: [{name: open, opens: edit}]`, `item: [{name: go, primitive: button, label: Ship it,
action: {name: ship, does: example.order.Ship, bind: {id: row.id}}}]` and an `expand` record with
`actions: [{name: plain, does: example.order.Plain, bind: {id: row.id}}]`; a drawer form `edit` with
`actions: [{name: void, does: example.order.Void}]`; a record section with `children: [{name: home,
primitive: button, label: Go home, action: {name: home, navigate: {to: orders.list}}}]`; a drawer
whose component is a collection with `actions: [{name: add, opens: create, label: Add line}]`):

```
failed  item 13 ... act pages/orders.list/overlays/edit/actions/void: ... the overlay does not offer Void order
failed  item 14 ... act pages/orders.list/sections/list/rows/o-1/item/go: ... is not an action
failed  item 15 ... act pages/orders.detail/sections/oops/children/home: ... is not an action
failed  item 16 ... expected "Add line" at pages/orders.list/overlays/lines
failed  item 25f ... expect_command example.order.Plain: command example.order.Plain was never sent; sent: []
```

## Acceptance

Journey tests in `crates/ui/ess-ui-test/tests/`, each red on 0.92.0:

- For each overlay kind (`dialog`, `drawer`, `fullscreen`, `popover`) and component (`record`,
  `form`), `act` at `pages/<p>/overlays/<o>/actions/<a>` runs the action and its `expect_command`
  holds; `upload` at such an action sends its command (the B11-1 reproduction).
- `act` at `…/rows/<key>/item/<node>` (a button primitive with an `action`) sends the action's
  command with the row's binds.
- `act` at `…/sections/<s>/children/<node>` runs the child button's action (`navigate` opens the
  page; `does` sends the command).
- A drawer whose body is a collection draws its `actions`; `act` at `…/overlays/<o>/actions/<a>`
  runs one (`opens` shows the overlay it names).
- `act` at `…/rows/<key>/expand/actions/<a>` sends the command with its binds, a `state.<name>`
  bind at the state's value.
- An action name the place does not offer is refused naming the place (`the overlay does not offer
  <label>`, `the row does not offer <label>`), and `act` never passes without pressing the
  action's key.
- Acting on a section, row, header or bulk action gives the result it gives on 0.92.0 (the existing
  `journey_controls` tests stay green).
- The Playwright spec emits each as a click on `[data-ui-path="<path>"]`; where the React project
  draws no element at that path, the test is `test.fixme` with the terminal's reason, through the
  parity rules (`crates/ui/ess-ui-test/src/parity.rs`).

## Scope

- cited: crates/ui/ess-ui-test/src/runner.rs
- cited: crates/ui/ess-ui-tui/src/app.rs
- cited: crates/ui/ess-ui-tui/src/view.rs
- inferred: crates/ui/ess-ui-test/src/parity.rs
- inferred: crates/ui/ess-ui-test/src/playwright.rs
- inferred: crates/ui/ess-ui-test/tests/journey_actions.rs
- inferred: website/docs/reference/ess-ui-test.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A documented step (`act`) at a documented node (an action, at any position the schema
   gives one) must run it; today only section, row, header, bulk and form actions run
   (`runner.rs:544-650`). Reproduction: § Need, `ess ui test` on
   0.92.0. Requester's ask: "a step ... for each, or a statement that the item is out
   of scope" (#722 § Asked).
2. **Class.** Defect. The reference says `act` "runs an action: a row's action, a header action, a
   section's action, a bulk action" and an overlay's actions (`website/docs/reference/ess-ui-test.md`
   § act), and the node path grammar addresses `item`, `children` and `actions` everywhere
   (`website/docs/reference/ess-ui.md` § NodePath). Item 16 is a renderer defect: a collection's
   `actions` (`ess-ui.md` § collection) are not drawn in an overlay.
3. **Already expressible?** No: no step or path reaches these actions (§ Need). The only
   workaround is to move the action to a section or header, which changes the page under test.
4. **Fit.** One mechanism instead of one rule per place: the terminal already records a region per
   drawn node by path (`crates/ui/ess-ui-tui/src/app.rs:59-71`, `Region`) and marks each row action
   at its hint (`view.rs:1044-1080`). Recording the key that runs an action on its region, wherever
   it is drawn, lets `act` press the key recorded at the path rather than scrape a line. It adds no
   authored surface. The language grids (`docs/design/language-grids.md`) do not apply: no ESS
   specification construct changes.
5. **Second adopter.** A support desk console whose ticket drawer has a "Close ticket" action and
   whose ticket rows carry an inline "Assign to me" button in their item template.
6. **Cost.** A key field on `ess_ui_tui::Region` (a library type, not a format), the runner's `act`
   path, the collection-in-overlay draw. No format version: `ess-ui-test/1` and `ess-ui/1` are
   unchanged (`website/docs/reference/formats.md:196-198`).
7. **Considered.** (a) Change nothing and document the places as out of scope: rejected, each is a
   documented node and the refusal hides commands from mutation audits. (b) Strip the border before
   matching the hint (the earlier scope of this story): rejected as the whole fix, it repairs
   overlays only and keeps text scraping, which also breaks on labels the hint cuts. (c) Chosen: the
   terminal records each offered action's key at its path, and `act` presses it.

## Decisions

Accept, redesigned (triage of beyond10x/ess#722, 2026-11-09): the earlier overlay-only fix
(B11-1) is widened to every action position (#722 items 13, 14, 15, 16, 25); the runner presses the
key the terminal records at the action's path. Proposed title: "A journey step acts on every action
the terminal draws, by the key recorded at its path". The B4-47 `bind:` on an upload action stays
filed elsewhere.

## Source

github:beyond10x/ess#722
