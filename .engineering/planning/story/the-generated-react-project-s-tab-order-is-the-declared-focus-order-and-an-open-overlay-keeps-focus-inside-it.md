---
format: aep.planning-md/3
id: story:the-generated-react-project-s-tab-order-is-the-declared-focus-order-and-an-open-overlay-keeps-focus-inside-it
kind: story
status: draft
title: The generated React project's Tab order is the declared focus order, and an open overlay keeps focus inside it until it closes
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-terminal-s-tab-follows-the-declared-focus-order-and-a-test-presses-tab-and-asserts-the-focused-node-in
revision: 1
---
## Need

The generated React project's focus order is the DOM order of its controls, with a few
`tabIndex={0}` elements (`crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl:299-303`),
and its overlay frame declares `aria-modal="true"` without keeping focus inside or placing it on
open (`templates/runtime/overlays.tsx.tmpl:64-96`, as read). After `g08a-focus-order-steps`, every
browser test with `press` or `focused` is `test.fixme`. Report item 8a, browser half.

## Outcome

In the generated project, Tab and Shift-Tab visit the focusable nodes of the declared order, in that
order; an opened overlay takes focus on its first node, Tab stays inside it, and closing it returns
focus to the node that opened it. The Playwright spec runs `press` and `focused` with no `fixme`.

## Acceptance

- First: the test of `g08a-focus-order-steps`'s first acceptance, emitted with `--playwright`, has
  no `test.fixme`, and a test in `crates/ui/ess-ui-react/tests/` that runs the generated runtime
  finds the declared order's paths, in order, as the tabbable elements' `data-ui-path`.
- With an overlay open, the tabbable elements are the overlay's only; on close, the opener has focus.
- A row set is one Tab stop; the row cursor stays on arrow keys, as the declared rule says.
- No `ess-ui/1` key and no styling construct; the focus outline is the existing `--ui-color-focus`.

## Fit review

Tree `bcfc51555`, read-only. Covers report item 8a with `g08a-focus-order` and
`g08a-focus-order-steps`.

1. **Need.** The browser follows the declared order.
2. **Class.** *Gap* (and, for the overlay, a missing containment that `aria-modal` announces).
3. **Already expressible?** No.
4. **Fit.** Template changes only; the order comes from the rule, not from per-widget code.
5. **Second adopter.** As `g08a-focus-order`.
6. **Cost.** Generated code changes for every project (focus moves on overlay open and close); no
   format change.
7. **Considered.** Positive `tabIndex` values: fragile and against common accessibility guidance;
   refused. Chosen: DOM order matching the rule, `tabIndex={-1}` for nodes the rule leaves out.

## Decisions

- P1-D7, P1-D9 (`g08a-focus-order`).

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/overlays.tsx.tmpl`,
  `crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl`
- inferred: other composite templates under `crates/ui/ess-ui-react/templates/runtime/composites/`,
  `crates/ui/ess-ui-react/src/emit.rs` (section order in a laid-out page)
- cited: `crates/ui/ess-ui-test/src/parity.rs` (the `fixme` removed)
- Size: L (about 450 changed lines).

## Depends on

- `g08a-focus-order-steps`.
