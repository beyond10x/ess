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
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/graph_editor.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/src/playwright.rs
revision: 6
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

- cited: `crates/ui/ess-ui-react/templates/runtime/composites/graph_editor.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl`
- inferred: `crates/ui/ess-ui-react/tests/graph_editor.rs`
- inferred: `crates/ui/ess-ui-test/src/playwright.rs`
- confidence: medium

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
