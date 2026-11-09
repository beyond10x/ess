---
format: aep.planning-md/3
id: story:an-expression-measure-that-skips-absent-rows-is-witnessed
kind: story
status: draft
title: 'An expression measure that skips absent values or declares where: is witnessed'
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#561
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/aggregate_measures_over_expressions.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/
- confidence: cited
  path: docs/design/generic-language-features.md
revision: 4
---
## Need

The issue's first figure is the mean time to recovery: `avg` of `ended_at - started_at` over the
windows of a group, where a window still open has no `ended_at` and is left out. Since `ess/36`
that view validates with `skip_absent: true`, and so does an expression measure with `where:`, but
synthesis refuses both, so the suite does not check the figure the specification states.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/b-open-windows`, `repro/b-where` of the
triage of beyond10x/ess#561; `Window.ended_at: Optional<Timestamp>`):

```yaml
  - name: demo.ledger.Mttr
    source: demo.ledger.Window
    group_by: [incident]
    fields:
      - {name: incident, type: String}
      - {name: mttr, type: Optional<Decimal>, aggregate: {avg: {expr: "ended_at - started_at"}, skip_absent: true}}
```

```text
$ ess specify validate --path b-open-windows
demo v1 — 1 file(s), valid
$ ess verify conform synthesize --path b-open-windows --out b-suite.yaml
refused: refusal[ESS-SYNTH-017]: view demo.ledger.Mttr has no scenario `demo.ledger.Mttr/aggregate`
  `demo.ledger.Mttr` cannot be arranged: `mttr` computes avg(ended_at - started_at, skip_absent), an expression measure that skips absent values; this arrangement witnesses an expression measure over every row of its group
```

`{sum: {expr: "duration_s * level.weight"}, where: "duration_s > 0"}` is refused the same way
("reads only the rows its condition admits").

## Acceptance

- `ess verify conform synthesize --path b-open-windows` reports 0 refusals and writes
  `demo.ledger.Mttr/aggregate` arranging, in one group, two closed windows of different durations
  and one open window; it expects `mttr` the mean of the two closed durations in seconds (six
  places, half-even); `ess verify conform run --target interpreted` passes it, and a target that
  reads the open window as zero fails it.
- A group whose every window is open is arranged and expects `mttr` absent.
- `ess verify conform synthesize --path b-where` reports 0 refusals; its scenario arranges a row
  the `where:` excludes beside rows it admits, and a target ignoring the condition fails it.
- Models without `skip_absent:` or `where:` on an expression measure keep their suite bytes.

Pinned by `crates/verify/ess-conformance/tests/aggregate_measures_over_expressions.rs`
(`an_expression_measure_skipping_absent_values_is_witnessed`,
`an_expression_measure_with_a_condition_is_witnessed`).

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/aggregate.rs (the ESS-SYNTH-017 refusal around line 1925)
- inferred: crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs (contrast rows varying every field the expression reads)
- cited: crates/verify/ess-conformance/tests/aggregate_measures_over_expressions.rs
- inferred: crates/verify/ess-conformance/tests/fixtures/
- cited: docs/design/generic-language-features.md (§ G3 "Expression measures and ordered enums", the "Synthesis" bullet)
- inferred: CHANGELOG.md
confidence: high

## Fit review

1. **Need.** The issue's MTTR over windows of which some are still open, checked by the suite.
   Reproduction above (run.txt § B). Requester's syntax (theirs): `aggregate: {avg: ended_at -
   started_at}`; the spelling shipped in `ess/36` as `{avg: {expr: …}}`.
2. **Class.** Gap in the check: the language states the figure (`ess/36`, `FORMAT_HISTORY` in
   `crates/specify/ess-domain/src/system.rs`; run.txt § B validate exit 0) but synthesis refuses
   it by design of the first cut (`docs/design/generic-language-features.md` § G3, "Synthesis":
   "An expression measure that also declares `skip_absent:` or `where:` is refused as
   `ESS-SYNTH-017` … not extended in this cut").
3. **Already expressible?** Expressible, not checked. An authored scenario can stand in; that
   leaves the figure to each adopter's hand, and policy 1 asks the suite to check it.
4. **Fit.** No language change, no grid cell moves. The arrangement already varies every field an
   expression reads, one ladder each (`synthesize/aggregate.rs:1917-1920`); the absent-row and
   contrast arrangements for a field measure (`aggregate/contrast.rs`) extend to the fields the
   expression reads. Targets: synthesis only; every runner executes the scenario.
5. **Second adopter.** Average fulfilment time over orders of which some are not shipped yet;
   mean handling time over tickets with an open end.
6. **Cost.** No source format, no key, no diagnostic code, no suite format (the scenario
   vocabulary is unchanged). Suites of models with such a measure gain a scenario and lose a
   refusal.
7. **Alternatives.** (a) Change nothing: adopters author the scenario. (b) Refuse `skip_absent`
   on expression measures in validation: takes the issue's figure back out of the language.
   (c) Chosen: extend the arrangement.

## Decisions

**Accept as proposed** (the requester's figure, in the spelling `ess/36` shipped): synthesis
witnesses an expression measure with `skip_absent:` or `where:`. No format change.

## Source

github:beyond10x/ess#561
