---
format: aep.planning-md/3
id: story:json-rpc-binds-an-exchange
kind: story
status: draft
title: JSON-RPC binds an exchange
summary: ess-jsonrpc/1 notifications and the opening method for a named exchange; size S-M
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- decomposes: epic:messages-are-exchanged-independent-of-the-wire
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
- depends_on: story:an-exchange-is-declared-in-the-specification
- depends_on: story:a-command-binds-to-json-rpc-and-grpc-beside-http
revision: 1
---
## Problem

ADR `messages-are-exchanged-independent-of-the-wire` § 2 and § 5: `ess-jsonrpc/1` (designed in
`docs/design/rpc-bindings.md`, not built; `story:a-command-binds-to-json-rpc-and-grpc-beside-http`)
is a binding of the same layer. Its methods bind implicit one-message exchanges; a named exchange
needs the method that opens it and notifications for the responder's messages.

## Acceptance

- `docs/design/rpc-bindings.md` states methods as bindings of implicit exchanges and adds
  `exchanges[]`: the opening method and a notification method per `responder_sends` message (and a
  method per `initiator_sends` command).
- An `ess-jsonrpc/1` document binding one exchange validates and compiles; documents written
  against the draft page's method syntax still validate.
- Refused by name: an unknown exchange; a responder message without a notification method; a
  notification method that collides with a request method.
- AsyncAPI shows the notifications as `send` operations of the exchange's channel.

## Size

S-M on top of the JSON-RPC story, which it depends on.
