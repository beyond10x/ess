---
format: aep.planning-md/3
id: story:an-exchange-is-declared-in-the-specification
kind: story
status: draft
title: An exchange is declared in the specification
summary: 'exchanges: at the next ess/N: who sends which message, correlation, open and close; size L'
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- decomposes: epic:messages-are-exchanged-independent-of-the-wire
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
revision: 1
---
## Problem

A specification cannot say that a command opens a conversation in which both sides then send
messages that belong to one row until something ends it (beyond10x/ess#650). ADR
`messages-are-exchanged-independent-of-the-wire` § 1, F1 (a) and F2 (a), puts that statement in the
specification.

## Design

`exchanges:` at the next `ess/N`: `name`, `opens` (a command), `follows` (the entity whose row the
opening outcome creates or addresses), `initiator_sends` (commands), `responder_sends` (events),
`closes` (`states:` of the followed lifecycle, a `command:`, or both), optional `max_followers`
with the error answered over it. Order is the followed row's lifecycle and the commands' guards; no
ordering syntax. Below the new format `exchanges:` is refused as `unsupported_format_version`
naming it; a specification without it keeps its IR bytes.

## Acceptance

- A specification with one exchange validates and compiles; its IR carries the exchange in a
  deterministic order (`cargo test -p ess-domain --locked --test exchanges`, new).
- Refused by name, each with its own `ValidationCode` and path: an `opens` command that creates or
  addresses no row of `follows`; an `initiator_sends` command that addresses no row of `follows`; a
  `responder_sends` event that carries no identity of `follows`; a `closes` state outside the
  lifecycle; a message listed twice or on both sides; `max_followers` below 1 or without an error.
- The same document one format lower is refused naming the new format.
- `FORMAT_HISTORY` gains the row; `ess verify diff` reports an added, removed or changed exchange.

## Size

L: ess-domain (parse, checks), ess-compiler (IR), ess-diff, format history, docs page
`docs/design/message-exchanges.md`.
