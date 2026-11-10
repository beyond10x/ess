---
format: aep.planning-md/3
id: story:synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields
kind: story
status: draft
title: Synthesis witnesses aggregate conditions over optional and state-filtered fields
summary: 'ESS-SYNTH-017 for where: on Optional fields and four state-filtered counts in one group (workbench report items 20, 24b); part A, avg over a field only a later command sets, is a backlog draft.'
tags:
- backlog
- defect
- external-request
- from-consumer-report
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/conditional_measures_cover_every_state.rs
revision: 11
---
## Need

An aggregate view that validates gets no scenario when its group's conditioned measures together
cover every state of the lifecycle. Item 24b of the 2026-11-09 workbench report
(`.engineering/repro/workbench-2026-11-09/report.md`); the example is the base specification
`examples/00-base/spec` plus the lines quoted, under
`.engineering/repro/workbench-2026-11-09/examples/24b/spec`. Re-run on 0.92.0 with
`ess verify conform synthesize --path spec --out suite.yaml` (exit 0) and
`ess verify conform run --target interpreted --path spec --report-format 2`.

**Conditions over every state of the lifecycle in one group (item 24b, example `24b`).**
`Session` moves `Idle` to one of `Running`, `Paused`, `Waiting`; `SessionTotals` (no `group_by`)
counts each state with `where: state == <S>`, four measures.

```text
refused: refusal[ESS-SYNTH-017]: view gap.demo.SessionTotals has no scenario `gap.demo.SessionTotals/aggregate`
  `gap.demo.SessionTotals` cannot be arranged: a measure's condition: no arrangement of one group's rows gives the condition of `idle`, `paused`, `running`, `waiting` rows it admits and rows it refuses that its measure tells apart (...)
```

The same view with one, two or three of the four measures (any of them, `idle` included)
synthesizes with 0 refusals and passes on the interpreted target. Inferred from
`crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs:506-516`: the contrast search
varies the values of group A's rows but not their number, which is fixed before the search
(`group_size`, `crates/verify/ess-conformance/src/synthesize/aggregate.rs:1473`, at least three
rows); four mutually exclusive conditions each need a row they admit, so a group with fewer rows
than conditions has no pattern.

**Not a defect: item 20 (example `20`).** `NotedItems` has `filter: note == "x"` and a measure
`with_x: {count: {}, where: note == "x"}`. The filter already removes every row the condition
refuses, so no target can drop or invert the condition observably; ESS-SYNTH-017 and its help
("or declare the measure without `where:`") are right. Without the view's `filter:`, the same
measure on the `Optional` field synthesizes with 0 refusals (11 scenarios, all pass).

Item 22 of the same report (an average over a field only a later command sets) is a gap, not a
defect, and is tracked outside this epic as its own backlog draft.

ESS-SYNTH-017 appears 8 times in the reporter's whole model.

## Fit review

1. **Need.** A set of state counts covering the whole lifecycle gets an aggregate scenario.
   Reproduction above, brand-free.
2. **Class.** Defect: the documented contrast (`contrast.rs:1-27`,
   `docs/design/conditional-aggregate-measures.md`) exists for four states, and the lifecycle state
   is a dimension the module says it varies.
3. **Already expressible?** Only by dropping a measure, which changes the specification the reader
   sees, so it is no idiom.
4. **Fit.** No authored surface. The change grows group A to at least one row per conditioned
   measure.
5. **Second adopter.** A status board counting every state.
6. **Cost.** No format. Suites of models with this shape gain an aggregate scenario; pinned digests
   in `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` move only for
   those rows.
7. **Alternatives.** (a) Change nothing. (b) Grow group A to one row per conditioned measure:
   chosen.

## Decisions

- Accept item 24b as a defect; item 20 is answered with the rule above and needs no change.
- Item 22 (arrangement through creators only) is a gap and is split off as its own backlog draft
  outside the epic, which takes defects only.

## Acceptance

Every bullet is a case in `cargo test -p ess-conformance --locked --test conditional_measures_cover_every_state`,
on brand-free copies of examples `24b` and `20`:

- Synthesis of example `24b` reports `0 refusal(s)` and writes `gap.demo.SessionTotals/aggregate`.
- That scenario's arrangement leaves at least one `Session` row in each of `Idle`, `Running`,
  `Paused` and `Waiting`, and its view expectation asserts all four counts.
- `ess verify conform run --target interpreted` passes `gap.demo.SessionTotals/aggregate`.
- Synthesis of example `20` still refuses `gap.demo.NotedItems/aggregate` with `ESS-SYNTH-017`.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes on the change,
  and every row of `tests/fixtures/synthesis-suite-bytes-table.tsv` that differs from the base
  belongs to a model with more conditioned measures in one group than the base group size held.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/aggregate.rs` (`group_size` :1473 and its
  call :2059)
- cited: `crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs` (`arrange` :497,
  group A and the search)
- inferred: `crates/verify/ess-conformance/tests/conditional_measures_cover_every_state.rs` (new)

confidence: medium

Shared surfaces: re-pins the suite-bytes rows it claims and proposes one `CHANGELOG.md` sentence.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
