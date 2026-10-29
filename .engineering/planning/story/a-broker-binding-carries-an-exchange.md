---
format: aep.planning-md/3
id: story:a-broker-binding-carries-an-exchange
kind: story
status: draft
title: A broker binding carries an exchange
summary: ess-transport/3 exchange channels; /1 and /2 keep their bytes; size M
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- decomposes: epic:messages-are-exchanged-independent-of-the-wire
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
- depends_on: story:an-exchange-is-declared-in-the-specification
revision: 1
---
## Problem

ADR `messages-are-exchanged-independent-of-the-wire` § 2, binding B: an exchange spoken over a
broker. `ess-transport/2` binds one event to one NATS subject and cannot say which subjects carry
the two directions of one conversation or how the followed row's identity appears in them.

## Design

`ess-transport/3`: `exchanges[]` entries name an `exchange` and give, per direction, a subject
template whose token is the followed row's identity, the broker, and `delivery`; `brokers[].protocol`
stays a closed set, extended one member at a time by its own decision. `/1` and `/2` documents keep
validating and keep their bytes.

## Acceptance

- An `ess-transport/3` document binding one exchange validates and compiles to
  `ess-transport-ir/3`; `/1` and `/2` fixtures compile to unchanged bytes.
- Refused by name: an unknown exchange; a subject template without the identity token; a message
  of the exchange left unbound; a subject captured by no stream on a JetStream broker.
- AsyncAPI shows the exchange's channel with its `nats` servers and subjects.
- Tests in `crates/specify/ess-transport/tests/exchanges.rs` (new).

## Size

M: ess-transport, ess-gen `asyncapi.rs`.
