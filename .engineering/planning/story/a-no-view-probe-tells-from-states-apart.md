---
format: aep.planning-md/3
id: story:a-no-view-probe-tells-from-states-apart
kind: story
status: implemented
title: The no-view arranged probe tells apart the states its branch runs from
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_d357_u3_pass1.rs
- confidence: cited
  path: docs/design/outcome-shapes.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T07:12:05Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-11-02T07:12:06Z", actor: "human:timo", revision: 8}
- {from: "active", to: "implemented", at: "2026-11-02T14:24:27Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":3,"review_outcome":2}}}
---
## Finding

Found by adversary pass 1 of wave D357 U3 (`review-result:adversary-d357-u3-pass-1`, F1; the claim was narrowed at 1962a5ff0 instead): the no-view probe of an arranged input-refusal half sends only the addressing branch's valid input, so a target that moves the record on refusal into another state that branch also runs from passes, even when another command's wrong-state branch would show the move. The red case (a `Staged` state from which `configure` also runs, a target swapping `New` and `Staged` on refusal) is kept by the wave lead for this story.

## Acceptance

- Where the probing branch runs from several states, the arranged half also sends a branch that tells the arranged state apart from the others (a wrong-state branch of another command), or the scenario is refused with a cause naming the states it cannot tell apart.
- The red case passes for the correct target and fails for the swapping target; the narrowed sentences in `existence.rs`, `docs/design/outcome-shapes.md` and CHANGELOG are widened again only as far as the new probe reaches.
- Committed models keep their suite bytes unless a model has such a branch, which is named.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave D368 on 6437975cb. Every line is **cited** (read from the story or the tree) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/existence.rs`, the no-view arranged half of an input refusal — cited (the story, `review-result:adversary-d357-u3-pass-1` F1, commits 1188481a0 and 1962a5ff0)
- **Symbols:** `refusals_on_an_arranged_row` :351 (narrowed doc sentence :329-350); `arranged_refusal` :399 (narrowed comment :434-441, `probe` call :450, probe steps appended :505-508); `addresses` :515, `runs_from` :524, `probe` :540-587 (the filter :546-555 looks only at the same command's branches); `UNPROBED` :591 — cited
- **Defect site:** `probe` sends the first addressing branch of `command` that runs from the arranged state and has a valid send, never another command's wrong-state branch, so a swap between two states that branch runs from answers the same branch — cited (:540-587, F1)
- **Also likely:** `crates/verify/ess-conformance/src/synthesize.rs` helpers `invoke_with` :7600, `reach` :8290, `admits_held_state` :7982, `Setup` :6437; a new refusal reason as a second `WitnessGap` reason beside `UNPROBED` — inferred
- **Documents:** `docs/design/outcome-shapes.md` § Conformance :283-292 (narrowed sentence :287-288) — cited; `CHANGELOG.md` gets a new `[Unreleased]` line (the narrowed line :10 is under released `[0.82.0]`) — cited headings, inferred treatment
- **Tests that pin today:** `tests/adversary_d357_u3_pass1.rs` (`STAGED` model :30, `Mode` :150, refusal arm :249, `failures` :295, controls :322, :334, F2 case :363); `tests/adversary_arrangement_pass2.rs` :526, :548, :590; `tests/adversary_arrangement_pass1.rs`; `tests/external_beside_held_guard.rs` — cited
- **Test changes:** the saved red case (`Mode::SwapsFromStateOnRefusal`, the `(mode, state)` match at :249, `no_view_arranged_half_catches_a_target_changing_the_from_state_on_refusal`) into `adversary_d357_u3_pass1.rs` — cited; a case for the "refused naming the states" branch — inferred
- **Pinned bytes:** `tests/fixtures/synthesis-suite-bytes-table.tsv`, `synthesis-component-suites-table.tsv` and `suites/generated/*/suite.json` should not move (1188481a0 moved none; no pinned model reaches `probe`) — inferred
- **Confidence:** high for where it lands; medium for the shape (which cross-command branch, whether a new refusal reason is needed)
- **Would collide with:** `existence.rs` :329-595 and the adjacent `held_state_refusals` :648 (story:a-no-view-held-state-refusal-probes-the-record), `outcome-shapes.md` § Conformance, the `CHANGELOG.md` `[Unreleased]` head, `tests/adversary_d357_u3_pass1.rs`

## Delivered

D368, 2026-11-02, with `story:a-no-view-held-state-refusal-probes-the-record` as one unit: the arranged half of a no-view input refusal tells the arranged state apart from every other state, through the shortest sequence of sends that a bounded search finds, extended one send at a time past it. A `wrong_state:` refusal counts among the answers. Where no sequence tells every state apart, it is withdrawn as `ESS-SYNTH-001` naming the states left. Unit e266be529, merged 550377957.

Fix round 1 (c5eddc4a2, merged 7cdd82c68) answered both review passes:
- the 64-try search no longer withdraws scenarios that a longer sequence keeps;
- a valid send that tells nothing apart is refused naming the states, not as "no valid send";
- a plain-typed input names a record only where a branch uses it as one.

The red D357 swap case passes.
