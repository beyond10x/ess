---
format: aep.planning-md/3
id: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
kind: architecture-decision-record
status: accepted
title: Messages are exchanged independent of the wire; WebSocket, broker, HTTP upgrade and JSON-RPC bind the exchange
summary: 'Proposed: a transport-neutral exchange layer, with A, B and C of #650 and ess-jsonrpc/1 as bindings side by side'
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- informed_by: architecture-decision-record:transport-and-idl-compile-targets
- informed_by: architecture-decision-record:ess-evolution-02-semantics-before-bindings
- informed_by: decision-blocker:websocket-upgrade-binding-form
revision: 3
transitions:
- {from: "proposed", to: "accepted", at: "2026-10-28T10:54:27Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"approval":1}}, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
Status: accepted 2026-10-28 by the operator: "F1, F2, F3, AAA, all as you suggested."
(`approval-record:exchange-adr-accepted-as-recommended`). The forks below are decided as
recommended: F1 (a), F2 (a), F3 (a).

## Operator direction

The operator, 2026-10-28, on `decision-blocker:websocket-upgrade-binding-form` (beyond10x/ess#650,
WebSocket half), first: "option A for the ESS WebSocket one."

Then, the same day, superseding that answer: "Actually, I think we also kind of need B. Or I mean, we
might even need A, B, and C. So it's just a matter of, yeah, how we want to specify things, and most
likely different ways should be possible. I think still the most important is a generic transport
one, yeah, because I mean this is about messages being exchanged, and there should be a way to
express that completely independent of whether you are using a WebSocket or JSON RPC or whatever. So
there need to be some interplay between them, and we shouldn't restrict us to one or the other
option."

A, B and C are the options of that blocker: a sibling `ess-websocket/1` (A), a broker binding in
`ess-transport` (B), an HTTP upgrade route key (C). This record proposes the transport-neutral layer
the second message asks for, and lets all three bind it.

## Context

- A command answers once. `ess-http/1` binds that one request and one answer to a route; the draft
  `ess-jsonrpc/1` and `ess-grpc/1` (`docs/design/rpc-bindings.md`) bind it to a method.
- An event is published once. `ess-transport/2` binds it to a NATS subject
  (`docs/design/event-transport-binding.md`); the protocol set is closed at `nats`.
- Nothing states a conversation: a party opens it, both sides send messages in an order the
  protocol fixes, the messages belong together, and something ends it. Four requests need exactly
  that and are blocked or drafted apart from each other:
  - #650, a `GET` answering `101` and then messages both ways on a socket (`ess-http/1` refuses
    `101` as an outcome, ESS-HTTP-023, `docs/design/http-presentation-binding.md:337`);
  - `story:a-response-stream-follows-one-row-until-a-terminal-state` and Amendment 1 of
    `architecture-decision-record:transport-and-idl-compile-targets` (a bound on followers);
  - `story:an-event-channel-may-be-bound-to-server-sent-events`;
  - `docs/design/rpc-bindings.md` § 6, which leaves open whether a streamed answer is binding-only or
    a model construct.
- Today each would be answered per wire, and the same conversation spoken over a WebSocket, a broker
  and JSON-RPC notifications would be specified three times, with nothing checking the three agree.

## Decision (proposed)

### 1. A message exchange, independent of the wire

An **exchange** is a named conversation between two parties, the *initiator* and the *responder*
(the component that serves it), stated with constructs the specification already has:

```yaml
exchanges:
  - name: voice.calls.Session
    opens: voice.calls.Connect          # a command the initiator sends; its success outcome opens the exchange
    follows: voice.calls.Call           # correlation: the row the opening outcome creates or addresses
    initiator_sends: [voice.calls.SendAudio, voice.calls.Hangup]          # commands
    responder_sends: [voice.calls.AudioReceived, voice.calls.CallEnded]   # events
    closes: {states: [Ended]}           # the followed row reaching one of these ends it
    max_followers: 4                    # optional; Amendment 1's bound, with its error
```

- **Which party sends which message:** the initiator sends commands, the responder sends events
  (and the outcome of each command the initiator sent).
- **Correlation:** every message belongs to the exchange of the row it addresses. A command
  addresses the followed row through its identity input; an event carries the identity.
- **Order:** the followed row's lifecycle and each command's guards. A command is admitted in the
  states its outcomes admit; an event is sent when an outcome emits it. No second ordering
  language (fork F2).
- **Open and close:** `opens` names the command; `closes` names terminal states, a command, or
  both. A message after close is refused with a declared error.
- **Implicit exchanges:** every command is already a one-message exchange (one request, one
  answer) and every event a one-message exchange with no initiator message. Nothing changes for
  them; they are what today's bindings bind.

The layer carries no wire word: no path, subject, frame, method name or status.

### 2. Bindings map an exchange to a wire, side by side

Each binding stays its own typed document with a closed vocabulary, pinned to the specification's
source digest, and refers to an exchange by its name. Several bindings of one exchange may coexist;
compiling each one checks it against the same exchange.

| binding | what it states for an exchange | format |
|---|---|---|
| A: `ess-websocket/1` (new sibling) | subprotocol, how a frame names its message (a discriminator field and the wire name per message), text or binary frames, ping interval; the path when no `ess-http` route carries the upgrade | new, starts at 1 |
| B: `ess-transport` broker channel | a subject or topic per message direction, the correlation token in the subject, delivery; `brokers[].protocol` gains members beyond `nats` one at a time | `ess-transport/3` |
| C: `ess-http` route key `upgrade:` | the handshake only: method, path, path parameters, the `101` and the refusals before it, and `upgrade: {exchange: <name>, protocol: websocket}` | additive in `ess-http/1` if the format policy admits it (as D137's operation ids), else `ess-http/2` |
| `ess-jsonrpc/1` (draft, not built) | a method per command (the implicit unary exchange); for a named exchange, notifications for the responder's messages and the method that opens it | its first version is written as a binding of this layer |
| SSE (the draft story) | an exchange with no `initiator_sends` beyond `opens`, over `text/event-stream` | the story's own format decision |

**Interplay of A and C:** C says where the socket is opened and what the handshake answers; A says
how messages travel once it is open. An exchange spoken over a socket opened by an HTTP route has
both; a socket server with no HTTP surface has A alone, with its path (fork F3).

### 3. How a binding refers to the exchange, and what is projected

- A binding names the exchange (`exchange: voice.calls.Session`) and, per message the exchange
  lists, its wire spelling. Compiling refuses a message the exchange does not list, a listed message
  the binding leaves unbound, and a binding of a message in the wrong direction, each by name.
- **AsyncAPI 3** (`ess-gen/src/asyncapi.rs`) derives from the exchange: one channel per bound
  exchange, a `receive` operation per `initiator_sends` message and a `send` operation per
  `responder_sends` message from the responder's side, `x-ess-exchange` naming the exchange, and the
  binding's own protocol section (`ws` from A, `nats` from B). Without a bound exchange, output keeps
  its bytes.
- **OpenAPI** keeps describing request and answer: a C route becomes a `GET` whose `101` response
  names the AsyncAPI channel in `x-ess-exchange`; OpenAPI does not describe the messages.
- **Conformance** synthesizes exchange scenarios from the layer alone (open, each admitted message
  in each state, a message after close, the follower bound), and a target adapter per binding
  carries them over its wire, as `ess-http/1` stage 3 does for routes.

### 4. What this reverses in this record, and what it keeps

- **Reversed:** the rejected alternative "One generic binding format with a transport field" was
  read so far as "bindings share nothing above the specification". This record adds one shared,
  typed layer above the wire, because the operator asked for messages to be expressible "completely
  independent of whether you are using a WebSocket or JSON RPC or whatever", and because the four
  requests above would otherwise each define the same conversation on their own wire.
- **Kept:** no generic binding document with a free `transport:` field, no facet registry and no
  property bag (`AGENTS.md` § Boundaries; ADR 02). The exchange is a typed construct with a closed
  set of keys; every binding is still its own format with its own vocabulary and refusal codes.
- **Withdrawn:** Amendment 2 as accepted on 2026-10-28 (A alone). A remains, as one binding of
  three.
- **Answered:** `docs/design/rpc-bindings.md` § 6: a streamed answer is an exchange; Amendment 1's
  bound is the exchange's `max_followers`.

### 5. Migration

- `ess-http/1`, `ess-transport/1` and `/2` documents keep validating and every artifact keeps its
  bytes: they bind implicit exchanges and name none.
- `ess-jsonrpc/1` is a draft design page; no crate parses it (`grep -ril jsonrpc crates/` finds only
  a vendored asset), so no document to migrate exists. Its draft syntax (methods keyed by command)
  is already a binding of implicit exchanges and stays valid as written.
- A specification without `exchanges:` keeps its source format and bytes.

### 6. Stages (after acceptance)

1. The exchange layer, its compile checks, AsyncAPI channels without a wire, synthesized exchange
   scenarios.
2. A, `ess-websocket/1` (closes #650 with stage 3).
3. C, the `upgrade:` route key.
4. B, `ess-transport/3` exchange channels.
5. `ess-jsonrpc/1` notifications and SSE, when their stories are scheduled.

## Forks still open

| fork | options | recommendation |
|---|---|---|
| F1 where the exchange lives | (a) in the specification, `exchanges:` at the next `ess/N`; (b) a sibling `ess-exchange/1` pinned to the specification, every binding reading both | (a): which party sends what, and when, is protocol semantics, which ADR 02 puts in the model; conformance synthesizes it without a wire; the cost is one `ess/N` |
| F2 how order is stated | (a) the followed row's lifecycle and the commands' guards; (b) an explicit sequence or state machine on the exchange | (a): no second ordering language, and the existing synthesis and interpreter already enforce it; (b) stays possible for a protocol whose order no row records |
| F3 a socket stated by A and C | (a) C carries the handshake and A the framing, A states a path only without C; (b) either may state everything and compiling refuses disagreement; (c) one of them per exchange | (a): each fact has one home, and both stay usable alone |

## Consequences

- `story:http-binding-carries-transport-framing` (WebSocket half), the response-stream story, the
  SSE story and the JSON-RPC notification design wait on this record and are re-planned against it.
- Nothing is built before acceptance. The open-issue batches proceed without #650.
