---
format: aep.planning-md/3
id: story:a-creator-guarded-by-a-related-row-and-a-row-set-drives-its-rows-for-another-commands-arrangement
kind: story
status: draft
title: A creator guarded by a related row and a row set drives its rows for another command's arrangement
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: cited
  path: crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/a_row_set_branch_arranges_rows_past_the_creators_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/row-set-refusal-beside-a-moved-row.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
revision: 8
---
## Outcome

- Take a creating branch whose command refuses a missing related row (`when_related: {via: input.x, exists: false}`) and also refuses on a row set over its own rows (`when_related: {entity: E, where: …, exists: true}`). That branch now creates rows for other scenarios: a command moving such a row, and its transition and state scenarios, are witnessed.
- The row-set branch is decided on the rows the run holds. It no longer counts as selected wherever its related row is present.

## Need

Written up from the coordinator section of `story:a-row-set-refusal-is-witnessed-beside-the-related-row-its-creator-needs` (wave branch `df7eb37a65`). It reproduces on the wave's unit binary (`wave-1122/s3/ess-s3`, which holds unit S2) and on the released 0.97.4. Both model directories hold their `run.txt`.

- `second-related-row/m-rs1/model.yaml` is the wave's generic fixture `crates/verify/ess-conformance/tests/fixtures/row-set-refusal-beside-a-named-related-row.yaml` with a lifecycle move added: `Ticket` `Pinned → Finished` through `FinishTicket` (`moves: shop.store.Ticket.finish`). The rows' creator `PinTicket` refuses `no-shelf` (`exists: false` through `input.shelf_id`) and `out-of-order` (a row set over `Ticket` where `shelf_id == input.shelf_id` and `seq >= input.seq`). The unit gives 8 scenarios, with `PinTicket/outcome/pinned` among them, and 3 refusals:

  ```text
  refused: refusal[ESS-SYNTH-004]: outcome shop.store.FinishTicket/finished has no scenario `shop.store.FinishTicket/outcome/finished`
    it needs an instance of `shop.store.Ticket` to move along `finish`, and the route runs through `shop.store.PinTicket/pinned`, which no input reaches
  refused: refusal[ESS-SYNTH-004]: transition shop.store.Ticket/finish has no scenario `shop.store.Ticket/transition/finish/by/shop.store.FinishTicket/finished`
  refused: refusal[ESS-SYNTH-004]: entity shop.store.Ticket has no scenario `shop.store.Ticket/state/Finished/refuses/shop.store.FinishTicket`
  ```

  The release gives the same three plus the two ESS-SYNTH-001 refusals that unit S2 fixed (6 scenarios, 5 refusals).
- Control `second-related-row/m-rs1-noset/model.yaml`, without `out-of-order`: 0 refusals and 10 scenarios on the unit, 1 refusal on the release (`ClearShelf/tickets-pinned`, the one S2 fixed). So the three come from the row-set branch alone.

Cause, read at `fd8a86bcf3` (inferred, not run in a debugger):
- `related_guard.rs` `selects` (line 2079) filters the branches to read. Its exclusion list does not name `ResolvedCondition::RelatedSet` (ess-compiler `ir.rs` 666). A row-set branch is not `Related`, so no predicate is read. It has no `input_guard` (line 2026 matches only `When` and `Related`), so it is pushed as selected wherever the related row is present.
- `answering_refusal` (2166) does not take it, because it is neither an input-guarded refusal (`synthesize.rs` 10784) nor a `when_related:` predicate refusal (`is_related_refusal`). So `selected` holds both `out-of-order` and `pinned`, and `selects` returns `None` (two branches selected).
- The draft's reading, "`answering_refusal` picks it over the accepting branch", is corrected here: the branch is not picked, it only makes the answer ambiguous. The effect is the same.
- The setup run of `pinned` (`created_owned_made` → `drive` → `searched` (3143) → `with_row` (5373) → `chooses` (5511) → `selects`) therefore finds no row on which `pinned` alone is selected. The run is refused, and the refusal is mapped to "which no input reaches" (story-3).

## Fit review

- Origin: a follow-up the wave's own unit S2 found, not an adopter request. It covers the ESS-SYNTH-004 part of that story's second acceptance row, whose ESS-SYNTH-001 part S2 fixed.
- Considered and rejected:
  - Excluding `RelatedSet` from `selects` like the held-state conditions. A setup run beside rows that do satisfy the selector would then be sent as accepted, and the target would refuse it.
  - Arranging a scenario-specific empty set. The run's rows are already known where `selects` is asked.

## Design

- In `selects`, a `ResolvedCondition::RelatedSet` branch is decided, not assumed. It is refuted when none of the rows the run holds is of its entity and matches its selector under the input, and selected when one does.
- `row_set.rs` already decides a selector against named rows (`row_set::rows_compared`, used by `refuted_on` at `related_guard.rs` 655). Reuse it.
- Where `selects` cannot see the run's rows, it answers `None`, as it does today for an undecided predicate (D1).
- `selects` stays a function of model, command, row and input: the `row_picks` memo (`related_guard.rs` 432) keys on exactly those. If the run's rows become an argument, they join the memo key (D1).
- No new refusal, no `ValidationCode`, no format version.

## Acceptance

- `m-rs1` as a fixture (generic, in `crates/verify/ess-conformance/tests/fixtures/`) synthesizes `FinishTicket/outcome/finished`, `Ticket/transition/finish/by/FinishTicket/finished` and `Ticket/state/Finished/refuses/FinishTicket` with 0 refusals. Every scenario passes on `Interpreted`. Tested in `tests/a_row_set_branch_arranges_rows_past_the_creators_guard.rs`, the binary unit S2 added.
- `PinTicket/outcome/out-of-order` keeps its scenario, which S2 added: the row-set refusal is still witnessed where its rows exist.
- The three pin tables change only by the new fixture's lines (`comm` proof). `stored_rows_decided_once_work` passes with unchanged bounds.

## Decisions

- D1. What `selects` decides a row-set branch on.
  - A (recommended): refuted when the run holds no row of the set's entity, and `None` otherwise. That is the smallest change, it covers the reproduction, and it keeps the memo key.
  - B: decide the selector on the rows the run holds through `row_set::rows_compared`. This is more precise, but adds the rows to `selects`' signature and to the `row_picks` key.
- D2. Order.
  - A (recommended): before story-3, which rewrites the text of the refusal this removes.
  - B: after it.
