---
format: aep.planning-md/1
id: story:subject-state-outcome-guards
kind: story
status: active
title: Define subject-state outcomes from a verified behavior witness
tags:
- priority-high
relations:
- decomposes: task:ess-gaps-measured-in-a-consumer-specification
- serves: vision:O2
- depends_on: story:closed-enum-outcome-coverage
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-gen/src/docs.rs
- confidence: cited
  path: crates/generate/ess-gen/src/http.rs
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/finite.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/subject_state.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/spec.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/subject_state.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/subject_state_adversary.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/subject_state_open_default_adversary.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/decision.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/subject-state-runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/subject-state.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/tests/subject_state.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: docs/design/subject-state-outcome-guards.md
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 5
---
## Outcome and priority

High-priority design and evidence checkpoint for gap 3, followed by the justified bounded capability. Confirm what the consumer actually does before implementing a held-state branch as its purported fix.

## Established capability and corrected witness

OutcomeCondition supports When, Otherwise, External and WrongState; typed When reads command input (ess-domain/src/command.rs:321,1395). ArrangeState already exists specifically for WrongState (:387). Wrong-state validation uses the complement of permitted transition-source states (entity.rs:1040), not selection among permitted states. Resolved conditions and conformance decision/input search have corresponding typed seams.

The consumer specification distinguishes six guarded report branches from unconditional confirmed/enriched updates. However, inspected reducers assign cs.State = newState directly after creating an absent record; they do not explicitly choose the two named branches by comparing old/new state. Establish whether the intended domain needs subject guards, direct reconciliation to reported state, or a corrected lifecycle model. Language absence is confirmed; that particular implementation branch is not.

## Acceptance

This story is complete when a verified held-state behavior witness selects different correct outcomes for identical inputs from different established states, its guard-ignoring target fails, and the matching consumer declarations and compatibility checks below pass. If investigation instead establishes model error, record and verify the correction and make an explicit reviewed disposition of this capability request; that investigation is not an implemented guard and does not satisfy the guard checks below. Until that disposition or a justified witness exists, keep the capability request open.

Required verification:

- First record the actual consumer held-state observation/identity and behavior witness. Do not infer a branch from a summary, fabricate fault injection, or alter producer values to satisfy a spec.
- If justified, design bounded equality against a declared subject lifecycle state, composition with input guards and absent-subject behavior. Refuse undeclared states, wrong/missing subjects or reads from another entity/view; do not redefine External/WrongState silently.
- For the justified guard-capability path, identical inputs with different established subject states select the correct outcomes. Use the shared bounded closed-domain coverage decision for exact input/state combinations; incomplete or unsupported coverage remains explicit.
- Establish required state via real reachable transitions or a separately adopted setup capability; no ConfigureExternalOutcome shortcut. A target ignoring the guard fails its witness.
- Account for source/IR condition semantics, previous readers, provenance, docs/HTTP/OpenAPI, semantic diff and native/conformance planning. No new persisted condition hidden under old-format meaning.
- Rework the two consumer commands only after matching actual reducer semantics. Their separate input-coverage correction does not prove held-state branching.

## Dependencies

Depends on story:closed-enum-outcome-coverage for the common finite input/state coverage proof. Typed authored arrangement is optional when existing create/move paths establish the witness; no artificial dependency on it. The design/consumer checkpoint starts before runtime implementation and does not stall independent compact/read-setup work.

## Historical source

Archived argument: story:outcome-selected-by-the-subjects-state. Preserved original snapshot and the corrections above define the new unit.


## Provenance and delivery

Decomposes task:ess-gaps-measured-in-a-consumer-specification under initiative:ess-evolution. Prioritized by the operator on 2026-09-11. Source-only scoping at ESS dcdc3343 and observed consumer commit 2497faf27959b59b6eb0700829bb321f851adf1c is retained at local-evidence:ess-evolution-20260910/priority-wave/gap-scoping/state-views.md. Original archived argument bytes/hashes remain under gap-scoping/source. Counts and historical test results are inherited evidence, not newly executed checks. Archived source entries stay terminal; this distinct unit records the maintainer's assessed implementation contract. Overlapping accessor source edits must be integrated or explicitly isolated before dispatch; that collision does not require completion of its downstream adoption acceptance. No full local/ownership gate or unchanged test reruns. Consumer-only PR pipeline failures are accepted; core feature evidence remains required.

## Scope

- Cited: crates/specify/ess-domain/src/command.rs.
- Cited: crates/specify/ess-domain/src/entity.rs.
- Cited: crates/specify/ess-compiler/src/ir.rs.
- Cited: crates/specify/ess-compiler/src/resolve.rs.
- Cited: crates/verify/ess-conformance/src/decision.rs.
- Cited: crates/verify/ess-conformance/src/input.rs.
- Cited: crates/verify/ess-conformance/src/synthesize.rs.
- Cited: crates/verify/ess-conformance/tests/synthesis.rs.
- Cited: crates/generate/ess-gen/src/docs.rs.
- Cited: crates/generate/ess-gen/src/http.rs.
- Cited: crates/generate/ess-gen/src/openapi.rs.
- Cited: crates/generate/ess-synth/src/plan.rs.
- Cited: crates/verify/ess-diff/src/diff.rs.
- Inferred: docs/design/subject-state-outcome-guards.md.

## Verified source checkpoint for a held-state witness

A direct production guard was located in the same consumer's call reducer: an incoming Ringing state updates an existing Init call, but preserves an existing Bridged call, while field enrichment continues in both cases. The call specification already documents that protection but its input-only ringing outcome cannot select by the held state. Exact private source positions/digests are retained in local-evidence:ess-evolution-20260910/priority-wave/gap-scoping/subject-state-real-witness.md.

Use this actual branch for the first justified subject-guard design and deciding runtime witness; do not invent a held-state distinction in the two unconditional campaign reducers. The latter still need the separately stated honest model/reconciliation treatment. Source inspection is not runtime proof: actual reducer and guard-ignoring target cases remain required before completion. No source capability or consumer declaration was changed by this checkpoint.

# Outcome selection from the held subject state

## Behavior and authority

An incoming Ringing report moves an existing Init call to Ringing but preserves an
existing Bridged call. Both paths enrich the same row. This branch was read from a
production reducer; the private source inventory is retained separately under
`local-evidence:priority-wave/gap-scoping/subject-state-real-witness.md`. The first
runtime witness must exercise identical input against those two established states,
and reject an implementation that always applies the incoming state.

Two other inspected reducers reconcile their reported state unconditionally. This
feature does not invent subject-state branching in those reducers, nor claim that
their input-coverage correction proves this feature's adoption.

## Declaration

An outcome may add `when_subject_state: Bridged` beside its existing input `when:`.
The two conditions are conjunctive. Omitting `when:` means any admitted input in
the named held state. The state is a declared lifecycle name, not a caller field.
It refers to the existing entity and identity already named by this outcome's
`moves:` or `updates:` and `instance:` declarations. The declaration is refused on
`creates:`, an outcome without a subject, or beside `external:` or `wrong_state:`.

All input-selectable outcomes of a subject-guarded command must name the same
existing entity and the same command-input identity field. This includes its
ordinary/default branches: one command cannot select against unrelated subjects.
A guarded move must admit its guard state in the transition's `from` set. The
state and identity are read from the target immediately before command selection.
An absent subject cannot satisfy this contract and is unavailable or reaches no declared outcome;
it is not implicitly created or treated as an initial-state row. This bounded
extension declares no absent-subject outcome, cross-entity read, view query, field
comparison, or arbitrary effect.

The legacy input namespace remains unchanged. In particular, an existing input
field named `subject` never acquires hidden meaning. An ordinary `when:` remains
an input predicate; a separate condition variant carries the state constraint.
WrongState retains its complement-of-move-sources meaning and External retains
its fault-injection meaning. A command using explicit state guards cannot also
declare WrongState: overlapping precedence is not inferred.

## Shared finite proof

Validation enumerates the Cartesian product of declared held states and the
closed input domains admitted by the existing finite guard checker. It shares
that checker's equality, membership, Boolean composition, type authority, truth
evaluation and resource bounds. State-only guards admit one empty input assignment.
The combined state/input table is capped at 64 assignments and guard syntax at
128 nodes; unsupported/open input guards still require a genuine default.

With no default, every admitted pair must select exactly one ordinary/state guard.
Uncovered pairs and overlapping pairs are named in validation diagnostics.
A real default selects only pairs for which no guard holds. Concrete witness
inputs are additionally validated against their declared types and invariants;
finite declared-domain coverage does not claim invariant inhabitation.

## Execution and conformance

A new internal test strategy constructs both an input and a held state. A scenario
establishes that state through reachable create/move commands or the typed entity
setup capability, verifies its identity/state, and then executes the command.
The input search decides every competing guard against the same established state.
For a default, it considers the held state too; input-only search cannot prove it.
The generated scenario records the entity/lifecycle and outcome dependencies.
The initial observation adapter uses a declared unfiltered immediate view exposing
both identity and lifecycle state; absent, eventual-only or insufficient projections
produce a typed synthesis refusal. Identity/state are checked before and after the
command using ordinary view assertions. Existing command-path arrangement is used
first; typed authored setup remains available for separate authored witnesses.
The runner observes the actual selected outcome and resulting row. It does not
inject the expected outcome or accept a target-supplied verdict as proof.

Native planning and docs/HTTP/OpenAPI projections carry the explicit state
condition. Rust and Go conformance witnesses must include successful execution,
absent/wrong identity refusal and a guard-ignoring mutation that fails. Unsupported
targets remain explicit refusals, not a substitute for the successful witness.

## Persisted formats and compatibility

The domain and resolved IR gain an explicit subject-state condition. Authored source
requires the wave's `ess/3` allocation when this condition occurs;
unchanged input-only models retain their existing identity and bytes. The compiled
subject-state witness uses existing command and view assertions with unchanged
meanings, so it needs no new suite vocabulary or version by itself. An authored
witness using typed setup requires the wave's extended suite 6/7 formats. Prior
ESS source readers refuse `ess/3`; prior suite readers may execute the ordinary
projected assertions because those assertions retain their meaning. No separate
version 8/9 allocation is introduced here.

Old-reader refusal, condition serialization, provenance/diff changes, correct
state/input pairing and ordinary-model byte preservation are deciding checks.
This page is the implementation contract; the initial source observation alone
does not establish runtime conformance or completed consumer adoption.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 ec45416ac72718f068dfef493a7d16fbba009c67e5edf1c091bb156a6c9e18b6, retained as local-evidence:runtime-gaps/publication-replay/snapshots/ec45416ac72718f068dfef493a7d16fbba009c67e5edf1c091bb156a6c9e18b6.md. Source creation recorded at 2026-09-11T00:26:30Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
