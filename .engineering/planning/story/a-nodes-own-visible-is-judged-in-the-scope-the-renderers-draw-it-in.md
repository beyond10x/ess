---
format: aep.planning-md/3
id: story:a-nodes-own-visible-is-judged-in-the-scope-the-renderers-draw-it-in
kind: story
status: draft
title: A node's own visible is judged in the scope the renderers draw it in
summary: ess ui check counts a node's own state as in scope for its own visible, written directly or handed in as a widget argument; the React project draws the condition outside the node's state layer, so the path is not resolved there; one ignored test holds the case
tags:
- backlog
- defect
- ess-ui
- from-finding
- review-finding
- severity-low
relations:
- serves: vision:O2
- informed_by: story:ui-check-resolves-the-root-and-section-of-every-binding-path
revision: 1
---
## Defect

`ess ui check` puts a node's own state in scope for that node's own `visible`. The generated React
project draws a node's condition outside the node's state layer (`crates/ui/ess-ui-react/src/emit.rs`,
`node`: the state layer is inside the hoisted component and `Visible` wraps it;
`core.tsx.tmpl`, `Visible` reads the parent scope), so `visible: state.open` on the node that
declares `open` is not resolved there. The check is silent and the React project shows the node by
a value that is not the node's state.

Two places hold the same scope rule:

- `crates/ui/ess-ui-check/src/filter.rs:276`, for a `visible` written directly on the node;
- `crates/ui/ess-ui-check/src/sites.rs`, `Walker.around`, for a widget argument substituted into
  the `visible` of the body node that declares the state (`visible: args.when` with
  `when: state.open`).

Found by the third and the fourth review pass of
`story:ui-check-resolves-the-root-and-section-of-every-binding-path`. Checked for the React project
only; what the terminal renderer does with a node's own state in its own `visible` is not read.

The case: `crates/ui/ess-ui-check/tests/adversary_expr_paths_pass4.rs`,
`a_state_the_reading_node_declares_does_not_cover_its_own_visible`, marked `#[ignore]` with this
story's id. It pins the stricter reading (the check reports the path) for the widget-argument form.

## What a design has to settle

Which side moves. Either the check holds a node's own `visible` to the scope around the node, as
the React project draws it, and the terminal renderer does the same; or the renderers draw a
node's condition inside its state layer, and the check stays. The reference page says which.

## Acceptance (sketch, to be fixed by the fit review)

- The reference states the scope of a node's own `visible`.
- The check, the React project and the terminal renderer agree on it for the direct form and for a
  widget argument, each with a test that is red before.
- The ignored case passes or is rewritten to the settled reading, and loses its `#[ignore]`.
- No finding moves in the on-disk documents for another cause.

## Fit review

Owed. No reproduction in a rendered project is in the repository; the React half is by reading.
