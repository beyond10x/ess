---
format: aep.planning-md/3
id: story:a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed
kind: story
status: implemented
title: A cascade onto rows whose creator needs the moving subject active and a second related row is witnessed
summary: 'affects: rows created only by a command guarded by the subject''s state and another related row get no arrangement (ESS-SYNTH-001); adopter report 2026-10-29, 0.76.0 and 0.77.0'
tags:
- ess-issues-3
relations:
- informed_by: story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_record_effects.rs
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T17:06:35Z", actor: "human:timo", revision: 10, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
- {from: "proposed", to: "active", at: "2026-10-29T17:06:36Z", actor: "human:timo", revision: 11, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
- {from: "active", to: "implemented", at: "2026-11-01T01:28:11Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":1}}}
---
## Problem

A downstream adopter's suite keeps one ESS-SYNTH-001 refusal on 0.76.0 and on the 0.77.0 head
(84671d6b8): a command that moves its subject and cascades onto rows of another entity through
`affects:` gets no scenario for that outcome, because synthesis arranges no affected row.

Shape, as neutral names:

- `CancelRun` moves `Run.cancel` on `instance: run_id` and lists
  `affects: [{entity: Generation, where: run_id == input.run_id, moves: Generation.cancel}]`.
- `Generation` rows are created only by `StartGeneration/started`. That command refuses first when
  the run is missing (`when_related: {via: input.run_id, exists: false}`), when the run is not
  Active (`when_related: {via: input.run_id, predicate: state != Active}`), and when a second
  related row, the node, is missing (`when_related: {via: input.node_id, exists: false}`).

The refusal on the 0.77.0 head:

```text
refusal[ESS-SYNTH-001]: outcome demo.graph.CancelRun/cancelled has no scenario `demo.graph.CancelRun/outcome/cancelled`
  no witness: `cancelled.affects[0]` is `demo.stream.Generation` and created only by `demo.stream.StartGeneration/started`, which no arrangement takes for a row the filter selects with the rows the scenario holds when it is sent, so no row is arranged
```

`story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed` (implemented) arranges the
creator past a guarded sibling branch. It does not take a creator guarded by a related row's state
(the run must be Active, which is also the subject `CancelRun` moves) together with a second
related row (the node) that must be created first.

Reported by the adopter's agent on 2026-10-29 (orchestrator D194). Its specification is public;
the reproduction is kept out of this repository.

## Acceptance

New tests, in `crates/verify/ess-conformance/tests/related_record_effects.rs`, over an inline
`demo.graph` model of the `## Problem` shape (`AddNode`, `StartRun`, `StartGeneration` with
`unknown-run`, `run-not-active`, `unknown-node` before `started`, `CancelRun/cancelled` with the
moving `affects:` entry over `Generation`), with a `skipping` counterpart as the feed tests have:

- `a_cascade_onto_rows_whose_creator_needs_the_subject_active_and_a_second_row_is_witnessed`:
  at `format: ess/25` and again at `format: ess/32`, synthesis has no refusal;
  `demo.graph.CancelRun/outcome/cancelled` exists, sends `demo.graph.AddNode` at least once and
  `demo.graph.StartGeneration` at least three times before `demo.graph.CancelRun`; the suite
  passes `Interpreted::for_model` and the scenario fails against the model whose cancel skips the
  move. Before: ESS-SYNTH-001 with the cause quoted under `## Problem`.
  `cargo test -p ess-conformance --locked --test related_record_effects a_cascade_onto_rows_whose_creator_needs_the_subject_active_and_a_second_row_is_witnessed`
- `a_cascade_onto_rows_whose_creator_guards_only_a_second_row_is_witnessed`: the same with
  `unknown-node` as the creator's only refusal (no run guard).
  `cargo test -p ess-conformance --locked --test related_record_effects a_cascade_onto_rows_whose_creator_guards_only_a_second_row_is_witnessed`
- `a_cascade_arranges_a_second_row_created_behind_its_own_guard`: the node is created by a
  command refusing an unknown `Pipeline` (`exists: false`) first; witnessed and passing as above.
  `cargo test -p ess-conformance --locked --test related_record_effects a_cascade_arranges_a_second_row_created_behind_its_own_guard`
- `a_cascade_refusal_names_the_guard_that_answers_first`: with `run-not-active` written
  `predicate: state == Active` (no generation of an Active run can start), synthesis keeps
  ESS-SYNTH-001 for `CancelRun/outcome/cancelled`, writes no such scenario, and the cause names
  `demo.graph.StartGeneration/started`, `cancelled.affects[0]` and `run-not-active`; for
  `feed(&[UNKNOWN_FEED, OPEN_FEED])` the cause names `cascade.feed.Post/posted` and `open-feed`.
  The help line is the `UNDRIVEN` one (`synthesize.rs:1352-1356`).
  `cargo test -p ess-conformance --locked --test related_record_effects a_cascade_refusal_names_the_guard_that_answers_first`

Kept green, unchanged (each whole binary, `cargo test -p ess-conformance --locked --test <file>`):

- `related_record_effects`, including `a_cascade_onto_rows_of_a_guarded_creator_is_witnessed`,
  `a_cascade_onto_rows_of_a_creator_refusing_a_related_state_is_witnessed`,
  `a_cascade_refusal_names_the_creator_it_could_not_drive` and
  `a_cascade_set_effect_refusal_carries_its_own_help` (the existing cascade story's cases keep
  their scenarios and bytes).
- `related_record_effects_adversary`, `adversary_related_record_effects_pass1`, `set_effects`,
  `interpreted_set_effects`, `affects_past_emitting_not_found`,
  `every_set_move_only_refusal_names_the_state`, `a_set_effect_binding_refusal_names_each_cause`,
  `set_effect_identity_pin`.
- The other callers of `arrange_toward_held` / `arrange_toward_with`: `row_set_several_selectors`,
  `a_row_set_branch_arranges_rows_past_the_creators_guard`, `adversary_b_creator_predicate_guard`,
  `adversary_b_creator_predicate_guard_2`, `binding_arranged_rows`.
- Bytes: `synthesis_suite_bytes_table` (`every_repository_model_keeps_its_synthesized_bytes`,
  `every_repository_component_keeps_its_synthesized_bytes`) and `external_beside_held_guard`
  (`repository_model_suites_change_only_where_claimed`) with no table edited (see `## Decisions`
  for the one allowed re-pin).

## Fit review

Findings covered: the adopter report of 2026-10-29 quoted under `## Problem` (orchestrator D194), the
one ESS-SYNTH-001 refusal the adopter's suite keeps on 0.76.0 and 0.77.0.

1. **Need.** Cancelling a run cancels every generation of that run. A generation is started only
   on an existing, Active run and an existing node, so the cascade's rows need two related rows
   arranged first: the run the cascade moves (Active, which is also the state the cancel needs)
   and a node the filter does not mention. Minimal neutral reproduction, a repro outside the
   repository (`demo.graph`, `ess/25`, kept with the prep notes): `AddNode`, `StartRun`,
   `StartGeneration` (refusals `unknown-run` `exists: false`, `run-not-active`
   `predicate: state != Active`, `unknown-node` `exists: false` over `input.node_id`, then
   `started` creating `Generation` with `sets: {run_id: input.run_id, node_id: input.node_id}`),
   `CancelRun/cancelled` moving `Run.cancel` with `affects: [{entity: Generation, where: run_id ==
   input.run_id, moves: Generation.cancel}]`. On the 0.77.0 binary (`ess 0.77.0`):
   `9 scenario(s) (0 authored), 1 refusal(s)`, `refusal[ESS-SYNTH-001]: outcome
   demo.graph.CancelRun/cancelled has no scenario` with the cause quoted under `## Problem`
   (`... created only by demo.graph.StartGeneration/started, which no arrangement takes for a row
   the filter selects ...`). `StartGeneration/outcome/started` is witnessed in the same suite.
   Variants on 0.77.0, which narrow the cause:

   | variant | result |
   |---|---|
   | all three refusals (the report's shape) | 1 refusal, as above |
   | without `unknown-node` (run guards only) | 9 scenarios, 0 refusals |
   | without `run-not-active` | 1 refusal, same text |
   | `unknown-node` only (no run guard at all) | 1 refusal, same text |
   | `unknown-run` only | 8 scenarios, 0 refusals |
   | the report's shape at `format: ess/32` | 1 refusal, same text |

   So the guard that blocks is not the state predicate on the run (that case was fixed by
   `story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed`): it is the creator reading a
   **second related row through an input the filter does not bind**. The title's "needs the
   subject active" is the adopter's shape, not the cause. Requester's reading (theirs): the
   creator "needs the subject active" — reproduced only together with the node guard.
2. **Class.** Defect. `docs/design/set-effects-over-filtered-instances.md:126-129`: a moving
   entry's changed rows rest in a `from` state "wherever the arranging commands reach one". The
   arranging commands reach a `Generation` row of the run: the suite's own
   `StartGeneration/outcome/started` scenario arranges it (node and run first, then the creator).
3. **Already expressible?** Yes; the model is complete. The adopter's workaround is an authored
   scenario (`ess-scenario/1`), which the help line of this refusal already names
   (`crates/verify/ess-conformance/src/synthesize.rs:1352-1356`). No idiom witnesses it otherwise
   short of deleting the node guard.
4. **Fit.** Read in code (line numbers at the 0.77.0 merge on `integration/ess-issues-3`):
   - The `affects:` rows are searched by `Selection::row_meeting`
     (`crates/verify/ess-conformance/src/synthesize/set_effects.rs:986-1047`); the filter is
     symbolic (`run_id` bound to the captured subject), so it calls `arrange_toward_held`
     (`set_effects.rs:1013`, `synthesize.rs:12488`) with the subject's held state and settled
     values, which runs `arrange_toward_with` (`synthesize.rs:12543`).
   - `arrange_toward_with` maps `fields` onto creator inputs and asks `creator_bindings`
     (`synthesize.rs:12590`, `12772`). With a bound row and a creator that routes through
     related guards, `creator_bindings` keeps the creator only if `related_guard::present_in`
     holds (`synthesize.rs:12797`). `present_in` returns `false` for a command reading several
     related rows (`related_guard.rs:446`, `several` at `:683`) and for a command whose related
     row is not the bound one (`bound.get(via)` at `:452-454`): the report's shape is the first,
     the node-only variant the second.
   - The per-input second pass (`synthesize.rs:12562-12566`, `12623-12633`) asks
     `related_guard::present_for` (`related_guard.rs:503`), which gates several rows on
     `several_present` (`ess/32`, `:515-525`, `:535-537`) and then requires every read row to be
     a captured row with known values (`present_on`, `:544-596`, `captured` at `:556-563`). The
     node is never captured, so it fails at every format (the `ess/32` variant above).
   - Every creator is skipped, `row_meeting` returns `None`, and `affects_segment_on` returns
     `undriven` (`set_effects.rs:2924-2930`, text `:394-395`, builder `:403-417`).
     `affects_segment`'s retries (`set_effects.rs:2653-2689`) only steer the subject's own
     fields, so they do not help.
   - The unbound path already does the right thing: `created_owned` → `related_guard::drive`
     (`synthesize.rs:6867-6904`) → `drive_several` (`related_guard.rs:1990-2091`) arranges every
     row the creator reads beside the one it drives, with `beside_row` (`related_guard.rs:1056`)
     choosing a row on which no branch over it answers before the creating branch. That is why
     `StartGeneration/outcome/started` is witnessed. The bound search (`created_bound`,
     `synthesize.rs:12819`, `invoke_with` at `:12873`) sends the creator naming only the bound
     row and never arranges the others.
   - Fix (inferred, not built): in the bound search, where a creator reads related rows through
     inputs the filter does not bind, arrange each of those rows with `beside_row` first, bind
     its instance to the input, hold its state and settled values beside the subject's, decide
     the creator with `present_on` over every row (all captured now, so no format gate is
     needed), and prepend the rows' steps to the created row's. Only from a new last retry of
     `affects_segment`, so every path that witnesses today keeps its arrangement.
   - Grid triage: no grid cell opens or closes; this is synthesis arranging rows the model
     already admits. It composes with `when_related` (exists and predicate), with `ess/22`
     several-row reads and with `ess/28` `all:` branches through `beside_row`'s own handling
     (inferred). Targets: synthesis only; the interpreter and generated runners run the new
     scenario as any other.
5. **Second adopter.** Closing a project closes its open tasks; a task is opened only on an open
   project and an existing assignee.
6. **Cost.** Synthesis only: no format, no keyword, no diagnostic code, no generated API, no wire
   change. Suites of models carrying this refusal gain the outcome's scenario. The refusal that
   remains where no arrangement exists gains the name of the branch answering first in its cause
   (same code, same reason constant, same help line).
7. **Alternatives.** (a) Change nothing: every cascade whose rows' creator reads a second related
   row stays unwitnessed, and adopters keep authored scenarios for it. (b) Seed the rows
   (`--synthesis-seed`): seeds admit the stored subject row only
   (`docs/design/synthesis-seeds.md`), as the earlier cascade story found; refused for the same
   reason. (c) Lift the `ess/32` gate in `present_for` for everyone: still fails (the node is not
   captured), and would move suites of row-set paths below `ess/32`. (d) Arrange the creator's
   unbound related rows beside it in the bound search, from a last retry (chosen): reuses the
   unbound path's `beside_row`, changes nothing that synthesizes today.

Not part of this cause, found while reproducing: an outcome with a second `affects:` entry over
rows whose filtered key is copied from a related row (`run_id: {related: {via:
input.generation_id, field: run_id}}`) still refuses on 0.77.0 once the first entry is fixed
(variant `two-entries` without `unknown-node`: `cancelled.affects[1]` ... `created only by
AppendSegment/appended`), and so does `.engineering/repro/related-copy-read-later/system.yaml`
(`CloseGroup/closed`, ESS-SYNTH-001, same cause text). That is the "second cause" of
`story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed`, not delivered there; split out as
`story:a-cascade-onto-rows-whose-filtered-key-is-copied-from-a-related-row-is-witnessed`.

## Decisions

**Accept as proposed** (a defect; the adopter proposed no syntax, only the scenario it expects).

- The cause is a creator reading a second related row through an input the `affects:` filter does
  not bind, not the state predicate on the moving subject (`## Fit review`, question 1). The
  acceptance names both shapes: the report's (run guards and node guard) and the node guard alone.
- Design: a last retry of `affects_segment` (`set_effects.rs:2653`), taken only where every
  earlier try returned the `UNDRIVEN` cause, runs the bound row search with the creator's unbound
  related rows arranged beside it (`related_guard::beside_row`, the row the unbound path's
  `drive_several` already arranges), decided by `present_on` over every row. Every outcome that
  synthesizes on 0.77.0 keeps its arrangement and bytes.
- No format gate. `present_for`'s `ess/32` gate (`several_present`) stays as it is for its own
  callers; the new path does not go through it, because it runs only where 0.77.0 refuses, so no
  suite below `ess/32` that synthesizes today moves. The adopter's model is `ess/25`.
- Where no arrangement exists (the creator's guard over the held subject contradicts the state
  the cascade needs), the refusal stays ESS-SYNTH-001 with the same `UNDRIVEN` reason and help
  line; the cause's subject text also names the creator's branch that answers first on the held
  row (`StartGeneration/run-not-active` in the contradiction variant, `Post/open-feed` in the
  existing feed model). The name goes into the `type_ref` text `undriven` already builds
  (`set_effects.rs:403-417`), not into the reason constant, so `synthesize.rs:1352` still picks
  the help line.
- D135: no wire bytes, generated API, format or documented contract changes. Suite bytes move only
  for a model whose suite carries this ESS-SYNTH-001 refusal: the refusal goes and the outcome's
  scenario appears, the scenario the specification already obliges (the same class as
  `story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed`, shipped without a gate). Not
  breaking. No new refusal: nothing for the adopter replay to admit beyond "no adopter's refused
  count rises".
- Bytes tables: no re-pin expected. The test models are inline in
  `tests/related_record_effects.rs` (no new fixture file in the walked tree). On the 0.77.0 binary
  none of the 14 repository models declaring `affects:` outside `.engineering/` (which the walker
  skips) prints the `UNDRIVEN` cause, so no pinned line can move. If the implementor adds a
  fixture file under a walked directory, re-pin with `SYNTHESIS_SUITE_BYTES_TABLE_WRITE` (one
  added line, header count +1, no existing line changed; the diff must show only that), and with
  `SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE` only if the fixture declares a component. Any other
  moved line is a regression, not a re-pin.
- D214 folds this story and
  `story:a-cascade-onto-rows-whose-filtered-key-is-copied-from-a-related-row-is-witnessed` into
  one unit, U14, implemented in one sitting, this story first. The adopter's second `affects:`
  entry copies its key (`run_id: {related: {via: input.generation_id, field: run_id}}`, confirmed
  by the orchestrator 2026-10-29), so this story alone would only move the adopter's refusal from
  `affects[0]` to `affects[1]`. The adopter-shape test with both entries is in that story's
  `## Acceptance`. The batch replay records which entry the adopter's refusal names before and
  after, and expects none after.

## Scope

Primary surface: synthesis of a moving `affects:` entry's rows (`ess-conformance`, the bound row
search behind `Selection::row_meeting`). Size: M, one implementor, one sitting.

- `crates/verify/ess-conformance/src/synthesize/set_effects.rs` (cited): `affects_segment`
  (`:2653`, one more retry after the steered ones), `affects_segment_on` (`:2762`, passes the
  mode on), `Selection` (`:744`, one flag), `Selection::row_meeting` (`:986`, the
  `arrange_toward_held` call at `:1013`), `undriven` (`:403`, names the branch answering first).
- `crates/verify/ess-conformance/src/synthesize.rs` (cited): `arrange_toward_held` (`:12488`),
  `arrange_toward_with` (`:12543`, the mode beside `(owner, named)`; its other callers pass it
  off), `creator_bindings` (`:12772`), `created_bound` (`:12819`, the beside rows' steps first).
  Not `arranged` (`:4946`) and not the help text at `:1352`.
- `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (inferred): one `pub(super)`
  entry over `beside_row` (`:1056`) and `present_on` (`:544`) for the bound search; no existing
  function's behaviour changes.
- `crates/verify/ess-conformance/tests/related_record_effects.rs` (cited): the four new tests and
  the inline `demo.graph` model.

Collisions with the other batch-3 units:

- U11 (#645, #693): edits `synthesize.rs::arranged` (`:4946`) and `synthesize/related_guard.rs`
  (#693, inferred around `related_guard::arranged`, `:2691`). U14 edits neither function: a
  textual merge in two shared files, different regions (U14's `synthesize.rs` edits are at
  `:12488-12873`, its `related_guard.rs` edit is an added function).
- U13 (parent-row tenant guard, synthesis arrangement): its creator-predicate-guard case starves
  the row-set reader's `found` through the same bound search: `row_set.rs:2334` calls
  `arrange_toward_held`, and `creator_bindings` / `related_guard::present_for` / `steering_for`
  decide its creator (inferred). The likely shared functions are `arrange_toward_with` (signature
  and the per-input pass), `creator_bindings` and `related_guard::present_for`. Run U13 and U14
  one after the other on these functions, or rebase the second onto the first; U14 keeps its
  change additive (a mode its callers pass off) to keep that rebase small.
- U15 (authored scenario reading): no shared file.
