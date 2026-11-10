---
format: aep.planning-md/3
id: story:a-graph-editor-draws-its-edges-and-a-menu-per-node
kind: story
status: active
title: A graph editor draws its edges and a menu per node
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/graph_editor.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/tests/graph_editor.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/graph_reads.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/src/playwright.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/graph_confirm.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:38:09Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:38:11Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A `graph_editor` section draws its nodes and its edges as two lists of text, and every node action
as a button on every node. On ess 0.92.0 (`ess generate ui --target react`, fixture rows
`{id: s1, label: Start}`, `{id: s2, label: End}`, `{id: e1, from_step: s1, to_step: s2}`):

- each node is a list item with every `node_actions` entry drawn inline
  (`crates/ui/ess-ui-react/templates/runtime/composites/graph_editor.tsx.tmpl:67`); the schema calls
  `node_actions` and `edge_actions` the "context menu of a node" and "of an edge"
  (`schemas/ui/ess-ui.schema.yaml` `graph_editor.fields`);
- each edge is a list item reading `Start → End` (`graph_editor.tsx.tmpl:79-80`); no line joins two
  nodes, so a flow reads as a list, not a graph.

## Acceptance

Named tests in `crates/ui/ess-ui-react/tests/graph_editor.rs`:

- `nodes_and_edges_are_drawn_as_a_graph`: nodes are placed in layers by a deterministic layout
  (longest path from the nodes no edge enters, an edge closing a cycle ignored for layering, ties in row order) and each edge is an SVG line or
  path between its two nodes, labelled by `edges.kind_by` when declared; the same rows give the same
  drawing.
- `node_actions_are_a_menu`: a node shows one menu control; opening it (click or the context-menu
  key) lists `node_actions`, and choosing one runs it with the node as `row`; `edge_actions` likewise
  on an edge. Each action keeps the path it is addressed by today (`…/node_actions/<name>` within the node's row path `…/rows/<key>`), so `ess-ui-test/1` steps and
  the Playwright export address it as today.
- The edge list stays in the page as the accessible text of the drawing (one entry per edge).
- `degrades: {no_graph_editor: collection}` and the terminal's drawing are unchanged.

## Scope

- cited: crates/ui/ess-ui-react/templates/runtime/composites/graph_editor.tsx.tmpl (every node action is drawn inline on every node at 67-69; each edge is a text list item at 79-80; the only markup is two `<ul>` lists at 56 and 75; layout, SVG edges and both menus land here, styling inline)
- cited: crates/ui/ess-ui-react/tests/graph_editor.rs (new; the acceptance names it and its two tests)
- inferred: crates/ui/ess-ui-test/src/playwright.rs (an `act` on `…/rows/<key>/node_actions/<name>` falls through to a direct click at 545 and 560, and `node_kind` at 740-772 has no menu case; a closed menu needs its control clicked first)
- inferred: crates/ui/ess-ui-test/tests/graph_confirm.rs (checks the Playwright spec for `rows/s-2/node_actions/remove` at 77-100)
- inferred: crates/ui/ess-ui-react/tests/graph_reads.rs (reads nodes by `ui-graph-node`, the label as `n.children[0]`, edge text from `ui-graph-edge` at 81-89)

confidence: medium — the template and the new test are read from the story and the tree; the two `ess-ui-test` lines depend on how the menu is built (whether a closed menu renders its items), a design choice not written anywhere (story-scoper, 2026-11-10)

Correction to the earlier pass: `crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl` is removed. Node positions are per row and go inline or as SVG attributes (precedent `chart.tsx.tmpl:69`, `core.tsx.tmpl:335-338`); the existing `.ui-menu`/`.ui-menu-item` rules (`styles.css.tmpl:71-73`) already style a pop-up menu, as the account menu uses them (`shell.tsx.tmpl:150-161`); and `styles.css` ships in every generated project (`assets.rs:166-169`), so editing it would move the `PLAIN_PROJECT` hash pin (`live_binding.rs:165`). `tests/graph_editor.rs` moves from inferred to cited; `graph_confirm.rs` and `graph_reads.rs` are added.

Shared surfaces this story moves (epic § Shared surfaces, not scope entries): `CHANGELOG.md`. No byte pin moves while `styles.css.tmpl` is untouched.

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 items B3 and "a flow map renders without edges".

1. **Need.** A reader of a flow sees which step leads where, and a node's actions do not crowd every
   node. Reproduction above (`repro/run.txt`). The requester's words, theirs: "node actions render on
   every card instead of a context menu", "a flow map renders without edges".
2. **Class.** *Defect*: the schema says context menu (`graph_editor.fields.node_actions`), and a
   "graph of nodes and edges" (`graph_editor.tsx.tmpl:9`) drawn without its edges draws less than it
   declares.
3. **Already expressible?** No; the drawing is the renderer's.
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). No key: the
   menu and the edges are what the existing keys say.
5. **Second adopter.** An approval workflow editor; a state machine viewer.
6. **Cost.** One runtime template and its styles; no key, no format change. The layout is computed
   in the runtime from the rows, so nothing persisted changes.
7. **Considered.** (a) Change nothing. (b) A layout library: the generated project's dependencies are
   React only (`ess generate ui --help`: "react and react-dom only"). (c) Chosen: a small layered
   layout in the template.

## Decisions

**Accept as proposed.** The requester asked for the drawing the schema already describes. No new
format version.

## Source

`github:beyond10x/ess#723`
