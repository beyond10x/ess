---
format: aep.planning-md/3
id: story:synthesis-reads-a-view-whose-filter-orders-a-minted-timestamp-against-now-by-observing-the-minted-value-and
kind: story
status: draft
title: Synthesis reads a view whose filter orders a minted `Timestamp` against `now`, by observing the minted value and holding the read's clock on each side of it, instead of skipping the read with ESS-SYNTH-005.
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#819
relations:
- serves: vision:O2
revision: 1
---
# d819: a view ordering a minted Timestamp against now is read

format: decided before acceptance (DF-D6; default A needs a new suite format version, no source
format).

Size: L (about 550 changed lines: suite step and runner, synthesis, Go and TypeScript runners,
docs, tests). If the critics find it over 600, split: (1) the suite step and the runners, (2)
synthesis writing it.
Fixed suite bytes of committed models: only models with a view filter ordering a minted
`Timestamp` against `now`, whose reads are skipped today (ESS-SYNTH-005); the suite of such a model
gains reads and moves to the new suite version. Every other suite keeps its version and bytes.
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
$ ess verify conform synthesize --path . --out suite.json
refused: refusal[ESS-SYNTH-005]: outcome shed.lending.LendTool/lent has a scenario `shed.lending.LendTool/outcome/lent` that skips its read of `shed.lending.OverdueLoans`
  `shed.lending.OverdueLoans` filters on `(state == OnLoan and due_at < now)`, which is undecided for an entity in `OnLoan`
    - `due_at` is bound by nothing a scenario knows
  help: filter the view on the entity's state, which is what a generated scenario knows after the command it ran
(the same for the `lend` transition's scenario)
6 scenario(s) (0 authored), 2 refusal(s), written to suite.json
```

## Outcome (DF-D6 A)

Where a minted `Timestamp` is published by a view or an event the scenario reads, synthesis binds
it as an observed value and reads the `now`-filtered view twice with the read's clock held: once at
an instant past the observed value (the row is shown) and once before it (the row is not). The suite
carries the held read clock in a new suite version; a suite without it keeps its version. Where
nothing publishes the minted value, ESS-SYNTH-005 stays, and its hint says to publish the field in
a view, not to filter on state alone.

## Where it lands (read in the source, not run)

Paths under `crates/verify/ess-conformance/src/`.

- ESS-SYNTH-005 is `RefusalCause::ViewUndecidable` (`synthesize.rs:1014`, code at 1823, message
  2006-2030, "bound by nothing a scenario knows" at 2027), raised at 11960-11990 when `shows_row`
  (16564) fails through `row_truth_with` (16600).
- A field compared with `now` is decided only when what synthesis set there is a
  `ScenarioValue::NowOffset` (16637-16671); a minted value never enters what synthesis set.
- `ScenarioValue::Observed {event, field}` exists (`scenario.rs:1965-1970`) and the runner resolves
  it from events seen (`runner.rs:3426-3436`), but only for a later command input.
- `hold_now.rs` (1-13, 49-50) holds the clock right before an `execute_command`; nothing holds a
  view read's clock. `hold_now` took suite/50 and /51 (`hold_now.rs:18-25`): the precedent for a
  suite pair bump. `story:a-suite-fixes-the-targets-clock` (implemented, #586) named a request clock
  for views in its plan; only command steps were built.

## Acceptance

1. Red today: synthesizing `repro/819/` writes no ESS-SYNTH-005; the `lent` scenario and the `lend`
   transition's scenario each read `OverdueLoans` once with the read clock held past the observed
   `due_at` (row expected) and once before it (row not expected).
2. That suite passes on the interpreted target, and an interpreted variant that ignores `due_at`
   in the filter fails it (the read checks the rule).
3. With `due_at` published by no view and no event, ESS-SYNTH-005 stays and its hint names
   publishing the field.
4. A suite without such a read keeps its suite version and bytes (the byte pins).
5. The Go and TypeScript runners execute the held read clock, or refuse the new suite version by
   name.
6. `website/docs/reference/formats.md` (suite rows) and `website/docs/reference/diagnostics.md:199`
   say what the new step does and when ESS-SYNTH-005 remains.

## Scope

`synthesize.rs` (`shows_row`, `row_truth_with`, the view-read writer), `scenario.rs` (a held clock
on a view read), `runner.rs` and the interpreter's view read, `hold_now.rs` or a sibling, the Go and
TypeScript runners' view read, the suite version bookkeeping, docs.

## Owned files

Named by the implementor after DF-D6; the files above are the expected set.

## Not yours

- A source spelling tying a minted instant to `now` plus an input offset (DF-D6 B).
- Any change to `{generated: true}` semantics.

## Tests first

`crates/verify/ess-conformance/tests/now_view_filter.rs` (existing): `a_minted_due_date_is_read_on_both_sides_of_now`
(acceptance 1, 2) and `an_unpublished_minted_instant_keeps_its_refusal` (acceptance 3). For the
clock hold, `tests/current_time_guard.rs` holds the command-step precedent. Each under 5 s expected
(bound 60 s per test, 120 s per step).

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`; `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test now_view_filter`, then `--test current_time_guard`
- `cargo test -p ess-cli --locked`
- `task test-xtask` (format version, docs, changelog)
- byte pins: `ess-conformance` `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes`, `external_beside_held_guard`; `ess-synth`
  `served_codec_names`, `go`; `ess-openapi` `adversary_ess_issues_2_u6`; `ess-compiler`
  `string_alphabet_and_length`; `ess-cli` `tutorial_page`; `task test-slow`

## Depends on

DF-D6.

## Changelog

Under `Added`:

- `ess verify conform synthesize` reads a view whose filter orders a minted `Timestamp` against
  `now` on both sides of the minted value it observed, with the read's clock held, in suite
  version N; such reads were skipped with ESS-SYNTH-005 (#819).

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
6. Cost: one suite version (A), runners in three languages; no source format.
7. Considered: change nothing (only a better hint); source spelling `due_at: {expr: now + …}` (B, a
   source format, and `now` as a value is refused at `ess/27`, `spec-versions.md:70`); suite-side
   observation and held read clock (A, chosen: it checks the model as written).

## Decisions

- Decision: accept, redesigned (the requester's first suggestion, in suite form).
- DF-D6: A (default) suite-side held read clock around an observed minted value, new suite version;
  B source spelling tying a minted instant to `now` plus an input offset, new source format; C keep
  the refusal and change only its hint.
