---
format: aep.planning-md/3
id: story:feature-request-395
kind: story
status: draft
title: ess generate client writes typed Go and Rust event publishers over a transport
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#395
relations:
- serves: vision:O2
- decomposes: epic:message-contract-clients
revision: 1
---
## Outcome

`ess generate client` writes a typed Go or Rust publisher for one component over its `ess-transport/1` channels, so a producer library is generated from the contract instead of hand-written per language.

## Acceptance

- One `publish_<event>` per event the component publishes and the transport binds; `array` channels also get `publish_<event>_now`; the publisher has `flush` and `close`; subject, batch and stream are constants (`crates/generate/ess-publisher/tests/generated.rs`).
- The core of each library depends only on what the types library does (`serde`, `serde_json`; Go standard library). The JetStream adapter is generated beside it as its own crate (`nats/`) or module (`natsjs/`), so the core builds offline.
- Generated Rust and Go compile offline and pass their own tests against an in-memory transport: an `array` channel flushes at `max_items` and on close, `_now` and `single` publish directly, a failed background flush reaches the error callback with its subject and item count and is dropped, a payload offered after close is refused (Go).
- `delivery: at_least_once` is refused by name (`unsupported_delivery`); a component that publishes nothing the transport binds is refused (`nothing_to_publish`); a stream with `owner: publisher` is reported as an obligation.
- `client-report.json` (`ess-client-report/1`) lists the operations and obligations (`crates/edge/ess-cli/tests/client.rs`).
- Verified outside the gate (2026-10-03): both adapters compile against `async-nats` 0.38 and `nats.go` v1.48.0, and a metering model's generated Go and Rust publishers each put 150 payloads into a local NATS 2.10 JetStream stream as 2 messages (100 + 50).

## Origin

beyond10x/ess#395. Depends on beyond10x/ess#393 (event payload types) and beyond10x/ess#390 (`ess-transport/1`).

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`, 2026-10-03.

1. **Need.** A producer library — publish an event to its subject with its envelope, batch, report failures under its delivery promise — is fully determined by the component's `publishes` and the transport document, and is hand-written per language today. Requester's proposal: `ess generate client --target go|rust --component <publisher>`.
2. **Class.** Gap: no target generates a publisher.
3. **Already expressible?** No. `ess generate types` emits data types only; the composition client (`crates/specify/ess-composition`, `ess-client-plan/1`) forwards request bytes for commands and queries, not events.
4. **Fit.** Builds on the types realizer and the transport IR, adds no authored surface. Follows the composition client's seam: an application-provided `Transport`, no networking in the core. The design page is `docs/design/event-publishers.md`.
5. **Second adopter.** An order service generating a Go publisher for `OrderPlaced` on a `single` channel.
6. **Cost.** One CLI verb, one report format, one output-owner family; no `ess/N` change. NATS adapters are not compiled by the offline gate.
7. **Alternatives.** (a) Change nothing: one hand-written client per language. (b) Async core on Tokio: the offline gate could not build it (no async runtime in the workspace lock), and Go needs no runtime. (c) Runtime-free core with opt-in adapters (chosen).

## Decisions

- **accept (2026-10-03):** runtime-free core over a `Transport` seam; adapters as separate crate/module; `at_least_once` refused until a retry policy is specified; source defaulting stays with the caller.
