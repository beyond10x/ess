---
format: aep.planning-md/3
id: story:a-related-boundary-refusal-names-the-scenario-it-leaves-standing
kind: story
status: draft
title: A related-boundary refusal says the branch's scenario stands and a row of its related guard is unwitnessed, where it said the branch has no scenario
summary: ESS-SYNTH-003 from related_boundaries prints 'has no scenario' for a branch whose scenario the suite holds; the refusal is right, its text is wrong
tags:
- defect
- issues-2026-11-26
relations:
- serves: vision:O2
- informed_by: story:a-creating-command-beside-an-exists-false-related-row-refusal-puts-its-second-row-under-the-subject-s-existing
- informed_by: story:a-setup-runs-refusal-names-the-real-cause-where-its-branch-has-a-scenario-of-its-own
revision: 2
---
## Need

A downstream application reported on 0.99.4 (received 2026-11-25) refusals worded "outcome <command>/<branch> has no scenario `<command>/outcome/<branch>`" for branches whose scenario the suite holds. The wording is wrong; the refusal is right.

Verdict: a wording defect only, in `related_boundaries` of `crates/verify/ess-conformance/src/synthesize.rs`, since 0.42.0 at least (`72e532a0aa`). `docs/design/cross-record-and-stored-field-guards.md` (lines 803 to 806) says a boundary row that no bounded arrangement reaches is refused under the branch's scenario id (`ESS-SYNTH-003`).

Cause. The branch's scenario stands: `exercise_as` writes it. `related_boundaries` then fails to arrange one isolating row of the related predicate's disjunction (one disjunct true, the other false) and pushes `Refusal::about` (`stands: false`), so the `Display` arm (`synthesize.rs`, lines 943 to 947) prints "has no scenario". `Refusal::beside` (`stands: true`) is the documented form for a row beside a standing scenario, but no `Display` arm words a standing `GuardUnsatisfiable`.

Reproduction, on the 0.99.5 binary: a generic model with a view whose related predicate is `any(state == Retired, tier == gold)` and a command that refuses unless the row is gold, so no row that is `Retired` and not gold exists. Synthesis prints `refusal[ESS-SYNTH-003]: outcome demo.shop.UseCoupon/blocked has no scenario ...` while the written suite holds that scenario.

Another site of the same class. `story:a-creating-command-beside-an-exists-false-related-row-refusal-puts-its-second-row-under-the-subject-s-existing` (issue 795) fixes `ESS-SYNTH-014` built with `about` beside a standing scenario, in `arrange_ranked`, with its own `Display` arm; that fix does not cover `related_boundaries`. `story:a-setup-runs-refusal-names-the-real-cause-where-its-branch-has-a-scenario-of-its-own` words `ESS-SYNTH-004` for a setup run. The two transition lines (`ESS-SYNTH-008`, also on 0.99.1) are a further site, not traced.

## Outcome

`related_boundaries` pushes `Refusal::beside` for its two `GuardUnsatisfiable` refusals, and `Display` has an arm for a standing `GuardUnsatisfiable` that says the scenario exists and a row of its related guard is left unwitnessed. The refusal's code, its scenario id and the suites' bytes do not change; only the text does.

## Acceptance

- The red test `related_guard::a_related_boundary_no_arrangement_reaches_is_refused_beside_the_scenario_it_leaves_standing` fails on 0.99.5 at its `stands` assertion and passes after the fix.
- The readers of `stands` (`arranged.rs`, `disclosure.rs`, `synthesize.rs`) treat a standing refusal as a missing row, which is this case; no committed byte row moves.
- A refusal whose branch really has no scenario keeps its text.

## Fit review

1. Need: a refusal that names a missing scenario where the suite holds it sends the author hunting for a defect that is not there.
2. Class: a defect of wording; no new code and no format step.
3. Already expressible: not applicable.
4. Fit: the documented `Refusal::beside` form, already used by the other sites.
5. A second adopter: any model whose view predicate has a disjunction no bounded arrangement isolates.
6. Cost: none beyond the text.
7. Considered: rewording the existing "has no scenario" arm for every `GuardUnsatisfiable` (rejected: it is right where the scenario is missing).

## Decisions

- None open. Taken as a defect, a wording fix on the documented form (the coordinator, 2026-11-26).

- D153 (the coordinator, 2026-11-26): N90B and N95 ship in 0.99.6, N93 in 0.99.7, each a patch release from the previous tag.

## Delivery

Branch `fix/n95-a-related-boundary-refusal-names-the-scenario-it-leaves-standing`, from tag 0.99.5, pushed. Red test `f61078c793`, fix `fd657ef3ec`. The gate ran every step with exit 0 (stamp, `task fmt-check`, clippy, `related_guard`, `related_guard_pass1_fixes`, `related_guard_pass2_fixes`, `adversary_related_guard_pass1`, `interpreted_related_guards`, the bytes table with its slow lanes, `cli_read`, `adversary_related_values_all_bytes`, `external_beside_held_guard`, `tutorial_page`); no committed byte row moved; the longest test was 51.3 s. Planned for 0.99.6.

## Scope

Cited: `crates/verify/ess-conformance/src/synthesize.rs` (`related_boundaries`, the `Display` arm of a refusal), `crates/verify/ess-conformance/tests/related_guard.rs`, `docs/design/cross-record-and-stored-field-guards.md`.
