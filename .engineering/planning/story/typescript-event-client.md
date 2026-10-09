---
format: aep.planning-md/3
id: story:typescript-event-client
kind: story
status: draft
title: ess generate client has a TypeScript event publisher target
tags:
- adopter-report
- deployment-chain
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

`ess generate client` has a TypeScript/Node target: an event publisher over an `ess-transport`
document, as the Rust and Go targets write.

## Evidence

An adopter on ess 0.55.0. On main `generate client`'s `Target` is `Rust | Go` only
(`crates/edge/ess-cli/src/client.rs:14-18`).

## Acceptance

- `ess generate client --target typescript` writes a publisher that type-checks and publishes the
  declared events with the transport's subjects and payload shapes, checked in a test against the
  same transport fixture the Rust and Go clients use.
