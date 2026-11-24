---
format: aep.planning-md/3
id: story:a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal
kind: story
status: draft
title: A test opens one section, overlay or widget of a document alone, at a given terminal width, and the terminal draws only that node
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A tool that judges one part of a screen (a card, a form, a status cell) must render the whole page
around it today, at one fixed size. Every `ess ui test` test starts from a page: `open` takes
`page`, `params` and `state` only (`crates/ui/ess-ui-test/src/spec.rs:60-68`), and the run draws
into a 160 x 240 backend that no test can change (`crates/ui/ess-ui-test/src/screen.rs:14-16`).
The terminal renderer can draw at any size (`App::render_text(width, height)`,
`crates/ui/ess-ui-tui/src/view.rs:198-216`) but always draws the whole screen (`App::draw`,
`:218-240`); its section and body drawing (`section_lines` at `:687`, `body_lines` at `:981`) is
private. No renderer can be pointed at one node.

What a "component" is in `ess-ui/1`: a composite kind (`collection`, `record`, `form`, `chart`, …,
`schemas/ui/ess-ui.schema.yaml:598-886`) standing as a section's body or nested in one, or an
app-defined widget declared under `widgets` with typed `params` and a `body`
(`schema:887-912`), used through a widget instance with `args` (`schema:914-929`). Each has a
canonical node path: `pages/<p>/sections/<s>`, `pages/<p>/overlays/<o>`, `widgets/<w>`
(`schema:211-251`).

Minimal reproduction, written fresh: a document with a widget `money` (`params: {value:
{type: number}}`, one `text` primitive) used in one table column. No test can draw `money` alone,
and no test can draw any page at 40 columns.

The requester proposed, in their words: "Render one component alone at given widths, with
fixtures." Their own specification records harness widths in pixels on each target.

## Outcome

An `ess-ui-test/1` test may name a terminal width, and its `open` step may name a node instead of a
page: a section of a page (drawn with the page's `params` given in the step), an overlay (with its
`params`), or a widget declaration (with `args` given as literals in the step). The terminal then
draws that node alone in a frame of the given width, reading reads from the fixtures as a page
does. Every other step and check works on the node opened as it does on a page. The Playwright spec
marks such a test `test.fixme` with the reason until `g05-react-node-harness` lands. No styling
construct is added.

## Acceptance

- First: a test file under `crates/ui/ess-ui-test/tests/` with `width: {cells: 40}` and
  `open: {node: widgets/money, args: {value: 1234.5}}`, then `expect: {at: widgets/money/body/amount,
  text: "1,234.5"}` (or the value's formatted text). It fails today when the file is read:
  "unknown key `node`; expected one of page, params, state" (`spec.rs:620`, `known` at `:746-757`).
- `open: {node: pages/<p>/sections/<s>, params: {…}}` draws that section's box alone; its reads,
  its live feed, its row actions and its overlays behave as on the page; `select`, `act`, `type`,
  `expect` at paths under it hold as on the page. A test pins one collection section at width 40
  and width 120, and the column widths differ (`view.rs` `column_widths`).
- `open: {node: pages/<p>/overlays/<o>, params: {…}}` draws the overlay alone.
- A node path of any other kind (a column, a row action, a primitive inside a section) is refused
  by `ess ui test` when the file is read, naming the three kinds it accepts.
- `width: {cells: <n>}` applies to the whole test, a page test included; absent, the run keeps
  160 x 240. A width below the terminal's minimum is refused when the file is read, naming the
  minimum. Height stays 240.
- A widget opened with `args` missing a required param, or with an arg of the wrong type, is
  refused with the message the loader gives for a widget instance (`schema:914-929`).
- In the Playwright spec, a test with `open: {node}` is `test.fixme` with one reason, from
  `ess-ui-test`'s parity rules (`crates/ui/ess-ui-test/src/parity.rs`), and the reference table
  "One verdict in both renderers" lists it.
- `ess-ui-test.md` documents `width` and `open: {node}`; `CHANGELOG.md` names them. No change to
  `ess-ui/1`.
- With `--trace` (`g04-step-trace`), the regions of a node opened alone start at `x = 0`.

## Fit review

Tree `bcfc51555`, read-only.

1. **Need.** Judging one part of a screen alone, at several widths, with sample data. Report item
   5 (the application's G11 bullet "a component harness"). Reproduction above; the requester's
   wording quoted separately.
2. **Class.** *Gap*: no step opens a node, no test sets a size (above).
3. **Already expressible?** Partly: a page holding only the one section, written for the test,
   draws it with the shell's navigation and header around it, at 160 columns. That changes the
   document under test, so it tests another document. No idiom.
4. **Fit.** Grids do not cover `ess-ui-test`.
   - Vocabulary: `open` already opens the thing a test starts from; it takes `node` beside `page`,
     a canonical node path as every step does. `args` is the widget instance's key (`schema:921`).
     `params` is the page's.
   - Siblings: the three kinds are the nodes that carry their own inputs (a section its read's
     params, an overlay its `params`, a widget its `params`). A composite nested under a section
     reads `row` or its parent's state; opening it alone needs a parent context and is refused by
     name (decision P1-D4).
   - Styling is out of scope: nothing here names a theme, a token or a template.
   - Every renderer: the terminal here, React in `g05-react-node-harness`, Playwright `fixme`
     until then.
5. **Second adopter.** A design-system team holds "the status widget never wraps at 30 columns" as
   a test; a reviewer checks a new form section at 60 and 120 columns without building its page.
6. **Cost.** Two optional keys in `ess-ui-test/1` (`width`, `open.node`, `open.args`). An existing
   test file keeps its meaning and its verdict; an older `ess` refuses a file that uses them, as it
   refuses every step added within `/1` (`ess-ui-test.md:49`, unknown keys refused). No version.
   One public entry on `ess_ui_tui::App` (open a node), no `ess-ui` change.
7. **Considered.**
   - (a) Change nothing: refused (question 3).
   - (b) A `harness:` section in the `ess-ui/1` document declaring widths and fixtures per
     component: puts test data into the UI document; refused, tests live in `ess-ui-test/1`.
   - (c) A CLI `ess ui render --node <path> --width <n>` printing the frame: useful, but a second
     way to drive the renderer beside tests; it can come later over the same `App` entry.
   - (d) Chosen: `open: {node}` and a test-level `width`. Widths in pixels are the browser's
     (decision P1-D3).

## Decisions

**Accept, redesigned.** What changed from the request: the harness is a test step, not a document
construct; widths are per renderer unit; nested composites wait.

- **P1-D3. How a width is written.** (A) `width: {cells: <n>, px: <n>}`, each renderer applies its
  own unit and a test that names only the other unit is `test.fixme` there. (B) One number in
  cells; the browser uses a fixed px per cell. (C) Cells only; the browser never runs these tests.
  **Recommended: (A).**
- **P1-D4. Which nodes open alone.** (A) A section, an overlay, a widget declaration. (B) Also any
  composite nested in a section, with a `row` given in the step. (C) Sections only.
  **Recommended: (A).** (B) can follow once a second need appears.

## Scope

- cited: `crates/ui/ess-ui-test/src/spec.rs` (`Step::Open`, `parse` of `open`, test-level keys)
- cited: `crates/ui/ess-ui-test/src/runner.rs` (`open`, `resolve`: a target under the opened node)
- cited: `crates/ui/ess-ui-test/src/screen.rs` (size per test)
- cited: `crates/ui/ess-ui-test/src/parity.rs`, `crates/ui/ess-ui-test/src/playwright.rs` (fixme)
- cited: `crates/ui/ess-ui-tui/src/app.rs` (a public `open_node`), `crates/ui/ess-ui-tui/src/view.rs`
  (`draw` of a single section, overlay or widget body)
- inferred: `crates/ui/ess-ui/src/expand.rs` (expanding a widget for given literal `args`; read
  only if a public entry already exists)
- cited: `website/docs/reference/ess-ui-test.md`; `CHANGELOG.md`
- inferred: a new test module in `crates/ui/ess-ui-test/tests/`
- Size: L (about 550 changed lines).

## Depends on

- None. `g05-react-node-harness` depends on this story.
