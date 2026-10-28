---
format: aep.planning-md/3
id: architecture-decision-record:transport-and-idl-compile-targets
kind: architecture-decision-record
status: accepted
title: 'Transport and IDL compile targets beyond ess-http/1: protobuf IDL projection, JSON-RPC 2.0 and gRPC bindings'
summary: Reverses the 2026-09-15 deferral of protobuf for the projection direction and admits ess-jsonrpc/1 and ess-grpc/1 sibling bindings; import of .proto stays deferred.
relations:
- serves: vision:O2
- informed_by: task:deferred-protocol-ui-bindings
- informed_by: approval-record:ess-evolution-20260915
- informed_by: architecture-decision-record:ess-evolution-02-semantics-before-bindings
- blocks: story:ess-projects-a-protobuf-idl-with-http-annotations
- blocks: story:a-command-binds-to-json-rpc-and-grpc-beside-http
- blocks: story:an-event-channel-may-be-bound-to-server-sent-events
- blocks: story:a-response-stream-follows-one-row-until-a-terminal-state
- blocks: story:an-authored-scenario-claims-a-binding-error-code
revision: 5
transitions:
- {from: "proposed", to: "accepted", at: "2026-10-23T19:36:17Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"approval":1}}, correlation: "ess-0690"}
---
Status: accepted 2026-10-23 by the operator, after reviewing all 15 proposed ADRs in full: "I accept all of them" (see this record's transitions).

## Context

ESS projects one transport today. A command or view binds to HTTP through `ess-http/1`
(`docs/design/http-presentation-binding.md`, scope decided on `decision-blocker:http-surface-scope`),
an event to a broker through `ess-transport/1` (`docs/design/event-transport-binding.md`), and
`ess generate` projects docs, site, docs-ir, schema, openapi and asyncapi
(`crates/edge/ess-cli/src/main.rs:550-562`). There is no IDL projection and no RPC binding.

On 2026-09-15 the operator deferred generic protobuf, with UI and Flutter, out of the ESS evolution
initiative: "Track generic protobuf, UI and Flutter as separate deferred follow-up"
(`approval-record:ess-evolution-20260915`), preserved as `task:deferred-protocol-ui-bindings` and
`docs/design/ess-evolution/feature-preservation.md:41`.

On 2026-10-22 an adopter specifying A2A (beyond10x/a2a) needs the same operations served over
HTTP+JSON, JSON-RPC 2.0 and gRPC, with a protobuf IDL as the normative message source. The
operator's words, relayed by the orchestrator: "technically ESS needs compile targets for all of
these specs".

## Decision

Reverse the 2026-09-15 deferral of protobuf for the projection direction, and admit RPC bindings:

1. **IDL projection.** `ess generate --kind proto` projects a protobuf IDL (`.proto`) from the
   compiled model, with `google.api.http` annotations from `ess-http/1` when present. Field numbers
   are wire identity and live in a sibling binding pinned to the source digest, not in `ess/N`.
   Design: `docs/design/protobuf-projection.md`.
2. **RPC bindings.** Sibling documents `ess-jsonrpc/1` and `ess-grpc/1`, shaped like `ess-http/1`:
   authored separately, pinned to the specification's source digest, compiled against the resolved
   IR into their own `*-ir/1`. Design: `docs/design/rpc-bindings.md`.
3. **The model does not change.** `ess/N` stays transport-free. The one model-level question these
   raise, a streamed answer, is decided in the design pages and, if it needs `ess/N`, takes the
   next format under its own decision.
4. **Still deferred.** Importing an existing `.proto` into ESS, and the UI and Flutter parts of
   `task:deferred-protocol-ui-bindings`.

## Rejected alternatives

- **Keep the deferral.** Adopters of multi-binding protocols keep a hand-written IDL and RPC
  adapters that nothing checks against the model.
- **Transport keys in `ess/N`.** Puts wire choices into the domain model, against ADR 02
  (`architecture-decision-record:ess-evolution-02-semantics-before-bindings`) and the precedent of
  `ess-http/1` and `ess-transport/1`.
- **One generic "binding" format with a transport field.** A generic facet registry, which
  `AGENTS.md` § Boundaries forbids.
- **Field numbers derived from declaration order.** Silent renumbering on reorder breaks every
  deployed client.

## Compatibility and migration

Opt-in only. Without a protobuf, JSON-RPC or gRPC binding every artifact keeps its bytes. Each new
format (`ess-proto/1`, `ess-jsonrpc/1`, `ess-grpc/1`) starts at 1 and refuses unknown keys by name.
Union `oneof` mapping depends on `story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant`.

## Acceptance evidence

None yet. Acceptance is the operator's move of this record to `accepted`, after the two design
pages are reviewed. Until then `story:ess-projects-a-protobuf-idl-with-http-annotations` and
`story:a-command-binds-to-json-rpc-and-grpc-beside-http` stay in draft and are not implemented.

## Amendment 1 (proposed 2026-10-22, accepted with this record 2026-10-23): a bound on response streams

Adopter repro (beyond10x/a2a, tracker row 37, ESS 0.67.0): A2A `SendStreamingMessage` and
`SubscribeToTask` keep a stream open that follows one task. The adopter caps the number of streams
following a task and answers a request over the cap with an error, all by hand (a2a `router.rs:514-518`).
The specification cannot state the cap or the error, so no suite checks either.

ESS has no response-stream construct yet: `story:a-response-stream-follows-one-row-until-a-terminal-state`
is blocked on this record, and `docs/design/rpc-bindings.md` § 6 leaves open whether a streamed answer
is binding-only or an `ess/N` construct. A bound belongs to the same decision.

| | what it is | cost |
|---|---|---|
| A | **The bound is part of the stream construct.** Wherever the streamed answer is declared (`ess/N` or binding, per § 6), it may declare `max_followers: <n>` and an error to answer a request over it. | One key and one error reference on the stream declaration; synthesis opens `n` streams on one row and expects the declared error on the next, with no effect. |
| B | **The bound is deployment configuration.** A runtime setting (as `--max-subscribers` is in the adopter's server), outside the specification. | Nothing in the specification; the cap and its error stay unchecked. |

Recommendation: A, decided together with § 6, so the bound and its over-bound error sit wherever the
stream is declared. `story:a-response-stream-follows-one-row-until-a-terminal-state` gains two
acceptance lines when this record is accepted: a stream declaration may carry a bound, and the request
over the bound is answered with the declared error and changes nothing.

Effect: no change beyond what § 6 decides for the stream itself; the bound adds one key, one error
reference and one synthesized scenario per bounded stream. Without a bound every artifact keeps its
bytes.

## Amendment 2 (accepted 2026-10-28, withdrawn the same day): a WebSocket upgrade is a sibling binding

Withdrawn 2026-10-28, the same day, by the operator: "Actually, I think we also kind of need B. Or I
mean, we might even need A, B, and C. [...] I think still the most important is a generic transport
one, yeah, because I mean this is about messages being exchanged, and there should be a way to
express that completely independent of whether you are using a WebSocket or JSON RPC or whatever.
[...] we shouldn't restrict us to one or the other option." The direction is proposed in
`architecture-decision-record:messages-are-exchanged-independent-of-the-wire`; nothing below is
built, and `ess-websocket/1` returns there as one binding of three. The text below is kept as it was
accepted.

Status: accepted 2026-10-28 by the operator: "option A for the ESS WebSocket one." Recorded in
`approval-record:websocket-upgrade-is-a-sibling-binding`, which clears
`decision-blocker:websocket-upgrade-binding-form`.

beyond10x/ess#650 asks for a WebSocket upgrade beside the JSON-RPC binding Decision 2 already
admits. `ess-http/1` keeps refusing `101` as an outcome (`docs/design/http-presentation-binding.md:337`):
an upgrade is not an HTTP answer to one command.

**Decision.** A third sibling document, `ess-websocket/1`, shaped like `ess-jsonrpc/1`: authored
separately, pinned to the specification's source digest, refusing unknown keys by name. It lists
sockets, each with a path, a subprotocol, the command that opens it, the commands the client sends on
it and the events the server sends on it. It compiles against the resolved IR into
`ess-websocket-ir/1` and is projected into AsyncAPI 3's `ws` binding by `ess-gen/src/asyncapi.rs`.
`ess/N` does not change.

**Rejected.** `ess-transport/3` with `brokers[].protocol: websocket`: the broker and stream
vocabulary does not fit a point-to-point socket, and it leans to the generic binding this record
rejects. An `ess-http/2` route key `upgrade:`: moves `ess-http/2` and `ess-http-ir/2`, needs AsyncAPI
from the HTTP crate, and breaks the sibling pattern.

**Compatibility.** Opt-in; without an `ess-websocket/1` document every artifact keeps its bytes.
