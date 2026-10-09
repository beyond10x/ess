---
format: aep.planning-md/3
id: story:rust-binding-wraps-a-plain-copy-into-an-optional-member
kind: story
status: draft
title: The Rust target wraps a plain binding copy into an Optional member
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The Rust synthesis target emits an event binding that copies a plain value into an Optional
member (wrapping it, `Some(value.clone())`), as the Go target already does, instead of refusing it.

## Evidence

An adopter on ess 0.56.0: the Rust target refuses "binding `<b>` emits a plain clone of `<Doc>`
for `payload` of type `Optional<…>`", and the same for a topic-to-channel member of type
`Optional<Channel>`; Go synthesizes the same model (56 capabilities, 0 refused).

On `main`, `crates/generate/ess-synth/src/rust/feasibility.rs:852-880` (`bindings`) raises
`BindingAssignment` when there is no conversion, the source and target types differ and the value
is not proved present. The only mismatch it accepts is the unwrapping direction (an Optional
source the binding's condition proves present); wrapping a plain source into an Optional target
is never emitted.

## Acceptance

- A Rust synthesis test: a binding that copies a `Doc` into an `Optional<Doc>` payload member and
  a `Channel` into an `Optional<Channel>` member synthesizes, compiles, and emits `Some` of the
  copied value; the Go target's output for the same model is unchanged.
- A mismatch that is neither wrapping nor proved unwrapping is still refused.
