---
format: aep.planning-md/3
id: approval-record:websocket-upgrade-is-a-sibling-binding
kind: approval-record
status: superseded
title: A WebSocket upgrade is a sibling ess-websocket/1 binding (operator, 2026-10-28)
summary: 'Option A for the WebSocket half of #650; ADR amendment 2 accepted'
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- decides: decision-blocker:websocket-upgrade-binding-form
- decides: story:http-binding-carries-transport-framing
- informed_by: architecture-decision-record:transport-and-idl-compile-targets
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:44:16Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
- {from: "approved", to: "superseded", at: "2026-10-28T10:47:35Z", actor: "human:timo", revision: 3, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

The operator, 2026-10-28, relayed verbatim by the orchestrator: "option A for the ESS WebSocket one."

This answers `decision-blocker:websocket-upgrade-binding-form` (beyond10x/ess#650, WebSocket half)
with option A: a sibling `ess-websocket/1` shaped like `ess-jsonrpc/1` (pins the specification; lists
sockets with path, subprotocol, opening command, client-sent commands and server-sent events),
compiled to `ess-websocket-ir/1` and projected into AsyncAPI 3's `ws` binding. It is written into
`architecture-decision-record:transport-and-idl-compile-targets` as Amendment 2, accepted with this
quote.

## Effect

- The decision-blocker is cleared.
- The WebSocket half of `story:http-binding-carries-transport-framing` is buildable: one unit, size
  M, a new crate, extending `ess-gen/src/asyncapi.rs`, planned into a later batch (it adds a new
  format, so not batch 1).

Decider: operator
