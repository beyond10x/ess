---
format: aep.planning-md/3
id: story:stored-field-equals-returned-response-value
kind: story
status: active
title: 'Store the value an outcome returns with {response: <field>} in sets:'
tags:
- ess-0.71.0
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#498
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/response_source.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/response_source_obligation.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/minted_value.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/one_time_response.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/set_effects.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/response_payload.rs
- confidence: inferred
  path: crates/verify/ess-diff/tests/response_source.rs
- confidence: cited
  path: docs/design/language-grids.md
- confidence: cited
  path: docs/design/minted-value-sources.md
- confidence: cited
  path: docs/design/typed-response-outcome-payloads.md
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:46:18Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:46:20Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

An implementation mints a value, returns it to the caller and keeps it, and a later command
compares its input with the kept value: a verification code shown to a user (RFC 5849 §2.2, §2.3),
a one-time PIN, a pickup claim code, a pairing code. ESS cannot say that the stored value is the
returned one: `{generated: true}` in `sets:` and a typed `response:` field are two independent
mints, and `{response: <field>}`, the existing spelling of "the value this invocation returned" in
an event payload, is refused in `sets:`.

Reproduced on 0.92.0 at `format: ess/36` (`example.order.Order` with `pin: String`;
`example.order.Place` creates it, `returns: true`, `response: [{name: pin, type: String}]`;
`example.order.Confirm` refuses with `when_subject: {predicate: pin != input.pin}`):

```yaml
      - name: placed
        creates: example.order.Order
        instance: order_id
        returns: true
        sets:
          title: input.title
          pin: {response: pin}
```

```console
$ ess specify validate --path stored
  - [conflicting_declaration] command.example.order.Place.outcomes.placed.sets.pin: response and generated sources are admitted only in event payloads
  - error[ESS-COMMAND-004]: response and generated sources are admitted only in event payloads
  --> system.yaml:46:11
```

The message is also wrong in naming generated sources: `sets: {pin: {generated: true}}` validates
on the same command (`run.txt` § 2). That variant states no equality, and synthesis then cannot
witness `Confirm` at all (`ESS-SYNTH-001` for `Confirm/mismatch`, `Confirm/confirmed`, the
`confirm` transition; `ESS-SYNTH-004` for `Confirmed`).

This story is the language half: design, validation, IR and the targets that refuse or keep an
obligation. The interpreter, the suite and synthesis follow in
`story:the-interpreter-answers-the-response-value-an-outcome-stores`,
`story:synthesis-witnesses-a-guard-over-a-stored-response-value` and
`story:the-go-and-typescript-runtimes-resolve-a-responded-value`.

## Acceptance

- `docs/design/minted-value-sources.md` gains a section "A stored value read from the response"
  and `docs/design/language-grids.md` grid R gains the column "own response" (`{response: f}`):
  yes in `sets:` (subject, nested struct leaves), `sets:` under `instances:`/`affects:` and event
  `payload:`, n/a in error `payload:` (an error outcome has no response) and every guard, view and
  binding position; `docs/design/typed-response-outcome-payloads.md:23` points at it.
- At `format: ess/37` (the epic's one source format, opened by
  story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs and shared with the epic's
  other constructs), `ess specify validate` on the reproduction succeeds and `ess specify compile` writes
  `ResolvedPayloadValue::ResponseField` for `pin` in the outcome's resolved `sets`.
- Refused as `conflicting_declaration` at the `sets` key, each naming the cause: an outcome without
  `returns: true`; an error outcome (including `compensates: true`); a field not in the command's
  `response:`; a response field whose type is not assignable to the stored field without a
  conversion; a response field listed in `one_time_response:` (the existing refusal kept).
- At `format: ess/36` the reproduction is refused as `unsupported_format_version` naming `ess/37`, and
  the text no longer names generated sources; `{generated: true}` in `sets:` keeps validating.
- A model without the source compiles to the same IR bytes; generated Rust and Go keep a command
  with the source an obligation named by it; Entity Runtime lowering refuses it by name; `ess
  verify diff` between `{generated: true}` and `{response: pin}` on one `sets:` entry reports
  `outcome-sets-changed`.

## Scope

- cited: crates/specify/ess-domain/src/command.rs
- inferred: crates/specify/ess-domain/src/command/minted_value.rs
- inferred: crates/specify/ess-domain/src/command/set_effects.rs
- inferred: crates/specify/ess-domain/src/command/one_time_response.rs
- inferred: crates/specify/ess-compiler/src/resolve.rs
- inferred: crates/specify/ess-domain/tests/response_payload.rs
- inferred: crates/generate/ess-synth/tests/response_source_obligation.rs
- inferred: crates/generate/ess-entity-runtime/tests/response_source.rs
- inferred: crates/verify/ess-diff/tests/response_source.rs
- cited: docs/design/minted-value-sources.md
- cited: docs/design/language-grids.md
- cited: docs/design/typed-response-outcome-payloads.md
confidence: medium

Moves the `ess/37` `FORMAT_HISTORY` row, `CHANGELOG.md`, `website/docs/reference/spec-versions.md`
and `schemas/generated/ess.schema.json` (coordinator-reconciled shared surfaces).

The refusal is `command.rs:5048-5064` (admits `Generated` from `ess/14`, never `ResponseField`);
the IR variant exists (`crates/specify/ess-compiler/src/ir.rs:1170`); the targets already handle
`ResponseField` in payloads: `crates/generate/ess-synth/src/determined.rs:642`,
`rust/behaviour.rs:2995`, `go/behaviour.rs:3849`, `crates/generate/ess-entity-runtime/src/subset.rs:142`,
`crates/verify/ess-diff/src/diff.rs:1272, 2420, 3029` under `OutcomeSetsChanged`
(`change.rs:2806`).

## Fit review

1. **Need.** "The value this command returned to the caller is the value it stored, and a later
   command is answered by comparing its input with it." Not expressible (reproduction above).
   Requester's proposal (theirs): admit `{response: <field>}` in `sets:`, or a payload/response
   source that reads what the outcome's own `sets:` wrote.
2. **Class.** Gap. The refusal is deliberate (`docs/design/typed-response-outcome-payloads.md:23`:
   "Response and generated sources are not admitted in entity `sets`"), and no construct relates a
   stored value to a returned one. Its message is a small defect: it names generated sources,
   which `sets:` admits from `ess/14` (`command.rs:5049-5052`).
3. **Already expressible?** No. (a) `{generated: true}` in `sets:` validates but relates nothing and
   leaves `Confirm` unwitnessed (`run.txt` § 2, four refusals). (b) Routing through an event,
   payload `pin: {response: pin}` and `sets: {pin: {payload: example.order.OrderPlaced.pin}}`
   (`minted-value-sources.md`), validates, but synthesis refuses the same four cases
   (`ESS-SYNTH-003`) and the interpreted run reports `Place/placed` unsupported, "an actual
   response field or its executable conversion is not interpreted yet" (`run.txt` § 3;
   `interpret/execute/determined.rs:414-427`); it also forces an event the domain may not have.
   (c) `sets: {pin: input.pin}` makes the caller the authority on a server-minted value. No idiom
   keeps the figure checked (policy 1, `docs/design/generic-language-features.md` § Policy).
4. **Fit.** Grid R (`docs/design/language-grids.md`) has no column for the outcome's own response;
   today `{response: f}` is read only in event `payload:`. The decision opens the column whole:
   every value position a successful outcome writes (`sets:`, nested leaves, `sets:` under
   `instances:`/`affects:`, `payload:`), n/a where no response exists (error payloads) and in
   guards, views and bindings (they run before or apart from the answer). Vocabulary: the same
   spelling as in payloads (`PayloadSource::ResponseField`, the IR variant at `ir.rs:1170`), so
   one idea keeps one spelling; the requester's second option would add a keyword for the
   reverse direction. Composition: `returns: true` only; `one_time_response:` keeps forbidding
   its plaintext in state (`docs/design/one-time-response-values.md`, "Static flow checking").
   Targets: interpreter (S2), suite and synthesis (S3), Go/TypeScript runners (S4), Rust/Go
   generated code keep the obligation, Entity Runtime refuses by name, diff renders it.
5. **Second adopter.** A device-pairing flow: `example.pairing.Start` returns a six-digit code shown
   on screen and stores it; `example.pairing.Complete` refuses when `code != input.code`.
6. **Cost.** No format of its own: the source lands in `ess/37`, the one source format every
   construct of the epic shares (policy 2); no new keyword; new refusal causes under the existing
   `conflicting_declaration`; no IR type; no migration (models without it keep their bytes). Suite
   cost is in S3 (`ess-conformance/64` and `ess-conformance/65`).
   Counted as general (one column of grid R opened whole, policy 3).
7. **Alternatives.** (a) Change nothing: the fact is lost and `Confirm` is never exercised.
   (b) A new `{minted: f}` source shared by `sets:` and the response: a second spelling for an
   existing source. (c) A response source reading `sets:` (the requester's second option): the
   converse direction, response derived from state, which `story:response-field-declares-its-value-source`
   (#506) designs as the response-field row; not this column. Chosen: `{response: f}` in every
   value position of a returning outcome.

## Decisions

Accept, redesigned. The requester's first option is the shape, widened to the whole grid-R column
(not `sets:` alone) and limited to `returns: true` outcomes, with the `one_time_response` refusal
kept, the refusal text corrected, and the conformance half split into its own stories so the
source never validates unwitnessed in a release. Earlier targets (`ess/24`, `ess/31`, `ess/33`)
shipped without it; this story targets `ess/37` at 0.92.0.

2026-11-10 (plan-critic round 1): `ess/37` is not allocated to this decision. It is the epic's one
source format (story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs, coordinator
decision of 2026-11-09), so this source and story:aggregate-group-keys-are-expressions both land
in it and neither has to land first; the `depends_on` edge to the format story is the only order.

## Source

github:beyond10x/ess#498
