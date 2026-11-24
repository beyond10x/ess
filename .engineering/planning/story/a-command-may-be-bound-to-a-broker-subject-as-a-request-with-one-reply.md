---
format: aep.planning-md/3
id: story:a-command-may-be-bound-to-a-broker-subject-as-a-request-with-one-reply
kind: story
status: draft
title: A command is bound to a broker subject as a request with one reply, its correlation, timeout, reply members and their reading stated by a protocol document
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-broker-binding-carries-an-exchange
- depends_on: story:an-ess-protospec-1-message-declares-header-fields-absent-fields-fixed-values-and-an-application-body
- depends_on: story:an-ess-protospec-1-reply-answers-on-the-exchange-of-the-message-it-received-and-a-credential-header-is-named
- depends_on: story:an-ess-protospec-1-participant-reads-named-members-of-one-message-in-any-case-and-may-ignore-the-others-and-a
revision: 4
---
**Revised on 2026-11-24 after the owner's note and the operator's additions.** Request/reply is a
command and its response, and the broker is only the transport. Correlation, the timeout, the
no-responders answer, the reply's members and the requester's case-insensitive reading of two of
them are wire parts. They are stated in the protocol document the binding names. The binding keeps
the broker, the request subject, and how reply values map to the command's existing outcomes.

## Need

A downstream application calls a service over a message broker with request/reply. Its component
is the requester. It publishes the request on a subject, one responder answers on the requester's
reply subject, and the requester stops waiting after a timeout. When no responder listens, the
broker answers "no responders" at once.

The replier writes its reply as a JSON object with capitalised member names: a status, an error and
two more. The requester reads the status and the error in any case and ignores the other two. The
application had to leave the call as an `UNMAPPED` marker (third report, finding 2, bus half).

What exists, and is not restated here:

- **The semantics.** The command, its outcomes and its errors. When the system is the requester, a
  model binding delivers the command, and that binding's `on_failure:` or per-refusal policies
  (`docs/design/conditional-binding-failure-policies.md:9-20`) already answer an untyped failure of
  the call. Synthesis writes those scenarios through a scripted port. The accepted ADR
  `messages-are-exchanged-independent-of-the-wire` counts every command as a one-message exchange
  with one request and one answer (§ 1, lines 87-89).
- **The wire parts.** They have no home today. `ess-transport/1` and `/2` bind events only
  (`crates/specify/ess-transport/src/lib.rs:23-32`, protocols closed at `nats` in `:46-58`), and
  `ess-transport/3` is refused (`tests/compile.rs:176`). Under the note they go to an
  `ess-protospec/1` document, which needs `pr0-protocol-message-structure`,
  `pr0-protocol-reply-exchange-and-credential-header` and `pr0-protocol-reader-tolerance`.

Minimal reproduction (written fresh, not run). Component `ledger` accepts `demo.ledger.ReserveFunds`,
and the system calls it from a binding `demo.orders.reserve-on-place` when `demo.orders.OrderPlaced`
is published.

```yaml
type: ess-transport/3
specification: {system: demo, version: v1, source_digest: "sha256:<hex>"}
protocol: "sha256:<compiled digest of ledger-call>"
brokers: [{id: bus, protocol: nats}]
requests:
  - command: demo.ledger.ReserveFunds
    broker: bus
    subject: ledger.reserve
    requester: requester
    request: request
    reply: {message: reply_envelope}
    failures: [timed_out, no_responders]
```

```console
$ ess specify transport validate --path ledger.transport.yaml --spec spec --protocol ledger-call.yaml
error: unexpected argument '--protocol' found
```

## Outcome

**The protocol document** (`ledger-call`, abbreviated; the `broker` participant, the channels and
the bounds are left out):

```yaml
participants:
  - name: requester
    initial: idle
    states: [idle, waiting, answered, failed]
    fields: [{name: pending, type: String, initial: ""}]
    inputs: [{name: call, fields: [{name: id, type: String}]}]
    timers: [timeout]
    reads: [{message: reply, any_case: [Status, Error], other_members: ignore}]
    transitions:
      - {name: send, from: idle, to: waiting, trigger: {kind: input, name: call},
         effects: [{kind: send, channel: requests, message: request, exchange: {kind: input, field: id}, logical: {kind: literal, value: q}, payload: {}},
                   {kind: set, field: pending, value: {kind: input, field: id}},
                   {kind: arm, timer: timeout, after: {kind: fixed, millis: 2000}}]}
      - {name: answer, from: waiting, to: answered, trigger: {kind: receive, channel: replies, message: reply},
         guards: [{left: {kind: exchange}, right: {kind: local, field: pending}}],
         effects: [{kind: cancel, timer: timeout}, {kind: emit, name: answered}]}
      - {name: nobody, from: waiting, to: failed, trigger: {kind: receive, channel: replies, message: no_responders},
         effects: [{kind: cancel, timer: timeout}, {kind: emit, name: no_responders}]}
      - {name: late, from: waiting, to: failed, trigger: {kind: timer, name: timeout}, effects: [{kind: emit, name: timed_out}]}
  - name: responder   # receive request; send reply on {kind: exchange}
messages:
  - {name: request, application: body}
  - name: reply
    fields: [{name: Status, type: Integer}, {name: Error, type: String, required: false}]
  - {name: reply_envelope, application: body}
  - name: no_responders
    headers: [{name: status, type: Integer, value: 503}]
properties: [{kind: event_count, name: one-answer, participant: requester, event: answered, max: 1}]
```

**Where each wire part lives, and why:**

| part | home | why |
|---|---|---|
| broker, **request subject** | the binding (`requests[].subject`) | A subject is a broker address, as a path is a binding fact in `ess-http/1`. `ess-transport` already owns subjects and their checks (publishable-subject rule 4, stream capture rule 7, address parameters in `/2`; `docs/design/event-transport-binding.md:68-81`). A protocol document is transport-neutral. (PR-D8 A) |
| reply subject | nobody authors it | It is the exchange identity the requester draws (`exchange: {kind: input, field: id}`). The NATS codec writes it as the message's reply-to, and the responder answers on `{kind: exchange}`. |
| correlation, one reply | the protocol | the guard on `{kind: exchange}`, and the `one-answer` property |
| timeout | the protocol | the `timeout` timer, 2000 ms |
| no-responders | the protocol | a message with the fixed header `status: 503`. The design page states that the NATS codec reads the broker's no-responders status as that header. |
| reply members, read in any case, others ignored | the protocol | the requester's `reads` entry. It is one reader's tolerance, and the replier writes one case. |
| request body | the protocol | `request.application: body`, typed by the binding as the command's input in ESS's JSON |
| reply values to outcomes | the binding | `reply.outcomes` (unit b), as `ess-http/1` maps outcomes to statuses |
| which requester ends are delivery failures | the binding | `failures`, read by the delivering binding's `on_failure:` |

**The binding** (`ess-transport/3`):

- A `requests[]` entry names `command`, `broker`, `subject`, the protocol participant `requester`,
  the protocol message `request` that carries the command's input, and a `reply`. The reply is one
  of two forms:
  - `{message}` naming a message with `application: body`. It carries the command's answer in the
    envelope a generated server answers (unit a).
  - `{message, outcomes: {<outcome>: {<member>: <value>, error?: {<error field>: <member>}}}}`. It
    reads the replier's own members (unit b):

    ```yaml
    reply:
      message: reply
      outcomes:
        Reserved: {Status: 200}
        InsufficientFunds: {Status: 409, error: {reason: Error}}
    ```

- `failures` lists the requester's `emit` names that end it without a reply. When a model binding
  delivers the command (the system is the requester), each one is a delivery failure that binding's
  `on_failure:` handles. The model gains no failure word.
- Members are named in their declared spelling. The binding never restates the case rule.
- A `credential` header on the request is refused: a broker binding has no authentication port.
- Top-level `protocol: sha256:<hex>`, with `--protocol <file>` beside `--transport` (PR-D2 A).

**No new semantics.** The command's scenarios and the delivering binding's failure scenarios are
unchanged, and no conformance format changes. Running them over the broker is a `--target process`
adapter that speaks it (PR-D7 A). The protocol obligations run against the requester's
implementation through the native protocol adapter, with the variants of
`pr0-protocol-reader-tolerance`: correlation, one reply, the timeout, no-responders, and a reply
whose `Status` and `Error` are spelled `status` and `ERROR` that is still read.

## Acceptance

Unit a (format, link, envelope reply):

- **Red first.** With `--protocol ledger-call.yaml`, the reproduction validates (exit 0), and
  `ess specify transport compile` writes it into `ess-transport-ir/3`, with entries in command
  order. Today the flag is an unexpected argument, and `/3` is refused. The cases are in
  `crates/specify/ess-transport/tests/compile.rs` and `crates/edge/ess-cli/tests/transport.rs`
  (existing binaries).
- **Old documents.** Every `/1` and `/2` fixture compiles to unchanged bytes. The same document at
  `/2` is refused, naming `protocol` and `requests` as `/3` keys (the old-reader test).
- **Refusals.** Each is refused with the transport crate's next free codes (`ESS-TRANSPORT-021`
  onward when the unit lands), each with its path:
  - a command no component accepts, or one bound twice;
  - a subject outside rule 4, or one another channel or request of the document uses;
  - an unknown broker;
  - `protocol:` without `--protocol`, or a digest that differs;
  - a `requester` that is not a protocol participant;
  - a `request` message without an application slot, or one the requester never sends;
  - a reply message the requester never receives, or (in the envelope form) one without
    `application: body`;
  - a `failures` name that is not an `emit` of the requester;
  - a credential header on the request.
- **Docs.** `docs/design/event-transport-binding.md` gains § "Requests". It covers the wire-part
  table above, the reply subject as the exchange identity, the NATS codec's no-responders reading,
  failures read by `on_failure:`, and "a responder that never answers is the requester's timeout,
  never a declared outcome".

Unit b (reply members mapped to outcomes):

- **Red first.** The `reply: {message: reply, outcomes: …}` form above validates and compiles. In
  the IR each outcome holds its member values and its error-field sources. Before unit b it is
  refused, naming `outcomes`. The case is in `crates/specify/ess-transport/tests/compile.rs`.
- **Refusals.** Each is refused by name:
  - an outcome of the command left unmapped, or a name that is not an outcome of it;
  - two outcomes on equal member values, which are indistinguishable;
  - a member that is not a field of the reply message, or is written in another spelling;
  - a value the member's type does not admit;
  - an `error` field the outcome's error does not declare, or one whose type a scalar member
    cannot carry;
  - an optional member used to tell outcomes apart.

Unit c (projections and generators):

- `ess generate --kind asyncapi --transport … --protocol …` writes, for the responder's component, a
  `receive` operation on `ledger.reserve` with a `reply` whose address is the request's reply-to.
  For the requester's component it writes a `send` operation with the same `reply`. Each message
  gets `headers` from the protocol's header fields, `x-ess-protocol` the digest, the reply message
  `x-ess-any-case: [Status, Error]` on the requester's side, and the no-responders message listed.
  A model without `requests:` keeps its AsyncAPI bytes. The case is in
  `crates/generate/ess-gen/tests/asyncapi_transport.rs`.
- `ess-publisher`, the one generator besides AsyncAPI that reads a transport document
  (`crates/generate/ess-publisher/src/lib.rs:1-2`), refuses a document with `requests:` by name
  (PR-D9 A). The case is in `crates/generate/ess-publisher/tests/generated.rs`.

Each test runs within 60 s, and each test step within 120 s.

Amendment (ninth gap report, 2026-11-24): the request's subject may read an input as a whole-token `{name}` parameter, as `ess-transport/2` subjects do.

## Scope

- cited: `crates/specify/ess-transport/src/lib.rs` (formats 23-32, `Protocol` 46-58, `SpecificationIdentity`
  35-45), `crates/specify/ess-transport/tests/compile.rs`
- cited: `crates/edge/ess-cli/src/transport.rs`, `crates/edge/ess-cli/src/main.rs` (`--transport`
  290, 1823-1851), `crates/edge/ess-cli/tests/transport.rs`
- cited: `crates/generate/ess-gen/src/asyncapi.rs`, `crates/generate/ess-gen/tests/asyncapi_transport.rs` (unit c)
- cited: `crates/generate/ess-publisher/src/lib.rs`, `crates/generate/ess-publisher/tests/generated.rs` (unit c)
- cited: `docs/design/event-transport-binding.md`, `website/docs/reference/formats.md`, `website/docs/reference/spec-versions.md`,
  `CHANGELOG.md`
- Size: unit a about 520 changed lines (M), unit b about 320 (M), unit c about 260 (S).

## Gate

- `task fmt-check`
- `cargo clippy -p ess-transport -p ess-cli -p ess-gen -p ess-publisher --all-targets --locked -- -D warnings`
- `cargo test -p ess-transport --locked` (its two binaries: `compile`, `adversary_391_pass1`)
- `cargo test -p ess-cli --locked --test transport`
- `cargo test -p ess-gen --locked --test asyncapi_transport` (unit c)
- `cargo test -p ess-publisher --locked --test generated` (unit c)
- `task ci-lint`
- `task test-xtask` (format row, `CHANGELOG.md`, docs)

Each test runs within 60 s, and each test step within 120 s. A build-pool job ends within 10 minutes
unless it names its time with `slot --timeout` (at most 30 minutes).

## Depends on

- `pr0-protocol-message-structure`, `pr0-protocol-reply-exchange-and-credential-header` and
  `pr0-protocol-reader-tolerance`.
- PR-D1, PR-D2 and PR-D3 (architecture). PR-D8 and PR-D9.
- `story:a-broker-binding-carries-an-exchange`: both open `ess-transport/3`. The one that lands
  first opens the format and adds `protocol:`, and the other adds its key. They cannot run in
  parallel on `lib.rs`.
- Unit b and unit c come after unit a.
- Not on the exchange layer: one command is an implicit exchange.

## Fit review

1. **Need.** A command carried as a broker request with one reply cannot be stated. That covers its
   subject, its correlation, its timeout, the broker's no-responders answer, a reply in the
   replier's own shape, and the requester's case-insensitive reading of two members. Reproduction
   above. Source: third report, finding 2 (bus half), and the operator's additions.
2. **Class: gap.** `ess-transport` binds events only.
3. **Already expressible?** No. Two events (request and reply) leave out the pairing, the timeout
   and the answer, so the suite checks none of them (policy 1 of
   `docs/design/generic-language-features.md`).
4. **Fit.**
   - The ADR's binding B for the implicit exchange of one command, in `/3`.
   - The wire parts live in the protocol document, under the note.
   - Reused: brokers, the subject rule, `on_failure:` of bindings, `ess-http`'s outcome mapping
     shape, and AsyncAPI 3's `reply` object (the projector that establishes the semantics, as the
     boundary rule asks).
5. **Second adopter.** A checkout service asks a pricing service with a 500 ms timer, and its
   binding's `on_failure: drop` falls back to a view of the last price.
6. **Cost.** `ess-transport/3` and `ess-transport-ir/3`, shared with the broker exchange story. Two
   new keys (`protocol`, `requests`), about ten codes, no `ess/N` change.
7. **Considered.**
   - (a) Keep `UNMAPPED`.
   - (b) A request/reply construct in the model (`ess/N`). Rejected: wire facts stay out of the
     model.
   - (c) Timeout, no-responders and case rule as binding keys (the earlier body). Refused by the
     note.
   - (d) The subject in the protocol document (PR-D8 B).
   - Chosen: binding B with the wire parts in the protocol document.

## Decisions

**Accept, redesigned again**, under AC-D2 (default A).

- PR-D8 (default A): the request subject is the binding's.
- PR-D9 (default A): no generated requester or responder.
- PR-D7 (default A).
- PR-D1, PR-D2 and PR-D3 (architecture).
- AC-D3, AC-D4 and AC-D5 are untouched and still open.

## Source

A downstream application, third report, finding 2 (ESS 0.98.0), and the reporter's detail on the
reply relayed on 2026-11-24.
