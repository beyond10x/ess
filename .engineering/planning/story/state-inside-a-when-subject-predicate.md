---
format: aep.planning-md/3
id: story:state-inside-a-when-subject-predicate
kind: story
status: active
title: A branch cannot be selected by the held state and a stored field together ("a subject fact has one selection authority")
refs:
- provider: github
  reference: beyond10x/ess#204
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:49Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:49Z", actor: "human:timo", revision: 3}
---
# Story: A branch cannot be selected by the held state and a stored field together ("a subject fact has one selection authority")

## Why

beyond10x/ess#204. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #204 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

The defect is still present on this tree (`ess-scope-next` at `e932781965`, release 0.40.0). There is no fix, and the fix needs a design choice first.

- **Refusal 1** (cited): `crates/specify/ess-domain/src/command.rs:4707-4721`. When `when_subject` is set beside `when_subject_state`, `when_state_changes`, `external` or `wrong_state`, the code returns `conflicting_declaration` "a subject fact has one selection authority". This matches the issue's first refusal word for word.
- **Refusal 2** (cited): `crates/specify/ess-domain/src/command/subject_fact.rs:286`. The `when_subject` predicate is checked against `DomainEnvironment::new(types, &entity.fields)`. That environment has no `state` pseudo-field, so `crates/specify/ess-domain/src/expression.rs:529` raises `unobservable_fact`. This matches the second refusal.
- **Third barrier** (cited): `subject_fact.rs:171-184` refuses a command where any branch reads a stored field while another branch reads the held state ("subject fact and lifecycle guards cannot be combined in one command"). Option A must lift this. Option B can leave it.
- **The refusals are designed behaviour** (cited): `docs/design/cross-record-and-stored-field-guards.md:162-164,184,186` excludes `state` from `when_subject` and refuses the combination. The fix is a new format feature, and that design page has to be amended.
- **Model** (cited): `OutcomeCondition::SubjectPredicate {predicate, input}` at `command.rs:413` has no state slot, and `SubjectState {state, predicate}` at `command.rs:420` has no stored-field slot. The document writer at `command.rs:5141-5160` maps each variant to one key.
- **A `state` operand already exists** (cited): entity invariants check expressions against `observable_fields()`, which includes `state` (`crates/specify/ess-domain/src/entity.rs:866-868`).

**Where the fix lands**
- **Option B, "admit `state` in the `when_subject` predicate"** (inferred): `subject_fact.rs::check` switches to the observable-fields environment, behind a new format version (`crates/specify/ess-domain/src/primitive_admission.rs`). The partition in `subject_fact.rs` treats `state` as a closed enum. Every evaluator that reads the stored row must also resolve `state`: `crates/verify/ess-conformance/src/decision.rs`, `crates/verify/ess-conformance/src/interpret/execute.rs`, `crates/generate/ess-entity-runtime/src/lib.rs`, `crates/generate/ess-gen/src/http.rs`. On the synthesis side, `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (`grounded` ~384, `search`/`successors` ~1021-1151) must arrange and observe the held state as part of the row.
- **Option A, "both keys on one branch"** (inferred): add `state: Option<StateName>` to `SubjectPredicate`. That touches `command.rs` (4707 guard, 4760 construction, 5141 writer), relaxes `subject_fact.rs:171`, touches the `command/subject_state.rs` partition (lines 67, 196, 259), and `crates/specify/ess-compiler/src/{ir,resolve}.rs`. The synthesis work is the same as for B.
- **Tests** (inferred): `crates/specify/ess-domain/tests/stored_field_guards.rs`, `crates/specify/ess-compiler/tests/stored_field_guards_ir.rs`, `crates/generate/ess-synth/tests/stored_field_guards.rs`, `crates/generate/ess-gen/tests/stored_field_guards.rs`, `crates/verify/ess-diff/tests/stored_field_guards.rs`.

**Collisions with the other issues**
- **#201** (inferred, high): the per-state `wrong_state` answer changes the same `command.rs` 4540-4600 and 4707 conflict logic and `command/subject_state.rs`. Sequence #201 and #204 one after the other, or give both to one implementor.
- **#198 and #199** (inferred, high): both are about finding the row for a `when_subject` branch in `synthesize/subject_fact.rs`, the same search and successors code.
- **#211** (inferred, medium): a guard over another entity is a new condition kind in `command.rs` `OutcomeCondition` and `outcome_condition`.
- **#209** (inferred, low): arranging a refusal is also in `synthesize/subject_fact.rs` (`refusal_input` ~629).
- **None expected** (inferred): #196, #210, #202, #203, #195, #193, #205.

**Design decision for the implementor**
- **Pick A or B.** I recommend B (inferred): it reuses the invariant environment's `state` pseudo-field and adds no enum variant or key. It also leaves the one-selection-authority rule standing. The costs are a format bump, `state` handling in every row evaluator, and synthesis arranging the held state for a row.
- **Precedence to settle either way** (inferred): does `wrong_state` answer before a `when_subject` predicate that reads `state`? The answer depends on how #201 is resolved.

Confidence: high on the defect and where the refusals come from (both cited). Medium on the list of evaluators that must change (grep only, no build).

Paths:
- crates/specify/ess-domain/src/command.rs
- crates/specify/ess-domain/src/command/subject_fact.rs
- crates/specify/ess-domain/src/command/subject_state.rs
- crates/specify/ess-domain/src/expression.rs
- crates/specify/ess-domain/src/entity.rs
- crates/specify/ess-domain/src/primitive_admission.rs (inferred)
- crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- crates/verify/ess-conformance/src/decision.rs (inferred)
- crates/verify/ess-conformance/src/interpret/execute.rs (inferred)
- crates/generate/ess-entity-runtime/src/lib.rs (inferred)
- crates/generate/ess-gen/src/http.rs (inferred)
- crates/specify/ess-compiler/src/ir.rs (inferred, Option A only)
- crates/specify/ess-compiler/src/resolve.rs (inferred, Option A only)
- docs/design/cross-record-and-stored-field-guards.md

Verdict: needs-design
