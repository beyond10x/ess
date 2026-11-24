---
format: aep.planning-md/3
id: story:a-command-may-be-bound-to-a-broker-subject-as-a-request-with-one-reply
kind: story
status: draft
title: A command may be bound to a broker subject as a request with one reply
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-broker-binding-carries-an-exchange
revision: 1
---
## Need

A service is called over a message bus with request/reply: the caller publishes the request on a
subject, one responder answers on the caller's reply subject, and the caller stops waiting after a
timeout. When no responder listens, the broker answers "no responders" at once. A downstream
application had to leave such a call as an `UNMAPPED` marker (third report, finding 2, bus half).

What ESS has, read in `integration/wave-2026-11-24` at `bcfc51555`:

- `ess-transport/1` and `/2` bind one **event** to one NATS subject. The protocol set is closed at
  `nats` (`crates/specify/ess-transport/src/lib.rs:46-51`), and `docs/design/event-transport-binding.md`
  has no command, reply or timeout. A command has no broker binding at all.
- The accepted ADR `messages-are-exchanged-independent-of-the-wire` says every command is already a
  one-message exchange with one request and one answer (§ 1, "Implicit exchanges"), and that binding B,
  `ess-transport/3`, maps an exchange to broker subjects (§ 2, stage 4).
  `story:a-broker-binding-carries-an-exchange` (draft, size M) designs `/3` for **named**
  exchanges, with a subject per direction. It has no request/reply of a single command, no timeout
  and no no-responders answer.
- A call the system makes is a binding: an event triggers a command another component accepts,
  with `delivery` and failure policies (`docs/design/conditional-binding-failure-policies.md`). An
  untyped port failure is answered by the binding's `on_failure` fallback, and synthesis already
  writes scenarios for it through a scripted port.

Minimal reproduction (written fresh; not run): a component `ledger` accepts
`demo.ledger.ReserveFunds`, and a binding `demo.orders.reserve-on-place` delivers it when
`demo.orders.OrderPlaced` is published. The real system carries `ReserveFunds` as a NATS request on
`ledger.reserve`, with a 2 s timeout.

```yaml
type: ess-transport/2
specification: {system: demo, version: v1, source_digest: "sha256:<digest>"}
brokers: [{id: bus, protocol: nats}]
requests:
  - {command: demo.ledger.ReserveFunds, broker: bus, subject: ledger.reserve, timeout: 2s}
```

```console
$ ess specify transport validate --path ledger.transport.yaml --spec spec
error: reading ledger.transport.yaml as transport YAML: unknown field `requests`, …
```

The requester's proposal is a request/reply call with a timeout and a rule for "no responders".

## Outcome

An `ess-transport/3` document may bind a command that a component accepts to a broker subject as a
request with one reply. It states the broker, the subject and the requester's `timeout`. The
responder's reply is the command's answer, the same envelope a generated server answers. When the
system is the requester, through a binding that delivers the command, a timeout and a
no-responders answer are delivery failures. The binding's existing `on_failure:` (or per-refusal
policy fallback) handles them, so the model gains no new failure vocabulary. AsyncAPI writes a
`receive` operation with a `reply` for the responder's component and a `send` operation with a
`reply` for the requester's.

## Acceptance

Unit a (format and compile):

- An `ess-transport/3` document with the `requests:` entry above validates (exit 0), and `ess specify
  transport compile` writes it into `ess-transport-ir/3`, entries in command order. Every `/1` and
  `/2` fixture compiles to unchanged bytes. The same document at `/2` is refused, naming `requests`
  as a `/3` key.
- Each of these is refused by name, with its path: a command no component accepts; a command bound
  twice; a subject outside the publishable-subject rule (`event-transport-binding.md` rule 4); a
  subject another channel or request of the document already uses; a `timeout` outside
  `1ms..=1h`; an unknown broker.
- `crates/specify/ess-transport/tests/requests.rs` (new) holds each case. Its first test is the
  refused document above, which turns green.
- `docs/design/event-transport-binding.md` gains § "Requests", which states that no-responders and
  timeout are delivery failures read by `on_failure:`, and that a responder that never answers is
  the requester's timeout, never a declared outcome.

Unit b (projection):

- `ess generate --kind asyncapi` with the transport document writes, for the accepting component, a
  `receive` operation on `ledger.reserve` with `reply` (address: the requester's reply subject,
  `x-ess-timeout: 2s`), and for the requesting component a `send` operation with the same `reply`. A
  model without `requests:` keeps its AsyncAPI bytes.
- Each generator that serves or calls commands and cannot serve a broker request refuses a
  transport document with `requests:` by name, instead of ignoring it. Which generators read a
  transport document is listed in the change.

## Fit review

1. **Need.** A command carried as a broker request with one reply, a requester's timeout and the
   broker's no-responders answer cannot be stated. Reproduction above. Report: third report,
   finding 2 (bus half). The requester's proposal is quoted under Need.
2. **Class: gap.** `ess-transport` binds events only (`lib.rs:46-51`, the design page's
   vocabulary table).
3. **Already expressible?** No. Modelling the call as two events (request and reply) leaves the
   pairing, the timeout and the answer out of the specification, so the suite checks none of them
   (policy 1 of `docs/design/generic-language-features.md`).
4. **Fit.** The ADR's binding B for the implicit exchange of one command, in the format the broker
   exchange story opens (`/3`). Reuses: the subject rule, brokers, `delivery` and `on_failure:` of
   bindings, and AsyncAPI 3's `reply` object (the projector that establishes the semantics, as the
   boundary rule asks). Siblings: an `ess-http` route is the HTTP binding of the same implicit
   exchange. Timeouts there are the realization's, so a timeout here is a requester's fact, stated
   in the binding and nowhere in the model.
5. **Second adopter.** A pricing service asked over the bus by a checkout service, with a 500 ms
   timeout and a fallback price when nobody answers: `on_failure: drop` plus a view of the last
   price.
6. **Cost.** `ess-transport/3` and `ess-transport-ir/3` (shared with the broker exchange story, one
   format for both). One new key, `requests:`. Refusal codes: the transport crate's next free codes.
   No `ess/N` change.
7. **Considered.** (a) Change nothing and keep `UNMAPPED`. (b) Chosen: a binding of the implicit
   command exchange. (c) A request/reply construct in the model (`ess/N`). This was rejected: the
   ADR keeps wire facts out of the model. (d) Wait for named exchanges and bind request/reply only
   as a two-message exchange. This was rejected: one command already is the exchange (ADR § 1).

## Decisions

Accept, redesigned, under **AC-D2** (default A): the bus half is designed as this story. The
reporter's "no responders rule" becomes the binding's existing failure policy, not a new rule.
Open: whether the responder's side also needs a generated NATS responder. Recommended: no, not in
this story. A generator that cannot serve a request refuses it by name.

## Scope

- cited: `crates/specify/ess-transport/src/lib.rs` (format constants 23-32, `Protocol` 46-51)
- inferred: `crates/specify/ess-transport/tests/requests.rs` (new)
- cited: `docs/design/event-transport-binding.md`
- inferred: `crates/generate/ess-gen/src/asyncapi.rs` (unit b)
- inferred: `website/docs/reference/formats.md`, `CHANGELOG.md`
- Size: unit a about 450 lines (M), unit b about 250 (S).

## Depends on

- `story:a-broker-binding-carries-an-exchange`: both open `ess-transport/3`. The one that lands first
  opens the format, and the other adds its key to it. They cannot run in parallel on `lib.rs`.
- Not on the exchange layer: one command is an implicit exchange and needs no `exchanges:`.

## Source

A downstream application, third report, finding 2 (ESS 0.98.0).
