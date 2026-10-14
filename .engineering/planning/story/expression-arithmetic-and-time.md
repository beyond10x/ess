---
format: aep.planning-md/3
id: story:expression-arithmetic-and-time
kind: story
status: draft
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
revision: 2
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
