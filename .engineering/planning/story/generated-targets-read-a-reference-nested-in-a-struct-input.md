---
format: aep.planning-md/3
id: story:generated-targets-read-a-reference-nested-in-a-struct-input
kind: story
status: draft
title: Generated targets read a reference nested in a struct input
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#640
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:via-and-instance-may-name-a-member-of-a-struct-input
- depends_on: story:synthesis-witnesses-a-row-addressed-through-a-struct-input-member
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/lowering.rs
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/struct_member_address.rs
- confidence: inferred
  path: website/docs/reference/entity-runtime-lowering.md
revision: 8
---
## Need

Every generated target must read a `via:` or `instance:` reference nested in a struct input, or
refuse it by name, once `story:via-and-instance-may-name-a-member-of-a-struct-input` admits it. The
generated Rust and Go behaviour look up a related row by a top-level input field: the resolved via
is one field name (`crates/specify/ess-compiler/src/ir.rs:1845-1853`), read in
`crates/generate/ess-synth/src/determined.rs:1660` (`related`). They already keep a
`List` via an obligation by name (`determined.rs:226-235`). Entity Runtime lowers related reads by
input field.

Minimal model: the parent story's `example.g.Make` with `via: input.settings.primary` and
`via: input.settings.parents`, and an update addressed by `instance: settings.primary`.

## Acceptance

- The generated Rust and Go behaviour for the model with `via: input.settings.primary` and
  `instance: settings.primary` pass the synthesized scenarios (`ess verify conform run` against
  each, every scenario passed), in a new `ess-synth` test target that builds both.
- With `via: input.settings.parents`, a list, `ess generate synthesize` for Rust and for Go keeps
  the command an obligation whose reason names the path `input.settings.parents`, as the top-level
  list does today.
- `cargo test -p ess-entity-runtime --locked --test lowering` passes a new case: lowering the
  minimal model refuses the update addressed by `instance: settings.primary` with
  `AmbiguousInstanceBinding` at the command's path, and the `via: input.settings.primary` guard
  with `RelatedGuardUnsupported`, as a top-level `via:` is refused today.
- `cargo test -p ess-entity-runtime --locked --test lowerable_subset` passes with
  `website/docs/reference/entity-runtime-lowering.md` regenerated, its `AmbiguousInstanceBinding`
  row naming an `instance:` that is a member of a struct input.
- `ess generate openapi` on the minimal model writes the operation's request body as the struct
  input, with `settings.primary` a member of it and no separate parameter for it.

## Scope

- cited: crates/generate/ess-synth/src/determined.rs
- inferred: crates/generate/ess-synth/src/rust/behaviour.rs
- inferred: crates/generate/ess-synth/src/go/behaviour.rs
- inferred: crates/generate/ess-synth/tests/struct_member_address.rs (new)
- cited: crates/generate/ess-entity-runtime/src/lib.rs (`supplied_identity`)
- cited: crates/generate/ess-entity-runtime/src/subset.rs (the `AmbiguousInstanceBinding` row)
- cited: crates/generate/ess-entity-runtime/tests/lowering.rs
- inferred: website/docs/reference/entity-runtime-lowering.md (regenerated from `subset.rs`)

confidence: low

Shared surfaces: no format row and no byte-pin row; it proposes its `CHANGELOG.md` sentence.

## Fit review

1. **Need.** A construct the parent story admits must not be ignored by a target. Reproduction: the
   parent story's model.
2. **Class.** Gap, following the parent story.
3. **Already expressible?** Not applicable.
4. **Fit.** Every target answers it or refuses it by name
   (`.agents/skills/assessing-external-requests/SKILL.md`, question 4).
5. **Second adopter.** As the parent story.
6. **Cost.** No format. Generated code changes only for models with a nested address.
7. **Alternatives.** (a) Refuse nested addresses in every generated target: rejected, because the
   generated code already decodes the struct input and reads its members for `sets:`. (b) Read the
   member: chosen. Entity Runtime refuses by name.

## Decisions

accept, redesigned, as the parent story. This story carries the generated targets.

Entity Runtime refuses the nested `instance:` rather than lowering it: its binding is
`InstanceBinding::Supplied { input_field }`, one top-level input field the host loads the instance
from (`crates/generate/ess-entity-runtime/src/lib.rs`, `supplied_identity`), which is what
`AmbiguousInstanceBinding` already says is missing.

## Source

github:beyond10x/ess#640
