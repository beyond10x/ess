---
format: aep.planning-md/3
id: story:an-ess-protospec-1-reply-answers-on-the-exchange-of-the-message-it-received-and-a-credential-header-is-named
kind: story
status: draft
title: An ess-protospec/1 reply answers on the exchange of the message it received, and a credential header is named but never valued
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-ess-protospec-1-message-declares-header-fields-absent-fields-fixed-values-and-an-application-body
revision: 1
---
## Need

Two wire parts that the owner's note of 2026-11-24 assigns to the protocol kind still have no form
after `pr0-protocol-message-structure`.

- **Correlation.** A request and its reply belong to one exchange. Every send names an `exchange`
  identity (`crates/specify/ess-domain/src/protocol.rs:185-198`). A responder, though, can take it
  only from a literal, one of its own fields, or a field of the input (`ValueSource`, `:143-162`).
  So a reply can share its request's exchange only when the request carries a declared correlation
  field. On a broker that is not true: the correlation is the reply-to address, which is not a
  field.
- **A credential header.** A token in `Authorization`, or an API key in a named header, is part of
  the protocol, and the note lets the protocol document name it. Today a header would be an
  ordinary field. A send writes its value, a guard reads it, and `ess-prototrace/1` records it in
  the transmission's `payload` (`crates/verify/ess-conformance/src/protocol/mod.rs:142-158`). A
  persisted trace would then hold a credential, against the rule that credentials never reach
  serialized bytes (`AGENTS.md` § Kubernetes credential edge, § Commits). It would also invite a
  binding to turn a credential into an ordinary input, which AC-D3 refuses.

Minimal reproduction (written fresh, not run): a responder transition with
`effects: [{kind: send, channel: replies, message: reply, exchange: {kind: exchange}, logical: {kind: literal, value: r}, payload: {}}]`
under a `receive` trigger, and a message `call` with
`headers: [{name: Authorization, type: String, credential: true}]`.

```console
$ ess specify protocol validate --path call.yaml
error: $ [MalformedProtocol]: unknown variant `exchange`, expected one of `literal`, `local`, `input`
```

## Outcome

- **Value source `{kind: exchange}`.** It reads the exchange identity of the delivered copy that
  triggered the transition. It is admitted only under a `receive` trigger, wherever a `String` is
  read: `send.exchange`, `send.logical`, a payload value, a guard operand, a `set` value. A reply is
  `send {exchange: {kind: exchange}, …}`. A requester checks it by comparing `{kind: exchange}` with
  the local field it stored when sending.
- **Header key `credential: true`.** The header carries a credential.
  - Its value has no place in the model. A send does not write it, a guard or value source cannot
    read it, and it can have no `value:`.
  - A transmission records its **presence** only. Replay refuses a trace whose `payload` holds a
    value under that name (`CredentialInTrace`), so no adapter can persist a token through this
    format.
  - Whether a binding may use such a header is the binding's question. Every binding story refuses
    to feed it to or from an input or view parameter. Only an `authentication` scheme of an
    `ess-http/1` binding may read it (AC-D3 A,
    `architecture-decision-record:a-binding-authenticates-the-caller-through-a-port`).
- `docs/design/protocol-specification.md` § "Message structure" (written by `pr0-protocol-message-structure`)
  gains "Correlation" and "Credential headers", with the redaction rule.

## Acceptance

- **Red first.** The reproduction validates (exit 0). Today it is refused with "unknown variant
  `exchange`". The case is in `crates/specify/ess-compiler/tests/protocol.rs`.
- **Refusals.** Each is refused with a string code and its path:
  - `{kind: exchange}` outside a `receive` trigger (`ExchangeOutsideReceive`);
  - `credential: true` on a body field (`CredentialBodyField`) or beside `value:` (`InvalidFixedValue`);
  - a guard, value source or send payload naming a credential header (`CredentialHeaderRead`).
- **Execution.** These are unit tests in `crates/verify/ess-conformance/src/protocol/tests.rs`:
  - a request and a reply on `{kind: exchange}` replay;
  - a reply sent on another exchange is refused by the requester's guard, and the trace fails;
  - a trace whose transmission holds a value for a credential header is refused as `CredentialInTrace`,
    and the refusal names the header but never prints the value;
  - a trace that records only its presence passes.
- **Old documents.** Both committed examples keep their digests, pinned in the test. The frozen
  released shapes refuse `{kind: exchange}` and `credential`.
- Each test runs within 60 s, and each test step within 120 s.

## Scope

- cited: `crates/specify/ess-domain/src/protocol.rs` (`ValueSource` 143-162, the header struct
  added by `pr0-protocol-message-structure`)
- cited: `crates/specify/ess-compiler/src/protocol.rs` (`source` 325-360, `expected` 361-377, send in `effect` 397-506)
- cited: `crates/verify/ess-conformance/src/protocol/engine.rs`, `monitor.rs`, `mod.rs`
  (`Transmission` 142-158), `protocol/tests.rs`
- cited: `crates/specify/ess-compiler/tests/protocol.rs`, `docs/design/protocol-specification.md`,
  `CHANGELOG.md`
- Size: S, about 260 changed lines.

## Gate

- `task fmt-check`
- `cargo clippy -p ess-domain -p ess-compiler -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-compiler --locked --test protocol`
- `cargo test -p ess-conformance --locked --lib protocol`
- `cargo test -p ess-domain --locked --lib`
- `cargo test -p ess-cli --locked --test protocol_cli`
- `task ci-lint`
- `task test-xtask`

Each test runs within 60 s, and each test step within 120 s. A build-pool job ends within 10 minutes
unless it names its time with `slot --timeout` (at most 30 minutes).

## Depends on

- `pr0-protocol-message-structure` (headers exist, and the same files).
- PR-D1, PR-D2 and PR-D3.

## Fit review

1. **Need.** A reply cannot answer on its request's exchange without an invented field. A credential
   header cannot be named without its value becoming model data and trace bytes. Reproduction above.
2. **Class: gap.**
3. **Already expressible?** Correlation partly: through a declared correlation field and a guard,
   which a broker's reply-to is not. Credential headers: no.
4. **Fit.** The exchange identity already exists on every transmission. This reads it. The
   credential rule mirrors the redaction tests of the Kubernetes edge (value never serialized,
   presence observable), and keeps AC-D3 where it is.
5. **Second adopter.** A request/response protocol over a message queue whose replies are matched by
   a correlation id the transport carries outside the body.
6. **Cost.** One value-source variant, one header key, about four string codes. No format version
   under PR-D3 A.
7. **Considered.**
   - (a) Require a declared correlation field. That states a wire field the transport does not have.
   - (b) Let credential headers be ordinary fields and redact in each adapter. The format would
     then persist what an adapter forgot to redact.
   - Chosen: both in the format.

## Decisions

- PR-D1, PR-D2 and PR-D3 (architecture).
- AC-D3 stays as decided (default A) and open for the owner; this story does not change it.
