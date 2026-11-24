---
format: aep.planning-md/3
id: story:the-generated-react-project-draws-one-section-overlay-or-widget-alone-at-a-given-width-and-the-playwright-spec
kind: story
status: draft
title: The generated React project draws one section, overlay or widget alone at a given width, and the Playwright spec runs the tests that open one
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal
revision: 1
---
## Need

After `g05-terminal-node-harness`, a test that opens one node runs in the terminal and is
`test.fixme` in the Playwright spec, because the generated React project draws only whole pages
behind routes: `page.goto` opens a page by its route (`website/docs/reference/ess-ui-test.md:450-460`).
Widths in pixels, which the requester records per target, exist only in the browser. Report item 5,
browser half.

## Outcome

The generated React project serves a harness route that draws one node alone (a section of a page
with given params, an overlay with given params, a widget declaration with given literal args),
with the fixtures it uses for pages. The Playwright spec opens that route for `open: {node}` and
sets the viewport to `width.px`, so a test that names `px` runs in the browser and holds or fails
there as in the terminal. The route is present only in a fixture-mode build. No styling construct
is added.

## Acceptance

- First: the test file of `g05-terminal-node-harness`'s first acceptance, with `width: {cells: 40,
  px: 360}`, emitted with `--playwright`, no longer marks the test `test.fixme`; a test in
  `crates/ui/ess-ui-react/tests/` builds the generated runtime and renders `widgets/money` alone
  through the harness entry under `node`, as `read_filter.rs` runs the generated runtime, and finds
  one element with `data-ui-path="widgets/money"` and none for any page.
- A section opened alone keeps its `data-ui-path` (`pages/<p>/sections/<s>`), so every step's
  selector is the one a page test uses.
- The harness route is not emitted in a build bound to a real backend; a generator test pins its
  absence.
- A test naming only `cells` stays `test.fixme` in the spec with the reason of P1-D3 (A), and one
  naming only `px` is `test.fixme` in the terminal with the same reason, from one place in
  `ess-ui-test`'s parity rules.
- The reference rows "In the browser" and "One verdict in both renderers" (`ess-ui-test.md`) are
  updated; `CHANGELOG.md` names it.

## Fit review

Tree `bcfc51555`, read-only.

1. **Need.** As `g05-terminal-node-harness`, in the browser, in pixels.
2. **Class.** *Gap*.
3. **Already expressible?** No: no route draws less than a page.
4. **Fit.** The generated project already knows every node path (`data-ui-path`,
   `ess-ui-test.md:495`); the harness reuses the page components and the fixture player. No
   `ess-ui/1` key; no styling.
5. **Second adopter.** A component gallery for reviewers, generated from the document's widgets,
   one harness URL per widget and width.
6. **Cost.** One generated route and one Playwright emission branch. No format change beyond
   `g05-terminal-node-harness`.
7. **Considered.** Storybook-style stories generated per widget: a third-party toolchain in every
   generated project; refused. Chosen: one route over the existing components.

## Decisions

- P1-D3 and P1-D4, recorded in `g05-terminal-node-harness`.

## Scope

- cited: `crates/ui/ess-ui-react/src/emit.rs` (page emission; a harness entry beside it)
- cited: `crates/ui/ess-ui-react/src/assets.rs` (routes and runtime files)
- inferred: `crates/ui/ess-ui-react/templates/runtime/` (a harness template)
- cited: `crates/ui/ess-ui-test/src/playwright.rs`, `crates/ui/ess-ui-test/src/parity.rs`
- cited: `website/docs/reference/ess-ui-test.md`; `CHANGELOG.md`
- inferred: a new test in `crates/ui/ess-ui-react/tests/`
- Size: L (about 500 changed lines).

## Depends on

- `g05-terminal-node-harness`.
