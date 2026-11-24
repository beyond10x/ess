---
format: aep.planning-md/3
id: story:a-ui-test-asserts-that-an-action-opened-an-external-address-in-the-terminal-and-in-the-browser-without-a
kind: story
status: draft
title: A UI test asserts that an action opened an external address, in the terminal and in the browser, without a network request
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-action-leads-to-an-address-outside-the-document-so-a-button-a-header-action-or-a-menu-item-needs-no-stub
revision: 1
---
## Need

Once an action can lead outside the document (`lp2-action-href`), a test must be able to say that
it did. Today `ess-ui-test/1` asserts the effects an action has: a command sent (`expect_command`)
and a view exported (`expect_export`) (`website/docs/reference/ess-ui-test.md:256-281`). An external
address leaves no trace on the screen that a step could read, so a test can neither pass nor fail
on it.

- The terminal records an export and a test reads it (`crates/ui/ess-ui-test/src/runner.rs:1189-1230`,
  `crates/ui/ess-ui-tui/src/app.rs:756`). `lp2-action-href` adds `App::visits()` beside it.
- The Playwright spec refuses `expect_export`: "the spec does not capture the read an export makes"
  (`crates/ui/ess-ui-test/src/playwright.rs:625-627`). A browser does see the new tab that
  `window.open` makes, so a visit can be captured there.
- `act` at a primitive in a section's `children` is refused in the terminal: "the terminal acts on
  actions of sections, rows, overlays and the header" (`runner.rs:653`). A `button` primitive's
  action is therefore not reachable by `act` today, and the reproduction's hero button is not
  either.
- The test step makes no network request (the organization's `AGENTS.md` § Tests stay fast). A browser
  that followed the address would make one.

Minimal reproduction, written fresh and not run, on the document of `lp2-action-href`:

```yaml
steps:
  - open: home
  - act: pages/home/header/actions/source
  - expect_visit: {href: "https://example.org/source"}
```

The step is refused as an unknown keyword when the file is read.

## Outcome

`ess-ui-test/1` has a step `expect_visit: {href}`. It holds when an action with that address has run
since the test started. It follows the shape of `expect_export`.

- In the terminal, it reads `App::visits()`. A failure lists the addresses visited, or says none
  was.
- In the browser, the generated spec routes the address to an aborted answer before the steps run,
  so no request leaves the machine. It waits for the `popup` whose URL is the address. Under LP-D6
  (B), a same-tab open, it waits for the navigation request instead.
- `act` reaches an action inside a section's child, at `…/children/<name>/action` for a `button`
  primitive, in both renderers. The terminal draws the child's action as a key hint on the
  section's box, as it draws a section's own actions. A `link` primitive stays a node that `expect`
  reads. Acting on it is refused by one parity rule with one reason in both renderers.

## Acceptance

1. The reproduction passes in the terminal and in the generated Playwright spec. Red today: the
   step is an unknown keyword.
2. With `expect_visit: {href: "https://example.org/other"}`, the step fails in both renderers. The
   terminal's failure names `https://example.org/source` as the address visited.
3. Before the `act`, `expect_visit` fails with "no external address was opened".
4. The Playwright spec run makes no network request. Its routes abort every request to the
   address. A test that counts the requests reaching the network sees none.
5. `act: pages/home/sections/hero/children/repo/action` runs the button's action in both renderers,
   and `expect_visit` holds for `https://example.org/repo`. `act` at a `link` primitive's path is
   refused in both renderers with the same reason, never with `no node at`.
6. `ess-ui-test.md` has a section `expect_visit`, and its `act` section names a child's button
   action. The step is listed wherever `expect_export` is listed (`spec.rs:137-180`, `:527`).
   `CHANGELOG.md` names it.
7. Format. `ess-ui-test/1` gains a step and keeps its version. A test file without the step reads
   and runs as before, so every committed `*.test.yaml` and corpus test gives the same report. An
   older `ess` refuses the step as an unknown keyword, as it refuses every step added within
   `ess-ui-test/1`.
8. Each new test runs in under 5 s, none may run over 60 s, and each test step stays within 120 s.
   The Playwright case runs in the lane that already runs the generated specs, under its own 60 s
   `timeout`.

## Fit review

Tree `bcfc51555` (branch `integration/wave-2026-11-24`, workspace 0.98.0). Received 2026-11-24 from
a downstream application, fifth list, item 2 (the test side).

1. **Need.** A test fails when a button no longer leads where it should.
2. **Class.** *Gap*. No step reads an external visit, and no step reaches a child's button.
3. **Already expressible?** No. `expect` reads cells, and a visit draws none in the browser.
4. **Fit.**
   - It is named for the effect, as `expect_command` and `expect_export` are, and compares as they
     do.
   - The parity rule keeps one verdict for both renderers.
   - Reaching a child's button closes a hole that predates this feature. The terminal already
     draws the child (`crates/ui/ess-ui-tui/src/view.rs:805-817`).
5. **Second adopter.** Any document with a help button or a status-page link in its header tests
   that the link still leads to the right address.
6. **Cost.** One step in `ess-ui-test/1`, no version. One addressing rule for a child's action.
7. **Considered.**
   - (a) No step: Rust tests only. Rejected: an adopter's own suite could not hold its links.
   - (b) Refuse in Playwright, as `expect_export` does. Rejected: the browser can capture a popup
     without the network, so refusing would leave the browser half of the parity empty.
   - (c) Chosen: `expect_visit` in both renderers.

## Decisions

**Accept.** It is the test side of `lp2-action-href`. It depends on LP-D6 for how the browser
opens the address, and on LP-D7 for the terminal, which records and opens nothing. It has no
decision of its own.

## Scope

- cited: `crates/ui/ess-ui-test/src/spec.rs` (`Step` 137-180, parse 527), `crates/ui/ess-ui-test/src/runner.rs`
  (`expect_export` 1189-1230, `act` 566-660), `crates/ui/ess-ui-test/src/playwright.rs`
  (625-627), `crates/ui/ess-ui-test/src/parity.rs` (366)
- cited: `crates/ui/ess-ui-tui/src/view.rs` (section children 805-817: a child's action hint)
- inferred: a journey test in `crates/ui/ess-ui-test/tests/` (a module of the crate's one test
  binary once `story:the-test-step-meets-the-test-budget` has merged them)
- inferred: `website/docs/reference/ess-ui-test.md`, `CHANGELOG.md`
- Size: M, about 300 changed lines.

## Depends on

- `lp2-action-href`, for `App::visits()` and the action.
- `story:the-terminal-addresses-every-node-the-browser-addresses` (active) changes the same
  addressing code in `runner.rs`, `screen.rs` and `parity.rs`. One lands after the other.
- P0's `g01-session-actions` lists the actions offered after each step. Once both land, an `href`
  action is listed there like any other.

## Source

A downstream application, 2026-11-24, fifth list, item 2.
