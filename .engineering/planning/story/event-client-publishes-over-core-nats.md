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
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/client.rs
- confidence: inferred
  path: crates/generate/ess-publisher/src/go_natscore.go.txt
- confidence: cited
  path: crates/generate/ess-publisher/src/lib.rs
- confidence: inferred
  path: crates/generate/ess-publisher/src/rust_natscore.rs.txt
- confidence: inferred
  path: crates/generate/ess-publisher/tests/generated.rs
- confidence: cited
  path: crates/specify/ess-transport/src/lib.rs
revision: 8
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

## Decisions

- **Write the adapter, do not refuse.** The outcome asks for it, the adopter's hand-written
  adapter is about 20 lines, and the transport document already carries `jetstream`.
- **The core adapter is `natscore/` in both languages.** `nats/` (Rust) and `natsjs/` (Go) keep
  their JetStream bytes, and a component whose channels reach a JetStream broker and a core broker
  gets both adapters side by side.
- **Checked by content, not by a live broker.** `Cargo.lock` holds no `async-nats` and the Go test
  runs with `GOPROXY=off`, so the adapters are checked as the JetStream ones are today: the core
  publisher compiles and behaves offline, and the adapter source is asserted.

## Acceptance

- `cargo test -p ess-cli --locked --test client`, new `a_core_nats_broker_gets_core_nats_adapters`:
  for the `jetstream: false` transport above, `ess generate client --target rust` exits 0 and writes
  `natscore/Cargo.toml` and `natscore/lib.rs`; the source holds `async_nats::Client` and calls
  `.publish(`, contains no `jetstream`, and no `nats/` directory is written.
- Same test: `--target go` exits 0 and writes `natscore/go.mod` and `natscore/natscore.go`, which
  imports `github.com/nats-io/nats.go`, calls `t.Conn.Publish` on a `*nats.Conn`, has no
  `nats.go/jetstream` import, and no `natsjs/` directory is written.
- Same target, new `a_component_on_both_broker_kinds_gets_both_adapters`: with brokers `events`
  (`jetstream: false`) and `stream` (`jetstream: true`), each bound by a channel of `producer`, Rust
  writes `nats/` and `natscore/` and Go writes `natsjs/` and `natscore/`.
- For a `jetstream: true` broker the output bytes are unchanged:
  `transport_1_publisher_files_keep_the_released_0_52_0_bytes`
  (`cargo test -p ess-publisher --locked --test generated`) and
  `a_rust_and_a_go_publisher_are_written_with_their_adapters_and_report`
  (`cargo test -p ess-cli --locked --test client`) pass unedited.
- `cargo test -p ess-publisher --locked --test generated`: `the_rust_publisher_compiles_offline_and_behaves`
  and `the_go_publisher_compiles_and_behaves` also run on the `jetstream: false` transport and pass.

## Scope

- cited: crates/generate/ess-publisher/src/lib.rs (`rust_nats` :665, `go` :681)
- cited: crates/specify/ess-transport/src/lib.rs (`Broker.jetstream` :551)
- inferred: crates/generate/ess-publisher/src/rust_natscore.rs.txt (new)
- inferred: crates/generate/ess-publisher/src/go_natscore.go.txt (new)
- inferred: crates/generate/ess-publisher/tests/generated.rs
- inferred: crates/edge/ess-cli/tests/client.rs

confidence: medium

Shared surfaces: proposes its `CHANGELOG.md` sentence; moves no byte-pin row.
