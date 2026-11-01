---
format: aep.planning-md/3
id: story:a-cascade-onto-rows-whose-filtered-key-is-copied-from-a-related-row-is-witnessed
kind: story
status: implemented
title: A cascade onto rows whose filtered key is copied from a related row is witnessed
summary: 'affects: rows whose creator copies the filtered field with {related:} get no arrangement (ESS-SYNTH-001); split out of U14'
tags:
- downstream
- ess-issues-3
relations:
- informed_by: story:a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set/copied.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_record_effects.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T17:06:36Z", actor: "human:timo", revision: 6, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
- {from: "proposed", to: "active", at: "2026-10-29T17:06:37Z", actor: "human:timo", revision: 7, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
- {from: "active", to: "implemented", at: "2026-11-01T01:28:11Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1}}}
---
## Problem

A moving `affects:` entry gets no scenario when its rows' creator writes the filtered field as a
copy from a related row (`run_id: {related: {via: input.generation_id, field: run_id}}`) rather
than from its own input. Synthesis refuses the outcome with ESS-SYNTH-001 and the `UNDRIVEN`
cause ("... created only by `<creator>`, which no arrangement takes for a row the filter selects
with the rows the scenario holds when it is sent ...").

The adopter of `story:a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed`
has this shape in the second `affects:` entry of the same cancel (confirmed by the orchestrator,
2026-10-29): the segment rows' creator sets `run_id: {related: {via: input.generation_id, field:
run_id}}`, and the cascade filters `run_id == input.run_id`. Fixing the first entry alone moves
the adopter's refusal from `affects[0]` to `affects[1]`.

`story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed` (implemented) lists this as its
"second cause" and proposed reusing `synthesize/row_set/copied.rs::steered`; its tests hold no
copied-key case, and its reproduction `.engineering/repro/related-copy-read-later/system.yaml`
still refuses on 0.77.0.

## Fit review

Findings covered: the "second cause" of the 2026-10-19 report
(`story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed`, `## Second cause and help line`)
and the second `affects:` entry of the 2026-10-29 report (D194).

1. **Need.** Closing a group closes its seats; a seat records its group by copying it from the
   tier it is issued under. Rows the cascade selects exist whenever a tier of the group exists and
   a seat is issued under it. Reproductions on the 0.77.0 binary (`ess 0.77.0`), neutral models
   kept as a repro outside the repository:

   | model | result |
   |---|---|
   | `demo.seats`: `Issue` refuses `unknown-tier` (`exists: false`), then `issued` sets `group_id: {related: {via: input.tier_id, field: group_id}}`; `CloseGroup/closed` `affects: [{entity: Seat, where: group_id == input.group_id, moves: Seat.close}]` | 7 scenarios, ESS-SYNTH-001 `closed.affects[0]` ... `created only by demo.seats.Issue/issued, which no arrangement takes ...` |
   | the same without `unknown-tier` | 6 scenarios, the same refusal |
   | `.engineering/repro/related-copy-read-later/system.yaml` | ESS-SYNTH-001 for `CloseGroup/closed`, same cause (plus an unrelated ESS-SYNTH-004 for `Seat/state/Closed/refuses/Use`) |
   | its `control-input-key.yaml` (the key taken as the creator's input) | `CloseGroup/outcome/closed` written, 12 scenarios; only the ESS-SYNTH-004 |
   | `demo.graph` two entries (the adopter's shape), `unknown-node` left out so the first entry is witnessable on 0.77.0 | ESS-SYNTH-001 `cancelled.affects[1]` ... `created only by demo.graph.AppendSegment/appended` |
   | `demo.graph` two entries with `unknown-node` | ESS-SYNTH-001 `cancelled.affects[0]` (the first story's cause, answered first) |

   Requester's reading (theirs, 2026-10-19): "the filter is mapped only through `sets: field:
   input.field`". Confirmed in code (question 4).
2. **Class.** Defect: `docs/design/set-effects-over-filtered-instances.md:126-129` (changed rows
   rest in a `from` state "wherever the arranging commands reach one"); the arranging commands
   reach such a seat (the creator's own `Issue/outcome/issued` scenario is in the same suite).
3. **Already expressible?** Yes; the model is complete. Writing the key from the creator's input
   (the control) witnesses it, but changes the model's meaning (the copy is the domain fact) and
   moves the checked figure out of the specification. An authored scenario covers it today.
4. **Fit.** Read in code at the 0.77.0 merge:
   - `arrange_toward_with` maps the filter onto creator inputs only through direct
     `sets: field: input.field` entries (`crates/verify/ess-conformance/src/synthesize.rs:12574-12585`,
     doc comment above `arrange_toward_bound`, `:12510-12512`). A `{related:}` source
     (`ResolvedPayloadValue::RelatedField`) is not mapped, so `creator_bindings`
     (`:12772`) finds no input for the bound field (`mapped.get(field.as_str())?` at
     `:12785`) and the creator is skipped; `row_meeting` returns `None` and
     `affects_segment_on` returns `undriven` (`set_effects.rs:2924-2930`).
   - The row-set path already arranges through a copy: `copied::steered`
     (`crates/verify/ess-conformance/src/synthesize/row_set/copied.rs:355`, called from
     `row_set.rs:2359`) splits the goal into conjuncts over copied fields, arranges the named
     related row toward them (`referenced`, `copied.rs:289-312`, via `arrange_toward_owned`),
     binds the input to it with `creator_bindings`, creates the row with `created_bound`, and
     settles each copied value from the related row (`after_referenced`, `copied.rs:317-348`).
     It works on literal goals only: `referenced` holds no captured row, so a goal bound to the
     subject instance (`run_id == <captured run>`) is not carried over.
   - `affects_segment_on` builds its `Selection` with `copies: Vec::new()`
     (`set_effects.rs:2915`); the `Link`/`Copied` machinery (`set_effects.rs:1770-1900`) serves
     `instances:` updates and binding triggers, where the copied field is not the filtered key.
   - Fix (inferred, not built): a held form of `copied::steered` whose `referenced` arranges the
     related row with `arrange_toward_held` (fields mapped through the copy: the seat's `group_id`
     becomes the tier's `group_id`, bound to the captured subject; the selection's held states and
     rows passed on), tried from `Selection::row_meeting`'s symbolic branch
     (`set_effects.rs:1011-1023`) after `arrange_toward_held` finds nothing, and only in the last
     retry U14's first story adds to `affects_segment`. The related row's own search runs in that
     same mode, so a generation row needing a node beside it (the first story) is arranged too.
   - Grid triage: no cell opens or closes; synthesis only. Composes with `when_related`
     `exists: false` on the copying input (`present_in` passes it on a captured row,
     `related_guard.rs:440-486`), and with the first story's beside rows.
5. **Second adopter.** Archiving a workspace archives its documents; a document records its
   workspace by copying it from the folder it is filed in.
6. **Cost.** Synthesis only: no format, keyword, diagnostic code, generated API or wire change.
   Suites of models carrying this refusal gain the outcome's scenario.
7. **Alternatives.** (a) Change nothing: copied keys stay unwitnessed; adopters keep authored
   scenarios. (b) Ask adopters to take the key as an input (the control): changes the domain
   model, refused. (c) Extend `Link` in `set_effects.rs` to the filtered key: duplicates
   `copied::steered`. (d) A held form of `copied::steered` from the same last retry (chosen):
   reuses the row-set path's arrangement, moves nothing that synthesizes today.

## Decisions

**Accept as proposed** (a defect; the earlier report proposed reusing `copied::steered`, which
this keeps, extended to a goal bound to a captured row).

- D214 folds this story and
  `story:a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed` into one unit,
  U14, implemented in one sitting, that story first: this story's related-row search runs in the
  mode that story adds, and the adopter's cancel needs both.
- The copied path is tried only in `affects_segment`'s last retry (that story's mode), after the
  bound search found nothing, so every outcome that synthesizes on 0.77.0 keeps its arrangement
  and bytes. `copied::steered` and its row-set caller are not changed; the held form is a new
  function beside it.
- Risk to check in the tests, inferred: in the two-entry shape the generation row arranged for a
  segment is itself a row the first entry's filter selects. The scenario's read-back of the first
  entry must account for it (or the second entry's related row must be one of the first entry's
  arranged rows). The adopter-shape test runs the scenario on the interpreter and against targets
  skipping either move, which catches a read-back that ignores it.
- D135: no wire bytes, generated API, format or documented contract changes. Suite bytes move only
  for a model whose suite carries this ESS-SYNTH-001 refusal: the refusal goes and the scenario
  the specification obliges appears. Not breaking. No new refusal.
- Bytes tables: no re-pin expected. Test models are inline in `tests/related_record_effects.rs`;
  on the 0.77.0 binary no repository model in the walked tree prints the `UNDRIVEN` cause (the
  only one is `.engineering/repro/related-copy-read-later/system.yaml`, which the walker skips).
  A fixture file added under a walked directory re-pins with `SYNTHESIS_SUITE_BYTES_TABLE_WRITE`
  (one added line, header count +1, nothing else), and with
  `SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE` only if it declares a component. Any other moved line
  is a regression.

## Acceptance

New tests in `crates/verify/ess-conformance/tests/related_record_effects.rs`, inline models:

- `a_cascade_onto_rows_whose_filtered_key_is_copied_from_a_related_row_is_witnessed`: the
  `demo.seats` model above, with and without `unknown-tier`: no refusal;
  `demo.seats.CloseGroup/outcome/closed` exists, sends `demo.seats.DefineTier` and
  `demo.seats.Issue` before `demo.seats.CloseGroup`, passes `Interpreted::for_model`, and fails
  against the model whose close skips the move.
  `cargo test -p ess-conformance --locked --test related_record_effects a_cascade_onto_rows_whose_filtered_key_is_copied_from_a_related_row_is_witnessed`
- `a_cascade_with_both_entries_of_the_reported_shape_is_witnessed`: the adopter's shape in
  `demo.graph` form. `CancelRun/cancelled` moves `Run.cancel` with two entries:
  `Generation` (`where: run_id == input.run_id`), created by `StartGeneration`, which refuses
  `unknown-run`, `run-not-active` (`state != Active`) and `unknown-node` before `started`; and
  `Segment` (`where: run_id == input.run_id`), created by `AppendSegment`, which refuses
  `unknown-generation` (`exists: false`) and copies `run_id: {related: {via: input.generation_id,
  field: run_id}}`. Synthesis has no refusal; `demo.graph.CancelRun/outcome/cancelled` exists and
  sends `AddNode`, `StartGeneration` and `AppendSegment` before `CancelRun`; the suite passes
  `Interpreted::for_model`; the scenario fails against a model skipping the `Generation` move and
  against one skipping the `Segment` move.
  `cargo test -p ess-conformance --locked --test related_record_effects a_cascade_with_both_entries_of_the_reported_shape_is_witnessed`
- `a_cascade_onto_copied_key_rows_names_the_creator_where_no_related_row_can_be_arranged`: in
  `demo.seats`, `DefineTier` refuses an open group (`when_related: {via: input.group_id,
  predicate: state == Open}`, beside `exists: false`), so no tier of an open group exists: the
  refusal stays ESS-SYNTH-001 for `CloseGroup/outcome/closed`, no such scenario is written, and
  the cause names `demo.seats.Issue/issued` and `closed.affects[0]`.
  `cargo test -p ess-conformance --locked --test related_record_effects a_cascade_onto_copied_key_rows_names_the_creator_where_no_related_row_can_be_arranged`
- Manual check with the built binary: `.engineering/repro/related-copy-read-later/system.yaml`
  writes `seats.desk.CloseGroup/outcome/closed`; its ESS-SYNTH-004 stays, as in
  `control-input-key.yaml`.
  `cargo run -p ess-cli --locked --bin ess -- verify conform synthesize --path .engineering/repro/related-copy-read-later/system.yaml`

Kept green: everything the first story's `## Acceptance` lists, plus the row-set callers of
`copied::steered`: `row_set_related_copy`, `row_sets`, `row_sets_suite`
(`cargo test -p ess-conformance --locked --test <file>`).

## Scope

Primary surface: synthesis of a moving `affects:` entry's rows through a creator that copies the
filtered key (`ess-conformance`). Size: S-M on top of the first story, same implementor, same
sitting.

- `crates/verify/ess-conformance/src/synthesize/row_set/copied.rs` (cited): a held form of
  `steered` (`:355`) and `referenced` (`:289`) beside the existing ones; visibility widened so
  `set_effects.rs` can call it (the module is private to `row_set`, `row_set.rs:65`).
- `crates/verify/ess-conformance/src/synthesize/set_effects.rs` (cited):
  `Selection::row_meeting`'s symbolic branch (`:1011-1023`), in the first story's mode only.
- `crates/verify/ess-conformance/src/synthesize/row_set.rs` (inferred): a re-export of the held
  form, if the implementor does not widen `copied`'s visibility instead.
- `crates/verify/ess-conformance/tests/related_record_effects.rs` (cited): the three new tests.

Collisions: U13 (row-set arrangement) may edit `row_set.rs::arrange_row` (`:2297`), which calls
`copied::steered`, and `copied.rs` (inferred); this story adds functions and leaves `steered`
and `referenced` as they are, so a merge is textual. U11 (`synthesize.rs::arranged`,
`related_guard.rs` for #693): no shared function. U15: no shared file.
