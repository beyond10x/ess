---
format: aep.planning-md/3
id: story:a-protocol-timer-takes-its-duration-from-a-message-field
kind: story
status: draft
title: An ess-protospec/1 timer takes its duration from a field of the received message or a local field, below a declared ceiling
summary: Timer durations are fixed or backoff only, so a server-advised reconnect interval cannot be modelled (adopter item 9, 2026-10-22); same on 0.67.0.
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
- depends_on: story:an-ess-protospec-1-message-declares-header-fields-absent-fields-fixed-values-and-an-application-body
- depends_on: story:an-ess-protospec-1-reply-answers-on-the-exchange-of-the-message-it-received-and-a-credential-header-is-named
- depends_on: story:an-ess-protospec-1-participant-reads-named-members-of-one-message-in-any-case-and-may-ignore-the-others-and-a
revision: 5
---
## Need

Reported by an adopter on 2026-10-22 (item 9 of the second batch, relayed by the orchestrator; recorded on 0.56.0, re-probed
here). In a pub/sub protocol the server's advice carries an `interval` in milliseconds that the client must wait before it
reconnects. `ess-protospec/1` timers take a fixed duration or a backoff of an earlier interval, so the adopter pins 1000 ms.

Needed: a timer duration read from a field of the received message (or a local field set from it), with a declared bound, so
the bounded model stays finite.

### The widened case (added)

The source widens from a received message field to any field a value source reads, and a second
case joins the draft's.

Two protocols need a timer whose duration is not a literal:

- **From the peer** (the draft's case): a server's advice carries `interval` in milliseconds, and the
  client waits that long before it reconnects.
- **From configuration** (this report): a requester bounds each request by a configured timeout, at
  most 30 s inside an outer bound. The protocol model pins one value, so its traces cannot show the
  behaviour under another configuration.

`ess-protospec/1` durations are `fixed` or `backoff` only (`crates/specify/ess-domain/src/protocol.rs:225-243`,
`Duration`). A value source already reads a local field or a field of the triggering input
(`protocol.rs:143-162`, `ValueSource`), but no duration reads one.

Minimal reproduction (written fresh; not run): participant `requester` with local field
`bound_ms: Integer`, input `configure {bound_ms: Integer}` setting it, and on `send` the effect
`{kind: arm, timer: bound, after: {kind: field, source: local, field: bound_ms, ceiling_ms: 30000}}`.

## Reproduction

Brand-free `ess-protospec/1` document: participants `client` (states `connected, waiting, reconnecting`, timer `R`) and
`server`; `server` sends message `advice {interval: Integer}` on channel `advice`; `client` on receiving it arms `R`, and on
`R` firing emits `reconnected`.

With `after: {kind: fixed, millis: 1000}`, `ess specify protocol validate --path fixed.yaml` on 0.68.0 and 0.67.0:
`reconnect-advice — valid ess-protospec/1 (52962b69...)`.

With `after: {kind: input, field: interval, ceiling_ms: 5000}`, exit 1 on 0.68.0 and 0.67.0, identical:

```text
error: $ [MalformedProtocol]: unknown variant `input`, expected `fixed` or `backoff`
```

## Where it lives

- `crates/specify/ess-domain/src/protocol.rs:228` (`Duration`): `Fixed { millis }` and `Backoff { timer, factor, ceiling_ms }`
  only.
- `ValueSource::Input { field }` (:146) already reads a field of the triggering message for `set` and `send` effects; the
  timer has no such source. The checker's time bound (`bounds.max_time_ms`) is where a declared ceiling would have to fit.

## Regression status

Not a 0.68.0 regression: identical refusal on 0.67.0; no commit in `0.67.0..52bd110dd` touches `protocol.rs`. Gap. Does not
block 0.68.0.

## Outcome

Under **AE-D14**, option A (the default):

- A duration may be `{kind: field, source: input | local, field: <name>, ceiling_ms: <n>}`. The
  field is an `Integer` of milliseconds.
- At execution, a value of 0 or less, or above `ceiling_ms`, refuses the step. The result is
  inconclusive, never passing, as every unsupported observation is
  (`docs/design/protocol-specification.md:19`). Nothing is clamped silently.
- Configuration is modelled as a local input, as above, so exploration offers each witness value
  from the inputs file. The state space stays finite through those witnesses and the ceiling
  (`max_time_ms` still bounds advancement).
- How a configuration file maps an out-of-range value to a default (for example "0 means 10 s") is
  configuration reading, not protocol behaviour. The witnesses give effective values, **AE-D14**.

Format: an additive `Duration` variant within `ess-protospec/1` under **PR-D3** A (the protocol
revision's decision). Under PR-D3 B it joins `ess-protospec/2`. Decided before acceptance, with
PR-D3.

## Acceptance

1. Red today: the reproduction is refused naming `field` (a case in
   `crates/specify/ess-compiler/tests/protocol.rs`).
2. After: it validates. `ess specify protocol validate` prints the digest. Both committed protocol
   examples keep their digests (canonical form unchanged when the variant is unused).
3. Refused at admission: a field that is not `Integer`; an unknown field; `source: input` on a
   transition whose trigger has no such field; a missing or zero `ceiling_ms`; a ceiling above
   `bounds.max_time_ms`.
4. Execution (`crates/verify/ess-conformance/src/protocol/engine.rs`): `configure {bound_ms: 2000}`,
   then `send`, arms `bound` for 2000 ms, and the trace replays (`ess-prototrace/1` unchanged). A
   witness of `0` or `40000` makes the run inconclusive and names the field and the ceiling. Cases go
   in `crates/verify/ess-conformance/src/protocol/tests.rs`.
5. The draft's peer-advice case (`source: input` on a receive trigger) passes the same checks.
6. `docs/design/protocol-specification.md` § "First format details" lists the duration source.

## Scope

- cited: `crates/specify/ess-domain/src/protocol.rs` (`Duration` 225-243, `ValueSource` 143-162)
- inferred: `crates/specify/ess-compiler/src/protocol.rs` (admission), `crates/verify/ess-conformance/src/protocol/engine.rs`
- cited: `docs/design/protocol-specification.md`

## Owned files

`ess-domain/src/protocol.rs` (`Duration::Field`), `ess-compiler/src/protocol.rs`,
`ess-conformance/src/protocol/engine.rs`, `src/protocol/tests.rs`, `ess-compiler/tests/protocol.rs`,
`docs/design/protocol-specification.md`.

## Not yours

The protocol revision's units (`pr0-protocol-message-structure`, `pr0-protocol-reader-tolerance`,
`pr0-protocol-reply-exchange-and-credential-header`). They touch `protocol.rs` (`Message`,
`Participant`, `ValueSource`): land one after another.

## Tests first

Acceptance 1, 3, 4. Pure. A bounded exploration of the reproduction stays under 1 s.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes. Run
`ess-conformance` one target at a time.

```console
task fmt-check
cargo clippy -p ess-domain -p ess-compiler -p ess-conformance --all-targets --locked -- -D warnings
cargo test -p ess-compiler --locked --test protocol
cargo test -p ess-conformance --locked --lib protocol
cargo test -p ess-cli --locked --test protocol_cli
task test-xtask
```

## Depends on

**PR-D3** (format consequence). After the `pr0-*` units that touch `protocol.rs`, or rebased on them.

## Changelog

- `ess-protospec/1`: a timer's duration may read an `Integer` field of the triggering message or a
  local field, below a declared ceiling; a value outside it makes a run inconclusive.

## Fit review

1. **Need.** A timer whose duration comes from the peer or from configuration. This report: a
   configured request bound the model can only pin. The draft's adopter: a server-advised reconnect
   interval. Neither proposed a syntax beyond the draft's.
2. **Class: gap** (`protocol.rs:225-243`).
3. **Expressible today?** No. The idiom of a timer per configured value multiplies transitions per
   value, and it still cannot take the peer's value.
4. **Fit.** It reuses `ValueSource`'s reading of local and input fields, and `backoff`'s
   `ceiling_ms`. Finite bounds stay mandatory (`protocol-specification.md:15`). Configuration as an
   input is how the format already models anything outside the participants.
5. **Second adopter.** The draft's (2026-10-22).
6. **Cost.** One `Duration` variant. The format consequence follows PR-D3.
7. **Considered.** (a) Change nothing. (b) Top-level `parameters:` swept by exploration and pinned in
   each trace (AE-D14 B): a new construct and an `ess-prototrace` field for what an input already
   carries. (c) Chosen: one widened duration.

## Decisions

Accept, redesigned (merged with the draft). **AE-D14**, default A. Format: PR-D3.

## Size

M, about 380 lines with tests.
