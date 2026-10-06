---
format: aep.planning-md/3
id: story:feature-request-441
kind: story
status: active
title: 'Derived values over aggregates: ratio, difference, scaling, rounding'
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#441
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/tests/read_api_view_idioms.rs
- confidence: cited
  path: docs/design/aggregate-views.md
- confidence: inferred
  path: docs/design/read-api-view-idioms.example/ratio.yaml
- confidence: inferred
  path: docs/design/read-api-view-idioms.example/system.yaml
- confidence: inferred
  path: docs/design/read-api-view-idioms.md
- confidence: cited
  path: website/docs/guides/specify/aggregate-views.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:23Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:24Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":4}}}
---
## Outcome
Resolve beyond10x/ess#441: Derived values over aggregates: ratio, difference, scaling, rounding.

## Origin
beyond10x/ess#441, filed 2026-10-05; an adopter's metrics read API, where 5 of 19 table-reading metrics are ratios, 1 is a difference, and several compare against a parameter scaled from seconds to milliseconds.

## Fit review
1. Need: a read returns, per group, a number computed from two measures of the same row: a ratio with a stated rounding and a stated value for a zero denominator, or a difference. A filter also compares a stored field with a parameter given in another unit. Minimal reproduction, written fresh (`<fit-review scratch>/probe-441/`): `queued = count`, `abandoned = count where abandoned`, `rate = abandoned / queued`. The requester's syntax, not adopted: `value: {divide: [n, d], on_zero: 0, round: half_up}`, `{subtract: [a, b]}`, `duration_sum > param.greater_than * 1000`.
2. Class:
   - Ratio and difference: a gap in what a suite can check, closed by a standing decision. Operator decision 3 of 2026-09-25 reads "No ratios or differences between aggregates; left to the consumer" (`story:aggregate-views` Decisions; docs/design/aggregate-views.md:23, :1031). That decision is not a defect.
   - Scaling: a unit conversion at the adopter's boundary, so local policy (see #444).
3. Existing idiom:
   - Both operands in one row with conditional measures (ess/22, aggregate-views guide:418-431). `{count: {}}` and `{count: {}, where: abandoned == true}` validate together (probe-441 c's operands).
   - The client computes the ratio, its rounding and its zero rule from two numbers the suite checks exactly.
   - `{divide: …}` is refused: "unknown variant `divide`" (probe-441 c).
   - A computed field is refused: "promises an observation nothing produces" / "neither an aggregate nor a group key" (probe-447 n4).
   - Additive offsets on a parameter validate: `duration_ms > param.limit_ms + 1000` (probe-441 b; spec-versions ess/22, "one fact moved by one constant").
   - Multiplication is read as text: "Text literal `param.limit_s * 1000`" (probe-441 a). Declaring the parameter in the stored unit is the idiom; the wire conversion belongs to the adapter.
4. Fit:
   - A derived field would be a second kind of view field beside `aggregate:`. It needs its own type rule (Decimal, scale), its own rounding mode, beside `avg`'s 6 digits with ties to even (aggregate-views guide:404-405), and a zero rule. It needs an evaluator in the interpreter, the generated Rust and Go queries, the Go and TypeScript runners and Entity Runtime lowering. It also needs a synthesis contrast that makes numerator and denominator decisive.
   - The decision gave the reason before: a consumer computes those. Nothing in the issue is a fact ESS gets wrong; it is a fact ESS chose not to state.
   - Scaling in predicates would bring multiplication into the expression vocabulary, which has one constant offset only (predicates.md:297-305).
5. Second adopter: any KPI read, such as conversion rate (orders / visits) or a margin (revenue − cost). The need is general, and that is why the decision is the operator's, not this review's.
6. Cost if accepted: a new view-field construct, ess/23, a suite pair carrying computed values, a diff classification, and every target. Cost of the idiom: a guide section and one validated example.
7. Alternatives:
   - (a) Change nothing: the operands are checked and the derived number is not.
   - (b) The requester's `value: {divide|subtract}` with `on_zero`/`round`: reverses decision 3.
   - (c) A narrower `ratio:` aggregate (sum or count over sum or count, Decimal, half-even at 6 digits, zero → absent): one construct, but still a reversal.
   - Chosen: keep decision 3 and document the idiom. (c) is recorded as the shape to start from if the operator reopens it.

## Decisions
decline, with the idiom — Decision 3 (2026-09-25) stands. The story body is the decline record.
- This story creates the shared idioms files (coordinator decision 2026-10-05): the note `docs/design/read-api-view-idioms.md` (title `# Read-API view idioms`), its validated example directory `docs/design/read-api-view-idioms.example/`, and `crates/verify/ess-conformance/tests/read_api_view_idioms.rs`. The test has one helper that returns a note section by its `## ` heading, splitting the way `crates/edge/ess-xtask/tests/d2_constraint_home_adversary.rs:116-123` does, and one that validates the example directory. #439, #442, #443, #444, #446 and #447 depend on this story (edges recorded); each adds its own `## ` section to the note, its own domain file to the example and its own named cases to the test.
- This story's note section is `## Derived values over aggregates are the consumer's`. A view returns the operands as conditional measures in one row, and the consumer derives the ratio or difference with its own rounding and zero rule. A parameter is declared in the stored unit, and the adapter converts the wire value.
- The guide gains `## What an aggregate view does not compute` in `website/docs/guides/specify/aggregate-views.md`, pointing at the note.
- `<path> * <n>` stays refused. The note names today's refusal of `param.limit_s * 1000` (`type_mismatch`) and quotes no hint text: the hint is #438's to build and accept, so this story does not wait for #438 and has no edge to it.
- One domain per idiom (coordinator decision 2026-10-05): `idioms.ratio` (this story), `idioms.correlated` (#442), `idioms.fold` (#443), `idioms.units` (#444), `idioms.latest` (#446), `idioms.joined` (#447) and `idioms.window` (#439). A source file carries the system header and at most one domain (`crates/specify/ess-domain/src/spec.rs:39-42`), so the example is a directory: `system.yaml` holds the header (`system: idioms`, no `domains:` roster, which is optional, `spec.rs:60-65`), and each idiom is its own file. The dependents therefore add disjoint files and no name can clash. This story writes `system.yaml` and `ratio.yaml`, domain `idioms.ratio`: entity `idioms.ratio.Call` (`queue_id`, `abandoned`, `duration_ms`), the aggregate view `idioms.ratio.QueueOutcomes` and the parameterised view `idioms.ratio.LongCalls`.
- If the operator reopens decision 3, start from alternative (c). It needs ess/23 and a suite pair.
- Reply to the requester naming decision 3 and the idiom.

## Acceptance
- derived_values_section_states_the_idiom: `docs/design/read-api-view-idioms.md` exists with the heading `## Derived values over aggregates are the consumer's`. The section contains "conditional measures", "one row", "the consumer derives", "rounding", "zero denominator", "decision 3", "stored unit", "`param.limit_s * 1000`" and "`ratio:`". The case reads the note and fails naming the missing heading or phrase.
- aggregate_views_guide_points_at_the_idioms_note: `website/docs/guides/specify/aggregate-views.md` has the heading `## What an aggregate view does not compute`. Its section names "ratio" and "difference" and links a target ending in `docs/design/read-api-view-idioms.md`. Same test file; `task site-build` does not resolve that link, so it is not the check.
- derived_value_example_validates: `docs/design/read-api-view-idioms.example/` validates as one specification, and `ratio.yaml` declares domain `idioms.ratio` with the view `idioms.ratio.QueueOutcomes` (`queued = count`, `abandoned = count where abandoned`, one row per group).
- derived_view_field_stays_refused: `{divide: …}` and a field with neither `aggregate:` nor a group key stay refused with today's messages.
- parameter_in_stored_unit_validates: `idioms.ratio.LongCalls`, filtered by `duration_ms > param.limit_ms + 1000`, validates in the example.

## Scope
- docs/design/aggregate-views.md  cited — decision 3, 23; out of scope, 1031
- docs/design/read-api-view-idioms.md  inferred — created here; shared design note that #439, #442, #443, #444, #446 and #447 extend
- docs/design/read-api-view-idioms.example/system.yaml  inferred — created here; the example's system header
- docs/design/read-api-view-idioms.example/ratio.yaml  inferred — created here; domain `idioms.ratio`
- crates/verify/ess-conformance/tests/read_api_view_idioms.rs  inferred — created here; section helper, validation and refusal cases
- website/docs/guides/specify/aggregate-views.md  cited — conditional measures, 418-431; new section
