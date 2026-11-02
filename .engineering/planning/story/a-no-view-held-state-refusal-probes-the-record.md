---
format: aep.planning-md/3
id: story:a-no-view-held-state-refusal-probes-the-record
kind: story
status: implemented
title: A no-view held-state refusal probes the record through a command
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/a_no_view_held_state_refusal_probes_the_record.rs
- confidence: cited
  path: docs/design/outcome-shapes.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T07:12:05Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-02T07:12:05Z", actor: "human:timo", revision: 10}
- {from: "active", to: "implemented", at: "2026-11-02T14:24:26Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":3,"review_outcome":2}}}
---
## Finding

Out of scope in adversary pass 1 of wave D357 U3 (`review-result:adversary-d357-u3-pass-1`): the no-view fallback that U3 replaced in `arranged_refusal` with a probe through the addressing command is still in `held_state_refusals` (`crates/verify/ess-conformance/src/synthesize/existence.rs`, near :613-618). A held-state refusal on a model with no identity view requires only the error and no event, so a target that changes the record on refusal and publishes nothing passes it.

## Acceptance

- On a model with no identity view, a held-state refusal scenario probes the record after the refused send through a branch that runs from the held state, as the arranged input-refusal half does, and a target that moves the record on refusal fails it; the correct target passes.
- Where no such branch has a valid send, the scenario is withdrawn and refused as `ESS-SYNTH-001` naming the branches tried.
- A model with an identity view, and every committed model, keeps its suite bytes.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave D368 on 6437975cb. Every line is **cited** (read from the story or the tree) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/existence.rs`, `held_state_refusals` :648-751; the no-view fallback is the `Err(_)` arm of `observe_selection_subject` at :697-715 (the story's ":613-618" is stale) — cited
- **Symbols:** `probe` :540-586, `UNPROBED` :591, `runs_from` :524, `addresses` :515, `arranged_refusal` :399 (the model to follow, probe at :450), `refused_inputs_in` :753; in `synthesize.rs`: `reach_in_state` :8007, `reach` :8290, `observe_selection_subject` :8165, `invoke_with` :7600, `Setup` :6437 — cited
- **Reuse gap:** `probe`'s `addresses` filter rejects every subject-less held-state refusal (`when_subject_state: Paused` + `error:`); called unchanged it would withdraw such scenarios; inputs must come from `reach_in_state` for that state — inferred (walked, unproven)
- **Today, observed:** `ess verify conform synthesize` (0.82.0) on a no-view probe model: the held-state halves send the refused input with error and no event, and no read-back follows; with a view they get `snapshot_complete_subject` / `expect_complete_subject_unchanged` — cited (run)
- **Committed model that moves:** `crates/verify/ess-conformance/tests/fixtures/occurrence-clock-offers.yaml` has no `views:`; its held-state input refusal `AcceptOffer/outcome/malformed` is filed today with no probe; pinned in `tests/fixtures/synthesis-suite-bytes-table.tsv:233`, `external-beside-held-guard-base.tsv:115`, `adversary-244b-base-digests.tsv:178-179` — cited that it has no probe; inferred that the bytes move. So acceptance line 3 holds for every model except this fixture, which the unit names and re-pins
- **Tests pinning today:** `tests/held_state_input_refusal.rs` (models with views), `tests/adversary_d357_u3_pass1.rs`, `tests/adversary_arrangement_pass2.rs`, `tests/synthesis_suite_bytes_table.rs` + `_cli_read.rs`, `tests/external_beside_held_guard.rs`, `tests/adversary_244b_window_free_bytes.rs` — cited
- **Test a unit would add:** `tests/a_no_view_held_state_refusal_probes_the_record.rs` (correct target passes, a target moving the record on refusal fails, a withdrawal case with `ESS-SYNTH-001` naming the branches tried) — inferred
- **Documents:** `docs/design/outcome-shapes.md` "Beside held-state branches" :298-320 and :295-297; `CHANGELOG.md` `[Unreleased]` — cited (passage), inferred (edit)
- **Confidence:** high on the site; medium on the bytes outcome and the probe design for refusal branches
- **Would collide with:** `existence.rs` `probe`/`UNPROBED`/`arranged_refusal`/`held_state_refusals`; `outcome-shapes.md` :276-320; the three digest tables above; story:a-no-view-probe-tells-from-states-apart (same `probe`, same texts: one unit)

## Decisions

Wave D368 (`approval-record:ess-d368-wave`, 2026-11-02): this story and
`story:a-no-view-probe-tells-from-states-apart` are one unit, because both rewrite `probe` in
`synthesize/existence.rs`. The third acceptance line holds for every model with an identity view
and every committed suite under `suites/`. The test fixture `occurrence-clock-offers.yaml` has no
views and a held-state input refusal, so its suite gains the probe; its pinned rows are re-pinned and
the model is named in the CHANGELOG line.

## Delivered

D368, 2026-11-02, with `story:a-no-view-probe-tells-from-states-apart` as one unit: on a model with no identity view, an input refusal beside held-state branches reads the record back after each held-state send, through sends from that state that tell it apart from every other state. Unit e266be529, merged 550377957. Only `occurrence-clock-offers` moved its suite bytes. Fix round 1 (c5eddc4a2, merged 7cdd82c68) answered both review passes; see the sibling story. The held-state case where an identity view exists but cannot observe the state became `story:a-held-state-refusal-reads-back-where-the-view-cannot-observe`, and the ignored window-free probe's stale rows became `story:the-244b-probe-stays-current`.
