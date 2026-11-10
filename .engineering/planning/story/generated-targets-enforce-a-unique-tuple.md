---
format: aep.planning-md/3
id: story:generated-targets-enforce-a-unique-tuple
kind: story
status: active
title: Generated targets enforce a unique tuple
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#638
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-identity-may-be-a-composite-of-fields
- depends_on: story:synthesis-witnesses-a-unique-tuple-collision
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/unique_tuple.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/http.rs
- confidence: inferred
  path: crates/generate/ess-openapi/tests/unique_tuple.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/determined.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/unique_tuple.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/change.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/compatibility.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/delta.rs
- confidence: inferred
  path: crates/verify/ess-diff/tests/unique_tuple.rs
- confidence: inferred
  path: website/docs/reference/entity-runtime-lowering.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:44:50Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:44:51Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Every target must enforce an entity's `unique: [fields]` (`story:an-identity-may-be-a-composite-of-fields`)
or refuse the model by name. The record says: "servers and the interpreter check the tuple on create
and on any update that writes a member; the storage port gains no method"
(`architecture-decision-record:a-compound-key-is-a-uniqueness-constraint-first`, § Effect). The
projections must state the constraint, and `ess verify diff` must classify a change to it.

Minimal model: the parent story's `example.g.Grant` with `unique: [account_id, app_id]`, and
`example.g.Bind`.

## Acceptance

Every bullet runs on the minimal model.

- Generated Rust and Go servers pass the synthesized `example.g.Bind/outcome/taken` and
  `/bound` scenarios under `ess verify conform run` against each; tested in
  `cargo test -p ess-synth --locked --test unique_tuple`.
- `ess generate openapi` answers the `existing_instance:` refusal of `example.g.Bind` with `409`,
  as for a duplicate identity; tested in `cargo test -p ess-openapi --locked --test unique_tuple`.
- The generated documentation lists the tuple `[account_id, app_id]` on `example.g.Grant`; tested in
  the same target.
- Entity Runtime lowering refuses `example.g.Grant` under the new lowering code
  `UniqueTupleUnsupported`, naming the entity's `unique:` path, and
  `website/docs/reference/entity-runtime-lowering.md` lists the code; tested in
  `cargo test -p ess-entity-runtime --locked --test unique_tuple` and the existing
  `lowerable_subset` target.
- `ess verify diff` from the model without `unique:` to the model with it reports `unique-added`
  with callers compatibility `breaking`; tested in `cargo test -p ess-diff --locked --test unique_tuple`.
- The same diff in reverse reports `unique-removed` with readers compatibility `breaking`; tested in
  the same target.

## Scope

This story moves only the `CHANGELOG.md` shared surface.

- inferred: crates/generate/ess-synth/src/rust/behaviour.rs
- inferred: crates/generate/ess-synth/src/go/behaviour.rs
- inferred: crates/generate/ess-synth/src/determined.rs
- inferred: crates/generate/ess-gen/src/http.rs
- cited: crates/generate/ess-entity-runtime/src/subset.rs
- cited: crates/generate/ess-entity-runtime/src/lib.rs
- inferred: crates/verify/ess-diff/src/delta.rs
- inferred: crates/verify/ess-diff/src/change.rs
- inferred: crates/verify/ess-diff/src/compatibility.rs
- inferred: website/docs/reference/entity-runtime-lowering.md
- inferred: crates/generate/ess-synth/tests/unique_tuple.rs
- inferred: crates/generate/ess-openapi/tests/unique_tuple.rs
- inferred: crates/generate/ess-entity-runtime/tests/unique_tuple.rs
- inferred: crates/verify/ess-diff/tests/unique_tuple.rs

confidence: low

## Fit review

1. **Need.** A constraint the specification declares must hold in every generated server.
   Reproduction: the parent story's model.
2. **Class.** Gap, following the parent story.
3. **Already expressible?** No.
4. **Fit.** Every target handles it or refuses it by name
   (`.agents/skills/assessing-external-requests/SKILL.md`, question 4). The answer reuses the
   duplicate-identity `409`.
5. **Second adopter.** As the parent story.
6. **Cost.** Two new `ess verify diff` kinds. Generated code changes only for models with `unique:`.
7. **Alternatives.** (a) Refuse `unique:` in generated servers: rejected, because the check is a
   lookup the storage port already serves. (b) Enforce it: chosen. Entity Runtime may refuse by name.

## Decisions

accept, redesigned, as the parent story. This story carries the generated targets, the projections
and the diff.

- Entity Runtime refuses `unique:` by name rather than lowering it: the runtime already refuses a
  branch chosen by whether a record exists (`ExistenceSelectionUnsupported` in
  `crates/generate/ess-entity-runtime/src/subset.rs`, needing the host's own exists answer), and a
  tuple check needs the same lookup across records.

## Source

github:beyond10x/ess#638
