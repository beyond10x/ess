---
format: aep.planning-md/3
id: story:generated-targets-enforce-a-unique-tuple
kind: story
status: draft
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
revision: 2
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

- Generated Rust and Go servers for the minimal model pass the synthesized
  `example.g.Bind/outcome/taken` and `/bound` scenarios (`ess verify conform run` against each).
- `ess generate openapi` answers the `existing_instance:` refusal of `example.g.Bind` with `409`,
  as for a duplicate identity, and the generated documentation lists the tuple on the entity.
- Entity Runtime lowering either lowers the check or refuses the entity with a named lowering code,
  listed in `website/docs/reference/entity-runtime-lowering.md`.
- `ess verify diff` between the model with and without `unique:` reports `unique-added` as
  breaking for callers. The reverse reports `unique-removed` as breaking for readers.

## Scope

- inferred: crates/generate/ess-synth/src/rust/behaviour.rs
- inferred: crates/generate/ess-synth/src/go/behaviour.rs
- inferred: crates/generate/ess-synth/src/determined.rs
- inferred: crates/generate/ess-gen/src/http.rs
- inferred: crates/generate/ess-entity-runtime/src/subset.rs
- inferred: crates/verify/ess-diff/src/delta.rs
- inferred: website/docs/reference/entity-runtime-lowering.md
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

## Source

github:beyond10x/ess#638
