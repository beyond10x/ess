---
format: aep.planning-md/3
id: story:related-guard-behaviour
kind: story
status: active
title: Commands guarded by when_related are generated in the Rust and Go targets
refs:
- provider: github
  reference: beyond10x/ess#319
relations:
- decomposes: epic:ui-live-apps
- serves: vision:O2
- depends_on: story:go-generated-behaviour
- depends_on: story:feature-request-310
- depends_on: story:related-via-optional-input
- depends_on: story:related-via-stored-reference
- depends_on: story:served-store-and-entry
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/declared_behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/related_guard_obligation.rs
- confidence: cited
  path: generated
- confidence: cited
  path: website/docs/guides/synthesize.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:53Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":6}}}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:54Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":6}}}
---
## Outcome

A command whose only undetermined part is a `when_related` guard is generated in the Rust and Go
targets, through an input via, an Optional input via and a stored-reference via: the behaviour
reads the related row through the storage port, reads no row for an absent reference, and applies
the guard's branches in the stated precedence.

## Fit review

1. **Need.** Every code target keeps a `when_related` command a whole obligation
   (`crates/generate/ess-synth/src/determined.rs:108,147`; `website/docs/guides/synthesize.md:223`),
   although the guard reads one row by identity (`crates/specify/ess-domain/src/command/related_guard.rs:6-14`),
   which `<Entity>Storage::get` answers. Minimal reproduction: the 0.49 `when_related` reference
   example (`website/docs/reference/predicates.md:770-795`); with ess/21, "complete a task unless the
   task blocking it is not done" (story:related-via-stored-reference).
2. **Class.** Gap in the code targets; nothing in the format changes.
3. **Already expressible.** Only as a hand-written behaviour.
4. **Fit.** The interpreter (the reference, `determined.rs:1-8`) already executes `when_related`
   (`crates/verify/ess-conformance/src/interpret/execute.rs:495-560`), so the generated behaviour has
   a reference to follow; synthesis already witnesses every branch (`predicates.md:770-795`); the
   storage port exists in Rust (`behaviour.rs:23-56`) and in Go after story:go-generated-behaviour.
   Web is a bridge over Rust; Clap handlers stay obligations; Entity Runtime keeps
   `RelatedGuardUnsupported` (`ess-entity-runtime/src/lib.rs:1281-1284`), refused by name.
5. **Second adopter.** Any model with a reference check: an order refused while its customer is
   Suspended, a shipment dispatched only while its payment is Captured.
6. **Cost.** No format change; generated code grows a related-row read; `TARGET.md` rows shrink;
   the plan marks these commands generated (plan bytes change for models that use `when_related`).
7. **Designs.** Change nothing: rejected by the operator. Generate only the input via: rejected
   (the stored-reference via of ess/21 is the case uilab needs). **Chosen:** generate every via
   ess/21 defines.

## Decisions

**Accept (operator decision, 2026-10-01).** `determined.rs` stops treating `when_related` as
undetermined when the related entity has a storage port in the component.

- Input via (required or Optional) and subject via (stored field, Optional included) are generated
  in Go and Rust, following the interpreter and the precedence of story:related-via-optional-input:
  an input via answers at step 1; a subject via after existence and held state, at
  story:feature-request-282's step.
- Present reference: the behaviour reads the row (`get`), answers `exists: false` first, then the
  predicate branches, then the rest. Absent reference: no row is read, no `when_related` branch is
  selected, selection carries on.
- `{related:}` value reads in the same command follow the same port; they stay owed while #285's
  semantics are unreleased.
- `a_stored_reference_guard_stays_an_obligation_with_its_contract` (story:related-via-stored-reference, `tests/related_guard_obligation.rs`)
  is replaced by the generation tests below.

## Acceptance

`crates/generate/ess-synth/tests/declared_behaviour.rs`:
- `a_related_guard_command_is_generated_and_passes_its_suite_rust`, `a_related_guard_command_is_generated_and_passes_its_suite_go` (the 0.49 reference example)
- `a_related_guard_reads_the_row_through_the_storage_port`
- `an_absent_optional_reference_reads_no_row_rust`, `an_absent_optional_reference_reads_no_row_go`
- `a_stored_reference_guard_is_generated_and_passes_its_suite_rust`, `a_stored_reference_guard_is_generated_and_passes_its_suite_go` (the blocked-by model of story:related-via-stored-reference)
- `a_plan_without_when_related_is_byte_identical` (billing and gatepass `PLAN.md` and `plan.json`
  unchanged)

Go tests need the PR's CI run with Go 1.25.10 as evidence that they ran.

## Scope

`crates/generate/ess-synth/src/determined.rs`, `src/rust/behaviour.rs`, `src/go/behaviour.rs`
(from story:go-generated-behaviour), `src/plan.rs`, `crates/generate/ess-synth/tests/declared_behaviour.rs`,
`crates/generate/ess-synth/tests/related_guard_obligation.rs` (the obligation test removed), `generated/*` that use `when_related`, `website/docs/guides/synthesize.md`.

## Sequencing

Last in the epic: after story:go-generated-behaviour (Go ports), story:served-store-and-entry
(shares `generated/` and `synthesize.md`), story:related-via-optional-input (shares
`declared_behaviour.rs` and `plan.rs`) and story:related-via-stored-reference (the via it
generates). `determined.rs` was changed by #310: base on 0.51.0. `CHANGELOG.md` is a merge-time
edit (epic).
