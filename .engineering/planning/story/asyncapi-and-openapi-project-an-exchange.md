---
format: aep.planning-md/3
id: story:asyncapi-and-openapi-project-an-exchange
kind: story
status: draft
title: AsyncAPI and OpenAPI project an exchange
summary: One AsyncAPI channel per exchange; the OpenAPI 101 route links to it; size M
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

ADR `messages-are-exchanged-independent-of-the-wire` § 3: AsyncAPI and OpenAPI must derive from the
exchange, so a generated document says which party sends which message whatever the wire.

## Acceptance

- `ess generate --kind asyncapi` emits one channel per exchange, with a `receive` operation per
  `initiator_sends` message and a `send` operation per `responder_sends` message from the
  responder's side, and `x-ess-exchange` naming the exchange; with a binding present, that
  binding's protocol section (`ws` from `ess-websocket/1`, `nats` from `ess-transport/3`).
- `ess generate --kind openapi` with an `ess-http` `upgrade:` route emits a `GET` whose `101`
  response names the AsyncAPI channel in `x-ess-exchange`.
- A specification without exchanges keeps its AsyncAPI and OpenAPI bytes (existing byte tests pass
  unmodified).
- Tests in `crates/generate/ess-gen/tests/asyncapi_exchange.rs` (new).

## Size

M: `crates/generate/ess-gen/src/asyncapi.rs`, `openapi.rs`.
