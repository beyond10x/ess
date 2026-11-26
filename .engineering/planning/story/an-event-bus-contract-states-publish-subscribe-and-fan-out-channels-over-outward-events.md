---
format: aep.planning-md/3
id: story:an-event-bus-contract-states-publish-subscribe-and-fan-out-channels-over-outward-events
kind: story
status: draft
title: An event-bus contract states publish-subscribe and fan-out channels over outward events
summary: ess-bus/1 pins specifications and states channels by pattern and delivery word, no wire; F1, F2 open
tags:
- design
- event-bus
- issues-2026-11-26
relations:
- serves: vision:O2
- decomposes: epic:outward-events-and-an-event-bus-contract
- depends_on: story:a-system-declares-which-of-its-published-events-are-outward
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
revision: 1
---
## Need

Outward events reach other systems through a bus, and what the bus promises (who receives what,
with which delivery word) has no home apart from a wire. `ess-transport/1` and `/2` bind the events
of one specification to NATS subjects (`docs/design/event-transport-binding.md:19-24`) and leave a
consumer side undecided (`:104-109`). Design: `docs/design/outward-events-and-event-bus.md` § 2
(branch `design/outward-events-and-event-bus`).

## Outcome

A bus contract (under F2 A a sibling `ess-bus/1`, compiled to `ess-bus-ir/1`) pins one or more
specifications by digest and states channels over their outward events, each with a pattern from
a closed catalog and a delivery word (`at_most_once`, `at_least_once`):

- `publish_subscribe`: every attached subscriber receives each event on the channel;
- `fan_out`: the contract names sinks, each a declared input of a pinned system, and each receives
  a copy.

The other patterns of the catalog are named in the design page and are not members yet. The
contract names no subject, socket or host.

## Acceptance

- `ess specify bus validate` and `compile` accept a contract with one publish-subscribe and one
  fan-out channel over two pinned specifications and write a deterministic IR.
- Refused by name with stable codes: an unresolved or mismatched pin; an event that is not outward
  in its system; a pattern outside the built members; a fan-out with no sink; a sink that is not a
  declared input of a pinned system or whose input type the event cannot fill; duplicate names.
- A reference machine per built pattern, as an `ess-protospec/1` document, passes bounded
  exploration (each subscriber receives each event at most once under `at_most_once`; a copy per
  sink under `fan_out`).
- Synthesis writes one scenario per channel, event and receiver from the compiled contract alone.
- Each test runs within 60 s, each test step within 120 s.

## Scope

- A new crate under `crates/specify/` for the format; `crates/edge/ess-cli/src/` for the verbs;
  `crates/verify/ess-conformance/src/synthesize/` for scenarios; `docs/design/`,
  `website/docs/reference/formats.md`.

## Decisions

Open, for the owner: F1 (where outwardness is declared) and F2 (A a sibling `ess-bus/1`, B in each
producer's specification, C an `ess-protospec/1` document per bus). The design page recommends F1
A and F2 A; no default is taken.
