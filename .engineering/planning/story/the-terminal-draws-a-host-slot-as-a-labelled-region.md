---
format: aep.planning-md/3
id: story:the-terminal-draws-a-host-slot-as-a-labelled-region
kind: story
status: implemented
title: The terminal draws a host slot as a labelled region
summary: 'ADR Effect (terminal): a placeholder region that ess-ui-test can address; steps inside it are refused'
tags:
- backlog
- queue-0.92.0
- rank-3
relations:
- derived_from: design:ui-host-slots
- serves: vision:O2
- depends_on: story:a-ui-document-declares-a-host-slot
scope:
- confidence: cited
  path: crates/ui/ess-ui-test/src/parity.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/playwright.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/runner.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/target.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src
- confidence: cited
  path: crates/ui/ess-ui-tui/tests/host_slots.rs
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-08T08:54:31Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-11-08T08:54:32Z", actor: "human:timo", revision: 11}
- {from: "active", to: "implemented", at: "2026-11-09T05:06:15Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":1}}}
---
## Need

The accepted ADR on host components, Effect (terminal): the terminal renderer cannot draw a host
component, so it marks the slot's place. The design is design:ui-host-slots, section Generators.

## Acceptance

- `ess generate ui --target tui` on a document with slots no longer refuses it
  (`ess-ui-tui/src/profile.rs:148`); the renderer draws a bordered region at each slot's path whose
  text is `<host>: <reason>` (today `Body::Host` draws nothing, `view.rs:822`).
- An `ess-ui-test/1` `text` check of `<host>: <reason>` at the slot's path passes against the terminal
  target, and a `not_text` check of that string at the slot's path fails; before this story the same check is refused as an undrawn
  node (`Runner::region`, `runner.rs:1069`). One test per check kind.
- A step that acts inside the region — `act`, `select`, `mark`, `move`, `upload`, `type` or `choose`
  addressing the slot's path or a path below it — is refused on every target by one rule in
  `parity.rs`, with the message `` `<path>` is host slot `<host>`: the host draws it, so a test
  cannot act inside it ``. One test per step kind on the terminal target; the Playwright emitter
  writes that same message as `test.fixme` for the React target (one test).
- This story owns that rule in `crates/ui/ess-ui-test/src/parity.rs`; the React story renders the
  `data-ui-path` it names.

## Scope

- inferred, confirmed by reading source: `crates/ui/ess-ui-tui/src/` (`profile.rs:148`,
  `view.rs:822`, `app.rs:1432`), `crates/ui/ess-ui-tui/tests/host_slots.rs` (rewritten),
  `crates/ui/ess-ui-test/src/` (`parity.rs`, `target.rs` `resolve`, `runner.rs` `resolve`/`region`,
  `playwright.rs` `test.fixme`).
- Shared with story:the-ui-test-runner-acts-on-an-action-inside-an-overlay (`runner.rs`,
  `ess-ui-tui/src/view.rs`): ordered, that story depends on this one and is not in this release.
- Shared, trivially merged: `CHANGELOG.md`, `website/docs/reference/ess-ui.md`.
