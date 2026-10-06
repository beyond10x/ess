---
format: aep.planning-md/3
id: story:feature-request-444
kind: story
status: implemented
title: Units on numeric fields
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#444
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-441
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/read_api_view_idioms.rs
- confidence: inferred
  path: docs/design/read-api-view-idioms.example/units.yaml
- confidence: inferred
  path: docs/design/read-api-view-idioms.md
- confidence: cited
  path: docs/design/review-expression-typechecking.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:25Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:26Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:48:32Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome
Resolve beyond10x/ess#444: Units on numeric fields.

## Origin
beyond10x/ess#444, filed 2026-10-05; an adopter's metrics read API, where every numeric metric has a unit (ms, s, count, ratio, epoch ms) carried as a comment, and one reporting view mixes units.

## Fit review
1. Need: a numeric field carries a unit that projections show, and comparing or aggregating values of different units is caught. Minimal reproduction, written fresh (`<fit-review scratch>/probe-444/`): `duration_ms` and `wait_s` on one entity. The requester's syntax, not adopted: optional `unit:` on fields and view fields, carried into docs, JSON Schema and OpenAPI, and checked where units are mixed.
2. Class: a convenience for declaring and projecting a unit, because the newtype idiom states it (question 3). The mixed-unit comparison check is a gap only against ESS's own typing rule, which deliberately lets differently named numeric wrappers compare (docs/design/review-expression-typechecking.md:60). That rule is not a defect.
3. Existing idiom: declare one newtype per unit (`{name: Millis, kind: newtype, of: Integer}`, `Seconds` likewise) and type fields with it.
   - It validates (probe-444 c). JSON Schema projects `"$ref": "#/$defs/probe.calls.Millis"` with `"title": "Millis"`, `"x-ess-kind": "newtype"` (probe-444 c/schema).
   - Newtypes are not assignable to one another: types.rs:3-5 says "`Email` and `InvoiceId` … are not interchangeable in the model".
   - `max` keeps the newtype: `Optional<Millis>` validates (probe-444 b).
   - A field's prose goes in `summary:` (name.rs:208-210), which OpenAPI uses as the parameter description (openapi.rs:1795).
   - A newtype itself takes no `summary:`: "unknown field `summary`, expected one of `of`, `alphabet`, `prefix`, `invariants`" (probe-444 first run).
   - Two limits remain. `duration_ms > wait_s` validates across `Millis` and `Seconds` (probe-444 a). `sum` drops the newtype: declaring the sum `Millis` is refused, "result type is Integer" (probe-444 d; view.rs:881-886).
4. Fit:
   - A `unit:` key would be a second way to say what a newtype name already says, and the two could disagree (`type: Seconds, unit: ms`). That is the red flag "names a concept ESS already spells differently elsewhere" (skill, red flags).
   - Checking units in comparisons would reverse the deliberate same-representation rule (review-expression-typechecking.md:60) for every newtype, not only units, and would break specifications that compare wrapped identifiers or amounts today (inferred).
   - The newtype idiom already composes with guards, bindings, views, conversions and every projection.
5. Second adopter: a billing service with `Cents` and `Euros`, or a sensor feed with `Celsius` and `Kelvin`. Both are served by newtypes. Neither needs a separate unit vocabulary.
6. Cost if accepted: a new key on `Field` and `RawViewField`, a unit grammar (`ms`, `s`, `count`, `ratio`, `epoch_ms`, open or closed?), ess/23, diff classes, projection changes, and a rule for what a sum's unit is. Cost of the idiom: a note section and an example in the shared idioms test.
7. Alternatives:
   - (a) Change nothing.
   - (b) The requester's `unit:` key.
   - (c) A `summary:` on newtypes, so a unit's prose reaches every projection: a small extension of an existing construct. Not needed for the stated need, since the newtype title already names the unit. Recorded as the next step if an adopter asks for prose on the type.
   - (d) Nominal comparison checking: reverses a typing decision.
   - Chosen: (a) plus documentation of the newtype idiom.

## Decisions
decline, with the idiom — A unit is a newtype (`Millis`, `Seconds`, `EpochMillis`, `Ratio` of `Decimal`). Its name reaches JSON Schema and OpenAPI as the schema title and `x-ess-name`, and a value of one is not assignable to a field of another. The story body is the decline record. The idiom is documented as the section `## A unit is a newtype` in `docs/design/read-api-view-idioms.md`, with its model in `docs/design/read-api-view-idioms.example/units.yaml`, domain `idioms.units`: the newtypes `idioms.units.Millis` and `idioms.units.Seconds` (of `Integer`), entity `idioms.units.Call` (`talk_ms: Millis`, `wait_s: Seconds`) and the mixed-unit reporting view `idioms.units.CallDurations`. Depends on #441 (edge recorded), which creates the note, the example directory and the test. The section states the two limits:
- comparisons across differently named numeric newtypes are admitted (review-expression-typechecking.md:60);
- `sum` returns the bare primitive (view.rs:881-886).

No format bump. Reply to the requester with the idiom.

## Acceptance
- unit_newtype_section_states_the_idiom: `docs/design/read-api-view-idioms.md` has the heading `## A unit is a newtype`. The section contains "`kind: newtype`", "`Millis`", "`Seconds`", "not assignable", "schema title", "comparisons across differently named numeric newtypes are admitted" and "`sum` returns the bare primitive". The case reads the note and fails on a missing heading or phrase.
- unit_newtype_example_validates: the example directory with `units.yaml` validates, and `idioms.units.CallDurations` projects one `Millis` and one `Seconds` field.
- unit_newtypes_are_not_assignable: setting a `Seconds` field from a `Millis` input is refused, and the schema projection carries `title: Millis`.
- unit_newtype_limits_hold_as_documented: a `Millis`/`Seconds` comparison validates and a `sum` declared `Millis` is refused, so the section cannot drift from behaviour.

All four cases live in `crates/verify/ess-conformance/tests/read_api_view_idioms.rs`.

## Scope
- docs/design/read-api-view-idioms.md  inferred — adds `## A unit is a newtype` (file created by #441)
- docs/design/read-api-view-idioms.example/units.yaml  inferred — domain `idioms.units`: unit newtypes and the reporting view (directory created by #441)
- crates/verify/ess-conformance/tests/read_api_view_idioms.rs  inferred — section, assignability, projection title, documented limits
- docs/design/review-expression-typechecking.md  cited — same-representation comparison rule, 60
- crates/specify/ess-domain/src/view.rs  cited — `sum` drops the newtype, 881-886 (read, not edited)
