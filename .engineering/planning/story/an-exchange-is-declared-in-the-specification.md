---
format: aep.planning-md/3
id: story:an-exchange-is-declared-in-the-specification
kind: story
status: active
title: An exchange is declared in the specification
summary: 'exchanges: at ess/37: who sends which message, correlation, open and close; size L'
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- decomposes: epic:messages-are-exchanged-independent-of-the-wire
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/exchange.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/lib.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/exchanges.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/error.rs
- confidence: cited
  path: docs/design/message-exchanges.md
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:39:20Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:39:22Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

A specification cannot say that a command opens a conversation in which both parties then send
messages that belong to one row until something ends it. A request answered by `101 Switching
Protocols` followed by messages both ways on the socket is the case reported; the same conversation
spoken over a broker or as JSON-RPC notifications has the same gap. The accepted ADR
`messages-are-exchanged-independent-of-the-wire` § 1 (F1 (a), F2 (a)) puts that statement in the
specification as `exchanges:`, carrying no wire word.

Minimal reproduction on 0.92.0 (`ess/36`, a `Call` entity with `Open`/`Ended`, a `Connect` command
creating it, a `HangUp` command moving it to `Ended`, a `CallEnded` event):

```yaml
exchanges:
  - name: example.calls.Session
    opens: example.calls.Connect
    follows: example.calls.Call
    initiator_sends: [example.calls.HangUp]
    responder_sends: [example.calls.CallEnded]
    closes: {states: [Ended]}
```

```text
$ ess specify validate --path model-exchange
  - system.yaml: unknown field `exchanges`, expected one of `format`, `system`, ... `outcome_groups`
exit 1
```

## Acceptance

- `docs/design/message-exchanges.md` exists, status accepted, before any code lands, and states the
  keys below, each refusal with its `ValidationCode`, the format `ess/37`, and for each target the
  story that builds it or the statement that it ignores an exchange (Fit review 4).
- At `format: ess/37` the specification above validates, and `ess specify compile` writes the
  exchange into the IR in name order; compiling it twice gives identical bytes
  (`cargo test -p ess-domain --locked --test exchanges`, new).
- Keys: `name`, `opens` (a command), `follows` (an entity), `initiator_sends` (commands),
  `responder_sends` (events), `closes` (`states:`, `command:` or both), optional
  `max_followers: {count: <n>, error: <error>}`. No ordering key.
- Each of these is refused with its own code and its path: an `opens` command none of whose
  accepting outcomes creates or addresses a row of `follows`; an `initiator_sends` command that
  addresses no row of `follows` through its identity input; a `responder_sends` event with no field
  of the identity type of `follows`; a `closes.states` member outside the lifecycle; a
  `closes.command` that is not in `initiator_sends` or whose accepting outcomes move the row into
  no `closes.states` member (into no terminal state when `states:` is absent); a message listed
  twice or on both sides; `max_followers.count` below 1; an unknown name anywhere.
- A message after close is answered by the command's own refusal in the closing state (the
  lifecycle already refuses it); the page says so and no after-close error key exists.
- The same document at `format: ess/36` is refused as `unsupported_format_version` naming `ess/37`;
  a specification without `exchanges:` keeps its IR bytes at every format.
- After `cargo xtask schema`, `schemas/generated/ess.schema.json` holds a top-level `exchanges`
  property, and `task projection-check` passes.

## Scope

- cited: docs/design/message-exchanges.md (new; named by the epic and the earlier body)
- cited: crates/specify/ess-domain/src/spec.rs (`RawSpecFile`, beside `outcome_groups`)
- inferred: crates/specify/ess-domain/src/exchange.rs (new, shaped like `outcome_group.rs`, which also holds its own format gate)
- inferred: crates/specify/ess-domain/src/lib.rs
- cited: crates/specify/ess-primitives/src/error.rs (`ValidationCode`, the new refusal codes)
- cited: crates/specify/ess-compiler/src/resolve.rs (lowers top-level constructs such as outcome groups)
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/specify/ess-domain/tests/exchanges.rs (new)

confidence: medium

Shared surfaces moved: proposes the `exchanges:` sentence of the `ess/37` `FORMAT_HISTORY` row and
its `CHANGELOG.md` sentence, and regenerates `schemas/generated/ess.schema.json` and
`website/docs/reference/spec-versions.md`.

## Fit review

1. **Need.** A conversation (who opens it, which messages each party sends, the row they belong to,
   what ends it) cannot be stated; reproduction above, 0.92.0 refuses `exchanges` as an unknown
   field (`triage/650/repro/run.txt`). Requester's syntax: a WebSocket upgrade route with the
   messages each side sends (issue body, "Expected"); not adopted as written, see 7.
2. **Class: gap.** No construct states a multi-message conversation: commands answer once, events
   are published once (ADR § Context, lines 41-58). Not a defect: `101` is refused on purpose
   (`docs/design/http-presentation-binding.md:405`).
3. **Already expressible?** No. The closest idiom, separate commands and events with no link
   between them, leaves the pairing and the direction of each message out of the specification, so
   the suite checks none of it (fails policy 1, `docs/design/generic-language-features.md:588-594`).
4. **Fit.** Not a cell of grid R or grid O (`docs/design/language-grids.md`): it reads nothing in a
   guard or value position and adds no operation; it names existing commands, events, an entity
   and lifecycle states. Order reuses the lifecycle and guards (F2 (a), ADR line 165), so guards,
   outcomes and views are untouched. Every target: the interpreter and synthesis in
   `story:conformance-synthesizes-exchange-scenarios`; `ess verify diff` in
   `story:ess-verify-diff-reports-a-changed-exchange`; AsyncAPI and OpenAPI in
   `story:asyncapi-and-openapi-project-an-exchange`; generated Rust, Go, TypeScript and Entity
   Runtime ignore it by design (it adds no behaviour to a command), which the design page states.
5. **Second adopter.** A price feed: `Subscribe` opens an exchange following a `Subscription` row,
   the server sends `QuoteChanged` events, `Unsubscribe` moves the row to `Closed`.
6. **Cost.** One source format (`ess/37`), one new top-level key, about eight validation codes, a
   diff kind, a schema regeneration. No migration: specifications without it keep their bytes
   (ADR § 5, lines 142-149).
7. **Alternatives.** (a) Change nothing: the WebSocket, broker and JSON-RPC forms would each state
   the conversation apart, unchecked against each other (ADR lines 55-58). (b) A sibling
   `ess-exchange/1` document (F1 (b)): rejected by the operator's acceptance of F1 (a). (c) Chosen:
   `exchanges:` in the model. The requester's route-level syntax became binding C
   (`story:an-http-route-upgrades-to-an-exchange`) on top of this.

## Decisions

**accept, redesigned.** The requester asked for a WebSocket binding on a route; the accepted ADR
`messages-are-exchanged-independent-of-the-wire` (accepted 2026-10-28, F1 (a), F2 (a), F3 (a))
moves the conversation into the specification and makes the WebSocket one binding of it. Refined
here: the after-close refusal is the lifecycle's own, so no error key is added for it, and
`max_followers` is a mapping `{count, error}`.

- Format: `ess/37`, the epic's one source format (epic § Shared surfaces), which
  `story:a-view-may-answer-values-no-row-holds` also joins; no other format satisfies this story.

## Source

github:beyond10x/ess#650
