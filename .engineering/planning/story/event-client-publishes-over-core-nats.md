---
format: aep.planning-md/3
id: story:event-client-publishes-over-core-nats
kind: story
status: draft
title: A core-NATS broker gets a core-NATS publisher adapter
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

When an `ess-transport` document declares a core-NATS broker (`jetstream: false`), the adapter
`ess generate client` writes publishes over core NATS, not through a JetStream context.

## Evidence

An adopter on ess 0.55.0 wrote a core-NATS `Transport` of about 20 lines by hand. On main the
bundled adapters are JetStream only: the Go `natsjs/` module publishes through
`jetstream.JetStream` (`crates/generate/ess-publisher/src/go_natsjs.go.txt:1-19`) and the Rust
`nats/` crate is "the `JetStream` adapter" (`crates/generate/ess-publisher/src/lib.rs:664`); the
publisher does not read `broker.jetstream`, which `crates/specify/ess-transport/src/lib.rs:551`
carries.

## Acceptance

- For a `jetstream: false` broker, the Rust and Go clients ship a core-NATS adapter (or the
  JetStream adapter is refused naming the broker); for `jetstream: true` the output bytes are
  unchanged.
- A client test generates both and publishes through each against the transport fixture.
