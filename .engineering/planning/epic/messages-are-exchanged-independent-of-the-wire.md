---
format: aep.planning-md/3
id: epic:messages-are-exchanged-independent-of-the-wire
kind: epic
status: draft
title: Messages are exchanged independent of the wire
summary: One exchange in the specification, bound side by side by WebSocket, broker, HTTP upgrade and JSON-RPC (#650)
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- derived_from: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
revision: 1
---
## Outcome

A conversation between two parties (who opens it, which messages each side sends and in what order,
which row they belong to, what ends it) is stated once in the specification, and every wire that
carries it binds that one statement: a WebSocket, a broker, an HTTP upgrade and JSON-RPC, side by
side. Projections and conformance derive from the statement, so the bindings cannot disagree.

Design: `architecture-decision-record:messages-are-exchanged-independent-of-the-wire`, accepted
2026-10-28 with F1 (a), F2 (a), F3 (a). Closes beyond10x/ess#650.

## Stories and order

| story | layer | size | after |
|---|---|---|---|
| `story:an-exchange-is-declared-in-the-specification` | `exchanges:` at the next `ess/N`, compile checks, IR | L | none |
| `story:asyncapi-and-openapi-project-an-exchange` | AsyncAPI channels and operations, OpenAPI `101` route link | M | the exchange |
| `story:conformance-synthesizes-exchange-scenarios` | synthesized scenarios and the interpreted target | L | the exchange |
| `story:a-websocket-binding-frames-an-exchange` | `ess-websocket/1`, new crate | M | the exchange |
| `story:an-http-route-upgrades-to-an-exchange` | `ess-http` `upgrade:` route key (handshake) | M | the exchange, the WebSocket binding |
| `story:a-broker-binding-carries-an-exchange` | `ess-transport/3` exchange channels | M | the exchange |
| `story:json-rpc-binds-an-exchange` | `ess-jsonrpc/1` notifications for a named exchange | S-M | the exchange, `story:a-command-binds-to-json-rpc-and-grpc-beside-http` |

The batch runs after the two batches that change no source format. A wave holds at most four units,
so it runs as two waves: the exchange, projections, conformance and the WebSocket binding first;
the HTTP upgrade, the broker binding, JSON-RPC and `story:a-view-may-answer-values-no-row-holds`
(beyond10x/ess#674, first cut, outside this epic) second. The exchange and the sourceless view share
one `ess/N` bump when they land in one release.
