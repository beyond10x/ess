---
format: aep.planning-md/3
id: story:a-set-effect-selection-reads-the-row-it-references-and-a-bulk-delete-removes-children-before-parents
kind: story
status: draft
title: A set-effect selection reads the row it references, and a bulk delete removes children before parents
summary: 'instances: and affects: read the candidate row and input only, and affects: is refused beside instances:, so a purge that reads a referenced row''s setting or state and removes children first cannot be declared (gap report, 2026-11-24)'
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:row-queries-navigate-a-reference
- informed_by: story:feature-request-452
- informed_by: story:a-selection-delete-may-bound-how-many-rows-one-call-removes
- informed_by: story:an-ordered-selection-may-be-counted-as-a-value-with-rows-before-this-one-and-its-head-may-be-the-subject-of-a
- informed_by: story:a-system-issued-creating-command-is-witnessed
revision: 2
---
## Need

A downstream application reported on 2026-11-24 (tenth gap report; reporter name kept out). A cleanup job removes, per
owner of a setting, the records older than a period, children before parents because the reference has no cascade, and
keeps a record while a related record is still pending. `instances:` and `affects:` read the candidate row and input only
(`docs/design/language-grids.md:27`, grid R), so the filter cannot read the referenced row's setting or state, and
`affects:` is refused beside `instances:`, so a job with no single subject cannot delete the children first.

Needed: a selection that reads the row a candidate references, and a bulk delete that removes the child rows before the
parent rows, with the order checked.

## Reproduction

Brand-free fixture, `format: ess/37`, domain `demo.ret`: `Tenant {retention_days}`, `Batch {tenant_id, age_days}` with
states `Pending`, `Done`, `Entry {batch_id: Optional, tenant_id, age_days}`. `ess specify validate`, installed 0.98.0:

| `PurgeEntries` outcome | result |
|---|---|
| `instances: {where: tenant_id == input.tenant_id and age_days > input.min_age}` (control) | valid, 8 scenarios |
| `where` reads `tenant_id.retention_days` | `ESS-COMMAND-003 cannot select retention_days from demo.ret.TenantId` |
| `where` reads `batch_id.age_days` | `ESS-COMMAND-003 cannot select age_days from Optional<demo.ret.BatchId>` |
| `where` reads relation `tenant.retention_days` | `reads tenant, which is not a declared observable root` |
| `instances:` plus `affects:` deleting the children | `ESS-COMMAND-009 declares affects: beside instances:; a secondary effect is admitted beside one subject in this cut` |
| `when_related:` beside `instances:` | `ESS-COMMAND-009 declares instances: under a condition other than when: or the default` |
| tenant as subject, `affects:` deletes Entry then Batch where age > `subject.retention_days` | valid, 9 scenarios, selected rows absent and kept rows present |

The last row is the idiom for one named tenant. It does not cover keep-while-pending, a selection over many owners, or
a setting reached from a candidate.

## Findings covered

- A filter cannot read the referenced row's field or state, including through an Optional reference.
- A bulk delete cannot be followed by the deletion of its child rows, and the order of the two is not checked.
- A setting cannot guard or filter a bulk effect (`when_related` is refused beside `instances:`).

## Fit review

1. Need: a bulk delete whose filter reads the referenced row, children removed before parents. Domain facts of any
   retention, archival or purge job.
2. Class: gap. Grid R (`language-grids.md:27`) stops at the candidate row; the G2 target opens "related row and row set:
   every command position" (`generic-language-features.md:217-300`) and no story covers `instances:`/`affects:`.
3. Existing idiom: the tenant-as-subject sweep, valid and checked (`affects:` entries apply in written order;
   `subject.retention_days`). Policy 1 holds for it; it needs one owned subject row and cannot read a related row's
   state, so it is not a decline.
4. Fit: no new keyword. `where:` gains the navigation of `ess/28` (`story:row-queries-navigate-a-reference`, implemented
   for views, values and guards), and `affects:` is admitted beside `instances:` for deleting entries
   (`crates/specify/ess-domain/src/command/set_effects.rs:1121` is the refusal). A reader of
   `docs/design/set-effects-over-filtered-instances.md` finds the page's "Out of scope: the order in which rows change"
   narrowed to rows within one entry.
5. Second adopter: token and subscriber purges (`story:feature-request-452`), a bounded cleanup (item 14).
6. Cost: the source format the G2 decision opens, not `ess/38` (policy 2); `ess-domain` refusals,
   `ess-conformance` synthesis (arrange the related row and decoys) and interpretation; Rust and Go targets owe the
   obligation and refuse by name until implemented; `ess/30` enforced-reference derivation treats a deleting branch of a
   referenced entity as defeating the proof, so it must hold a child-then-parent delete in one outcome; `RawSpecFile`
   schema unchanged unless a key is added (then `cargo xtask schema`).
7. Alternatives: change nothing (the job is listed as unmapped); a `cascade:` on the relation (declined cross-domain in
   `story:feature-request-452`); only the existence form of `when_related` (SW-D2 B).

## Decisions

Accept, redesigned. The coordinator decided on 2026-11-24, under the owner's delegation, each point at the default of its option list:

- SW-D2, how a selection reads the related row. Decided: A, the navigation views and guards already use (`ess/28`): a reference of the candidate row, any hop count the grid allows, including an Optional reference, with the same rule for an absent value that guards use. B was an existence or predicate form like `when_related` only; C the G2 related-row selection forms after G2 ships.
- SW-D3, format placement. Decided: A, the source format the G2 decision opens (policy 2: one format per design decision). B was `ess/38`.
- SW-D7, order of changes across entries. Decided: A, the written order of `affects:` entries (and `instances:` before `affects:`) is a promise; a target double that refuses deleting a referenced row while a child remains fails a reversed order, and the design page's "order in which rows change" out-of-scope line narrows to rows within one entry. B was to leave the order unspecified, which cannot check children-first.

The cap, the dry-run count and the system-issued await are not carried here: they join `story:a-selection-delete-may-bound-how-many-rows-one-call-removes`, `story:an-ordered-selection-may-be-counted-as-a-value-with-rows-before-this-one-and-its-head-may-be-the-subject-of-a` and `story:a-system-issued-creating-command-is-witnessed`.

## Acceptance

- selection_reads_the_referenced_row_of_each_candidate: a `where:` that reads a field of the row a candidate references
  selects exactly the rows whose referenced row satisfies it; arranged rows with a matching and a non-matching
  referenced row read absent and present, and a mutant that drops the related predicate fails the scenario.
- selection_through_an_absent_optional_reference_follows_the_guard_rule: a candidate whose Optional reference is absent
  is selected or kept by the same rule a guard uses for an absent value, and the suite arranges that candidate.
- selection_reads_the_referenced_row_state: a `where:` that requires the referenced row to be in a named state keeps the
  candidate while that row is `Pending` and selects it once the row is `Done`.
- setting_of_the_owner_filters_the_bulk_delete: with two owners holding different settings, one `instances:` delete
  removes each owner's rows older than that owner's setting only.
- bulk_delete_then_child_delete_removes_children_first: `affects:` deleting entries are admitted beside `instances:`,
  apply in written order, and a target double that refuses deleting a referenced row while a child remains passes the
  child-first spelling and fails the reversed one.
- set_effect_beside_instances_with_a_non_deleting_secondary_stays_refused: the refusal the 0.98.0 text names stays for
  any other combination, naming the admitted one.
- below_the_opening_format_refuses_naming_it: one refusal naming the format, no cascade of `ESS-COMMAND-005`.
- unimplemented_targets_refuse_by_name: Entity Runtime, Rust, Go, Web and Clap targets that cannot apply the form
  refuse by name; the interpreted scenario is green and a delete-nothing mutant fails it.
- enforced_reference_derivation_holds_the_child_first_delete: with the children removed in the same outcome, the `ess/30`
  derivation of an enforced reference does not read the parent delete as defeating the proof.
- grids_and_selection_effects_page_state_the_cell: `docs/design/language-grids.md` row R names the related row for
  `instances:`/`affects:`, and `website/docs/guides/specify/selection-effects.md` shows the keep-while-pending example.

## Scope

- crates/specify/ess-domain/src/command/set_effects.rs  cited — `RawInstances` (99, one field `where`), `affects:` beside `instances:` refusal (1121), `instances:` under a condition (1803), `{count: changed}` refusal (1891)
- crates/specify/ess-domain/src/expression.rs  cited — unobservable-fact path (1161), not a declared observable root (990); `where:` reading a reference
- crates/verify/ess-conformance/src/synthesize/set_effects.rs  cited — `UNWRITTEN` ESS-SYNTH-001 (358); arrange the related row and decoys
- crates/verify/ess-conformance/src/interpret/  inferred — apply entries in written order
- crates/specify/ess-compiler/src/  inferred — `ess/30` enforced-reference derivation
- docs/design/language-grids.md, docs/design/set-effects-over-filtered-instances.md, website/docs/guides/specify/selection-effects.md  cited

## Format

The source format the G2 decision opens (policy 2, SW-D3); not `ess/38`. Documents valid today keep their meaning: a
`where:` that reads a reference and `affects:` beside `instances:` are refused today.
