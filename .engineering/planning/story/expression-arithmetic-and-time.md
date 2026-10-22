---
format: aep.planning-md/3
id: story:expression-arithmetic-and-time
kind: story
status: active
title: 'Expressions compute arithmetic and time: Timestamp difference, Decimal division, now as an operand'
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#560
- provider: github
  reference: beyond10x/ess#567
relations:
- serves: vision:O2
- decomposes: epic:generic-expression-language
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T00:20:54Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-21T00:20:54Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The expression language computes arithmetic and time: `Timestamp - Timestamp` yields a `Duration`,
`+ - * /` work over Integer and Decimal with a declared rounding, and `now` (with a `Duration`
offset) is an operand wherever the binding design's position table admits a clock; each in
`sets:`, `payload:`, response fields and computed view fields, checked by conformance.

Binding design: `docs/design/generic-language-features.md` (being drafted), feature 2, arithmetic and time.
Parent: `epic:generic-expression-language`.

## Fit review

Findings covered: https://github.com/beyond10x/ess/issues/560 and
https://github.com/beyond10x/ess/issues/567 (a downstream incident-ledger specification, ess 0.59.0,
`ess/24`).

1. **Need.** (a) Incident and impact durations and detection lag are figures the reader sees:
   `sets: {duration_s: input.ended_at - input.started_at}` is refused (`ESS-COMMAND-001`,
   `crates/specify/ess-domain/src/command/input_path.rs:164`), the offset form is refused
   (`ESS-COMMAND-002`), and a view field has no `from:` (https://github.com/beyond10x/ess/issues/560).
   (b) A view filter relative to now (`started_at >= now - 30d`) is refused
   (`crates/specify/ess-domain/src/expression.rs:2282`); low priority for the reporter, whose
   dashboard sends `from` and `to` (https://github.com/beyond10x/ess/issues/567).
2. **Class.** Gap. Both reopen declines: https://github.com/beyond10x/ess/issues/442 ("arbitrary
   arithmetic … not in this design", `docs/design/value-expressions.md:394`) and
   https://github.com/beyond10x/ess/issues/439 (the caller resolves the window,
   `docs/design/read-api-view-idioms.md:198-232`). The reporter's reason: the idiom moves the figure
   out of the specification, so conformance never checks the number the reader sees.
3. **Already expressible?** Only by storing both instants and letting the consumer subtract, and by
   sending `from`/`to`; neither is checked as the figure.
4. **Fit.** One set of arithmetic and time functions over typed operands, the same in every position, instead of one
   constant offset (`docs/design/expression-family-source22.md:29-33`). `now` in a view filter needs a
   read-time clock a suite can set: the binding design states it (the suite already sets the clock
   for command guards, `story:interpreted-cli-target-decides-now-at-the-step-instant`), or keeps it
   refused by name. Targets: validation (types: `Timestamp - Timestamp -> Duration`), interpreter,
   synthesis (bounded linear-arithmetic witness search), Rust/Go/TypeScript exact evaluation, Entity
   Runtime, diff, generated docs.
5. **Second adopter.** The correlated begin/end duration of
   https://github.com/beyond10x/ess/issues/442; any SLA figure (time to acknowledge).
6. **Cost.** A source format version; a witness search over linear arithmetic in synthesis; exact
   evaluation in three runners (they already do exact `i128` offsets).
7. **Alternatives.** (a) Change nothing: figures stay out of conformance. (b) Per-position keys
   (`from:` on a view field, `offset:` forms): one cell per request. (c) One set of arithmetic and time functions (chosen).

## Decisions

Accept, redesigned into the expression language's arithmetic and time functions. The declines of
https://github.com/beyond10x/ess/issues/439 and https://github.com/beyond10x/ess/issues/442 are
reopened by the 2026-10-13 decision to build the four generic features; `now` in a view filter is
admitted only if the binding design gives the suite a read clock, otherwise refused by name with
the `from`/`to` idiom in its hint.

## Acceptance

- `timestamp_difference_is_a_duration_in_sets_payload_and_view_field`: the #560 shapes validate and
  their scenarios assert the computed duration.
- `decimal_division_rounds_as_declared`: a division states its rounding and zero rule and is
  witnessed on a tie and a zero denominator.
- `now_relative_view_filter`: admitted with a suite-set read clock and witnessed either side of the
  bound, or refused naming the `from`/`to` idiom (whichever the binding design decides).

## Intake 2026-10-16

The suite-set clock `now_relative_view_filter` depends on is
`story:a-suite-fixes-the-targets-clock` (https://github.com/beyond10x/ess/issues/586), one clock
capability for command guards, stored `Timestamp` guards and view filters.

## State 2026-10-21

Shipped: every acceptance test of this story passes on the 0.67.0 batch branch
(`crates/verify/ess-conformance/tests/current_time_guard.rs`,
`crates/verify/ess-conformance/tests/now_stored_rows_runtimes.rs`,
`crates/verify/ess-conformance/tests/expression_arithmetic_and_time.rs`,
`crates/specify/ess-domain/tests/expression_values.rs`). Not done: `now` in a view filter read at a
suite-set request clock. Today it is refused naming the `from`/`to` idiom
(`now_relative_view_filter` takes that branch). What it needs: the view-filter current-time site
admitted from the open source format; the interpreted view read deciding the filter at the command
clock's instant; synthesis sending the filtered field's carrier input as `now_offset` values either
side of the bound (latency-safe margin) and, at the boundary, a `hold_now` before the read; a suite
pair admitting `hold_now` before `query_view` in Rust, Go and TypeScript admission; generated
queries and Entity Runtime refusing it by name.

## Adopter report, recheck batch (2026-10-22)

### Need

Reported by an adopter on 2026-10-22 (recheck batch item 16, the second team's L9, relayed by the orchestrator; adopter names
kept out). The system stores and receives times as epoch milliseconds in an `Integer` field (the wire contract fixes that
representation), and refuses a request whose time is in the past. `now` is ordered only against a `Timestamp`, so the guard
cannot be stated over the field the system actually has.

Needed: an epoch-milliseconds (or epoch-seconds) `Integer` may be ordered against `now` with an offset, through a declared
representation (an epoch-time newtype or an explicit conversion), in a command `when:` and a `when_subject:` predicate; or a
recorded idiom (store a `Timestamp` beside the wire `Integer`) that keeps the check in the specification.

Appended here because it is a case of beyond10x/ess#560 (one typed expression language) and of this story's `now`-as-operand
outcome. That outcome names `Timestamp` and `Duration` beside `now` and no
conversion from an `Integer` epoch value; the fit review decides whether the conversion belongs here.

### Reproduction

Brand-free fixture, `format: ess/29`, domain `demo.server`: entity `Token {token_id, expires_ms: Integer, expires_at:
Timestamp}`, `IssueToken` creating it, `UseToken(token_id, sent_ms: Integer, sent_at: Timestamp)` with an `expired` refusal
first, `unknown_instance`, and a `used` update. `ess specify validate --path spec.yaml`, identical output on 0.68.0 and 0.67.0:

| `expired` guard | result |
|---|---|
| `when: sent_at < now - 60s` (control) | valid; interpreted conformance 6/6 passed, both versions, byte-identical suites |
| `when_subject: {predicate: expires_at < now}` (control) | valid; 5/5 passed, both versions |
| `when: sent_ms < now - 60s` | ESS-COMMAND-002 `type_mismatch`: "operator `<` does not admit `Integer` (Number) and `Text literal `now - 60s`` (Text)" |
| `when_subject: {predicate: expires_ms < now}` | ESS-COMMAND-002 `type_mismatch`: "operator `<` does not admit `Integer` (Number) and `Text literal `now`` (Text)" |

Same refusal with `format: ess/30` on 0.68.0. Against an `Integer`, `now` is read as a text literal, not the clock.

### Where it lives

- `crates/specify/ess-domain/src/expression.rs:2404`: the `type_mismatch` message, raised when the operand types do not admit
  the operator.

### Regression status

Not a 0.68.0 regression: identical refusals on 0.67.0 and 0.68.0. Gap. Does not block 0.68.0.
