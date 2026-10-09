---
format: aep.planning-md/3
id: story:event-client-generates-the-subscriber-side
kind: story
status: draft
title: ess generate client writes the subscriber side of a transport
tags:
- adopter-report
- deployment-chain
- design-first
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

`ess generate client` can write the consumer side of an `ess-transport` document: a subscriber
that receives the declared events (wildcard subjects included), decodes them into the generated
types and hands them to the caller, as the publisher side does today.

Design first: `docs/design/event-transport-binding.md` lists "A consumer-side binding (durable
consumers, acknowledgement, queue groups)" under "Not decided here" (lines 104-107); the design
decides it before code.

## Evidence

An adopter on ess 0.55.0 hand-wrote the subscriber: `generate client` emits only the publisher
(`crates/edge/ess-cli/src/client.rs`, `crates/generate/ess-publisher`). Still so on main.

## Acceptance

- The design page decides the consumer binding (core subscription versus durable consumer,
  acknowledgement, queue groups, wildcard subjects).
- Rust and Go subscribers are generated and checked against the transport fixture the publisher
  tests use: a published event arrives decoded; an undeclared subject is not delivered.
