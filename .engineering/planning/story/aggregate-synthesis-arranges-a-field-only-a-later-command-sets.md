---
format: aep.planning-md/3
id: story:aggregate-synthesis-arranges-a-field-only-a-later-command-sets
kind: story
status: draft
title: Aggregate synthesis arranges a field only a later command sets
relations:
- informed_by: story:synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields
revision: 1
---
## Need

An aggregate view over a field that only a later command writes gets no scenario. Item 22 of the
2026-11-09 workbench report (`.engineering/repro/workbench-2026-11-09/report.md`); the example is
the base specification `examples/00-base/spec` plus the lines quoted, under
`.engineering/repro/workbench-2026-11-09/examples/22/spec`. Re-run on 0.92.0 with
`ess verify conform synthesize --path spec --out suite.yaml` (exit 0) and
`ess verify conform run --target interpreted --path spec --report-format 2`.

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

Raised at `crates/verify/ess-conformance/src/synthesize/aggregate.rs:470` (`unchosen`): the
arrangement writes aggregated fields only through the creating command.

Split off from `story:synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields`,
which keeps item 24b, the defect.

## Fit review

1. **Need.** An aggregate over a field a later command writes gets an aggregate scenario.
   Reproduction above, brand-free.
2. **Class.** Gap: aggregate arrangement goes through creators only, and nothing documents that it
   reaches update steps.
3. **Already expressible?** Only by making the creator set the field, which changes the
   specification the reader sees, so it is no idiom.
4. **Fit.** No authored surface. It reuses the update steps synthesis already runs for row-set
   arrangements.
5. **Second adopter.** A mean rating set by a later review.
6. **Cost.** No format. Suites of models with this shape gain an aggregate scenario; pinned digests
   in `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` move only for
   those rows.
7. **Alternatives.** (a) Change nothing. (b) Arrange the field through the updating command after
   creation.

## Acceptance

Every bullet is a case in `cargo test -p ess-conformance --locked --test aggregate_over_a_later_set_field`,
on a brand-free copy of example `22`:

- Synthesis reports `0 refusal(s)` and writes `gap.demo.MeanScore/aggregate`.
- That scenario's arrangement sends `ScoreItem` for some rows after `CreateItem`, and leaves at
  least one row with `score` unset.
- `ess verify conform run --target interpreted` passes `gap.demo.MeanScore/aggregate`.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes on the change,
  and every row of `tests/fixtures/synthesis-suite-bytes-table.tsv` that differs from the base
  belongs to a model aggregating a field only an updating command sets.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/aggregate.rs` (`unchosen` :460, the
  arrangement through the creator)
- inferred: `crates/verify/ess-conformance/tests/aggregate_over_a_later_set_field.rs` (new)

confidence: medium

Shared surfaces: re-pins the suite-bytes rows it claims and proposes one `CHANGELOG.md` sentence.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, item 22, with its example under `examples/22`.
