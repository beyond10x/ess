---
format: aep.planning-md/3
id: story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed
kind: story
status: draft
title: 'A moving affects: entry is witnessed when its rows'' creator has a guarded sibling branch'
tags:
- adopter-report
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set/copied.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_record_effects.rs
revision: 6
---
## Outcome

A moving `affects:` entry gets its scenario when the rows it moves are created by a command whose
creating branch sits below a guarded sibling (a `when:` refusal or a `when_related:` refusal). Today
synthesis refuses the outcome with `ESS-SYNTH-001` and a cause that names the wrong reason ("moved by
its entry only from the state the move arrives in … a target that skips the move reads the same")
and a help line about finite field types that does not apply.

## Fit review

Findings covered: downstream specification report of 2026-10-19, findings 1 and 2 (an `affects:`
cascade onto rows whose creator carries a guard; `--synthesis-seed` cannot stand in).

1. **Need.** Cancelling a parent moves every pending child row it selects (`affects: {entity:
   Item, where: feed_id == input.feed_id, moves: Item.discard}`). The children are created by
   `Post`, which first refuses an unknown parent. Minimal reproduction:
   `.engineering/repro/affects-creator-related-guard/system.yaml` (`when_related: {via:
   input.feed_id, exists: false}` refusal before `posted`), `when-only.yaml` (a `when: urgent ==
   true` refusal instead), `noguard.yaml` (control, no sibling). Observed on ess 0.60.0, 0.62.0 and
   an integration-branch build (0.62.0 + the 2026-10-13 port, `ess-0063-port` tree at `15db24d97`):
   `system.yaml` and `when-only.yaml` give `6 scenario(s), 1 refusal(s)`, `refused:
   refusal[ESS-SYNTH-001]: outcome cascade.feed.Cancel/cancelled has no scenario`;
   `noguard.yaml` gives `6 scenario(s), 0 refusal(s)`. The reporter saw it with a `when_related`
   guard only; the minimal model shows any guarded sibling triggers it. Requester's reading
   (theirs): "synthesizes when the creator has only `when:`" — not reproduced.
2. **Class.** Defect: `synthesize/set_effects.rs:1401-1406` documents that changed rows rest in a `from` state
   "wherever the arranging commands reach one", and the creator does reach `Pending` (its own
   `Post/outcome/posted` scenario exists in the same suite).
3. **Already expressible?** Yes; the model is complete. No idiom avoids it except deleting the
   creator's refusal.
4. **Fit.** Inferred from code, not traced: the symbolic-filter path
   (`Selection::row_meeting` → `arrange_toward_bound`,
   `crates/verify/ess-conformance/src/synthesize.rs:10837-10935`) draws creator inputs from
   `when(creator.outcome)` alone and keeps one only if `selects_branch(…, held: None, …)` is
   `Ok(true)` (`synthesize.rs:10889-10897`). A default branch below a `when:` sibling needs that
   sibling refuted (`plain_guards`, `synthesize.rs:7258-7263`), which the candidate list does not
   ask for; below a `when_related` sibling, `has_subject_guards` with no held state returns
   `Err(StrategyWithoutGuard)` (`synthesize.rs:7231-7238`). Every creator is skipped, `affect_rows`
   falls through to its last arm and prints the arrival-state message
   (`synthesize/set_effects.rs:1408-1444`). The fix reuses how the non-bound arrangement reaches the
   same branch (it does: `Post/outcome/posted` is witnessed). The refusal text should name the
   creator that could not be driven, not the arrival state.
5. **Second adopter.** Closing an account discards its open payment requests, created by a command
   that refuses an unknown account.
6. **Cost.** Synthesis only. Suites of affected models gain the outcome's scenario; no format, no
   diagnostic code change (the cause text changes).
7. **Alternatives.** (a) Change nothing: every cascade onto guarded-creator rows stays unwitnessed.
   (b) Seed the child rows (`--synthesis-seed`, finding 2): seeds admit the stored subject row only,
   by design (`docs/design/synthesis-seeds.md:87-93`; related and owned-parent seeds are out of
   scope, `:173`), and the rows are reachable through the declared creator, so a seed would restate
   the model. (c) Drive the guarded creator's default branch in the bound search (chosen).

## Decisions

Accept as a defect. Finding 2 is answered by this fix, not by widening seeds: seeds stay limited to
the stored subject row (`docs/design/synthesis-seeds.md`).

## Acceptance

- `a_cascade_onto_rows_of_a_guarded_creator_is_witnessed`: `system.yaml` and `when-only.yaml`
  from the reproduction synthesize with 0 refusals and a `Cancel/outcome/cancelled` scenario that
  passes `--target interpreted`, and fails a target that skips the move.
- `a_cascade_refusal_names_the_creator_it_could_not_drive`: where no creator can be driven at all
  (its creating branch unreachable), the cause names that creator, not the arrival state.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs` (`arrange_toward_bound`, ~10837) — inferred.
- `crates/verify/ess-conformance/src/synthesize/set_effects.rs` (`affect_rows` message, ~1427) — cited.
- Test: extend `crates/verify/ess-conformance/tests/related_record_effects.rs` (the `issue_229_*`
  model gains a refusal branch before `StartSession/started`, once `when:` and once `when_related`).

One unit, today.

## Second cause and help line

Finding 8, second cause in the same function: an `affects:` filter over a field the creator writes with `{related: {via, field}}` is not steered through the source row, because `arrange_toward_bound` maps a filter only through `sets: field: input.field` (`synthesize.rs` doc comment above `arrange_toward_bound`). Reproduction: `.engineering/repro/related-copy-read-later/system.yaml` (`CloseGroup/closed`, `ESS-SYNTH-001` with the same arrival-state message on all three binaries) against `control-input-key.yaml` (the same filter with the key taken as a creator input: witnessed). Reuse `synthesize/row_set/copied.rs::steered`, which the integration branch added for row-set guards over a copy (https://github.com/beyond10x/ess/issues/601). Test: a `{related:}`-copied key case in `tests/related_record_effects.rs`.

The `help:` line printed under this refusal ("give the field a type that has a finite value, or drop it from the command's input", `synthesize.rs:1210`, the `NoWitness` fallback) does not apply to a set-effect cause; give set-effect causes their own help.

## Further report

Source: public repository beyond10x/inference, main 8e72194, `ess/` (reported on 0.60.0 to 0.67.0; reproduced on 0.68.0 and on the 0.69.0 integration build, commit 52bd110dd, which still prints `ess 0.68.0`). `ess verify conform synthesize --path ess` refuses five moving `affects:` outcomes with ESS-SYNTH-001: `inference.duplex.CloseSpeechSession/closed` (duplex.yaml:1132), `FailSpeechSession/failed` (duplex.yaml:1166), `inference.graph.CancelRun/cancelled` (graph.yaml:1854), `inference.stream.CancelGeneration/cancelled` (stream.yaml:711), `SupersedeGeneration/superseded` (stream.yaml:681).

```
refused: refusal[ESS-SYNTH-001]: outcome inference.stream.CancelGeneration/cancelled has no scenario `inference.stream.CancelGeneration/outcome/cancelled`
  no witness: `cancelled.affects[0]` is `inference.stream.OutputSegment`, which is moved by its entry only from the state the move arrives in, as far as the arranging commands reach, and the entry writes no field, so a target that skips the move reads the same
```

The duplex pair prints "has no three rows the filter selects that the arranging commands can produce" instead (`crates/verify/ess-conformance/src/synthesize/set_effects.rs:800`, `NO_MATCHING`); the arrival-state message is `:1498`; both are reached from `Selection::row_meeting` (`:655`).

What it narrows, verified by probes on the 0.69.0 build: the `exists: false` sibling of this story's own reproduction no longer blocks (`affects-creator-related-guard/system.yaml` synthesizes 7 scenarios, 0 refusals, on 0.68.0 and 0.69.0). `when-only.yaml` and `related-copy-read-later/system.yaml` still refuse. The guard that blocks here is a `when_related` state predicate on the creator, over the related row the filter keys on (`EmitTextDelta/stale-generation: state != Active`, `ReceiveSessionEvent/session-not-open: state != Open`). Dropping only that sibling clears the two stream refusals and the two duplex refusals; keeping `exists: false` beside it does not matter.

`CancelRun/cancelled` needs more. With both state predicates dropped it still refuses `affects[0]` (Generation, created by `StartGeneration`, which reads two related rows, run and node). With `unknown-node` dropped as well, it refuses `affects[1]` (OutputSegment, whose `run_id` is written `{related: {via: input.generation_id, field: run_id}}`). That is this story's second cause, and probably also `story:a-command-with-two-related-selectors-arranges-both` (inferred).

Not the cause of `story:seeded-state-reaches-related-rows` (an `unsatisfied` ESS-SYNTH-003 with an `Unwitnessable` cascade) or of `story:an-existence-outcome-passes-a-related-list-membership-guard` (ESS-SYNTH-004 from the existence family's steered search): different function, different message.

Workaround in the source: authored scenarios under `ess/scenarios/` execute all five.

Added acceptance: a creator whose creating branch sits below a `when_related` state-predicate sibling over the filter's key (`state != Active`) gets its moving `affects:` scenario, and the inference CancelGeneration, SupersedeGeneration, CloseSpeechSession and FailSpeechSession outcomes synthesize and pass on `--target interpreted`.
