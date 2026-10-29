---
format: aep.planning-md/3
id: decision-blocker:websocket-upgrade-binding-form
kind: decision-blocker
status: cleared
title: How a WebSocket upgrade is bound
summary: 'ADR amendment for the WebSocket half of #650; recommendation A'
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- blocks: story:http-binding-carries-transport-framing
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-28T10:44:16Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
# How a WebSocket upgrade is bound (beyond10x/ess#650, WebSocket half)

The JSON-RPC half of #650 is decided (ADR `transport-and-idl-compile-targets` § Decision 2,
`docs/design/rpc-bindings.md:19-48`). The WebSocket half needs an amendment to that ADR, and an ADR
amendment is the operator's (orchestrator decision D134, 2026-10-28: "#650 (WebSocket half) and #674:
both need an ADR amendment, which is the operator's. File a decision-blocker for each ... Batches proceed
without them; those units wait."). On 0.74.0 `101` stays refused as an outcome
(`docs/design/http-presentation-binding.md:337`).

## Options

| option | what it does | cost |
|---|---|---|
| A | Sibling `ess-websocket/1`, shaped like `ess-jsonrpc/1`: pins the specification and lists sockets (path, subprotocol, opening command, commands the client sends, events the server sends); compiles to `ess-websocket-ir/1`, projected into AsyncAPI 3's `ws` binding | ADR amendment 2; new format and crate; extends `ess-gen/src/asyncapi.rs`; size M; breaks nobody |
| B | `ess-transport/3` with `brokers[].protocol: websocket` | a format change; broker and stream vocabulary does not fit a point-to-point socket; leans to the ADR's rejected generic binding; size M |
| C | `ess-http/2` route key `upgrade: {websocket: …}` | moves `ess-http/2` and `ess-http-ir/2`; messages need AsyncAPI from the HTTP crate; against the ADR's sibling pattern; size M-L |

## Recommendation

A, because it follows the accepted ADR's sibling-document pattern and needs no `ess/N` change.

## What waits

The WebSocket half of `story:http-binding-carries-transport-framing`. Every batch proceeds without
it.

Decider: operator
