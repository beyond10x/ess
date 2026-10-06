---
format: aep.planning-md/3
id: story:feature-request-452
kind: story
status: implemented
title: An outcome that removes many records, and cascade removal
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#452
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-429
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/mod.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/set_effects.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/interpreted_set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/set_effects.rs
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: crates/verify/ess-diff/tests/set_effects.rs
- confidence: cited
  path: docs/design/set-effects-over-filtered-instances.md
- confidence: cited
  path: website/docs/guides/specify/selection-effects.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T20:40:14Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-10-05T20:40:14Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:48:53Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":6}}}
---
## Outcome
Resolve beyond10x/ess#452: An outcome that removes many records, and cascade removal.

## Origin
beyond10x/ess#452, filed 2026-10-05; downstream specifications that remove rows in bulk (replace a set, everything a parent owns) and model a user deletion that revokes tokens and releases resources in other domains. Reproduced minimally in `<fit-review scratch>/probe-452/`.

## Fit review
1. Need: three facts. (a) One command removes every stored row a filter selects: "revoke every token of this user". (b) Removing a record also removes the rows that belong to it in the same domain: "delete the user and its tokens". (c) One deletion causes removals in other domains, with stated ordering and partial-failure behaviour. Requester's proposal (theirs): "a bulk-removal effect over a filter, and a stated cascade rule (one event, several bindings, ordering and partial failure)". "Replace a set" is the delete half of #459's per-element write and is handled there or later, not here.
2. Class: (a) and (b) are gaps, with a documentation defect: `website/docs/guides/specify/selection-effects.md:4` describes the page as "An outcome that changes, moves or deletes every record a filter selects", while `:31` refuses `instances:` on `deletes:`. (c) is a convenience: expressible today (Q3).
3. Existing idiom: (a) is refused. On installed 0.52.0, `deletes: demo.auth.Token` + `instances: {where: user_id == input.user_id}` gives `error[ESS-COMMAND-009]: … a set subject is admitted beside moves: and updates: only`, from `crates/specify/ess-domain/src/command/set_effects.rs:171-182` (unchanged in 0.53.0). (b) is refused: `deletes:` + `affects:` gives `ESS-COMMAND-009 … a secondary effect is admitted beside moves: and updates:` (set_effects.rs:326-338). The same probe also prints a cascading `ESS-COMMAND-007 … declares no outcomes`, which the set-effects design says must not happen (`docs/design/set-effects-over-filtered-instances.md:4-8`); fix it in passing. (c) is expressible: one event, one binding per receiving domain, each with required `delivery:` and `on_failure:` (`retry | drop | escalate`, and per refusal since 0.53.0) (`website/docs/guides/specify/bindings-and-components.md:37-76`, `:95`). Each receiving command removes its own rows, by (a) once it exists. ESS states no order between two bindings of one event (inferred: neither the guide nor `docs/design/binding-delivery-guarantees.md` names one). Atomicity across a set is explicitly out of scope (`set-effects-over-filtered-instances.md:161-164`, `selection-effects.md:93-94`).
4. Fit: extend the set-effect family rather than add a construct. (a) `deletes: <Entity>` + `instances: {where}`: same selection as `moves:`/`updates:` (design note :28-33); zero rows is an accepted answer; `{count: changed}` counts removed rows (:35-37); `sets:` stays refused (deletes takes no `sets:`, commands-and-outcomes.md:176-177). (b) Two lifts. An `affects:` entry may declare `deletes: <Entity>`, spelled like the entry `moves: <Entity>.<transition>` from `ess/22` (:100-110): it names the entry's own entity, another entity is `conflicting_declaration`, and `sets:` beside it is refused. `affects:` is also admitted beside a `deletes:` subject, whose row exists before the outcome so `subject.<field>` reads it as today (:39-43). A deleting entry beside any other entry over the same entity is `conflicting_declaration`, the way two moving entries already are (:119-122). Cross-domain: `affects:` stays within what it reaches today. A domain boundary is where bindings belong. Targets: interpreter removes; Entity Runtime `SetEffectUnsupported`, Rust/Go/Web/Clap `MissingRepresentation`, as for every set effect (:153-159); `ess-diff`'s `outcome-set-effect-changed` line names the deletion; generated docs say which rows are removed.
5. Second adopter: a mailing list that purges every bounced subscriber of a list, and a project deletion that removes its draft documents. Both are domain facts.
6. Cost: source format `ess/23`, introduced by #429 (`crates/specify/ess-domain/src/system.rs:53-55`); this story extends it. No new keyword: `deletes:`, `instances:`, `affects:` already exist. No new suite step: the witness uses the `instances:` arrangement (three selected rows, excluded rows per conjunct, zero-match call; design note :65-80) and reads selected rows as absent with `deletes:`'s absence check (`ess-conformance/22`, commands-and-outcomes.md:178-179), so no conformance major is expected. Below `ess/23` both forms keep their current refusal codes and messages, naming `ess/23`.
7. Alternatives: (a) change nothing: the guide keeps claiming deletion and the rows stay unchecked. (b) A new `removes:` or `cascade:` construct: a second spelling for `deletes:` and `affects:`. (c) Declare cascade on the relation (`owns` deletes children): ESS relations carry no lifecycle semantics (`docs/design/ess-entity-relations-design-v0.1.md` §1-3), and an implicit effect would be one no outcome names. (d) Chosen: lift two refusals in the existing family; answer the cross-domain half with bindings. The requester's bulk effect is taken; their "stated cascade rule" is answered with the binding idiom instead of new syntax.

## Decisions
accept, redesigned — from `ess/23`, `instances:` is admitted beside `deletes:`, and `affects:` gains a `deletes: <Entity>` entry and is admitted beside a `deletes:` subject. Cross-domain cascade is declined, with the idiom: one event, one binding per receiving domain, each with its own `delivery:` and `on_failure:`; no order between bindings, and atomicity stays out of scope. This body is the decline record for that half. Document it as the section `## Removal in other domains is one binding per domain` in `selection-effects.md`, beside a new section `## Delete every record a filter selects`, and in the design note as the new section `## Deleting the selected rows (ess/23, beyond10x/ess#452)` of `docs/design/set-effects-over-filtered-instances.md`, before `## Targets`. Fix the guide's description to match what is admitted, and stop the `ESS-COMMAND-007` cascade after the refusal.

Format: `ess/23`, introduced by #429. This story depends on #429 (edge recorded) and extends `ess/23`; it does not edit `system.rs` or `spec-versions.md`: report the one sentence for the `ess/23` row under `## Changelog lines`, and the coordinator merges it into the row #429 writes. In `command.rs` it shares `TryFrom<RawOutcome> for Outcome` (5858-6070) with #448, which edits the `raw.when` / `outcome_condition` call (5900-5946): the two run concurrently in wave 2 on disjoint line ranges, and the coordinator dry-runs `git merge-tree` before the second of them merges. It also reuses #429's `deletion_witness` (`synthesize.rs:10736`) after #429 has changed it. #459 depends on this story (edge recorded) and changes `affects_beside` (`set_effects.rs:305-350`) after it. No conformance major expected.

## Acceptance
- bulk_delete_removes_every_selected_row_and_keeps_the_rest: three selected rows read absent, every excluded row reads as arranged, and `{count: changed}` is 3.
- bulk_delete_zero_match_is_accepted_with_count_zero: a second call selecting nothing takes the same outcome, count 0, rows unchanged.
- bulk_delete_with_sets_is_refused: `sets:` beside `deletes:` + `instances:` is refused by name.
- affects_delete_entry_removes_owned_rows_with_subject: deleting the subject removes every row `where: owner == subject.<id>` selects, and leaves others.
- affects_delete_entry_naming_other_entity_is_conflicting: an `affects:` entry `deletes: demo.auth.Session` under `entity: demo.auth.Token` is refused `conflicting_declaration` at `<command>.outcomes.<outcome>.affects[<i>]`, the message names both entities, and no IR is written.
- affects_delete_beside_other_entry_same_entity_is_conflicting: a deleting entry and a setting or moving entry over the same entity in one `affects:` are refused `conflicting_declaration` at the second entry. Over two different entities, the same pair validates.
- set_delete_below_ess23_is_refused_naming_ess23_without_cascade: one refusal, no `ESS-COMMAND-007`.
- set_delete_targets_refuse_by_name: Entity Runtime `SetEffectUnsupported`; Rust/Go/Web/Clap `MissingRepresentation`; interpreted scenario green, and a delete-nothing mutant fails it.
- set_delete_diff_line_names_the_deletion: `outcome-set-effect-changed` carries the deletion.
- selection_effects_page_states_admitted_forms_and_cascade_idiom: `website/docs/guides/specify/selection-effects.md` says in its first paragraph (:4) that `deletes:` takes `instances:`. Its refusal list (:31) no longer names `deletes:`. It has the headings `## Delete every record a filter selects` and `## Removal in other domains is one binding per domain`. The second section contains "one event", "one binding per receiving domain", "`delivery:`", "`on_failure:`", "no order between bindings" and "atomicity". A case in `crates/specify/ess-domain/tests/set_effects.rs` reads the page and fails on any of these.
- set_effects_note_records_bulk_delete: `docs/design/set-effects-over-filtered-instances.md` has the heading `## Deleting the selected rows (ess/23, beyond10x/ess#452)`, before `## Targets`. The section contains "`deletes:` with `instances:`", "`{count: changed}`", "`affects:`", "`deletes: <Entity>`", "`SetEffectUnsupported`" and "one binding per receiving domain". Its `## Out of scope` list still names cross-domain cascade. A case of that name in `crates/specify/ess-domain/tests/set_effects.rs` reads the note and fails naming the missing heading or phrase.
- ess_23_row_names_bulk_delete: the coordinator's merged `ess/23` row of `website/docs/reference/spec-versions.md` names `deletes:` with `instances:` and the deleting `affects:` entry. Checked at integration by the coordinator, not by a unit test.

## Scope
- crates/specify/ess-domain/src/command/set_effects.rs  cited — `set_subject` (138-226, refusal at 171-182) and `affects_beside` (305-350) refusals; `RawAffect` (55) gains the `deletes` entry key
- crates/specify/ess-domain/src/command.rs  inferred — in `TryFrom<RawOutcome> for Outcome` (5858-6070): the calls to `set_effects::affects` (5878) and `affects_beside` (6008) take a `deletes:` subject (#448 edits 5900-5946 of the same function concurrently; coordinator `git merge-tree` dry run before the second merge); the `ESS-COMMAND-007` cascade is cut where `TryFrom<RawCommandSpec>` (6276-6319) gathers refused outcomes before `validate_shape` (2695-2763) reports "declares no outcomes" (2704)
- crates/verify/ess-conformance/src/synthesize/set_effects.rs  cited — arrangement and read-back for removed rows
- crates/verify/ess-conformance/src/synthesize.rs  cited — `deletion_witness` absence check (10736), after #429
- crates/verify/ess-conformance/src/interpret/execute/set_effects.rs  cited — interpreter removal
- crates/generate/ess-entity-runtime/src/subset.rs  cited — `SetEffectUnsupported`
- crates/generate/ess-synth/src/rust/mod.rs  inferred — named refusal
- crates/verify/ess-diff/src/change.rs  cited — `outcome-set-effect-changed`
- crates/specify/ess-domain/tests/set_effects.rs  cited — domain cases and the page case
- crates/verify/ess-conformance/tests/set_effects.rs  cited — witness cases
- crates/verify/ess-conformance/tests/interpreted_set_effects.rs  cited — interpreter cases
- crates/verify/ess-diff/tests/set_effects.rs  cited — diff line
- docs/design/set-effects-over-filtered-instances.md  cited — new `## Deleting the selected rows (ess/23, beyond10x/ess#452)` before `## Targets` (153); `## Out of scope` (161) kept
- website/docs/guides/specify/selection-effects.md  cited — description (4), refusals (31), two new sections
