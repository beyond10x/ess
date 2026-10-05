---
format: aep.planning-md/3
id: story:feature-request-459
kind: story
status: draft
title: An outcome cannot create or update one record per element of a collection
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#459
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-429
- depends_on: story:feature-request-452
- depends_on: story:feature-request-448
- depends_on: story:feature-request-458
- depends_on: story:feature-request-454
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/mod.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/outcome_shapes.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/set_effects.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/interpreted_set_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/set_effects.rs
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: docs/design/set-effects-over-filtered-instances.md
- confidence: cited
  path: website/docs/guides/specify/selection-effects.md
revision: 11
---
## Outcome
Resolve beyond10x/ess#459: An outcome cannot create or update one record per element of a collection.

## Origin
beyond10x/ess#459, filed 2026-10-05; found while specifying a seen-documents marker in a downstream specification on ess 0.52.0 (`ess/20`). The issue's own shape is already minimal and brand-free (`demo.feed.*`).

## Fit review
1. Need: one command run carries N items and must leave one stored record per item, keyed by the item. If a record with that key exists it is updated, otherwise created. A refused run writes none of them. Requester's proposal (theirs): "an upsert set effect over an input collection — for each element, create the record its identity names or update the existing one, with `sets:` reading the element", witnessed by one arranged record and two elements.
2. Class: gap. Each piece exists for one record only. `instances:` selects stored rows and is refused on `creates:` (`crates/specify/ess-domain/src/command/set_effects.rs:159-170`). `affects:` writes one value to every selected row and cannot create (`docs/design/set-effects-over-filtered-instances.md:39-48`). Create-or-update addresses one record by one input identity (`website/docs/guides/specify/commands-and-outcomes.md:93-130`). The workaround, a command per element, changes the command surface and loses the run's atomicity.
3. Existing idiom: none for N records from one outcome. Vocabulary to reuse: the binder `{in: <list>, as: <name>}` of `distinct:` and the list quantifiers (`website/docs/reference/predicates.md:331-349`, `:598-615`, `ess/22`); the existence pair's semantics, with update when held and create when not, and its "exactly one row for the identity" witness (commands-and-outcomes.md:142-149); the `affects:` entry as the place an outcome writes rows other than its subject (design note :39-48, :100-122). Not probed: a list-element binder in `sets:` does not exist in any version, so there was nothing to run.
4. Fit: a third `affects:` entry form, beside filter-setting and filter-moving entries:
   `affects: [{entity: demo.feed.SeenDocument, each: {in: input.applied, as: doc}, instance: doc.document_id, sets: {source_id: subject.source_id, content_hash: doc.content_hash, applied_at: doc.applied_at}}]`.
   For each element, the row whose identity `instance:` reads is updated if held, created in `initial` if not. `sets:` reads `<as>.<path>`, `input.*` and `subject.*`, with the same refusals as other `affects:` sources (`{related:}`, `{increment:}`, `{caller:}`). `each:` and `where:` on one entry are `conflicting_declaration`. `moves:`/`deletes:` beside `each:` are refused in this cut. Duplicate keys: an entry is admitted only where the list's identity member is held distinct by a declared `distinct: {in, as, by}` (an input refusal guard or the list type's invariant). Otherwise two elements naming one identity give an order-dependent result, and validate says `missing_declaration`. A required field an entity invariant reads must be written by the entry, as for any creation (ESS-COMMAND-018, `website/docs/guides/specify/fields-and-invariants.md:47-64`). Atomicity: the existing rule that a refusal changes nothing already gives "a failed run records none". Mid-run partial failure stays out of scope, as for every set effect (design note :161-164). Siblings: `affects:` needs a subject (set_effects.rs:339-345), so a subject-less import command is refused with a hint naming that. Admitting `each:` as an outcome's own set subject is a labelled follow-up, not this cut. Targets: interpreter executes; Entity Runtime `SetEffectUnsupported`, Rust/Go/Web/Clap `MissingRepresentation` (design note :149-159); `outcome-set-effect-changed` names the entry; generated docs say "one row per element of `applied`".
5. Second adopter: an order command `SetLines {order_id, lines: List<Line>}` that writes one line row per product. A directory sync records each member it saw with a last-seen time. Both are domain facts.
6. Cost: source format `ess/23` (`crates/specify/ess-domain/src/system.rs:53-55`), introduced by #429; this story extends it. New surface: one entry key `each:` and one value-source root `<as>.` scoped to the entry. No new suite step: arrange through creations, `execute_command`, then `expect_view contains` per element plus the existence pair's one-row-per-identity check. A conformance major is not expected; confirm in the design note. It touches the same `affects:` entry code as #452 and depends on it.
7. Alternatives: (a) change nothing and document the per-element command: the suite cannot tie N rows to one run, and a refused run's rows are unchecked. (b) The requester's outcome-level "upsert set effect": the outcome's own subject is already `Source`, so a second subject on one outcome breaks the one-thing rule (`crates/specify/ess-domain/src/command.rs:6181-6197`). (c) `instances:` over an input list: `instances:` means selection from the store (design note :20-26), so one key would carry two meanings. (d) Chosen: an `affects:` entry form, reusing the binder spelling of `distinct:`/quantifiers and the existence pair's semantics. The requester's semantics and witness are kept; their placement is changed.

## Decisions
accept, redesigned — from `ess/23`, an `affects:` entry may declare `each: {in: input.<list>, as: <name>}` with `instance: <name>.<member>`. Each element creates or updates the row its identity names. Admitted only with a declared `distinct:` over that member, and refused without a subject, beside `where:`, `moves:` or `deletes:`. Replace-a-set (removing rows the list no longer names) is not in this cut. It composes with #452's deleting entry later. Format: `ess/23`, introduced by #429. Design note first: the section `## One row per element of an input list (ess/23, beyond10x/ess#459)` in `docs/design/set-effects-over-filtered-instances.md`, after #452's section and before `## Targets`, settles the shape and whether a conformance major is needed before code. Depends on #429 (edge recorded): it gates on `FormatVersion::V23`, does not edit `system.rs` or `spec-versions.md` (the implementor reports the one sentence for the `ess/23` row under `## Changelog lines`, and the coordinator merges it into the row #429 writes), and shares `outcome_shapes.rs`, `value_expression.rs`, `set_effects.rs` and the outcome conversion in `command.rs` with #429. Depends on #452 (edge recorded): it extends `affects_beside` (`set_effects.rs:305-350`), the `RawAffect` entry forms, the design note and the `selection-effects.md` sections after #452 has changed them. Further edges (recorded): after #448, which edits the same `TryFrom<RawOutcome> for Outcome` at 5900-5946 while this story edits the `affects_beside` call at 6008; after #458, which edits `check_subject` and `existing_subject_field` in `value_expression.rs` while this story edits `validate_affect`; after #454, which edits `held_state_refusals` and `refusals_on_a_stored_row` in `synthesize/existence.rs` while this story raises the visibility of two helpers there.

## Acceptance
- each_entry_updates_held_and_creates_new: one arranged record plus two elements (one naming it, one new) leaves exactly two rows with the element values, and a decoy row unchanged.
- each_entry_one_row_per_identity: the held identity reads as exactly one row after the run.
- each_entry_empty_list_writes_nothing: an empty list is accepted, and every row reads as arranged.
- each_entry_refused_run_writes_none: where the command has an input refusal, the refused run leaves no element row.
- each_entry_without_distinct_is_refused: `missing_declaration` naming the list and member.
- each_entry_beside_where_or_move_or_delete_is_refused and each_entry_without_subject_is_refused: each by name with a hint.
- each_entry_below_ess23_is_refused_naming_ess23: one refusal, no cascade.
- each_entry_targets_refuse_by_name: Entity Runtime and Rust/Go/Web/Clap named refusals; interpreted scenario green; a create-only mutant and an update-only mutant each fail it.
- each_entry_diff_line_names_the_entry: `outcome-set-effect-changed` carries `each`.
- set_effects_note_records_each_entry: `docs/design/set-effects-over-filtered-instances.md` has the heading `## One row per element of an input list (ess/23, beyond10x/ess#459)`, before `## Targets`. The section contains "`each: {in: input.`", "`distinct:`", "created in `initial`", "updated if held", "a refused run writes none", "replace-a-set" and "conformance major". A case of that name in `crates/specify/ess-domain/tests/set_effects.rs` reads the note and fails naming the missing heading or phrase.
- each_entry_guide_section_states_the_form: `website/docs/guides/specify/selection-effects.md` has the heading `## Write one record per element of an input list`, after #452's two sections. The section contains "`each:`", "`instance: <name>.<member>`", "`distinct:`", "an empty list writes nothing" and "not removed", and its fenced model validates. A case of that name in `crates/specify/ess-domain/tests/set_effects.rs` reads the page and fails naming the missing heading, phrase or invalid model.
- ess_23_row_names_each_entry: the coordinator's merged `ess/23` row of `website/docs/reference/spec-versions.md` names the `each:` entry of `affects:`. Checked at integration by the coordinator, not by a unit test.

## Scope
- crates/specify/ess-domain/src/command/set_effects.rs  cited — `affects:` entry forms, refusals (159-170, 305-350)
- crates/specify/ess-domain/src/command.rs  cited — `subject_of` (6161-6271), the one-thing rule (6181-6197), read and untouched; the `affects_beside` call in `TryFrom<RawOutcome> for Outcome` (6008) passes `each:` entries, after #429, #452 and #448 (#448 edits 5900-5946 of the same function)
- crates/specify/ess-domain/src/command/value_expression.rs  cited — `validate_affect` (283-333) puts the entry's element binder in its `Context` so an `affects:` `sets:` source may read `<as>.<member>`; #458 edits `check_subject` (927-949) and `existing_subject_field` (952-1010) first (edge recorded); #429's identity read in `identity_paths` (231-279, 257-262) is untouched
- crates/specify/ess-domain/src/command/outcome_shapes.rs  cited — existence semantics reused
- crates/verify/ess-conformance/src/synthesize/set_effects.rs  cited — new segment
- crates/verify/ess-conformance/src/synthesize/existence.rs  cited — `identity_views` (728-740) and `one_row_view` (745-753) raised to `pub(super)` for the per-element one-row claim in `synthesize/set_effects.rs`, bodies unchanged; `one_row` (765) is not reused, because it reads an updating branch's own scenario; #454 edits `held_state_refusals` (506) and `refusals_on_a_stored_row` (1157) first (edge recorded)
- crates/verify/ess-conformance/src/interpret/execute/set_effects.rs  cited — per-element write
- crates/generate/ess-entity-runtime/src/subset.rs  cited — `SetEffectUnsupported`
- crates/generate/ess-synth/src/rust/mod.rs  inferred — named refusal
- crates/verify/ess-diff/src/change.rs  cited — `outcome-set-effect-changed`
- crates/specify/ess-domain/tests/set_effects.rs  cited — domain cases
- crates/verify/ess-conformance/tests/set_effects.rs  cited — witness cases
- crates/verify/ess-conformance/tests/interpreted_set_effects.rs  cited — interpreter cases
- docs/design/set-effects-over-filtered-instances.md  cited — new `## One row per element of an input list (ess/23, beyond10x/ess#459)` after #452's section, before `## Targets`
- website/docs/guides/specify/selection-effects.md  cited — new section `## Write one record per element of an input list`, after #452's sections
