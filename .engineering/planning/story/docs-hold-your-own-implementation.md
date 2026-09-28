---
format: aep.planning-md/3
id: story:docs-hold-your-own-implementation
kind: story
status: draft
title: The conformance guide shows how to hold your own implementation to the suite
relations:
- decomposes: epic:public-docs-overhaul
scope:
- confidence: cited
  path: website/docs/guides/verify-conformance.md
revision: 2
---
## What

`guides/verify-conformance.md` gains a section on holding your own implementation to the suite: the
Go and TypeScript packages `ess verify conform synthesize --target go|typescript` writes, the
nine-method `Target`, `ErrUnsupported`, and the report. It replaces the Rust-only "Add a target"
paragraph. The input-selection section is rewritten in plain terms. The exploration and history
sections are not touched.

## Acceptance

- The new section names the generated files and methods exactly as `ess` 0.38.0 writes them.
- No line of the "Explore random command sequences" section changes.
- `task site-build` passes.

## Scope

- website/docs/guides/verify-conformance.md
