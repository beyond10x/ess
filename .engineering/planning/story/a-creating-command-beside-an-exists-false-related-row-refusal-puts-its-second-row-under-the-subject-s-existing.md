---
format: aep.planning-md/3
id: story:a-creating-command-beside-an-exists-false-related-row-refusal-puts-its-second-row-under-the-subject-s-existing
kind: story
status: active
title: 'A creating command beside an `exists: false` related-row refusal puts its second row under the subject''s existing related row, so the ordered view''s order is checked and no ESS-SYNTH-014 names a scenario the suite holds.'
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#795
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T02:31:06Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T02:31:07Z", actor: "human:timo", revision: 4}
---
# d795: an order check beside a missing-related-row refusal is witnessed

Size: M (about 220 changed lines: 90 source, 130 test).
Fixed suite bytes of committed models: none expected. The four rows of
`tests/fixtures/synthesis-suite-bytes-table.tsv` holding both `exists: false` and `order_by:`
(`coalesce-selector`, `nested-ordered-heads`, `ordered-related-head`, `related-rows-quantifier`)
order a related head selection, not a view; no model under `examples/` or `generated/` has the
shape. `docs/design/read-api-view-idioms.example` is the one repository system holding both
`exists: false` (`joined.yaml:82`) and an ordered view (`latest.yaml:60`); run on 0.98.0
(`repro/795-idioms/`): 18 scenarios, 4 refusals (ESS-SYNTH-025 once, ESS-SYNTH-017 three times),
none ESS-SYNTH-014, so its row of `synthesis-suite-bytes-table.tsv` (line 466) is not expected to
move. The byte pins below decide.
Patch-release candidate: no (synthesis changes the suite of every model with the shape, and the
refusal text changes).

## Need

beyond10x/ess#795. A command that creates a row under an existing parent row, guarded by a refusal
for a missing parent (`when_related: {via: input.<parent>, exists: false}`), loses the order check
of a view that ranks the created rows per parent: synthesis puts the second row under a second
parent, so the view never holds two rows, and it refuses ESS-SYNTH-014 saying the outcome "has no
scenario" while the suite holds that scenario and it passes. Without the refusal the same model
gets the order check.

## Reproduction

Reproduced on 0.98.0. Model `repro/795/` (the issue's model, `format: ess/37`): `Shelf` owns many
`Item` via `shelf_id`; `PlaceItem(shelf_id, position)` with `no-such-shelf`
(`when_related: {via: input.shelf_id, exists: false}`) then `placed` (`creates: Item`, minted
`item_id`); view `ShelfItems` (param `shelf_id`, filter `shelf_id == param.shelf_id`,
`order_by: [position asc]`).

```console
$ ess verify conform synthesize --path . --out suite.json
refused: refusal[ESS-SYNTH-014]: outcome depot.shelf.PlaceItem/placed has no scenario `depot.shelf.PlaceItem/outcome/placed`
  `depot.shelf.ShelfItems` declares an order and this scenario can put 1 row(s) in it: comparing two rows needs two `depot.shelf.Item`s, and no declared move reaches it from `Placed`
3 scenario(s) (0 authored), 1 refusal(s), written to suite.json
$ ess verify conform run --path . --suite suite.json --target interpreted --report-format 2 --report-out report.json
  passed depot.shelf.PlaceItem/outcome/placed
  3 scenarios: 3 passed, 0 failed, 0 error, 0 unsupported
```

The `placed` scenario sends its two `PlaceItem` to `shelf-138601` and `shelf-194045` and checks
only `contains` and `excludes`; the issue's `noguard/` copy sends both to one shelf and adds
`counts` and `ranked`.

## Outcome

Where a ranked view needs a further row under the subject's parent and the creating branch reads
that parent through a related guard, synthesis reuses the parent row the scenario already holds
(it exists, so `exists: false` does not hold for it) and sends the second creation there. The
`placed` scenario checks `counts` and `ranked` on `ShelfItems`, as without the guard. Where an order
check still cannot be arranged, the refusal stands beside the scenario ("has a scenario … without
its order of `<view>`") and its reason names the related-guard cause, not "no declared move
reaches it".

## Cause (read in the source, not run)

Paths under `crates/verify/ess-conformance/src/`.

- `arrange_ranked` (`synthesize.rs:12320-12409`) asks `arrange_beside` (`:15392`) for a further row
  under the subject's owner (`owner_of_row` `:8791`, `shares_owner` `:8784`), through `under_owner`
  (`:8685`) → `created_bound` (`:16088`) → `created_owned` → `created_by`, which binds
  `{shelf_id: <subject's shelf>}` (`:8927-8936`).
- `created_owned_made` calls `related_guard::drive` (`:8867-8883`). `drive`
  (`synthesize/related_guard.rs:2672`) returns `Err(unarranged())` whenever `bound` already holds
  the guarded field, unless `subject_fact::held` finds the row on the thread-local stack
  (`:2652-2667`), which it does not during a view read. The candidate is skipped
  (`synthesize.rs:15899-15901`) and the search reports `Unreachable::NoPath` (`:15411-15415`,
  printed at `:3314`).
- The refusal is built with `Refusal::about` (`:825`, `stands: false`), so `Display` (`:941-946`)
  prints "has no scenario". `ViewUndecidable` already uses `Refusal::beside` (`:845`, printed at
  `:890-901`), the shape to reuse.

## Acceptance

1. Red today: synthesizing `repro/795/` writes no refusal, and the scenario
   `depot.shelf.PlaceItem/outcome/placed` sends two `PlaceItem` with one `shelf_id` instance and
   expects `ShelfItems` with `counts` and `ranked`.
2. That suite passes on the interpreted target (all scenarios passed).
3. A model where the second row truly cannot be arranged: the reproduction with a branch
   `shelf-full` (`when_related: {entity: depot.shelf.Item, where: shelf_id == input.shelf_id,
   exists: true}`, an error) declared before `placed`. It gets ESS-SYNTH-014 whose first line is
   ``refused: refusal[ESS-SYNTH-014]: outcome depot.shelf.PlaceItem/placed has a scenario `depot.shelf.PlaceItem/outcome/placed` without its order of `depot.shelf.ShelfItems` ``
   and whose reason line is
   `` `depot.shelf.ShelfItems` declares an order and this scenario can put 1 row(s) in it: a second `depot.shelf.Item` under one `shelf_id` is answered by `shelf-full` of `depot.shelf.PlaceItem` ``;
   it does not say "no declared move reaches it".
4. A bound guarded field counts as held when `known` (the steps `drive` receives) holds the step
   that created the row `bound[field]` names; such a row is reused. A bound field whose row no
   step of `known` created is still refused (`unarranged`), so `exists: false` is never sent at a
   row that is there. No new thread-local state (`every_static_of_the_crate_is_classified` stays
   green). The `bound.contains_key(field)` stop at `related_guard.rs:2672` keeps its meaning for
   the several-rows and subject cases.
5. The byte pins listed under Gate keep every row; a row that moves names #795 beside its re-pin.

## Scope

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs`, `drive` (`:2652-2672`): besides
  `subject_fact::held` (the `HELD` stack), treat `bound[field]` as held when a step of `known`
  created that row; no new thread-local state.
- `crates/verify/ess-conformance/src/synthesize.rs`:
  - `arrange_ranked` (`:12345`, `:12383`): record `OrderUnwitnessed` with `Refusal::beside`;
  - `impl Display for Refusal` (`:863`): an arm for a standing 014;
  - the 014 reason (`:2031-2040`) and `Unreachable` reporting (`:15411-15415`): carry the
    related-guard cause where `drive` refused.

## Owned files

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs`
- `crates/verify/ess-conformance/src/synthesize.rs` (the functions above only)
- `crates/verify/ess-conformance/tests/view_filter_identity.rs` (new `#[test]`s)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (rewritten by the stamp
  command under Gate)
- `CHANGELOG.md`

## Not yours

- `story:a-parameterised-view-is-read-in-every-scenario-that-reaches-its-rows` (draft): its Shape 3
  (`models/g2r-required-parent`) shows the same refusal text, and its Design asks for the same
  headline change. Its cause there is a bound parameter in `arrange_beside`, not the related guard.
  This unit owns the headline (`Refusal::beside` for 014); that draft drops its headline bullet and
  rebases on this unit. If a run of its Shape 3 shows a `when_related` guard on `FileCard`, it is
  this defect and the draft's Shape 3 acceptance moves here.
- `website/docs/reference/diagnostics.md:208` (the code's summary stays true).

## Tests first

In `crates/verify/ess-conformance/tests/view_filter_identity.rs` (existing target; it holds the
owner-linked ordered ledger and the 014 test at `:378`):

- `an_order_beside_a_missing_parent_refusal_is_checked` (acceptance 1, 2): the reproduction model
  inline, synthesize, assert the steps, run on the interpreted target.
- `an_order_that_cannot_hold_two_rows_stands_beside_its_scenario` (acceptance 3).

Each synthesizes one small model in process: under 5 s expected (bound 60 s per test, 120 s per
step). Time both before adding them; one over 5 s is named in the report.

## Gate

One `--test` at a time; no build-pool job over 10 minutes (name `SLOT_TIMEOUT=` up to 30m only if a
single `ess-conformance` test build needs it).

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test view_filter_identity`, then `--test synthesis`,
  `--test read_api_view_idioms` (they check 014)
- `cargo test -p ess-cli --locked` (synthesis in `ess-conformance`: the table adds `ess-cli`)
- byte pins: `ess-conformance` `--test synthesis_suite_bytes_table`,
  `--test synthesis_suite_bytes_table_cli_read`, `--test adversary_related_values_all_bytes`,
  `--test external_beside_held_guard`; `ess-synth` `--test served_codec_names`, `--test go`;
  `ess-openapi` `--test adversary_ess_issues_2_u6`; `ess-compiler` `--test
  string_alphabet_and_length`; `ess-cli` `--test tutorial_page`
- the sources stamp, rewritten in this change:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
  re-run after every rebase, never merge the stamp by hand
- the byte comparison, once per unit, its measured time named in the report:
  `SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table -- --ignored every_repository_model_keeps_its_synthesized_bytes`
- `task test-xtask` (the unit edits `CHANGELOG.md`)

## Depends on

Nothing. Lands before the draft `a-parameterised-view-is-read-in-every-scenario-that-reaches-its-rows`
(shared `Display` arm).

## Changelog

Under `Fixed`:

- `ess verify conform synthesize` checks the order of a view over rows created under an existing
  parent where the creating command refuses a missing parent (`when_related: … exists: false`): the
  second row goes under the same parent, where it went under a new one and ESS-SYNTH-014 said the
  outcome had no scenario. A 014 that stands now says the scenario stands without its order (#795).

## Fit review

1. Need: a ranked view per parent must be order-checked whenever the creating command can put two
   rows under one parent; a guard on the parent's existence must not cost the check. No syntax.
2. Class: defect. `website/docs/reference/diagnostics.md:208`: 014 is "the scenario cannot put two
   rows in it"; here it can (two `PlaceItem` to one existing shelf). The head "has no scenario"
   contradicts the suite.
3. Expressible: dropping the guard restores the check, which removes a checked refusal: no answer.
4. Fit: no surface; synthesis and refusal text.
5. Second adopter: any "add a line to an existing order" command guarded on the order existing.
6. Cost: suite bytes of models with the shape (none committed, as read); refusal text of 014.
7. Considered: change nothing; only fix the headline (rejected: the order stays unchecked); reuse
   the bound row in `drive` and stand the refusal beside the scenario (chosen).

## Decisions

- Decision: accept as proposed (both parts of the issue's expected behaviour).
