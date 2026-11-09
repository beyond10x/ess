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
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: website/docs/reference/entity-runtime-lowering.md
revision: 4
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
  `instance: settings.primary` pass the synthesized scenarios (`ess verify conform run` against each).
- With `via: input.settings.parents`, a list, both keep the command an obligation. Its reason names
  the list path, as the top-level list does today.
- Entity Runtime lowering of the minimal model either lowers the nested read or refuses it with a
  named lowering code listing the outcome, and `website/docs/reference/entity-runtime-lowering.md`
  names it.
- `ess generate openapi` for the model is unchanged except where the IR change is visible. The
  operation's request body stays the struct input.

## Scope

- cited: crates/generate/ess-synth/src/determined.rs
- inferred: crates/generate/ess-synth/src/rust/behaviour.rs
- inferred: crates/generate/ess-synth/src/go/behaviour.rs
- inferred: crates/generate/ess-entity-runtime/src/subset.rs
- inferred: website/docs/reference/entity-runtime-lowering.md
confidence: low

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
   member: chosen. Entity Runtime may refuse by name.

## Decisions

accept, redesigned, as the parent story. This story carries the generated targets.

## Source

github:beyond10x/ess#640
