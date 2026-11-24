---
format: aep.planning-md/3
id: story:synthesis-sends-the-steps-that-arrange-a-held-state-and-the-refusal-step-itself-as-a-caller-whose-attribute
kind: story
status: draft
title: Synthesis sends the steps that arrange a held state, and the refusal step itself, as a caller whose attribute the command's `when:` caller guard admits for the arranged row, so ESS's interpreted target passes the held-state refusal scenario.
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#794
relations:
- serves: vision:O2
revision: 1
---
# d794: a state-arranging step is sent as a caller the input guard admits

Size: M (about 300 changed lines: 140 source, 160 test).
Fixed suite bytes of committed models: none expected. No committed model has a `when:` guard
comparing an input with `caller.<attr>`; the committed `caller.` guards are `when_subject:` over a
stored field (`crates/generate/ess-synth/tests/fixtures/declared-behaviour/domains/ticket.yaml:213`,
`crates/verify/ess-conformance/tests/fixtures/{one-time-followup/caller-guard.yaml,
adversary-0740/*/d.yaml, caller-model-seed/attr/d.yaml}`). A model with a literal identity guard
(`<id> != "x"`) on a wrong-state command or an arranging move could move; the byte pins decide.
Patch-release candidate: no (synthesis changes suites of every model with the shape).

## Need

beyond10x/ess#794. When a command refuses with an input guard over a caller attribute
(`when: bay_id != caller.bay_id`), the synthesized held-state refusal scenario arranges the
subject's state by sending that command as a caller whose attribute does not match the row, so
the arranging step is answered by the guard's refusal and ESS's own interpreted target fails the
scenario. The refusal step has the same fault.

## Reproduction

Reproduced on 0.98.0. Model `repro/794/` (the issue's model, `format: ess/37`): entity `Bay`
(`Open` → `Closed` by `close`); actor `Holder` with attribute `bay_id`; `CloseBay(bay_id)` with
`not-holder` (`when: bay_id != caller.bay_id`), `closed` (`moves: Bay.close`), `wrong-state`.

```console
$ ess verify conform synthesize --path . --out suite.json
5 scenario(s) (0 authored), 1 refusal(s), written to suite.json
$ ess verify conform run --path . --suite suite.json --target interpreted --report-format 2 --report-out report.json
  failed depot.bay.Bay/state/Closed/refuses/depot.bay.CloseBay
  5 scenarios: 4 passed, 1 failed, 0 error, 0 unsupported
```

Steps of the failing scenario (command, caller `bay_id` or expected outcome, input `bay_id`):

```
["depot.bay.OpenBay",null,"00000000-0000-4000-8000-924d17c0f0ed"]
["depot.bay.CloseBay","00000000-0000-4000-8000-a590987ebfbb","bay"]   expected `closed`
["depot.bay.CloseBay","00000000-0000-4000-8000-a490987ebe08","bay"]   expected `wrong-state`
```

## Outcome

In every scenario synthesis writes, a step expected to take a branch is sent with an input and a
caller under which no earlier-answering input refusal holds. For this model the held-state
refusal scenario creates the bay under the identity the arranging caller's `bay_id` names (or sends
a caller whose `bay_id` is the bay's), and sends the refusal step as a caller the guard admits for
that bay; the scenario passes on the interpreted target, and so does its swapped-callers run.
Where no caller reading can satisfy both the arranging step and the refusal step, that reading is
not written and another reading (all steps as one caller) is kept; if none remains, the scenario
is refused naming the guard, never written to fail.

## Cause (read in the source, not run)

Paths under `crates/verify/ess-conformance/src/`.

- The scenario comes from `lifecycle` (`synthesize.rs:16909`) → `wrong_state_scenario` (`:17039`) →
  `refused_here` (`:17142`); the bay and input from `refusal_arrangement` (`:18756`) → `arrange`
  (`:8480`) and `reach` (`:10471`, called at `:18804`); the arranging steps from `advance`
  (`:8556`) → `invoke` (`:8611`, defined `:9064`), where `supply` (`:9941`) overwrites the identity
  input with the captured row.
- Callers: `Callers::of` (`synthesize/caller.rs:242`) draws two values and rewrites
  `caller.bay_id` into a literal per reading (`written`, `:1166`); `InvocationModels::mark`
  (`:205`, `:1076`) stamps arranging steps first, acting steps second. This scenario is the
  `Reading::Cross` (`caller/readings.rs:136-155`) kept by `merge_cross` (`caller.rs:843`,
  `:866-899`); the swapped run is appended at `caller.rs:877-878` → `append_independent_about`
  (`:1384`).
- The path that works, `CloseBay/outcome/closed` → `arranged_plain` (`synthesize.rs:7358`) →
  `identity_sent` (`:7374`, defined `:7135`) → `sent_keeps_branch` (`:7043`) and
  `identity_steered` (`:7069`), re-creates the row under an identity that keeps the chosen
  branch's guard decided. Neither `refusal_arrangement` nor `advance`/`invoke` calls it, so
  `supply` replaces the guard-satisfying identity with a row nobody checked.
- In the Cross reading the arranging step (first caller) and the refusal step (second caller)
  need the bay's identity to equal two different values; no row satisfies both, and
  `refused_here` never checks the acting input against the acting caller's guards.

## Acceptance

1. Red today: the suite of `repro/794/` passes on the interpreted target: 5 or more scenarios, 0
   failed, `depot.bay.Bay/state/Closed/refuses/depot.bay.CloseBay` passed.
2. In that scenario every `CloseBay` step expected `closed` or `wrong-state` is sent with a caller
   whose `bay_id` equals the identity of the bay its input names.
3. The variant with the guard over a stored field (`holder` set by `OpenBay`,
   `when_subject: {predicate: holder != caller.bay_id}`) keeps its suite bytes (10 scenarios, all
   passing, per the issue).
4. A step expected to take a branch that an earlier-answering input refusal would answer is not
   written: a synthesis-time check over every written step of `refused_here` and the arranging
   route (or a test-only oracle over the whole suite of the reproduction) finds none.
5. The byte pins listed under Gate keep every row; a row that moves names #794 beside its re-pin.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs`:
  - `refusal_arrangement` (`:18756`) and `advance`/`invoke` (`:8556`, `:9064`): steer the arranged
    identity through `identity_sent`/`identity_steered` (or `identity_kept`, `:7557`) as
    `arranged_plain` does, so `supply` cannot break a held input guard;
  - `refused_here` (`:17142`): check the refusal step's input against the acting caller's guards.
- `crates/verify/ess-conformance/src/synthesize/caller.rs`, `merge_cross` (`:866-899`): drop a
  Cross reading whose arranging and acting steps cannot both hold, keep the all-first reading.

## Owned files

- `crates/verify/ess-conformance/src/synthesize.rs` (the functions above)
- `crates/verify/ess-conformance/src/synthesize/caller.rs` (`merge_cross`)
- `crates/verify/ess-conformance/tests/identity_guard_on_the_row_sent.rs` (new `#[test]`s)
- `CHANGELOG.md`

## Not yours

- The reproduction's other refusal, ESS-SYNTH-001 for `depot.bay.CloseBay/outcome/wrong-state`
  ("`bay_id` … has too few values inside the guards the input meets"). The issue does not report
  it. If this unit's steering removes it, record that in the report; otherwise it is a separate
  defect.
- `story:an-identity-guard-is-decided-on-every-row-synthesis-arranges` (draft): the follow-up paths
  in `row_set::prepare`, `existence.rs`, `subject_fact::refusal_witness` and minted identities. It
  names neither `refused_here`/`refusal_arrangement` nor `advance`→`invoke` and has no caller
  reading. Land this unit first; that draft rebases on the steering this unit adds.
- `story:a-command-acts-on-the-row-a-caller-attribute-names` (active): `instance: caller.<attr>`
  validation, a different construct.

## Tests first

In `crates/verify/ess-conformance/tests/identity_guard_on_the_row_sent.rs` (existing target; inline
models run on the interpreted target):

- `a_caller_guarded_wrong_state_scenario_arranges_as_an_admitted_caller` (acceptance 1, 2).
- `a_cross_reading_that_no_row_satisfies_is_not_written` (acceptance 4).

Each synthesizes and runs one small model: under 5 s expected (bound 60 s per test, 120 s per
step). Time them; one over 5 s is named in the report.

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test identity_guard_on_the_row_sent`, then
  `--test caller_swap_caller_bound_selected_input`
- `cargo test -p ess-cli --locked`
- byte pins: `ess-conformance` `--test synthesis_suite_bytes_table`,
  `--test synthesis_suite_bytes_table_cli_read`, `--test adversary_related_values_all_bytes`,
  `--test external_beside_held_guard`; `ess-synth` `--test served_codec_names`, `--test go`;
  `ess-openapi` `--test adversary_ess_issues_2_u6`; `ess-compiler` `--test
  string_alphabet_and_length`; `ess-cli` `--test tutorial_page`; and `task test-slow` for the
  byte comparison of `synthesis_suite_bytes_table`.

## Depends on

Nothing. Touches `synthesize.rs` like d795 and d809a/d804; different functions (see `MAP.md`).

## Changelog

Under `Fixed`:

- `ess verify conform synthesize` sends the steps that arrange a held state, and the refusal step,
  as a caller whose attribute the command's `when:` guard over `caller.<attr>` admits for the
  arranged row, where it sent another caller and the interpreted target failed the scenario (#794).

## Fit review

1. Need: a synthesized suite must pass on ESS's own interpreted target; a caller guard over an
   input breaks that. No syntax proposed.
2. Class: defect. `website/docs/guides/specify/values-and-views.md:295`: "Synthesis uses two
   callers: the refusal is sent by one caller on the other's note"; here the arranging act is sent
   by a caller the model refuses, and `ESS-SYNTH-019`'s row in `diagnostics.md:213` calls such a
   step a defect in ess.
3. Expressible: moving the guard onto a stored field works (the issue's variant), but changes the
   model's rule; no answer.
4. Fit: no surface; synthesis only.
5. Second adopter: any "only the holder may act" guard over an input identity (a seat, a locker, a
   document the caller owns).
6. Cost: suites of models with the shape.
7. Considered: change nothing; refuse the scenario (rejected: an admitted caller exists); steer
   the arranged identity and drop the impossible Cross reading (chosen).

## Decisions

- Decision: accept as proposed.
