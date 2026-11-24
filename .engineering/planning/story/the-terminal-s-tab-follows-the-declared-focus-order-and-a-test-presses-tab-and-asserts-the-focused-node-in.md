---
format: aep.planning-md/3
id: story:the-terminal-s-tab-follows-the-declared-focus-order-and-a-test-presses-tab-and-asserts-the-focused-node-in
kind: story
status: draft
title: The terminal's Tab follows the declared focus order, and a test presses Tab and asserts the focused node in both renderers
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-keyboard-focus-order-of-a-ui-state-is-declared-by-the-document-and-read-from-a-traced-test-run-the
revision: 1
---
## Need

After `g08a-focus-order`, the declared order is exposed, but the terminal's Tab still cycles only
the navigation and the sections (`crates/ui/ess-ui-tui/src/app.rs:2856-2880`), and no test can press
a key or assert focus (`crates/ui/ess-ui-test/src/lib.rs:43-58`, `spec.rs:195-208`). A test cannot
hold a renderer to the order. Report item 8a, renderer half.

## Outcome

The terminal's Tab and Shift-Tab move through the declared focus order of the state shown, and the
other keys keep their meaning. `ess-ui-test/1` gains a step `press: tab | backtab` and a check
`focused: true` at a path; the terminal runs both, the Playwright spec emits `keyboard.press` and a
check that the active element carries that `data-ui-path`, `test.fixme` in the browser until
`g08a-focus-order-react` lands.

## Acceptance

- First: a test that opens a list page, presses Tab four times and expects `focused: true` at the
  node the declared order puts fifth. It fails today when the file is read (unknown step `press`).
- Shift-Tab from the first node reaches the last; with an overlay open, Tab never leaves the
  overlay's nodes, and its close action is in the order.
- Every existing terminal test stays green (letter keys, row cursor, form editing keep their keys).
- `press` accepts `tab` and `backtab` only; other keys are refused when the file is read, naming
  the two.
- The reference `ess-ui-test.md` documents `press` and `focused`, lists the browser `fixme` in "One
  verdict in both renderers", and the terminal table row for `press`.

## Fit review

Tree `bcfc51555`, read-only. Covers report item 8a with `g08a-focus-order` and
`g08a-focus-order-react`.

1. **Need.** Holding a renderer to the declared order in a test.
2. **Class.** *Gap*.
3. **Already expressible?** No: no key step, no focus check.
4. **Fit.** `press` is the one step that names a key; the reference avoids key names elsewhere, so
   it is limited to the two keys the order defines. `focused` joins `checked` as a check on the
   control.
5. **Second adopter.** As `g08a-focus-order`.
6. **Cost.** One step and one check in `ess-ui-test/1`, no version (existing files keep meaning;
   an older `ess` refuses the new step). Terminal key handling changes for Tab only.
7. **Considered.** A general `press: <any key>` step: binds tests to terminal key letters the
   browser does not share; refused.

## Decisions

- P1-D7, P1-D9 (`g08a-focus-order`).

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs` (`cycle_focus`, `handle`)
- cited: `crates/ui/ess-ui-test/src/spec.rs`, `crates/ui/ess-ui-test/src/runner.rs`,
  `crates/ui/ess-ui-test/src/lib.rs` (`STEPS`), `crates/ui/ess-ui-test/src/playwright.rs`,
  `crates/ui/ess-ui-test/src/parity.rs`
- cited: `website/docs/reference/ess-ui-test.md`, `CHANGELOG.md`
- Size: L (about 500 changed lines).

## Depends on

- `g08a-focus-order`.
