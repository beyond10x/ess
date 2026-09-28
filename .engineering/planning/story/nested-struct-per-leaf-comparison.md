---
format: aep.planning-md/3
id: story:nested-struct-per-leaf-comparison
kind: story
status: draft
title: A nested struct target is compared leaf by leaf
refs:
- provider: github
  reference: beyond10x/ess#179
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

A nested `sets:`/payload mapping with one `{generated: true}` leaf still asserts every determined leaf, and presence and type for the generated one. The suite-format step that `value-expressions.md` E5 defers.

## Acceptance

The #179 shape (four input leaves, one generated) yields a scenario that fails when an implementation drops `lead.number`.
