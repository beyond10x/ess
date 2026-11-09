---
format: aep.planning-md/3
id: story:synthesis-arranges-the-related-rows-a-guard-or-read-needs
kind: story
status: draft
title: Synthesis arranges the related and filtered rows a guard or read needs
summary: ESS-SYNTH-001 for guards over related row states and counts, ESS-SYNTH-005 for an optional-filtered read (workbench report items 21, 23, 24a)
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
revision: 3
---
## Need

Commands whose guards or reads validate get no witness. Items 21, 23 and 24a of the 2026-11-09
workbench report (`.engineering/repro/workbench-2026-11-09/report.md`); each example is the base
specification `examples/00-base/spec` plus the lines quoted. Re-run on 0.92.0 with
`ess verify conform synthesize --path spec --out suite.yaml` (exit 0) and
`ess verify conform run --target interpreted --path spec --report-format 2` (every synthesized
scenario passes).

**A. The related row a creator needs is the guarded command's own subject (item 21, example `21b`).**
`CreateItem` refuses a missing parent (`no-parent: when_related: {via: input.parent_id, exists:
false}`); `MergeParent` (subject a `Parent`, `moves: Parent.close`) refuses while items of that parent
are open:

```yaml
- name: items-in-flight
  when_related:
    entity: gap.demo.Item
    where:
      all:
      - parent_id == input.id
      - state: [Open]
    exists: true
```

```text
refused: refusal[ESS-SYNTH-001]: outcome gap.demo.MergeParent/items-in-flight has no scenario `gap.demo.MergeParent/outcome/items-in-flight`
  no witness: `...`parent_id` created by `gap.demo.CreateItem`, whose `no-parent` refuses a missing `gap.demo.Parent` ...` is `row set`, which has no arrangement of rows its selector selects: the command creating them refuses a missing related row through the input the selector compares, and no arrangement left a related row for the rows and the send to name
15 scenario(s) (0 authored), 1 refusal(s)
```

Dropping `state: [Open]` changes nothing; dropping `no-parent` gives 0 refusals. The 0.69.0 fix
(CHANGELOG, "The accepting branch of a row-set command gets its scenario when the rows it
selects by `field == input.<x>` are created by a command that refuses a missing related row")
arranges a separate related row; here that row must be the subject `MergeParent` addresses, which
`CreateParent` creates in `Open`, a state `CreateItem` accepts.

**B. A view filtered on an Optional field the creator leaves unset (item 23, example `23`).**
`Item.score: Optional<Integer>` is set only by `ScoreItem`; the view `HighScores` has
`filter: score == 5`.

```text
refused: refusal[ESS-SYNTH-005]: outcome gap.demo.CreateItem/created has a scenario `gap.demo.CreateItem/outcome/created` that skips its read of `gap.demo.HighScores`
  `gap.demo.HighScores` filters on `score == 5`, which is undecided for an entity in `Open`
    - `score` is bound by nothing a scenario knows
12 scenario(s) (0 authored), 3 refusal(s)
```

The same refusal names `FinishItem/outcome/finished` and `Item/transition/finish/...`. After
`CreateItem` the field is absent, and an absent `Optional` compared with a literal is false, so the
row is decidedly outside the view. `shows_row` is asked with `settled` holding only fields the
arrangement wrote (`crates/verify/ess-conformance/src/synthesize.rs:10341`, refusal at `:10360`).

**C. A guard counting rows no input scopes (item 24a, example `24a`).** `CreateItem/halted` is
`when_related: {entity: Switch, where: [state == Engaged, scope == global], count: {gte: 1}}`.
On 0.92.0 both `CreateItem/halted` and `CreateItem/created` are refused as ESS-SYNTH-001 "selects
rows by no equality between a String or Uuid field and the input or the addressed subject, so the
rows it counts are not only the scenario's own". That is a deliberate rule
(`crates/verify/ess-conformance/src/synthesize/row_set.rs:3136`, test
`adversary_463_pass1::adv_integer_member_alone_is_refused_as_unscoped`). Aggregate views already
lift the same rule under `Empty` provenance, since a run starts on an empty namespace
(`docs/design/aggregate-group-selection.md` § "State-only and other unscoped groups"). The help
("give the field a type that has a finite value, or drop it from the command's input") does not fit a
field compared with a literal. The report's quoted message for 24a ("a refused subject is observed
through a view of the entity projecting its identity, and none shows this row") is the wrong-state
refusal of `Switch`, which has no view; its help names the fix and it is not part of this story.

ESS-SYNTH-001 appears 27 times and ESS-SYNTH-005 14 times in the reporter's whole model.

## Fit review

1. **Need.** A command refusing while related rows of its own subject exist, a read of a view
   filtered on an Optional field, and a count guard over global rows each get scenarios. Brand-free
   reproductions above.
2. **Class.** A and B: defect. The help of A names a declared command creating the related row,
   which exists; B is decidable under three-valued comparison. C: gap against a documented rule.
3. **Already expressible?** Only by an authored scenario (`ess-scenario/1`), which the help of A
   names; no idiom keeps the generated witness.
4. **Fit.** No authored surface. A extends the related-row arrangement of row sets to the subject;
   B settles fields the creator leaves unset as absent for view reads; C reuses the `Empty`
   provenance lift that aggregate views use for row-set guards.
5. **Second adopter.** A project that cannot close while it has open tasks; a "flagged" view on an
   optional flag; a maintenance switch that halts every creation.
6. **Cost.** No format. Suites of models with these shapes gain scenarios; pinned digests in
   `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` move only for rows
   with these shapes.
7. **Alternatives.** (a) Change nothing: authored scenarios per branch. (b) A and B only, keep C's
   rule. (c) All three.

## Decisions

- Accept A and B as defects.
- C needs a decision on lifting the unscoped row-set rule under `Empty` provenance; until then only
  its help text is in scope (name a literal-compared field and the authored-scenario route).

## Acceptance

- Example `21b` synthesizes `gap.demo.MergeParent/outcome/items-in-flight` with 0 refusals, and the
  interpreted run passes it: the arranged item names the merged parent.
- Example `23` synthesizes with 0 refusals; the `CreateItem/outcome/created` scenario reads
  `HighScores` and expects no row, and a `ScoreItem` scenario with `score: 5` expects the row.
- Example `24a`: ESS-SYNTH-001 on `CreateItem/halted` either is gone (if C is lifted) or its help no
  longer suggests a finite type for a field compared with a literal.
- No pinned suite digest moves except rows whose model has one of these shapes.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/row_set.rs` (related-row arrangement, `decidable`/`scoped`)
- cited: `crates/verify/ess-conformance/src/synthesize.rs` (view reads, `shows_row` settled fields)
- inferred: `crates/verify/ess-conformance/tests/synthesis.rs`
- inferred: `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv`

confidence: medium

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
