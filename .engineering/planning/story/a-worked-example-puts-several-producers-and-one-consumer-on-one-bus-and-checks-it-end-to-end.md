---
format: aep.planning-md/3
id: story:a-worked-example-puts-several-producers-and-one-consumer-on-one-bus-and-checks-it-end-to-end
kind: story
status: proposed
title: A worked example puts several producers and one consumer on one bus and checks it end to end
summary: examples/ with outward events, a bus contract, a route and two bindings; after F1-F7
tags:
- design
- event-bus
- issues-2026-11-26
relations:
- serves: vision:O2
- decomposes: epic:outward-events-and-an-event-bus-contract
- depends_on: story:a-bus-route-forwards-an-event-to-a-sink-when-its-condition-holds
- depends_on: story:a-bus-contract-binds-to-nats-or-a-unix-socket-and-a-bus-implementation-is-checked-over-each
- depends_on: story:a-periodic-aggregate-event-publishes-an-aggregate-view-s-answer-every-period
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-26T16:09:56Z", actor: "human:operator", revision: 3}
---
## Need

The first producers are a build-slot pool (job changes and pool statistics), an LLM gateway
(finished calls and per-route statistics, with a hosted GPU backend and a local model server
behind it), a runbook daemon (run events), a knowledge service (no feed yet) and later a perception
layer; the first consumer, an operator console, injects one bus implementation and shows every
producer on one architecture page. Their specifications live in their own repositories; ESS needs
one worked example that holds the whole path in its own tree, so a change to any of the formats
above is caught end to end. Design: `docs/design/outward-events-and-event-bus.md` § 6 (branch
`design/outward-events-and-event-bus`).

## Outcome

`examples/` gains, with generic names, three producer specifications declaring outward events (one
publish-subscribe, one periodic aggregate, one fed by an existing server-sent-event feed), one
consumer specification declaring an input, one bus contract with a route, a NATS binding and a
unix-socket binding.

## Acceptance

- `ess specify validate` and `compile` pass on every document of the example.
- Synthesis and the in-memory reference bus behind each adapter pass every scenario; the graph
  projection (`ess inspect`) shows the systems, channels and routes of the contract as data.
- `projection-check` holds the example's committed projections byte for byte, from a committed
  file with a digest of its sources, never built per test.
- Each test runs within 60 s, each test step within 120 s.

## Scope

- `examples/` (new directory), `crates/generate/ess-gen/src/graph.rs`, `Taskfile.yml`
  (`projection-check`).

## Decisions

F1 to F7 are decided: F1 A, F2 A, F3 A, F4 C, F5 B, F6 B, F7 A (the owner, 2026-11-26, relayed by
the operator session). The example uses `ess-bus/1`, `ess-bus-binding/1` and `ess-mcp/1` as the
design page states them.
