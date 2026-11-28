---
format: aep.planning-md/3
id: epic:outward-events-and-an-event-bus-contract
kind: epic
status: draft
title: Outward events, an event-bus contract, routing, transports and ingress
summary: A system declares outward events; a bus contract states patterns apart from the wire; routes, NATS and unix-socket bindings, MCP and A2A ingress
tags:
- design
- event-bus
- issues-2026-11-26
relations:
- serves: vision:O2
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
revision: 2
---
## Outcome

A system declares the events it offers to other systems (outward events). A bus contract states,
apart from any wire, who receives them and with which promise: publish-subscribe, fan-out, a
content-based router and a periodic aggregate first, the rest of the enterprise integration
pattern catalog named for later. A bus binding maps the contract to NATS or a unix socket, and a
bus implementation is checked over each binding against a reference machine per pattern. On the
way in, a system declares the protocols it accepts input on (MCP, A2A beside HTTP) with the same
separation of semantics, protocol port and transport.

Design: `docs/design/outward-events-and-event-bus.md` (branch
`design/outward-events-and-event-bus`, draft, 2026-11-26).

## Need

The owner, 2026-11-26 about 12:32 CET (relayed): "there should be the concept of outward events
each system can declare (ess) - when an eventbus is configured (contract) - then other system can
receive these events [...] i mean this is just the transport ... the other is the messaging
semantics ... similar to ess protospec thing where we seperate the exchange patterns/logic from the
transport". About 12:35 CET: "this eventbus could have a simple routing logic - if (condition)
forward -> sink_x". About 12:45 and 12:46 CET (relayed): a periodic aggregated statistics event per
route; the lower-layer producers all emit through the bus and one consumer shows them on one
architecture page. About 12:47 CET: "rfc based or conformance based "inputs" ingresses? could also
include mcp,a2a,etec".

## Stories and order

| story | section | after |
|---|---|---|
| `story:a-system-declares-which-of-its-published-events-are-outward` | § 1 | none |
| `story:an-event-bus-contract-states-publish-subscribe-and-fan-out-channels-over-outward-events` | § 2 | outward events |
| `story:a-periodic-aggregate-event-publishes-an-aggregate-view-s-answer-every-period` | § 2 | outward events |
| `story:a-bus-route-forwards-an-event-to-a-sink-when-its-condition-holds` | § 3 | the bus contract |
| `story:a-bus-contract-binds-to-nats-or-a-unix-socket-and-a-bus-implementation-is-checked-over-each` | § 4 | the bus contract |
| `story:a-system-declares-its-ingress-as-mcp-and-a2a-protocol-bindings-beside-http` | § 5 | none |
| `story:a-worked-example-puts-several-producers-and-one-consumer-on-one-bus-and-checks-it-end-to-end` | § 6 | all of the above |

## Decisions

Decided by the owner, 2026-11-26, relayed by the operator session. The design page
(`docs/design/outward-events-and-event-bus.md`, section "Decided") states each pick as the design.

- F1 = A: `outward:` on a component's `publishes:`, in the specification, at the next `ess/N`.
  Rejected: B nowhere, C a sibling `ess-outward/1`.
- F2 = A: a sibling `ess-bus/1` that pins several specifications. Rejected: B in each producer, C
  an `ess-protospec/1` document per bus.
- F3 = A: an outward event sampled from an aggregate view every period. Rejected: B a time step's
  event, C aggregated in the bus.
- F4 = C: ESS's `where:` fragment extended by numeric comparison and list membership, plus a tested
  translation into `canon-expr/1`. Rejected: A without the translation, B `canon-expr/1` unchanged.
- F5 = B: no sink kind of its own; a trigger evaluator is a system with a declared input. Rejected:
  A a sink kind of its own, C outside the bus.
- F6 = B: a sibling `ess-bus-binding/1` per bus. Rejected: A in `ess-transport`, C inside the
  contract.
- F7 = A: one format per protocol, `ess-mcp/1` and `ess-a2a/1`. Rejected: B one `ess-ingress/1`,
  C methods of `ess-jsonrpc/1`.

## Not in this epic

A broker, a bus implementation or a deployment; request and reply (the broker-subject story); a
conversation (`epic:messages-are-exchanged-independent-of-the-wire`); trigger rules; "exactly
once".
