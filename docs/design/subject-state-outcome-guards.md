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
