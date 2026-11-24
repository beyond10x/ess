---
format: aep.planning-md/3
id: story:synthesis-reads-a-view-whose-filter-orders-a-minted-timestamp-against-now-by-observing-the-minted-value-and
kind: story
status: active
title: Synthesis reads a view whose filter orders a minted `Timestamp` against `now`, by observing the minted value and holding the read's clock on each side of it, instead of skipping the read with ESS-SYNTH-005.
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#819
relations:
- serves: vision:O2
- depends_on: story:a-conformance-suite-s-view-read-may-hold-the-target-s-clock-at-an-instant-set-from-an-observed-value-and-every
- depends_on: story:state-param-binds-from-post-command-state
- depends_on: story:synthesis-arranges-the-related-rows-a-guard-or-read-needs
- depends_on: story:the-suite-pair-ess-conformance-66-and-67-is-open-read-wherever-64-and-65-are-and-written-by-nothing-yet
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:25:12Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-24T03:25:14Z", actor: "human:timo", revision: 5}
---
# d819 part 2: a view ordering a minted Timestamp against now is read

format: a suite format, at the suite pair the one pull request opens once (DF-D6 A and D77); no source format.

Part 2 of 2 for beyond10x/ess#819: synthesis writes the held-clock view read that part 1,
`d819-a-view-read-step-holds-the-read-clock`, adds to the suite and every runner. Split by layer
(C20): suite step and runners there, synthesis here.

Size: M (about 320 changed lines: 110 synthesis, 20 diagnostics text and docs, 190 test).
Fixed suite bytes of committed models: only models with a view filter ordering a minted
`Timestamp` against `now`, whose reads are skipped today (ESS-SYNTH-005); the suite of such a model
gains reads and moves to the suite version of part 1 (`ess-conformance/66`/`/67`). Every other suite keeps its version and bytes. No committed model is
known to have the shape; the byte comparison under Gate decides, and a moved row names #819 beside
its re-pin.
Patch-release candidate: no.

## Need

beyond10x/ess#819 (trial finding; same on 0.90.0, not a regression). A service sets a due date from
its own clock and serves "the rows whose due date has passed". Modelled as the code reads it (the
due date `{generated: true, present: true}`, the view `filter: {all: [state == OnLoan, due_at <
now]}`), synthesis cannot decide the filter and skips the view's read in every scenario that reaches
the row, with ESS-SYNTH-005. Its hint (filter on state alone) changes the rule the service
implements. The view's rule is then never checked.

## Reproduction

Reproduced on 0.98.0. Model `repro/819/` (`format: ess/37`): `Tool {member, due_at:
Optional<Timestamp>}`, `Available` → `OnLoan` by `lend`; `LendTool/lent` sets `due_at: {generated:
true, present: true}`; view `Tools` publishes `due_at`; view `OverdueLoans` filters
`{all: [state == OnLoan, due_at < now]}`.

```console
$ ess verify conform synthesize --path repro/819 --out suite.json
refused: refusal[ESS-SYNTH-005]: outcome shed.lending.LendTool/lent has a scenario `shed.lending.LendTool/outcome/lent` that skips its read of `shed.lending.OverdueLoans`
  `shed.lending.OverdueLoans` filters on `(state == OnLoan and due_at < now)`, which is undecided for an entity in `OnLoan`
    - `due_at` is bound by nothing a scenario knows
  help: filter the view on the entity's state, which is what a generated scenario knows after the command it ran
(the same for the `lend` transition's scenario)
6 scenario(s) (0 authored), 2 refusal(s), written to suite.json
```

## Outcome (DF-D6 A)

Where a minted `Timestamp` is published by a view or an event the scenario reads, synthesis binds
it as an observed value and writes the `now`-filtered view read twice with part 1's held clock:
once at an instant past the observed value (the row is shown) and once before it (the row is not).
Where nothing publishes the minted value, ESS-SYNTH-005 stays, and its hint says to publish the
field in a view, not to filter on state alone.

## Where it lands (read in the source, not run)

Paths under `crates/verify/ess-conformance/src/`, lines at `41dd4aa9e`.

- ESS-SYNTH-005 is `RefusalCause::ViewUndecidable` (`synthesize.rs:1014`, code at 1823, message
  2006-2030, "bound by nothing a scenario knows" at 2027), raised at 11960-11990 when `shows_row`
  (16564) fails through `row_truth_with` (16600).
- A field compared with `now` is decided only when what synthesis set there is a
  `ScenarioValue::NowOffset` (16637-16671); a minted value never enters what synthesis set.
- `ScenarioValue::Observed {event, field}` (`scenario.rs:1965-1970`) is resolved by the runner from
  events seen (`runner.rs:3426-3436`); part 1 lets a view read's held clock use it.

## Acceptance

Suite pair (D77, 2026-11-24): this story's suite vocabulary is written at `ess-conformance/66` (ordinary) and `/67` (coverage), opened by story:the-suite-pair-ess-conformance-66-and-67-is-open-read-wherever-64-and-65-are-and-written-by-nothing-yet; where this text names `ess-conformance/64` and `/65`, "the new suite pair", "the next suite pair" or a pair "after `/63`" as the pair this story writes, read `/66` and `/67` (`/64` and `/65` opened in 0.93.0 and are written since 0.95.0).

1. Red today: synthesizing `repro/819/` writes no ESS-SYNTH-005; the `lent` scenario and the `lend`
   transition's scenario each read `OverdueLoans` once with the read clock held past the observed
   `due_at` (row expected) and once before it (row not expected).
2. That suite passes on the interpreted target, and an interpreted variant that ignores `due_at`
   in the filter fails it.
3. With `due_at` published by no view and no event, ESS-SYNTH-005 stays and its hint names
   publishing the field.
4. A suite without such a read keeps its suite version and bytes (the byte pins).
5. `website/docs/reference/diagnostics.md:199` says when ESS-SYNTH-005 remains.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs`: `shows_row` (16564) and `row_truth_with`
  (16600) decide a `now` comparison against an observed minted value; the view-read writer (around
  11960-11990) writes the two held reads instead of skipping; the `ViewUndecidable` help (2006-2030)
  gains the "publish the field" clause.
- `website/docs/reference/diagnostics.md`, `CHANGELOG.md`.

## Owned files

- `crates/verify/ess-conformance/src/synthesize.rs` (`shows_row`, `row_truth_with`, the view-read
  writer, the `ViewUndecidable` help)
- `crates/verify/ess-conformance/tests/now_view_filter.rs` (new `#[test]`s)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (rewritten by the stamp
  command under Gate)
- `website/docs/reference/diagnostics.md`, `CHANGELOG.md`

## Not yours

- The suite step, the runners, the interpreter's read instant, the suite version: part 1.
- A source spelling tying a minted instant to `now` plus an input offset (DF-D6 B).
- Any change to `{generated: true}` semantics.
- A view field computed from a clock read at query time: the active
  `story:a-view-field-computes-from-now-and-from-a-view-parameter` holds that out as "a later
  story"; this unit reads a filter, not a field, and does not build it.

## Tests first

`crates/verify/ess-conformance/tests/now_view_filter.rs` (existing):
`a_minted_due_date_is_read_on_both_sides_of_now` (acceptance 1, 2) and
`an_unpublished_minted_instant_keeps_its_refusal` (acceptance 3). Each under 5 s expected (bound
60 s per test, 120 s per step); one over 5 s is named in the report.

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`; `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test now_view_filter`, then `--test current_time_guard`
- `cargo test -p ess-cli --locked`
- `task test-xtask` (diagnostic text, docs, `CHANGELOG.md`)
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

## Depends on

- DF-D6 (A).
- Part 1, `d819-a-view-read-step-holds-the-read-clock` (the step this unit writes).
- `story:state-param-binds-from-post-command-state` (active): edits the `ViewUndecidable` help and
  `bound()`. It edits the 005 hint first; this unit adds its clause to the hint that story leaves.
- `story:synthesis-arranges-the-related-rows-a-guard-or-read-needs` (active): edits the fields
  `shows_row` is asked with. It lands first; this unit rebases its `shows_row` change on it.
- `story:a-view-field-computes-from-now-and-from-a-view-parameter` (active): no file in common
  named; ordered only so its "later story" is this one (as "Not yours").

## Changelog

Under `Added`:

- `ess verify conform synthesize` reads a view whose filter orders a minted `Timestamp` against
  `now` on both sides of the minted value it observed, with the read's clock held, where such reads
  were skipped with ESS-SYNTH-005 (#819).

## Fit review

1. Need: check a "past due" view whose due date the implementation mints. Requester's suggestions,
   theirs: read the minted value back and hold the clock past it; or a spelling that ties a minted
   instant to `now` plus an input offset.
2. Class: gap. The filter is admitted from `ess/30`, and synthesis cannot read it for a minted
   field; ESS-SYNTH-005's documented hint (`diagnostics.md:199`) changes the rule.
3. Expressible today: the issue's workaround is to keep the filter and accept the refusal; the read
   stays unchecked (no idiom checks it).
4. Fit: reuses `Observed` values and the `hold_now` clock; extends the clock hold from command steps
   to view reads, a cell `story:a-suite-fixes-the-targets-clock` already planned.
5. Second adopter: expiring sign-ins, overdue invoices, stale reservations, all minted by the
   service's clock.
6. Cost: one suite version (A, part 1), runners in three languages (part 1); no source format.
7. Considered: change nothing (only a better hint); source spelling `due_at: {expr: now + …}` (B, a
   source format, and `now` as a value is refused at `ess/27`, `spec-versions.md:70`); suite-side
   observation and held read clock (A, chosen: it checks the model as written); one unit (rejected:
   over 600 lines, three runners beside synthesis).

## Decisions

- Decision: accept, redesigned (the requester's first suggestion, in suite form), split in two.
- DF-D6: A (default) suite-side held read clock around an observed minted value, new suite version;
  B source spelling tying a minted instant to `now` plus an input offset, new source format; C keep
  the refusal and change only its hint. Taken: A (D77).
