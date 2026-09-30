---
format: aep.planning-md/3
id: story:ui-spec-react-renderer
kind: story
status: implemented
title: An ess-ui document generates a React application
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- depends_on: story:ui-spec-schema
scope:
- confidence: inferred
  path: crates/ui/ess-ui-react
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T00:35:24Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T01:41:54Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T05:07:14Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Outcome

An `ess-ui/1` document generates a React application that renders the same pages, states and live
behaviour as the TUI, reading fixtures by default.

## Acceptance

- `ess generate ui --target react --path <document> --out <dir>` emits a Vite + React + TypeScript
  project: one component per page and section, the composite kinds and primitives as a small
  generated component set, routing from navigation, section lifecycle states, live channels
  (server-sent events for one-way channels, WebSocket for both-way) with a fixture player, and state
  placement honoured per `store:`.
- Every rendered node carries `data-ui-path="<canonical path>"` so tests select by path.
- The generated project type-checks (`tsc --noEmit`) offline in a test; generation is deterministic.
- The generator is Rust; no hand-written TypeScript is committed outside generated output.
