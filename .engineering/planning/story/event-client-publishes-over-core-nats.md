---
format: aep.planning-md/3
id: story:event-client-publishes-over-core-nats
kind: story
status: draft
title: A core-NATS broker gets a core-NATS publisher adapter
tags:
- adopter-report
- defect
- deployment-chain
- feature-request
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-event-client
- decomposes: epic:zero-open-issues-2026-11
revision: 3
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
carries. `client-report.json` names no obligation for it either.

Reproduction on 0.92.0: a one-event model with a component `producer` publishing
`metering.items.UsageRecorded`, and this transport document (digest from
`ess generate --kind schema`):

```yaml
type: ess-transport/1
specification: {system: metering, version: v1, source_digest: sha256:<model digest>}
brokers:
  - {id: events, protocol: nats, jetstream: false}
channels:
  - event: metering.items.UsageRecorded
    broker: events
    subject: usage
    envelope: array
    delivery: at_most_once
    batch: {max_items: 100, max_delay_ms: 5000}
```

```text
$ ess generate client --path model --component producer --transport transport.yaml --target rust --package metering --out out-rust
1 publish operation(s) for `producer`, written to out-rust; obligations in client-report.json
exit=0
$ grep -n JetStream out-rust/nats/lib.rs
10:pub struct JetStreamTransport {
11:    context: ::async_nats::jetstream::Context,
```

`--target go` likewise exits 0 and writes `natsjs/natsjs.go`, which calls `t.JetStream.Publish`.

## Acceptance

- For the `jetstream: false` transport above, `ess generate client --target rust` writes a `nats/`
  adapter that publishes through `async_nats::Client::publish` (no `jetstream::Context`), and
  `--target go` writes an adapter that publishes through `nats.Conn.Publish` (no `jetstream`
  import); or both exit non-zero naming broker `events` and `jetstream: false`.
- For a `jetstream: true` broker the output bytes are unchanged
  (`crates/edge/ess-cli/tests/client.rs` and `crates/generate/ess-publisher/tests/generated.rs`
  pins hold).
- A client test generates both adapters and publishes one event through each against the transport
  fixture.

## Scope

- cited: crates/generate/ess-publisher/src/lib.rs
- cited: crates/generate/ess-publisher/src/go_natsjs.go.txt
- cited: crates/generate/ess-publisher/src/rust_nats.rs.txt
- cited: crates/specify/ess-transport/src/lib.rs
- inferred: crates/generate/ess-publisher/tests/generated.rs
- inferred: crates/edge/ess-cli/tests/client.rs
- inferred: crates/edge/ess-cli/src/main.rs

confidence: medium
