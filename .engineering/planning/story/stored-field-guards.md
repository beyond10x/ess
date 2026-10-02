---
format: aep.planning-md/3
id: story:stored-field-guards
kind: story
status: implemented
title: An outcome can be guarded by the addressed entity's stored fields
refs:
- provider: github
  reference: beyond10x/ess#75
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-25T21:42:44Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-25T21:43:14Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-10-02T09:41:01Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"verification":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

"Express parcels over 20 kg are refused at dispatch" is a checked ESS guard over the addressed
entity's stored fields.

## Why

GitHub issue beyond10x/ess#75, rule 1. The design is
`docs/design/cross-record-and-stored-field-guards.md` as reworked in PR #90 (merged into this
wave's integration branch): `when_subject: {predicate: …}` beside today's `{field, equals}`,
refusal branch taking its subject from sibling branches, goal-directed input choice through `sets:`
input mappings, a typed fact source over the arranged row, ess/8.

## Acceptance

- Everything the design's acceptance and format sections require, including ess/8 as a new source
  major with unchanged bytes for models that do not use the construct, the new IR variant beside
  the unchanged one, and `cargo xtask schema`.
- Red first: the parcels example from the design validates, synthesizes a refusal scenario
  (21 kg, Express) and a success scenario (20 kg), and a mutant that ignores the stored weight
  fails the suite.
- Predicates in `when_subject` may use every form the #93/#94/#95 stories admit, including
  `defined` over an Optional stored field (the design's open question 2 is answered by #93).
- A `creates` outcome copies declared command input into the created entity's fields where the
  design's `sets:` mapping says so (the SYNTH-005 prerequisite reported in #96).

## Out of Scope

Rule 2 (non-overlap, cross-record constraints) — out of scope per the design. The duplicate-identity
refusal on `creates:` is left for a later story.

## Acceptance reconciliation 2026-10-02

The finalized source major is ess/9, correcting this story's stale ess/8 reference: string operators occupied ess/8. Binding design cross-record-and-stored-field-guards.md and stored_field_guards domain tests preserve old-model bytes and require the new source version only for the new construct. This records finalized version allocation, not a waiver of versioning.

Implementation e2d7e998e, integrated with string operators by0462e027e, is contained in public0.51.0. Domain tests stored_field_guards.rs:86/:123/:295 hold old bytes, admission and Optional/list/quantifier forms; compiler stored_field_guards_ir.rs:26/:52 holds old and new IR shapes. Conformance stored_field_guards.rs:122/:177 checks Express21kg refusal and20kg success, :465-479 honest/mutant behavior, :567/:635 Optional and string cases; arrangements exercise creates input-to-field mappings. Retained c-scratch/logs/red-conformance.log has1passed/7failed, EXIT=101; conf-new.log has13passed, EXIT=0. Schema generation and current projection are retained in xtask-schema.log and xtask-schema-check.log.

Rule2 set-wide non-overlap remains explicitly outside this story, as its original Out of Scope and binding design state. Later stored-guard reports retain separate ownership; this reconciliation does not close them.
