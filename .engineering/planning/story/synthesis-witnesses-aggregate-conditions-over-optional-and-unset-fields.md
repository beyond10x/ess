---
format: aep.planning-md/3
id: story:synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields
kind: story
status: draft
title: Synthesis witnesses aggregate conditions over optional, unset and state-filtered fields
summary: 'ESS-SYNTH-017 for where: on Optional fields, avg over an unset field, four state-filtered counts in one group (workbench report items 20, 22, 24b)'
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
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: inferred
  path: crates/verify/ess-conformance/tests/synthesis.rs
revision: 5
---
## Need

Aggregate views that validate get no scenario. Items 20, 22 and 24b of the 2026-11-09 workbench
report (`.engineering/repro/workbench-2026-11-09/report.md`); each example is the base
specification `examples/00-base/spec` plus the lines quoted. Re-run on 0.92.0 with
`ess verify conform synthesize --path spec --out suite.yaml` (exit 0) and
`ess verify conform run --target interpreted --path spec --report-format 2`.

**A. An average over a field only a later command sets (item 22, example `22`).**
`Item.score: Optional<Integer>` is written by `ScoreItem` (`updates: Item`, `sets: {score:
input.score}`), never by `CreateItem`:

```yaml
- name: gap.demo.MeanScore
  source: gap.demo.Item
  consistency: eventual
  fields:
  - {name: items, type: Integer, aggregate: {count: {}}}
  - {name: mean, type: Optional<Decimal>, aggregate: {avg: score, skip_absent: true}}
```

```text
refused: refusal[ESS-SYNTH-017]: view gap.demo.MeanScore has no scenario `gap.demo.MeanScore/aggregate`
  `gap.demo.MeanScore` cannot be arranged: the creating command does not set `score`, which `mean` aggregates
```

Raised at `crates/verify/ess-conformance/src/synthesize/aggregate.rs:470`: the arrangement writes
aggregated fields only through the creating command.

**B. Conditions over every state of the lifecycle in one group (item 24b, example `24b`).**
`Session` moves `Idle` to one of `Running`, `Paused`, `Waiting`; `SessionTotals` (no `group_by`)
counts each state with `where: state == <S>`, four measures.

```text
refused: refusal[ESS-SYNTH-017]: view gap.demo.SessionTotals has no scenario `gap.demo.SessionTotals/aggregate`
  `gap.demo.SessionTotals` cannot be arranged: a measure's condition: no arrangement of one group's rows gives the condition of `idle`, `paused`, `running`, `waiting` rows it admits and rows it refuses that its measure tells apart (...)
```

The same view with one, two or three of the four measures (any of them, `idle` included)
synthesizes with 0 refusals and passes on the interpreted target. Inferred from
`crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs:506-516`: the contrast search
varies the values of group A's rows but not their number, which is fixed before the search; four
mutually exclusive conditions each need a row they admit, so a group with fewer rows than conditions
has no pattern.

**Not a defect: item 20 (example `20`).** `NotedItems` has `filter: note == "x"` and a measure
`with_x: {count: {}, where: note == "x"}`. The filter already removes every row the condition
refuses, so no target can drop or invert the condition observably; ESS-SYNTH-017 and its help
("or declare the measure without `where:`") are right. Without the view's `filter:`, the same
measure on the `Optional` field synthesizes with 0 refusals (11 scenarios, all pass).

ESS-SYNTH-017 appears 8 times in the reporter's whole model.

## Fit review

1. **Need.** An aggregate over a field a later command writes, and a set of state counts covering
   the whole lifecycle, each get an aggregate scenario. Reproductions above, brand-free.
2. **Class.** A: gap (arrangement through creators only). B: defect: the documented contrast
   (`contrast.rs:1-27`, `docs/design/conditional-aggregate-measures.md`) exists for four states, and
   the lifecycle state is a dimension the module says it varies.
3. **Already expressible?** A: only by making the creator set the field. B: only by dropping a
   measure. Both change the specification the reader sees, so neither is an idiom.
4. **Fit.** No authored surface. A reuses the update steps synthesis already runs for row-set
   arrangements; B grows group A to at least one row per conditioned measure.
5. **Second adopter.** A mean rating set by a later review; a status board counting every state.
6. **Cost.** No format. Suites of models with these shapes gain an aggregate scenario; pinned
   digests in `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` move
   only for those rows.
7. **Alternatives.** (a) Change nothing. (b) B only. (c) A and B.

## Decisions

- Accept B as a defect, A as a gap; item 20 is answered with the rule above and needs no change.

## Acceptance

- Example `22` synthesizes `gap.demo.MeanScore/aggregate` with 0 refusals: its rows take `score`
  through `ScoreItem`, at least one row is left unscored, and the interpreted run passes it.
- Example `24b` synthesizes `gap.demo.SessionTotals/aggregate` with 0 refusals: rows are moved into
  each of the four states, every count is asserted, and the interpreted run passes it.
- Example `20` keeps its ESS-SYNTH-017 refusal (regression test that a measure condition the
  filter implies stays refused).
- No pinned suite digest moves except rows whose model has one of these shapes.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/aggregate.rs` (arrangement through the creator)
- cited: `crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs` (group A size and search)
- inferred: `crates/verify/ess-conformance/tests/synthesis.rs`
- inferred: `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv`

confidence: medium

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
