---
format: aep.planning-md/3
id: story:ui-spec-test-language
kind: story
status: active
title: UI behaviour is tested by node path in a compact test language
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- depends_on: story:ui-spec-schema
- depends_on: story:ui-spec-tui-renderer
- depends_on: story:ui-spec-checks
scope:
- confidence: inferred
  path: crates/ui/ess-ui-test
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T00:35:24Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T07:47:24Z", actor: "human:timo", revision: 4}
---
## Outcome

UI behaviour is tested in a compact language that selects nodes by their document path, declares
backend state as fixtures and live events as a script, and controls time — so one test runs
against every renderer.

## Acceptance

- A test file format (`ess-ui-test/1`, YAML, one line per step where possible) with steps: open a
  page, select by path (with row key for collection items), type, choose, act, expect text / rows /
  state (`loading`, `empty`, `stale`), play a live event, advance time, expect a command sent with
  given input; plus fixture setup per test.
- `ess ui test --path <document> <tests>` runs the tests against the TUI renderer headless; the
  generated React project maps the same steps onto `data-ui-path` selectors (emitted as a Playwright
  spec from the same file).
- The language and its design notes are documented, including a comparison with selector-based
  browser tests (data attributes, CSS classes, visible text) and the brittleness it removes.
- Example tests cover the example application: a list page (filter, page, delete with confirm), a
  live update landing in a filtered list, and a stale channel.

## Depends on

The TUI renderer (the runner drives it) and the checks (node paths).
