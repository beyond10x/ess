---
format: aep.planning-md/3
id: story:an-interval-union-measure-is-clipped-to-a-period
kind: story
status: active
title: An interval-union measure is clipped to a period
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#561
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:derived-measures-and-interval-union
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src/view_query.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: docs/design/generic-language-features.md
- confidence: inferred
  path: docs/design/language-grids.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:40:09Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:40:11Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Downtime per group is the length of the union of the `[started_at, ended_at)` intervals of its
rows, clipped to the requested period `[param.from, param.to)`: two overlapping windows count their
overlap once, and a window across two periods counts its part in each. A `sum` of durations counts
the overlap twice and ignores the period, so the figure a reader of an availability report sees is
not stated in the specification. Today no measure takes two fields, and no aggregate function is a
union.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/c-covered` of the triage of
beyond10x/ess#561):

```yaml
  - name: demo.ledger.Downtime
    source: demo.ledger.Window
    params: [{name: from, type: Timestamp}, {name: to, type: Timestamp}]
    filter: [started_at < param.to]
    group_by: [incident]
    fields:
      - {name: incident, type: String}
      - {name: down, type: Duration, aggregate: {covered: {from: started_at, to: ended_at, within: [param.from, param.to]}}}
```

```text
$ ess specify validate --path c-covered
system.yaml: unknown variant `covered`, expected one of `count`, `count_distinct`, `sum`, `min`, `max`, `avg`
exit 1
```

The requester's `{union: …}` is refused the same way (beyond10x/ess#562).

## Acceptance

At the next source format (`ess/37`), on a view `Downtime` over `Window` grouped by `incident`,
with `params: [{name: from, type: Timestamp}, {name: to, type: Timestamp}]` and
`{name: down, type: Duration, aggregate: {covered: {from: started_at, to: ended_at, within: [param.from, param.to]}}}`:

- `ess specify validate` exits 0; `covered` over fields that are no `Timestamp`, a `within:` that is
  no pair of `Timestamp` parameters or literals, or a result type other than `Duration` is refused
  as `type_mismatch`; an `Optional` bound needs `skip_absent: true`, as every measure does;
- the interpreter answers, for windows `[10:00, 10:30)` and `[10:20, 10:50)` of one incident and a
  period `[10:00, 11:00)`, `down: 3000s`; for one window `[09:50, 10:10)` and the period
  `[10:00, 11:00)`, `down: 600s`; for a window outside the period, `0s`;
- `ess verify conform synthesize` writes `Downtime/aggregate` with an overlapping pair, an
  adjacent pair, a window spanning a period edge and one outside, and `ess verify conform run
  --target interpreted` passes; a target summing durations fails it;
- `covered` composes with a derived field (`story:derived-measures-and-interval-union`): `1 - down /
  (param.to - param.from)` validates and is witnessed;
- below `ess/37` `covered` is refused as `unsupported_format_version` naming `ess/37`; a model
  without it keeps its IR bytes;
- generated Rust and Go view queries keep such a view an obligation naming the measure; Entity
  Runtime refuses it by name; `ess verify diff` reports a changed measure as
  `field-aggregate-changed`.

## Scope

- cited: crates/specify/ess-domain/src/view.rs (`AggregateFunction`, `RawAggregate`, `result_type`)
- inferred: crates/specify/ess-compiler/src/ir.rs (`ResolvedAggregate`)
- inferred: crates/specify/ess-compiler/src/resolve.rs
- inferred: crates/verify/ess-conformance/src/aggregate.rs (`evaluate`: union of clipped intervals, exact seconds)
- inferred: crates/verify/ess-conformance/src/interpret/views.rs
- inferred: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- inferred: crates/generate/ess-synth/src/view_query.rs
- inferred: crates/verify/ess-diff/src/change.rs
- inferred: crates/specify/ess-domain/src/system.rs
- inferred: schemas/generated/ess.schema.json
- cited: docs/design/generic-language-features.md (§ G3, the `covered` subsection decided before code)
- inferred: docs/design/language-grids.md (grid O, measure row)
- inferred: website/docs/reference/spec-versions.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** The measure of the union of intervals clipped to a period, per group, checked by the
   suite. A consumer cannot derive it from counts or sums: it needs the rows. Requester's syntax
   (theirs, beyond10x/ess#562 under the #561 feature): `aggregate: {union: {from, to, clip}}`.
2. **Class.** Gap: no measure reads two fields, none unions intervals
   (`docs/design/generic-language-features.md:360-365`, "one of six functions over one top-level
   field").
3. **Already expressible?** No. Copy-at-write cannot know later overlapping rows; a consumer-side
   computation moves the figure out of conformance (policy 1).
4. **Fit.** G3 decides it as a library function, not a key family: "`covered(from, to, within: [a,
   b])`, the measure of the union of intervals clipped to a period"
   (`generic-language-features.md:383-386`). The YAML spelling follows the measure shape
   `{<function>: <operand>}` already used (`{sum: …}`, `{avg: {expr: …}}`), with named operands
   because the function takes three. It composes with `where:` and `skip_absent` as every measure
   does, and with derived fields. Grid O: the measure row gains `covered` for Timestamp pairs only.
   Targets: interpreter and synthesis exact in seconds; Rust and Go obligations, named; Entity
   Runtime by name; diff.
5. **Second adopter.** Room occupancy per day from overlapping bookings; agent logged-in time per
   shift from overlapping sessions.
6. **Cost.** The next source format (`ess/37`), counted under G3; one aggregate function name; no
   suite format (a Duration figure is already a scenario value since `ess/36`).
7. **Alternatives.** (a) Change nothing. (b) The requester's `union:` with `clip:`: a key family
   for one measure, and `union` reads as a set operation; changed to G3's `covered`. (c) Chosen:
   `covered` as a measure function with named operands.

## Decisions

**Accept, redesigned.** `covered: {from, to, within: [a, b]}` is a measure function over a pair of
Timestamp fields, answering a Duration: the length of the union of the rows' intervals clipped to
`[a, b)`. From `ess/37`, counted under G3; G3 gains a decided subsection before code.

## Source

github:beyond10x/ess#561 (finding beyond10x/ess#562)
