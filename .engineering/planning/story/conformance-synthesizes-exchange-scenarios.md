---
format: aep.planning-md/3
id: story:conformance-synthesizes-exchange-scenarios
kind: story
status: draft
title: Conformance synthesizes exchange scenarios
summary: Open, admitted and refused messages, close, follower bound; interpreted target passes; size L
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

ADR `messages-are-exchanged-independent-of-the-wire` § 3: conformance synthesizes exchange
scenarios from the specification alone, and a binding's target adapter carries them over its wire.

## Acceptance

- `ess verify conform synthesize` emits, per exchange: opening it; each `initiator_sends` command in
  each state of the followed row its guards admit, and its refusal in a state they do not; each
  `responder_sends` event where an outcome emits it; a message after close refused with the declared
  error; with `max_followers`, the follower over the bound answered with the declared error and no
  effect.
- `ess verify conform run --target interpreted` passes every synthesized exchange scenario.
- A specification without exchanges keeps its suite bytes (the `synthesis_suite_bytes_table` rows do
  not move).
- Tests in `crates/verify/ess-conformance/tests/exchange_scenarios.rs` (new); the suite format moves
  only if the scenarios need a new step kind, decided in the story's design section.

## Size

L: ess-conformance synthesis and interpreter.
