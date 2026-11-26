---
format: aep.planning-md/3
id: story:a-bus-contract-binds-to-nats-or-a-unix-socket-and-a-bus-implementation-is-checked-over-each
kind: story
status: draft
title: A bus contract binds to NATS or a unix socket, and a bus implementation is checked over each
summary: ess-bus-binding/1, a target adapter per transport, traces checked against a machine per pattern; F6 open
tags:
- design
- event-bus
- issues-2026-11-26
relations:
- serves: vision:O2
- decomposes: epic:outward-events-and-an-event-bus-contract
- depends_on: story:an-event-bus-contract-states-publish-subscribe-and-fan-out-channels-over-outward-events
- informed_by: story:a-command-may-be-bound-to-a-broker-subject-as-a-request-with-one-reply
- informed_by: story:a-broker-binding-carries-an-exchange
revision: 1
---
## Need

A bus contract names no wire. A consumer injects one conforming implementation, NATS or a unix
socket, and nothing states that binding or checks an implementation against the contract.
`ess-transport`'s protocol set is closed at `nats` (`crates/specify/ess-transport/src/lib.rs:46-52`)
and binds one specification's events, not a bus. Design:
`docs/design/outward-events-and-event-bus.md` § 4 (branch `design/outward-events-and-event-bus`).

## Outcome

Under F6 B a bus binding (`ess-bus-binding/1`) pins a compiled bus contract by digest and maps each
channel to one wire, from a closed set:

- `nats`: a subject per channel (a stream on JetStream), with `ess-transport`'s subject and stream
  checks; `at_least_once` only on JetStream;
- `unix_socket`: a publish and a subscribe path under a declared runtime directory, framing
  `ndjson` or `length_prefixed`, a maximum message size; `at_most_once` only.

A target adapter per transport publishes the synthesized scenarios' events over the bound wire,
records what each sink receives as an `ess-prototrace/1` trace with origin `target`, and checks it
against the pattern's reference machine.

## Acceptance

- A NATS binding and a unix-socket binding of one contract validate and compile; refused by name:
  a channel left unbound, a subject outside the NATS rules or captured by no stream or two, a
  delivery word the transport cannot keep, a pattern it cannot carry, a digest that differs.
- Against an in-memory reference bus behind each adapter, the scenarios pass; against a
  deliberately faulty bus (one that drops one subscriber's copy, one that duplicates under
  `at_most_once`) the check fails naming the property; incomplete capture is inconclusive, never a
  pass.
- The NATS adapter is opt-in like the generated publishers' broker adapter, so the offline gate
  runs without a broker; a live broker run stays outside the gate.
- Each test runs within 60 s, each test step within 120 s.

## Scope

- A new crate under `crates/specify/` for the binding, `crates/verify/ess-conformance/src/` for the
  adapters and trace check, `crates/generate/ess-publisher/` for a bus seam, `docs/design/`.

## Decisions

Open, for the owner: F6 (A in `ess-transport` per producer with `unix_socket` as a new protocol
member, B a sibling `ess-bus-binding/1` per bus contract, C transport keys inside the contract).
The design page recommends B; no default is taken.
