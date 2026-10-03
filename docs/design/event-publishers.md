# Generated event publishers

Status: accepted for beyond10x/ess#395 (2026-10-03).

## Problem

A component's `publishes` list and an `ess-transport/1` document together say everything a
producer library does: which events, which payload types, which subject, whether one message
carries one payload or an array, how a batch fills, and what delivery the publisher promises.
Adopters still hand-write that library once per language, and the hand-written copies drift from
the contract and from each other.

## Decision

`ess generate client --target rust|go --component <c> --transport <doc> --package <p>
[--module <m>] --out <dir>` writes one library per target:

- the data types of every event the component publishes and their closure, from the types
  realizer (`docs/design/types-only-realizations.md`, event roots);
- one publisher for the component, with one `publish_<event>` per published event the transport
  binds, and the subject, envelope and batch policy of its channel as constants;
- a `Transport` seam the application implements or takes from an opt-in adapter;
- `client-report.json` (`ess-client-report/1`): the component, the transport IR digest, every
  generated operation, and every obligation the library does not discharge.

### Runtime-free core

The core depends on nothing beyond what the types library already depends on (`serde`,
`serde_json` in Rust; the standard library in Go). Batching runs on a standard thread (Rust) or a
goroutine (Go) with a timer; it needs no async runtime. The seam is synchronous in Rust and
`context`-taking in Go:

```rust
pub trait Transport: Send + Sync + 'static {
    type Error: std::fmt::Display + Send + 'static;
    fn publish(&self, subject: &str, payload: &[u8]) -> Result<(), Self::Error>;
}
```

```go
type Transport interface {
    Publish(ctx context.Context, subject string, payload []byte) error
}
```

An adapter for a broker the transport document names is generated beside the core, opt-in:
Rust feature `nats` (a JetStream publish that waits for the ack, through `async-nats` on a
`tokio` runtime handle), Go subpackage `natsjs` (`nats.go/jetstream`). The core never imports it,
so the offline gate compiles and tests the core with an in-memory transport, and an adopter that
already holds a connection writes three lines instead.

### Behaviour per channel

| channel | `publish_<event>` | `publish_<event>_now` | `flush` / `close` |
|---|---|---|---|
| `envelope: single` | encodes and publishes one message | — | nothing buffered |
| `envelope: array` | encodes, buffers; a flush publishes the buffer as one JSON array at `max_items` or `max_delay_ms` | publishes the given payloads as one array, bypassing the buffer | publishes what is buffered; `close` also stops the flusher |

`delivery: at_most_once`: a failed publish is reported to the error callback (default: none) and
the batch is dropped; nothing retries it. `delivery: at_least_once` is refused by the generator in
this version, by name: it needs a retry policy (attempts, backoff, what happens at `close`) the
transport document does not state.

Every stream the component's channels land on is touched by nothing generated. A stream with
`owner: publisher` is reported as an obligation (`stream creation is the application's`).

A payload is encoded when it is accepted, so a value the types library refuses to serialize fails
the call that offered it, not a later flush.

### Refusals

- the component publishes no event the transport binds;
- a channel with `delivery: at_least_once`;
- a broker protocol without a generated adapter is not a refusal: the core is still generated and
  the report says no adapter exists.

## Not decided here

- Consumer clients (subscribe, acknowledge, redeliver).
- Retry policies and an `at_least_once` publisher.
- Publishing an event the transport document does not bind.
- Source defaulting or any other producer-side filling of payload fields: the model states none.
