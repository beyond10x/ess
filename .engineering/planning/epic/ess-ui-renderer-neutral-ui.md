---
format: aep.planning-md/3
id: epic:ess-ui-renderer-neutral-ui
kind: epic
status: active
title: One renderer-neutral UI document renders as a TUI and as React, checked, documented and tested by node path
relations:
- serves: vision:O2
- informed_by: task:deferred-protocol-ui-bindings
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T00:35:24Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-30T00:35:25Z", actor: "human:timo", revision: 3}
---
## Outcome

A renderer-neutral UI document, `ess-ui/1`, describes an application's shell, navigation, pages,
sections, composites and primitives, what each reads (ESS views), does (ESS commands) and follows
live (ESS events and channels), its loading states, where each piece of state lives (memory, URL,
browser storage, server session, server), and how it degrades on a renderer that lacks a capability.
One document renders as a TUI and as generated React, is documented from its own schema, is checked
with node paths that stay stable, and is tested with a compact test language that selects nodes by
path.

## Origin

- The parked `task:deferred-protocol-ui-bindings` (typed UI model; loading, empty, failure,
  permission and reconnect states; drafts separate from service state).
- A retrofit of a Vue 2 admin application (about 46 routes, 289 components) into a draft of the
  format, kept outside this repository because it carries a third party's names.
- Requirements from a planned consumer that edits a document node by node through LLM structured
  output (app-defined widgets, a primitive layer, unbound placeholder reads, stable node paths, page
  layout, a brand-free example).

## Decisions

- `ess-ui/1` is a sibling format of the ESS specification, like `ess-cli/1`: it references an ESS
  model and never changes `ess/N`.
- Types in the schema are YAML structure (`{enum: [...]}`, `{list: T}`, `{map: {key, value}}`), never
  type expressions inside strings; shorthands are data (`accepts` / `expands_to`).
- Running code is Rust; the React renderer is a generator that emits React source.
