---
format: aep.planning-md/3
id: story:the-ui-test-runner-acts-on-an-action-inside-an-overlay
kind: story
status: active
title: The ui test runner acts on an action inside an overlay of every kind
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-07T21:42:29Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-07T21:42:29Z", actor: "human:timo", revision: 3}
---
## Need

Defect B11-1, reported by a downstream frontend on 0.90.0: `ess ui test` cannot `act` on or
`upload` to an action placed inside an overlay, for every overlay kind (dialog, drawer, fullscreen,
popover) and either overlay component (record or form). The step fails with
`<path>/actions/<name>: the overlay does not offer <label>`, while `ess ui check` reports no error
for the same document. It blocks that consumer's delivery.

Cause, read in the source: the runner finds an overlay action by its key hint on a screen line
(`crates/ui/ess-ui-test/src/runner.rs`, the overlay branch near `:584-597`, and `hint_key` at
`:1401`). The terminal renderer draws every overlay kind inside a bordered block
(`crates/ui/ess-ui-tui/src/view.rs`, the overlay block near `:647-695`), so each line of the overlay
starts with the border and the hint never matches. No ESS test acts on an overlay action, so
nothing caught it.

Minimal reproduction: a page whose header action `opens: up`, and an overlay `up` (kind `dialog`,
component `record`) holding `{name: start, upload: {accept: [text/csv], does: <a command>}, label:
Start upload}`. Journey: open the page, act on the header action, then `upload: {at:
pages/<p>/overlays/up/actions/start, file: x.csv}` and `expect_command <the command>`.

## Fit review

1. **Need, not syntax.** The request names no new syntax: a documented step (`act`, `upload`) on a
   documented node (an overlay action) does not work. Adopted as a defect.
2. **Generic.** The fix is in how the runner reads an action's key hint from any bordered region,
   not a special case for one overlay kind; the same lookup serves sections and rows.
3. **Format.** No `ess-ui/1` or journey format change.
4. **Cost.** `ess-ui-test` runner and its tests; possibly the terminal renderer if the hint is
   better exposed than scraped. Size S.

## Decision

Adopted as reported: a defect fix in the next wave, with a test per overlay kind.

## Acceptance

- For each overlay kind (`dialog`, `drawer`, `fullscreen`, `popover`) and each overlay component
  (`record`, `form`), a journey that opens the overlay and `act`s on an action inside it executes
  that action and its `expect_command` passes.
- The reproduction above passes: `upload` at `pages/<p>/overlays/up/actions/start` sends the
  command.
- An action name the overlay really does not offer is still refused with
  `the overlay does not offer <label>`.
- Acting on a section or row action, which already works, gives the same result as before.

Separately reported and already filed elsewhere: an `upload` action's `bind:` is dropped by the
runner (B4-47). Not part of this story.

## Scope (inferred)

`crates/ui/ess-ui-test/src/runner.rs` (`hint_key` and the overlay branch), its tests under
`crates/ui/ess-ui-test/tests/`; possibly `crates/ui/ess-ui-tui/src/view.rs`; `CHANGELOG.md`.
Collides with story:a-client-paged-collection-offers-sizes-and-a-range on `runner.rs` (its
`key_of` change), so it runs in the wave after that one.
