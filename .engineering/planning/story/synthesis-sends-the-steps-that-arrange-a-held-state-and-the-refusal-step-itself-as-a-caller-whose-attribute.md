---
format: aep.planning-md/3
id: story:synthesis-sends-the-steps-that-arrange-a-held-state-and-the-refusal-step-itself-as-a-caller-whose-attribute
kind: story
status: active
title: Synthesis sends the steps that arrange a held state, and the refusal step itself, as a caller whose attribute the command's `when:` caller guard admits for the arranged row, so ESS's interpreted target passes the held-state refusal scenario.
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#794
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T02:31:01Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T02:31:04Z", actor: "human:timo", revision: 4}
---
# d794: a state-arranging step is sent as a caller the input guard admits

Size: M (about 300 changed lines: 140 source, 160 test).
Fixed suite bytes of committed models: none expected. No committed model has a `when:` guard
comparing an input with `caller.<attr>`; the committed `caller.` guards are `when_subject:` over a
stored field (`crates/generate/ess-synth/tests/fixtures/declared-behaviour/domains/ticket.yaml:213`,
`crates/verify/ess-conformance/tests/fixtures/{one-time-followup/caller-guard.yaml,
adversary-0740/*/d.yaml, caller-model-seed/attr/d.yaml}`). A model with a literal identity guard
(`<id> != "x"`) on a wrong-state command or an arranging move could move; the byte pins decide.
Patch-release candidate: no by content (synthesis changes suites of every model with the shape);
the operator ordered it built now in the hotfix wave on the released tag (2026-11-24, 03:10 CET).

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
not written (`Note::CrossCallerUnwitnessed`) and another reading (all steps as one caller) is
kept; if none remains, the scenario is refused as ESS-SYNTH-003 naming the guard, never written to
fail.

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
4. A model whose guard makes the arranging step and the refusal step need two different bay
   identities under the Cross reading (the reproduction is one) writes no scenario that sends two
   callers for one row: `merge_cross` records `Note::CrossCallerUnwitnessed` (`caller.rs:857`) for
   `depot.bay.Bay/state/Closed/refuses/depot.bay.CloseBay` naming the guard `not-holder`, and keeps
   the all-first-caller reading, which passes. If no reading remains, synthesis records
   ESS-SYNTH-003 for that scenario id naming `not-holder`; it never writes a scenario that fails on
   the interpreted target.
5. The swapped-callers run of that scenario (`caller.rs:877-878`) is asserted too: either it is
   written and passes, or `Note::CrossCallerUnswapped` (`caller.rs:819`) names it.
6. The byte pins listed under Gate keep every row; a row that moves names #794 beside its re-pin.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs`:
  - `refusal_arrangement` (`:18756`) and `advance`/`invoke` (`:8556`, `:9064`): steer the arranged
    identity through `identity_sent`/`identity_steered` (or `identity_kept`, `:7557`) as
    `arranged_plain` does, so `supply` cannot break a held input guard;
  - `refused_here` (`:17142`): check the refusal step's input against the acting caller's guards.
- `crates/verify/ess-conformance/src/synthesize/caller.rs`, `merge_cross` (`:843`, `:866-899`):
  drop a Cross reading whose arranging and acting steps cannot both hold, record
  `Note::CrossCallerUnwitnessed`, keep the all-first reading. `merge_cross` is the function the
  landed `story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan`
  made; its fixed plan of readings is kept, only the keep/drop rule for a Cross reading changes.

## Owned files

- `crates/verify/ess-conformance/src/synthesize.rs` (the functions above)
- `crates/verify/ess-conformance/src/synthesize/caller.rs` (`merge_cross`)
- `crates/verify/ess-conformance/tests/identity_guard_on_the_row_sent.rs` (new `#[test]`s)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (rewritten by the stamp
  command under Gate)
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
- `a_cross_reading_that_no_row_satisfies_is_noted_and_the_first_caller_reading_kept`
  (acceptance 4, 5): asserts the note, its scenario id and guard, and the swapped run.

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
  string_alphabet_and_length`; `ess-cli` `--test tutorial_page`
- the sources stamp, rewritten in this change:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
  re-run after every rebase, never merge the stamp by hand
- the byte comparison, once per unit, its measured time named in the report:
  `SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table -- --ignored every_repository_model_keeps_its_synthesized_bytes`
- `task test-xtask` (the unit edits `CHANGELOG.md`)

## Depends on

Overlaps the active, not yet built `story:wrong-state-branch-beside-a-subject-guard-is-witnessed`:
both edit `refused_here` (`synthesize.rs:17142`) and `refusal_arrangement` (`:18756`), and that
story also edits `wrong_state_scenario` (`:17039`). That story's Decisions rest on
`refusal_arrangement` answering an unreachable state with `RefusalCause::InstanceRequired`
(ESS-SYNTH-004); this unit keeps that answer unchanged and only steers the identity of a state it
does reach. Landing order: this unit first (it is built now in the hotfix wave on the released
tag); the wrong-state story rebases on it when the one pull request of open issues is built. Both
re-pin `synthesis-suite-bytes-table.tsv` rows with a wrong-state scenario: the second to land
re-synthesizes those rows on the merged tree, never merges them by hand.

Also touches `synthesize.rs` like d795 and d804; different functions (see `MAP.md`).

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
