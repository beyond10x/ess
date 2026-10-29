---
format: aep.planning-md/3
id: story:an-http-route-upgrades-to-an-exchange
kind: story
status: draft
title: An HTTP route upgrades to an exchange
summary: 'ess-http upgrade: route key carries the handshake and the 101; size M'
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- decomposes: epic:messages-are-exchanged-independent-of-the-wire
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
- depends_on: story:an-exchange-is-declared-in-the-specification
- depends_on: story:a-websocket-binding-frames-an-exchange
revision: 1
---
## Problem

beyond10x/ess#650: a `GET` that answers `101` cannot be bound (`ess-http/1` refuses `101` as an
outcome, ESS-HTTP-023, rightly). ADR `messages-are-exchanged-independent-of-the-wire` § 2, binding
C, and F3 (a): the HTTP route carries the handshake.

## Design

An `ess-http` route key `upgrade: {exchange: <name>, protocol: websocket}` on the route of the
exchange's `opens` command: method, path, path parameters and the refusals answered before the
upgrade stay ordinary route keys; the accepting outcome answers `101` through the key, not through
`status:`. Additive within `ess-http/1` if the format policy admits it (as the additive
`operation_id` keys did), otherwise `ess-http/2`; the story's design section decides with the
policy cited.

## Acceptance

- A route with `upgrade:` validates and compiles into `ess-http-ir`; `101` as an outcome `status:`
  stays refused (ESS-HTTP-023).
- Refused by name: `upgrade:` on a route whose command opens no exchange, or naming an unknown
  exchange; a protocol outside the closed set (`websocket`).
- The OpenAPI projection shows the `101` response linked to the AsyncAPI channel.
- Bindings without `upgrade:` keep their bytes. Tests in `crates/specify/ess-http/tests/upgrade.rs`
  (new).

## Size

M: ess-http, ess-gen `openapi.rs`.
