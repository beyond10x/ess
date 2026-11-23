---
format: aep.planning-md/3
id: story:a-setup-run-of-a-creating-branch-that-reads-several-related-rows-creates-its-row-with-the-field-values-another-scenario-chose
kind: story
status: active
title: A setup run of a creating branch that reads several related rows creates its row with the field values another scenario chose
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: cited
  path: crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/created_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/a-creator-chosen-values-beside-several-rows.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/a-creator-reading-several-related-rows.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_guard_multiple_vias.rs
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T22:32:32Z", actor: "human:timo", revision: 15}
- {from: "proposed", to: "active", at: "2026-11-22T22:32:33Z", actor: "human:timo", revision: 16}
---
## Outcome

- Some scenarios need a row whose field values the scenario chooses, for example a grant whose `name` and `mode` a guard reads. When that row's creating branch reads two or more related rows, the scenario is now witnessed: through the arrangement that branch's own scenario uses, with the chosen values sent. Before, the scenario was refused.
- This covers three kinds of scenario:
  - an accepting branch that needs such a row;
  - each leg of an `any:` guard over it;
  - the same row held in a later lifecycle state.
- Setup runs that already succeed, and the refusals of runs that still fail, keep their bytes.

## Need

Found while measuring unit S3 of the wave of 2026-11-22. That unit is `fd8a86bcf3` on `unit/w1122-s3`, story `a-creator-reading-several-related-rows-arranges-its-row-for-other-scenarios`. It arranges such a setup run as the branch's own scenario, but only when the run carries no chosen input (`related_guard.rs` at `fd8a86bcf3`, `drive_several`, the condition `&& input.is_none()` at line 2906).

All models below were run with the unit's release binary (`wave-1122/s3/ess-s3`, `ess 0.97.4`) and with the released 0.97.4. Each model directory holds its `run.txt`.

- `second-related-row/m-g20/model.yaml` (148 lines):
  - `IssueGrant` reads a site and a desk through its input, with an `elsewhere` guard that compares the desk's `site_id` with `input.site_id`, and creates a `Grant` carrying `name` and `mode`.
  - `UseGrant` refuses `grant-refused` on `any: [state == Revoked, name none_of [chat], mode == denied]`.
  - Release: 9 scenarios, 5 refusals (all ESS-SYNTH-004 "which no input reaches"). Unit: 14 scenarios, 3 refusals, one per leg:

  ```text
  refused: refusal[ESS-SYNTH-003]: outcome shop.desk.UseGrant/grant-refused has no scenario `shop.desk.UseGrant/outcome/grant-refused`
    no candidate of the 3 tried satisfies `a related row on which `name not in [chat]`, `mode == denied` is false and `state == Revoked` is true`
    no candidate of the 3 tried satisfies `a related row on which `state == Revoked`, `mode == denied` is false and `name not in [chat]` is true`
    no candidate of the 3 tried satisfies `a related row on which `state == Revoked`, `name not in [chat]` is false and `mode == denied` is true`
  ```
- `second-related-row/m-g21/model.yaml`: the same with `none_of [mail]`, so the accepting branch needs a value that is not the enum's first. On the unit it also refuses the accepting branch:

  ```text
  refused: refusal[ESS-SYNTH-003]: outcome shop.desk.UseGrant/used has no scenario `shop.desk.UseGrant/outcome/used`
    no candidate of the 2 tried satisfies ``shop.desk.Grant` stored related row selecting this branch, over the rows 2 bounded arrangements left`
  ```

  In `m-g20` and in the unit's fixture, the accepting branch passed only because the default arrangement sends the first enum variant (`chat`, `free`).
- `second-related-row/m-g22/model.yaml`: the unit's fixture with `none_of [mail]`. On the unit it gives 15 scenarios and 4 refusals. Besides two legs, the receiving command's `received` and `ignored` are refused on the row beside its focus:

  ```text
  refused: refusal[ESS-SYNTH-003]: outcome shop.desk.ReceiveTicket/received has no scenario `shop.desk.ReceiveTicket/outcome/received`
    no candidate of the 1 tried satisfies `a row of `shop.desk.Grant` for `input.grant_id` of `shop.desk.ReceiveTicket` on which no branch answering before `received` is selected, beside the row the scenario is arranged around`
  ```
- Controls, each 0 refusals on both binaries:
  - `m-g20-one`: `IssueGrant` reads only the desk, 12 scenarios;
  - `m-g20-noelse`: both rows read, no guard reads the input, 13 scenarios;
  - `m-g21-one`: as `m-g20-one`, 12 scenarios.

  So the gap appears only where the strict search fails and the unit's fallback would be needed.
- The unit's fixture `crates/verify/ess-conformance/tests/fixtures/a-creator-reading-several-related-rows.yaml` gives 17 scenarios and 3 refusals on the unit. The three are the three legs of `ReceiveTicket/grant-refused`, with the texts above (`of the 5 tried`).
- On an adopter's downstream specification, the unit removes 81 of the 101 refusals rooted in such a setup run. Most of the remaining 20 are rooted in the same gap:
  - One receiving branch, and its sibling, need a related row beside the focus whose enum field carries a chosen value that is not the enum's first. They are refused "no candidate of the 1 tried satisfies `a row of … beside the row the scenario is arranged around`".
  - 18 refusals of the downstream commands, which need the row that branch creates, follow from it (inferred from the earlier hand-lift).
  - The unit surfaced 6 refusals that are not in the prediction. Four are the legs of an `any:` refusal over that row in four commands, refused as in `m-g20`. One is refused as in `m-g21` ("stored related row selecting this branch, over the rows 2 bounded arrangements left"). One gives no detail (inferred to be the same).

Where the candidate set ends, reading `fd8a86bcf3`:
- The fact search makes the candidates for a row with chosen values in `subject_fact.rs` `make_creations` (line 5692). Each creating input `toward` the goal is tried through `created(.., Some(&input))` (line 5784). One that fails is dropped without a cause (`if let Ok(arrangement)`).
- `created` reaches `created_owned_made` (`synthesize.rs` 8769). That calls `related_guard::drive(.., input)` (8792), then `drive_several` (2880), then `drive_several_strict`. The strict search arranges each row beside for every input and refuses the `elsewhere` link.
- The fallback is skipped because `input` is `Some`. Only the default creation is left, and it carries the first enum variants.
- `drive_toward` (2748) refuses `several(driver.command)` outright (line 2760). That is the same gap for the existence family.

## Fit review

- Origin: a defect found by measurement in the wave's own unit, not an adopter request. It covers about 26 of the adopter's 57 remaining refusals (the 20 root-caused ones minus the one separate case, plus the 6 surfaced legs; inferred for the downstream ones). It also covers the three legs left on the unit's fixture.
- Considered and rejected:
  - Sending every enum variant from the default arrangement. The search is bounded per candidate, and that multiplies it.
  - Lifting the `several` refusal in `drive_toward` alone. That path chooses rows the strict way and fails on the same guard.
- The separate case on that specification (a branch decided on a stored related row, "16 bounded arrangements") is not covered, as before.

## Design

- In `related_guard.rs` `drive_several`, admit the fallback for a chosen input as well. When `input` is `Some(chosen)`:
  - arrange the run as its own scenario (`arranged`);
  - set each chosen value on the arranged input, except the input fields that name a row (`rows(driver.command)`, `via.field()`), which the arrangement owns;
  - check with `selects` on the arranged rows that the branch is still `driver.outcome`;
  - otherwise return the strict refusal unchanged.
- `instance` (a moving run) and `known` stay as the unit left them. A later lifecycle state is reached by the fact search's moves after the creation.
- `drive_toward`'s `several` refusal is changed in the same way only if decision D2 says so.
- No new refusal, no `ValidationCode`, no format version: only synthesis changes. Suites of models where the strict search succeeds keep their bytes.

## Acceptance

- `m-g21` (moved into `crates/verify/ess-conformance/tests/fixtures/` as a generic fixture) synthesizes with 0 refusals, and every scenario passes on `Interpreted`. That includes `UseGrant/outcome/used` and the three related-boundary legs of `grant-refused`. Tested in `tests/related_guard_multiple_vias.rs`.
- `m-g22`, as an in-test variant of the unit's fixture, synthesizes `ReceiveTicket/outcome/received` and `ignored` with 0 refusals.
- The unit's fixture `a-creator-reading-several-related-rows.yaml` gives 0 refusals (17 scenarios today with 3 refusals). Its test asserts no refusal.
- The three pin tables change only by the new fixture's lines and the two fixtures' moved rows (`comm` proof against the base).
- `stored_rows_decided_once_work` and `once_per_model_work` pass with unchanged bounds. No test over 60 s.
- Synthesis of the adopter's specification costs no more CPU than on the unit (10.9–15.7 s task-clock today, against 1.8–2.1 s on 0.97.4). This is checked by the coordinator, not by a test.

## Decisions

- D1. How the chosen values reach the run.
  - A (recommended): overlay them on the input `arranged` chose, then check with `selects`.
  - B: thread a fixed-input argument through `arranged` → `arranged_beside` → `with_row`, as `drive_toward` passes `(Some(input), &fixed)` to `searched`. B is more precise when a guard over a row reads one of the chosen fields, but touches four signatures.
- D2. The existence family's `drive_toward`.
  - A (recommended): leave its `several` refusal. No measured case needs it.
  - B: route it to the same fallback.
- D3. The CPU cost the unit added on the adopter's specification (5 to 7 times).
  - A (recommended): measure in this story whether it comes from the 67 new scenarios or from failed fallback attempts, and bound the fallback by a memo keyed like `created_memo` if it is the latter.
  - B: accept it.

## Scope

Derived 2026-11-22 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/related_guard.rs` - cited (story Design: `drive_several`, the `input.is_none()` condition; read at line 2906)
- **Files:** `related_guard.rs` `drive_several` (2880), `as_own_scenario` (2936), `own_scenario_applies` (2919) - cited
- **Symbols:** `drive_several`, `as_own_scenario`, `arranged` (3711), `arranged_beside` (3776), `selects` (2079), `rows`, `own_opening`/`OWN_OPEN` (2837) - cited; `drive_toward` (2748) only if D2 = B - cited
- **Also likely:** `crates/verify/ess-conformance/src/synthesize/created_memo.rs` or a memo local to `related_guard.rs`, for D3 A - inferred
- **Read, not changed:** `subject_fact.rs` `make_creations` (5692) and `synthesize.rs` `created_owned_made` (8769) - cited; the store lists `subject_fact.rs` as inferred, and I read no change there (inferred)
- **Tests:** `crates/verify/ess-conformance/tests/related_guard_multiple_vias.rs` (the 17-scenario / 3-refusal pin at 712-729, the `LEGS` const at 698); new fixture `tests/fixtures/a-creator-chosen-values-beside-several-rows.yaml` - cited / inferred
- **Existing fixture whose row moves:** `tests/fixtures/a-creator-reading-several-related-rows.yaml` (`refusals=3` becomes 0) - cited
- **Pin tables:** `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` (new line plus the moved row), `crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv` and `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv` (new line each) - cited, by `fd8a86bcf3`'s own diff
- **Documents:** `CHANGELOG.md` `[Unreleased]` / Changed entry - inferred
- **Confidence:** medium - the story names the sites and I read them; whether the overlay satisfies `selects` over several rows is unproven
- **Would collide with:** any unit touching `drive_several`/`as_own_scenario` in `related_guard.rs`, `selects`, `beside_refused`, `arranged_beside`, or the same three pin tables and `related_guard_multiple_vias.rs`
- **Safety fact:** the fallback applies only after the strict search fails (`drive_several` match arm), so runs the strict search finds keep their bytes - 2 (`related_guard.rs:2903-2913`), unproven

## Decisions taken

Taken for the wave of 2026-11-23, after round 1 of the plan critics. The unit departs from two of
the story's decisions, and says so in its own `## Decisions`:

- D1 option B, against the recommendation A: the chosen values enter the arrangement as `fixed`
  input fields, extended onto every candidate input of the focus row's search, not as an overlay
  on the finished input. Reason: with A the finished input must be re-checked against every row
  beside and the focus row by hand, and `selects` takes one row; with B the existing checks
  decide it, once. If the model of the unit's fixture does not exercise `fixed`, the unit stops
  unlanded and reports; the fallback is option A, which is an operator decision.
- D3, no memo and no CPU split: the unit neither measures whether the added CPU comes from the new
  scenarios or from failed fallbacks nor bounds the failed fallbacks with a memo. The repeated
  arrangement is left to
  `story:the-own-scenario-arrangement-of-a-setup-run-finds-an-equal-arrangement-again-instead-of-building-it-once-per-path`.
  The only measurement kept is the synthesis time of the two fixtures before and after.

The outcome is 0 refusals with no leg left over; a leg that stays refused is a stop, not a pinned
count. The nesting bound stays at 1 and decides the interaction.
