---
format: aep.planning-md/3
id: story:feature-request-439
kind: story
status: draft
title: 'Clock-relative view windows: named ranges in a time zone and sliding windows'
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#439
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-438
- depends_on: story:feature-request-441
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/aggregate_group_selection.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/calendar_windows.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/read_api_view_idioms.rs
- confidence: cited
  path: docs/design/aggregate-group-selection.md
- confidence: cited
  path: docs/design/calendar-window-guards.md
- confidence: inferred
  path: docs/design/read-api-view-idioms.example/window.yaml
- confidence: inferred
  path: docs/design/read-api-view-idioms.md
- confidence: cited
  path: website/docs/reference/predicates.md
revision: 12
---
## Outcome
Resolve beyond10x/ess#439: Clock-relative view windows: named ranges in a time zone and sliding windows.

## Origin
beyond10x/ess#439, filed 2026-10-05; an adopter's metrics read API resolves 14 named ranges (`TODAY`, `LAST_7_DAYS`, …) against request time in a caller's zone, plus last-N-minutes windows and one read-time running duration.

## Fit review
1. Need, in three parts:
   - (a) a read counts rows whose instant falls in a range that is named relative to the request time, in a named zone;
   - (b) the same over the last N minutes;
   - (c) one value is the time a row has spent in its current state, measured at read time.
   Minimal reproduction, written fresh (`<fit-review scratch>/probe-438/`, variants l, m, n, rng). The requester's syntax, not adopted: `window: {ranges: [...], zone: param.tz}` as a parameter kind.
2. Class:
   - (a) and (b) are a convenience at the source level. The bounds can be stated as two `Timestamp` parameters (question 3). Resolving `TODAY` in a zone is the caller's work, so it is local policy.
   - There is a conformance gap: that idiom gets no witness on an aggregate view (`ESS-SYNTH-017`, probe rng).
   - (c) is outside the model. `now` is refused in a view filter (predicates.md:753-755; current-time-guards.md:41), and a view field cannot be computed (#441).
3. Existing idiom:
   - `params: [{name: from, type: Timestamp}, {name: to, type: Timestamp}]` with `filter: [started_at >= param.from, started_at < param.to]` validates. It compiles to instant comparisons, `started_at >= param.from as timestamp` (probes l and rng, 0.53.0 debug build).
   - `started_at >= now - 1h` in a filter is refused with `type_mismatch`, naming the guard positions (probe m).
   - A calendar `window:` in a filter is refused the same way (probe n).
   - Named zones are refused by a coordinator decision of 2026-10-05: there is no zone database across three evaluators, and DST-following windows are "a new construct with a pinned zone-data dependency" (calendar-window-guards.md:8-25, :221-222; predicates.md:805-809).
4. Fit:
   - A window parameter kind would bring the zone database that calendar-window-guards.md:15-19 rejected into views, which that decision refuses everywhere. It would also need a clock at read time, which no suite step can set (calendar-window-guards.md:225).
   - The caller-resolved `from`/`to` idiom reuses existing vocabulary and is evaluated as instants in every lane.
   - What is missing is a witness. Aggregate parameter selection admits only top-level `field == param.x` (`crates/verify/ess-conformance/src/synthesize/aggregate.rs:1157-1163`).
   - Extend it so that, at top level, a `Timestamp` field ordered against a parameter (`>=`, `>`, `<`, `<=`) is a range selector. Arrange rows a second either side of each bound. Spell one row at an offset under which the written clock read as UTC falls on the other side, as calendar windows do (predicates.md:817-822), so a target comparing text fails.
5. Second adopter: an SLA report for "this week in the customer's zone", or a monitoring view of the last 15 minutes. Each sends the instants it resolved.
6. Cost:
   - No new source key, no ess/23, and no zone data.
   - Synthesis only; Timestamp parameter values are RFC 3339 literals.
   - Whether the existing suite pairs carry a Timestamp view parameter unchanged: inferred yes, because a scalar parameter already travels (probe agg); confirm before closing.
7. Alternatives:
   - (a) Change nothing: the idiom validates but stays unwitnessed.
   - (b) The requester's window parameter kind with zones: contradicts the 2026-10-05 zone decision and needs a read-time clock.
   - (c) `now` in view filters for sliding windows: refused by current-time-guards.md:41. No suite can fix the target's clock.
   - Chosen: a design note saying the caller resolves windows, plus a witness for the range idiom.

## Decisions
accept, redesigned — No window construct, named zone or read-time clock in views.
- Documented: the caller resolves a named range in its zone, or a last-N-minutes window, into a `from`/`to` instant pair. ESS checks rows against those instants. The resolver's correctness (`TODAY` in a zone) is tested by the adopter outside ESS.
- Documented as out of scope: (c), the running duration at read time.
- Built: aggregate synthesis treats a top-level ordering of a `Timestamp` field against a parameter as a range selector, with boundary and offset-spelled rows.
- Where it is documented: the section `## A clock-relative window is resolved by the caller` in `docs/design/read-api-view-idioms.md`, with its model in `docs/design/read-api-view-idioms.example/window.yaml`, domain `idioms.window`: entity `idioms.window.Call` (`call_id`, `started_at: Timestamp`) and the view `idioms.window.CallsInRange` with parameters `from` and `to`. Depends on #441 (edge recorded), which creates the note, the example directory and the test.

No ess/23, and no suite pair expected. Depends on #438 (edge recorded): it extends the parameter selectors in `aggregate.rs` (1150-1163) after #438 has changed them.

## Acceptance
- aggregate_timestamp_range_parameters_select_rows: a view filtered by `started_at >= param.from` and `started_at < param.to` is synthesized. The rows at `from - 1s`, `from` and `to - 1s` are counted as they fall, and `to` is excluded.
- aggregate_timestamp_range_compares_instants_not_text: a row spelled at an offset whose text sorts on the other side of a bound is counted by its instant; a text-comparing target fails.
- aggregate_timestamp_range_faults_fail: a target that ignores a bound, makes `to` inclusive or compares text fails the named observation in the interpreter and the Go and TypeScript runners.
- window_parameters_stay_refused: `now` and `window:` in a view filter keep their `type_mismatch` refusals (existing calendar_windows and current-time controls unchanged).
- caller_resolved_window_section_states_the_idiom: `docs/design/read-api-view-idioms.md` has the heading `## A clock-relative window is resolved by the caller`. The section contains "`param.from`", "`param.to`", "the caller resolves", "zone", "last N minutes", "no zone data" and "running duration". The case `caller_resolved_window_section_states_the_idiom` in `crates/verify/ess-conformance/tests/read_api_view_idioms.rs` reads the note and fails on a missing heading or phrase. The case `caller_resolved_window_example_validates`, in the same file, validates the example directory with `window.yaml` in it and finds `idioms.window.CallsInRange`.

## Scope
- crates/verify/ess-conformance/src/synthesize/aggregate.rs  cited — parameter selectors, 1150-1163
- crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs  inferred — row arrangement around range bounds
- crates/verify/ess-conformance/tests/aggregate_group_selection.rs  inferred — range-selector controls beside the existing ones
- crates/verify/ess-conformance/tests/calendar_windows.rs  cited — refusal controls kept
- docs/design/calendar-window-guards.md  cited — zone decision, 8-25, 221-226
- docs/design/aggregate-group-selection.md  cited — selector rule, 33
- docs/design/read-api-view-idioms.md  inferred — adds `## A clock-relative window is resolved by the caller` (file created by #441)
- docs/design/read-api-view-idioms.example/window.yaml  inferred — domain `idioms.window`, the `from`/`to` model (directory created by #441)
- crates/verify/ess-conformance/tests/read_api_view_idioms.rs  inferred — adds the two named cases
- website/docs/reference/predicates.md  cited — `now` in views refused, 753-755
