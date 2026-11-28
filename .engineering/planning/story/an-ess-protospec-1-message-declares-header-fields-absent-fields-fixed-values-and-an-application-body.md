---
format: aep.planning-md/3
id: story:an-ess-protospec-1-message-declares-header-fields-absent-fields-fixed-values-and-an-application-body
kind: story
status: draft
title: An ess-protospec/1 message declares header fields, absent fields, fixed values and an application body
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 4
---
## Need

The owner's note of 2026-11-24 says that a header and a frame are concepts of a protocol, that the
protocol kind (`ess specify protocol`, `ess-protospec/1`) carries them, and that a transport binding
names the protocol document. Three binding stories need message structure that `ess-protospec/1`
cannot state today: an SSE route (frames), a broker request with one reply (a reply's members, a
status), and an HTTP route that reads a request header.

- A message is a name and body fields, and nothing else (`crates/specify/ess-domain/src/protocol.rs:89-98`).
  No field is a header.
- Every field is required ("A required scalar field", `protocol.rs:65-74`). A header or member that
  may be absent has no form.
- No field has a fixed value. A frame name in SSE's `event:` line, a discriminator member
  (`"kind": "shipment"`) or a status cannot be written as part of the message.
- A message cannot carry an application message: an event's payload, a command's input or a
  command's answer. Only `String`, `Boolean` and `Integer` are admitted
  (`crates/specify/ess-compiler/src/protocol.rs:128-137`), and the document references nothing
  outside itself (`protocol.rs:13`).

Minimal reproduction (written fresh, not run):

```yaml
format: ess-protospec/1
name: job-progress
participants:
  - name: server
    initial: open
    states: [open]
    inputs: [{name: advance}]
    transitions:
      - name: frame
        from: open
        to: open
        trigger: {kind: input, name: advance}
        effects:
          - {kind: send, channel: stream, message: advanced, exchange: {kind: literal, value: s},
             logical: {kind: literal, value: f}, payload: {}}
  - {name: client, initial: open, states: [open], transitions: []}
messages:
  - name: advanced
    headers: [{name: event, type: String, value: advanced}]
    application: body
channels:
  - {name: stream, from: server, to: client, ordering: fifo, capacity: 8, loss: false, duplication: false}
bounds: {max_steps: 50, max_states: 1000, max_time_ms: 60000}
```

```console
$ ess specify protocol validate --path job-progress.yaml
error: $ [MalformedProtocol]: unknown field `headers`, expected `name` or `fields`
```

## Outcome

`ess-protospec/1` gains four optional message keys. A document without them reads, compiles and
executes as before.

- `messages[].headers: [{name, type, required?, value?}]` declares header fields, apart from the
  body `fields`. `name` is the header's name on the wire and follows the identity grammar
  (`compiler/protocol.rs:108-126`). Header and body names of one message share one namespace, so
  `{kind: input, field}` reads either kind of field, and a transmission's `payload` map holds both.
- `required: false` is admitted on a header or a body field. The field may be absent from a send
  and from a transmission. No guard or value source reads it.
- `value: <literal>` gives a header or a body field a fixed value, admitted against its type. A
  send does not write the field: the runtime writes the declared value. A transmission that holds
  another value, or lacks the field, is not that message. This is how a frame name, a discriminator
  member or a status is stated.
- `messages[].application: body | {member: <name>}` says that the message carries one application
  message, whose type the binding that names this message supplies. `body` means the whole body is
  that value, and the message declares no body `fields`. `{member: m}` means the body is an object
  that holds the scalar `fields` and that value under `m`. The runtime and `ess-prototrace/1` never
  read an application value. Its content is checked by the ordinary conformance suite through the
  binding.
- `docs/design/protocol-specification.md` gains § "Message structure". A protocol document states
  structure: places, names, presence and fixed values. It never states bytes. The transport a
  binding names writes the bytes by its standard: HTTP header fields (RFC 9110 § 5), SSE fields
  (the event-stream format of the HTML Living Standard), the message headers of a broker protocol that `ess-transport` names (`nats`). A scalar header is
  written as text (an `Integer` in decimal, a `Boolean` as `true` or `false`), and an application
  value in ESS's JSON encoding. Each binding refuses a protocol message its transport cannot write.
  The boundary sentence (`:19`) names what is now admitted.
- A read API for binding compilers: `CompiledProtocol::message(name)` returns the message's
  headers (name, type, required, fixed value), body fields and application slot, and the channels
  that carry it.

## Acceptance

- **Red first.** The reproduction validates (exit 0), and `ess specify protocol compile` writes the
  new keys into its canonical JSON. Today it is refused with "unknown field `headers`". The case is
  in `crates/specify/ess-compiler/tests/protocol.rs` and a CLI case in `crates/edge/ess-cli/tests/protocol_cli.rs`
  (both existing binaries).
- **Refusals.** Each is refused with a string code and its path, in the style of the existing codes:
  - a header named like another header of the same message ignoring ASCII case, or like one of its
    body fields (`DuplicateIdentity`);
  - a `value` the field's type does not admit (`InvalidFixedValue`);
  - a send whose payload writes a fixed field (`FixedFieldWritten`);
  - a guard or value source that reads a `required: false` field (`OptionalFieldRead`);
  - `application: body` beside body `fields` (`ApplicationBesideFields`);
  - `application: {member}` naming one of the message's fields (`DuplicateIdentity`).
- **Execution.** These are unit tests in `crates/verify/ess-conformance/src/protocol/tests.rs`:
  - a send writes each fixed value;
  - replaying a trace whose sent transmission lacks a fixed value, or holds another one, fails as a
    contradicting observation;
  - an absent optional field is absent from the transmission;
  - the reproduction's `advanced` frame replays with `event: advanced`.
- **Old documents.** `examples/protocols/lost-ack.yaml` and `terminal-response.yaml` keep their
  digests, pinned in the test. An old-reader test holds a frozen copy of the released `Message` and
  `Field` shapes (`deny_unknown_fields`) and shows that each new key is refused by it.
  `ess-prototrace/1` is unchanged.
- **Docs.** `docs/design/protocol-specification.md` § "Message structure" is written. The
  `ess-protospec/1` row of `website/docs/reference/spec-versions.md` names the keys, and
  `CHANGELOG.md` has an entry.
- Each test runs within 60 s, and each test step within 120 s.

## Scope

- cited: `crates/specify/ess-domain/src/protocol.rs` (`Message` 89-98, `Field` 66-74)
- cited: `crates/specify/ess-compiler/src/protocol.rs` (admission 145-240, send check 416-440,
  accessors 37-43)
- cited: `crates/verify/ess-conformance/src/protocol/engine.rs`, `monitor.rs` (fixed values, absent
  fields), `protocol/tests.rs`
- cited: `crates/specify/ess-compiler/tests/protocol.rs`, `crates/edge/ess-cli/tests/protocol_cli.rs`
- cited: `docs/design/protocol-specification.md`, `website/docs/reference/spec-versions.md`, `CHANGELOG.md`
- Not touched: `models/protospec` (it models no message, and three pinned tables list it), `ess/N`,
  `ess-ir/1`, every conformance suite format, `ess-prototrace/1`.
- Size: M, about 450 changed lines.

## Gate

- `task fmt-check`
- `cargo clippy -p ess-domain -p ess-compiler -p ess-conformance -p ess-cli --all-targets --locked -- -D warnings`
- `cargo test -p ess-compiler --locked --test protocol`
- `cargo test -p ess-conformance --locked --lib protocol`
- `cargo test -p ess-cli --locked --test protocol_cli`
- `cargo test -p ess-domain --locked --lib`
- `task ci-lint` (public API of `ess-domain` and `ess-compiler`)
- `task test-xtask` (`CHANGELOG.md`, `docs/`)

Each test runs within 60 s, and each test step within 120 s. A build-pool job ends within 10 minutes
unless it names its time with `slot --timeout` (at most 30 minutes). No Python.

## Depends on

Nothing in the plan. PR-D1, PR-D2 and PR-D3 are answered first (all three are architecture). This
story lands before `pr0-protocol-reply-exchange-and-credential-header`,
`pr0-protocol-reader-tolerance`, the SSE binding, the broker request story and the header-input
story. It shares `crates/specify/ess-domain/src/protocol.rs` with the two other `pr0` units and with
the draft `story:a-protocol-timer-takes-its-duration-from-a-message-field`, so these merge one after
another.

## Fit review

1. **Need.** A binding cannot name the header, frame name, status or data place of a protocol
   message, because the protocol kind cannot state them. Reproduction above. The source is the
   owner's note and three downstream gaps (SSE frames, broker reply, request header).
2. **Class: gap.** The owner names it as the protocol kind's job.
3. **Already expressible?** No. A body field cannot be a header, be absent, be fixed, or carry an
   ESS type (Need).
4. **Fit.** This is the protocol kind's own vocabulary. It is not a cell of the language grids
   (`docs/design/language-grids.md` covers `ess/N`, and this format is a sidecar). It reuses the
   identity grammar, the `TypeRef` scalars, `ValueSource::Input` and the transmission `payload` map.
   It is one design decision with one format (policy 2 of `docs/design/generic-language-features.md`):
   the three `pr0` units open one set of keys in `ess-protospec/1`.
5. **Second adopter.** A device protocol whose notifications are named by a `type` member and
   whose replies carry an optional `retry-after` header.
6. **Cost.** Four optional keys and about six string codes. No format version under PR-D3 A.
7. **Considered.**
   - (a) Frame and header detail in each binding (ADR § 2 as written). Rejected by the note.
   - (b) The document references ESS types through a `specification:` pin. That is a references
     change, so `ess-protospec/2`, and it binds a protocol to one system (PR-D2 B).
   - (c) A codec format of its own (PR-D1 C).
   - Chosen: structure in the protocol document, with the data's type supplied by the binding
     through the application slot.

## Decisions

- PR-D1, PR-D2 and PR-D3 (architecture, defaults A).
- No other open question.

- PR-D1: A, taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET; reversible until a release carries it

- PR-D3: A, taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET; reversible until a release carries it

- A release that carries this key names this pick in its pull-request line; the operator session merges that release only after the owner has read the list.
