---
format: aep.planning-md/3
id: story:an-ess-protospec-1-participant-reads-named-members-of-one-message-in-any-case-and-may-ignore-the-others-and-a
kind: story
status: draft
title: An ess-protospec/1 participant reads named members of one message in any case and may ignore the others, and a target run proves it
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-ess-protospec-1-message-declares-header-fields-absent-fields-fixed-values-and-an-application-body
revision: 1
---
## Need

A downstream application calls a service over a message broker as a request with one reply. Its
own component is the requester and only reads the reply. The replier writes the reply's JSON member
names in one fixed case (capitalised). The requester reads two of the members, a status and an
error, in any case, and does not read the other two. The owner's operator, 2026-11-24: this
case-insensitive reading is a wire rule of the protocol. It belongs in the protocol document, not
in the binding and not in the command.

It is a property of **one participant reading named members of one message**. It is not a property
of the message: the replier writes one case, and another reader of the same message may be strict.
`ess-protospec/1` cannot state it.

- Every name is compared exactly: identities (`crates/specify/ess-compiler/src/protocol.rs:108-126`),
  payload maps (`crates/specify/ess-domain/src/protocol.rs:347-376`), and a trace's transmission
  payload (`crates/verify/ess-conformance/src/protocol/mod.rs:155-157`).
- Every message is closed: "no undeclared payload fields are admitted" (`protocol.rs:89`). A reader
  that ignores members it does not read cannot be stated.
- A delivery hands the reader the transmission exactly as sent (`Action::Deliver`, `mod.rs:29-33`),
  so no run can show that a reply spelled in another case is still read.

Minimal reproduction (written fresh, not run). A message
`reply {fields: [{name: Status, type: Integer}, {name: Error, type: String, required: false}]}`, and
on the participant `requester`:

```yaml
reads:
  - {message: reply, any_case: [Status, Error], other_members: ignore}
```

```console
$ ess specify protocol validate --path call.yaml
error: $ [MalformedProtocol]: unknown field `reads`, expected one of `name`, `initial`, `states`, `fields`, `inputs`, `timers`, `transitions`
```

## Outcome

- **`participants[].reads: [{message, any_case?, other_members?}]`.** It states how this
  participant reads one message it receives.
  - `any_case: [<field>…]` names header or body fields of that message. The participant matches
    each of them by name ignoring ASCII case. The declared spelling is the one a sender writes.
  - `other_members: ignore` means the participant accepts the message when it holds members the
    declaration does not name, and reads none of them. Absent, the message is closed, as today.
  - Absent `reads`, every name is exact and every message closed, which is today's meaning.
- **Admission.** The message is one the participant receives (a `receive` trigger names it). Each
  `any_case` name is a field of that message. No two fields of the message are equal ignoring case
  when one of them is listed. One entry per message.
- **Runtime and trace.** A `deliver` action may say how the copy was written. That takes two
  optional keys, both skipped when empty, so additive in `ess-prototrace/1` under PR-D3 A:
  - `as_written: {<declared field>: <spelling>}` respells a field that differs from the declared
    name only in ASCII case;
  - `extra_members: [<name>…]` adds members the declaration does not name.
  The runtime enables such a delivery only for a reader whose `reads` admits it. Otherwise it is
  not an enabled action, and replay of a trace that holds it fails.
- **What a target run sends.** `tolerance_variants(model, actions)` is a new public function next
  to `run_target` (`protocol/target.rs:94-110`). It expands an action sequence. For every `deliver`
  to a tolerant reader it adds runs in which:
  - the `any_case` fields are written all lower-case, then all upper-case, skipping a spelling equal
    to the declared one;
  - with `other_members: ignore`, one undeclared member is added.
  A target (the requester's implementation behind the native adapter) passes when each variant run
  shows the same observations as the run with the declared spelling. A reply in another case is
  still read. `explore` offers the same variants within the declared bounds.
- `docs/design/protocol-specification.md` § "Message structure" gains "Reader tolerance". It
  describes one reader, says what a target run sends, and states that a binding never restates the
  rule.

## Acceptance

- **Red first.** The reproduction validates (exit 0). Today it is refused with "unknown field
  `reads`". The case is in `crates/specify/ess-compiler/tests/protocol.rs`.
- **Refusals.** Each is refused with a string code and its path:
  - a `reads` entry for a message the participant never receives (`ReadsUnreceivedMessage`);
  - an `any_case` name that is not a field of the message (`UnknownField`);
  - a listed field equal ignoring case to another field of the message (`AmbiguousAnyCase`);
  - two entries for one message (`DuplicateIdentity`).
- **Runtime.** These are unit tests in `crates/verify/ess-conformance/src/protocol/tests.rs`:
  - a reply delivered with `as_written: {Status: status, Error: ERROR}` takes the same transition
    as the declared spelling;
  - the same delivery to a participant without `reads` is not enabled, and the trace fails;
  - `extra_members: [Trace]` is enabled only under `other_members: ignore`;
  - `tolerance_variants` yields the lower-case, upper-case and extra-member runs for the
    reproduction and nothing for a strict reader;
  - a hand-written strict fixture target fails the upper-case variant, and a tolerant one passes it.
- **Old documents.** Both committed examples keep their digests, pinned in the test. Both
  `examples/protocols/*.actions.json` files simulate to the same observations as before. The frozen released shapes refuse `reads`,
  `as_written` and `extra_members`.
- Each test runs within 60 s, each test step within 120 s, and exploration in tests stays inside
  the examples' declared bounds.

## Scope

- cited: `crates/specify/ess-domain/src/protocol.rs` (`Participant` 43-63)
- cited: `crates/specify/ess-compiler/src/protocol.rs` (`participant` 212-282, `trigger` 283-324)
- cited: `crates/verify/ess-conformance/src/protocol/mod.rs` (`Action` 18-48), `engine.rs`,
  `monitor.rs` (`check` 56-162, `explore` 163-248), `target.rs` (`run_target` 94-110), `protocol/tests.rs`
- cited: `crates/specify/ess-compiler/tests/protocol.rs`, `docs/design/protocol-specification.md`,
  `CHANGELOG.md`
- Size: M, about 420 changed lines.

## Gate

- `task fmt-check`
- `cargo clippy -p ess-domain -p ess-compiler -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-compiler --locked --test protocol`
- `cargo test -p ess-conformance --locked --lib protocol`
- `cargo test -p ess-domain --locked --lib`
- `cargo test -p ess-cli --locked --test protocol_cli`
- `task ci-lint` (new public function)
- `task test-xtask`

Each test runs within 60 s, and each test step within 120 s. A build-pool job ends within 10 minutes
unless it names its time with `slot --timeout` (at most 30 minutes).

## Depends on

- `pr0-protocol-message-structure` (optional fields, and the same files).
- PR-D1, PR-D2 and PR-D3.
- It may run beside `pr0-protocol-reply-exchange-and-credential-header` only if the two are merged
  one after the other: they share `protocol.rs` in all three crates.

## Fit review

1. **Need.** A reader's case-insensitive matching of named members, and its indifference to other
   members, cannot be stated or checked. Reproduction above. The source is a downstream
   application's broker reply, relayed by the owner's operator on 2026-11-24.
2. **Class: gap.**
3. **Already expressible?** No. Declaring every case variant as its own message multiplies messages
   by 2^n and still says nothing of the members the reader ignores.
4. **Fit.** It is one participant's reading rule. It sits on the participant beside its `inputs`,
   because a participant's view of its own inputs is already local to it ("cannot inspect another
   participant's unobserved state", `.engineering/planning/design/ess-protospec.md:95-98`). The
   check reuses the native target seam and the trace's delivery action. The suite's semantic
   scenarios do not change.
5. **Second adopter.** An HTTP client that reads `Content-Type` and a JSON `errorCode` from a server
   that writes `errorcode`.
6. **Cost.** One participant key, two optional delivery keys, one public function, about four string
   codes. No format version under PR-D3 A.
7. **Considered.**
   - (a) A message-level `member_names: case_insensitive`. It is wrong for a strict second reader,
     and it cannot express "reads two, ignores two".
   - (b) The rule in the broker binding. That is a wire rule outside the protocol document, which
     the note refuses.
   - (c) Case-insensitive decoding of the command's answer type in the model. That puts a wire rule
     in the command.
   - Chosen: on the reading participant.

## Decisions

- PR-D1, PR-D2 and PR-D3 (architecture).
- No other open question.
