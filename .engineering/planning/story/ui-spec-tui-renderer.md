---
format: aep.planning-md/3
id: story:ui-spec-tui-renderer
kind: story
status: proposed
title: Any ess-ui document runs as a terminal application from fixtures
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- depends_on: story:ui-spec-schema
scope:
- confidence: inferred
  path: crates/ui/ess-ui-tui
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T00:35:23Z", actor: "human:timo", revision: 3}
---
## Outcome

Any `ess-ui/1` document runs as a terminal application, reading fixtures instead of a backend.

## Acceptance

- `ess ui run --tui --path <document>` (ratatui) renders the shell (navigation pane, page outlet,
  overlay, notifications), pages with their layout (degrading columns and areas to a stack), and
  every composite kind and primitive, with keyboard navigation (move, open, filter, page, act,
  close), section lifecycle states (loading, empty, failed, stale) and live updates played from a
  fixture event script.
- Capabilities the TUI lacks are declared in its renderer profile and handled by the document's
  `degrades`; a document without the needed `degrades` is refused with the failing node's path.
- A test drives the example application headless (ratatui test backend): opens a page, filters a
  collection, plays one live event and asserts the rendered buffer.
