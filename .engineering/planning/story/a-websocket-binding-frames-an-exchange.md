---
format: aep.planning-md/3
id: story:a-websocket-binding-frames-an-exchange
kind: story
status: draft
title: A WebSocket binding frames an exchange
summary: ess-websocket/1, a new sibling binding and crate; size M
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

beyond10x/ess#650: messages after a WebSocket upgrade cannot be bound. ADR
`messages-are-exchanged-independent-of-the-wire` § 2, binding A, and F3 (a): `ess-websocket/1`
carries the framing.

## Design

A sibling document `ess-websocket/1`, shaped like `ess-jsonrpc/1`: `specification` pins system,
version and source digest; `sockets[]` each name an `exchange`, a `subprotocol`, `frames: text |
binary`, a discriminator field and the wire name per message, optional `ping_seconds`, and a `path`
only when no `ess-http` route upgrades to the exchange. Compiles against the resolved IR into
`ess-websocket-ir/1`; unknown keys refused by name; codes `ESS-WEBSOCKET-001` onward.

## Acceptance

- A binding of one exchange validates and compiles to `ess-websocket-ir/1` deterministically.
- Refused by name: an unknown exchange; a message the exchange does not list; a listed message left
  unbound; two messages with one wire name; a `path` on an exchange an `ess-http` route already
  upgrades to; a missing `path` when none does.
- `ess generate --kind asyncapi --websocket <file>` adds the `ws` binding section.
- Tests in the new crate `crates/specify/ess-websocket/tests/`.

## Size

M: new crate `ess-websocket`, CLI flag, `crates/generate/ess-gen/src/asyncapi.rs`.
