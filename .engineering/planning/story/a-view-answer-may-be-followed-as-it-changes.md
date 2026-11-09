---
format: aep.planning-md/3
id: story:a-view-answer-may-be-followed-as-it-changes
kind: story
status: draft
title: A view's answer may be followed as it changes
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#757
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-exchange-is-declared-in-the-specification
- depends_on: story:an-event-channel-may-be-bound-to-server-sent-events
revision: 2
---
## Need

A caller opens one read and receives the view's answer at once, then again every time it changes,
over one open connection; reconnecting gives the current answer again, and nothing is replayed.
beyond10x/ess#757's `GET /stats` is that read, over SSE. The accepted exchange
(`architecture-decision-record:messages-are-exchanged-independent-of-the-wire` § 1) is opened by a
command, follows one entity row, and carries events an outcome emits until the row reaches a closing
state. A followed view has no opening command, follows no row, carries state rather than events, and
never closes. Nothing in ESS states it. Minimal reproduction (view `demo.queues.QueueStats(queue_id,
waiting)`, `consistency: eventual`; ess 0.92.0):

```console
$ ess specify http validate --path spec --binding stream.http.yaml          # body: stream
error[ESS-HTTP-024]: routes[0].body: view `demo.queues.QueueStats` answers `rows`, `array` or `single`, and `stream` is not a view's body
$ ess specify validate --path spec-exchange                                   # exchanges: […]
  - domains/queues.yaml: unknown field `exchanges`, expected one of `format`, …
```

What the request also asks, and where it stands:

- **Frames of a type no entity holds** (a snapshot the component computes): a view without a
  source is `story:a-view-may-answer-values-no-row-holds` (beyond10x/ess#674); its first cut reads
  settings and literals only.
- **A snapshot scoped to the caller** (the caller resolved from a bearer token): refused by decision.
  Views read no caller; read authority is the view grant (`docs/design/language-grids.md:53-54`).
- **`status` frames `loading` / `live` with `catchup_ms`**: the server's own warm-up, not a fact
  about the domain; a view's `consistency: eventual` already says its answer may lag.
- **`keepalive`**: a framing key of the SSE binding,
  `story:an-event-channel-may-be-bound-to-server-sent-events`.

## Acceptance

Written for the recommended answer (an exchange may follow a view); rewritten if the decision goes
the other way.

- `exchanges: [{name: demo.queues.StatsFeed, follows: {view: demo.queues.QueueStats}}]` validates
  and compiles at the exchange's `ess/N`: no `opens`, no `initiator_sends`, no `closes`; the
  responder's one message is the view's answer. Refused by name: `opens`, `initiator_sends`,
  `responder_sends`, `closes` or `max_followers` beside `follows: {view}`; a view that does not
  exist; a paged view.
- An `ess-http` view route may carry `stream: {exchange: demo.queues.StatsFeed, frames: {answer:
  default}, keepalive: 30s}`: `GET /stats` answers `200 text/event-stream`, one unnamed `data:`
  frame with the view's answer on connect and one after each change of its answer, never closing.
- `ess verify conform synthesize` emits, per followed view: the answer on connect; after a command
  that changes a row the view reads, a frame equal to a plain read of the view; after one that
  changes none, no frame within the view's consistency bound; `ess verify conform run --target
  interpreted` passes them.
- A specification without a followed view keeps its IR and suite bytes.

## Scope

- inferred: crates/specify/ess-domain/src/exchange.rs
- inferred: crates/specify/ess-compiler/src/ir.rs
- cited: crates/specify/ess-http/src/lib.rs
- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: docs/design/message-exchanges.md
confidence: low

## Fit review

1. **Need.** A read whose answer is re-sent whenever it changes, over one connection. Reproduction
   above. The requester's proposal (theirs): a route `stream: <type>` with `media:
   text/event-stream`, `frames`, `keepalive` and `replay: last | none | from_id`, in two forms, "a
   stream over a view" and "a stream of a named struct the component computes, which no entity
   holds", the IR saying that only the type is checkable.
2. **Class: gap.** No construct reads a view more than once per request (`docs/design/compiled-view-delivery.md`
   states what a view is; no design page under `docs/design/` defines a followed or live view).
3. **Expressible today? No.** Polling the view is the caller's loop, not the protocol; a stream of
   events through the exchange carries what changed, not the answer the reader sees, so it moves
   the checked figure out of the specification (`docs/design/generic-language-features.md`
   § Policy, policy 1).
4. **Fit.** Grid R's caller column decides one part: views read no caller, so the per-caller scoping
   is refused as decided, not reopened here. The remaining shape is a fork of the accepted ADR that
   its forks F1 to F3 do not cover (`messages-are-exchanged-independent-of-the-wire.md:162-166`):
   whether an exchange may follow a view instead of a row. Reusing the exchange keeps the
   operator's direction, "a way to express that completely independent of whether you are using a
   WebSocket or JSON RPC or whatever" (ADR, "Operator direction"), so one followed view binds to
   SSE and WebSocket alike. A `body: stream` view route in `ess-http` alone would be the per-wire
   answer the ADR was written to avoid. Either way the requester's free-type stream is not taken:
   frames carry a declared view's answer, so the suite checks what the reader sees.
5. **Second adopter.** A trading screen follows `GET /markets/{market_id}/book`, an aggregate view
   of open orders by price, re-sent after every order change.
6. **Cost.** If an exchange: one more `follows:` form in the exchange's own `ess/N` (no format of its
   own, policy 2), a synthesized scenario family and an interpreter loop. If a view-route form: an
   `ess-http` key, and a second stream construct beside the exchange.
7. **Alternatives.** (a) Change nothing: the stream stays unchecked. (b) The requester's
   `stream: <type>`: a free type the suite can only type-check. (c) `follows: {view}` on the exchange
   (recommended). (d) An `ess-http` view body `stream`, outside the exchange layer.

## Decisions

**Accept, redesigned.** The need is real. The design needed a decision that extends an
operator-accepted ADR:

> May an exchange follow a view's answer, opened by a read rather than a command and never closing,
> with the view's answer re-sent on connect and after each change as its one message
> (`follows: {view: <name>}`), so that SSE and WebSocket bindings carry it like any exchange; or
> is a followed view an `ess-http` view-route form outside the exchange layer?

Decided 2026-11-09 by the coordinator of the zero-open-issues run, taking the triage's
recommendation under the operator's standing rule "decide, don't ask; take the reversible option"
(recorded in the run's non-interactive approval-record): the exchange, (c). A followed view is an
exchange with `follows: {view: <name>}`; it adds no second streaming layer beside the exchange, so
the ADR's rule that a stream is an exchange holds unchanged. Also decided: the per-caller scope is refused (grid R, views read no caller); the
`loading`/`live` status frames are local policy and stay out; `replay` is not a key.

## Source

github:beyond10x/ess#757
