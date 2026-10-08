---
format: aep.planning-md/3
id: story:generated-bundle-doc-names-only-emitted-traits
kind: story
status: draft
title: The Generated<P> doc names only the context traits the synthesis emits
tags:
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The doc comment ess-synth writes on `Generated<P>` names only the traits the same synthesis
emits: `TryContext` and the legacy `Context` blanket adapter are mentioned only when they are
generated.

## Evidence

An adopter's regenerated Rust crate (ess 0.56.0, `behaviour.rs:44`) documents `Generated<P>` as
implementing "`TryContext` (or its legacy `Context` blanket adapter)", while the same synthesis
emits neither once its context-asking commands become obligations (commands with an `external:`
outcome). The text is the constant `GENERATED` in
`crates/generate/ess-synth/src/rust/behaviour.rs` (the `Generated<P>` doc, "`TryContext` (or its
legacy `Context` blanket adapter) where one asks it anything"), written whatever the plan emits;
the trait itself is written at the `pub trait TryContext` emitter in the same file.

## Acceptance

- A plan that emits no `TryContext` produces a `Generated<P>` doc that does not name it or the
  `Context` adapter; a plan that emits them keeps today's text and bytes.
- A generated-crate test pins both cases, and the byte-pinned generated crates in the repository
  change only where they emitted no `TryContext`.
- Documentation only: no type, trait or signature changes.
