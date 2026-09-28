---
format: aep.planning-md/3
id: story:docs-first-run-walkthrough
kind: story
status: draft
title: The walkthrough goes from an empty directory to a green conformance run
relations:
- decomposes: epic:public-docs-overhaul
scope:
- confidence: cited
  path: website/docs/getting-started.md
- confidence: cited
  path: website/docs/index.md
revision: 2
---
## What

The walkthrough takes a reader from an installed `ess` to a one-file specification they wrote, an
OpenAPI document generated from it, and a green conformance run of a generated TypeScript package
against a small in-memory implementation. The repository example stays as the second half. The
introduction page stops claiming that `site` stops before HTML.

## Acceptance

- Every command in the new walkthrough section was run against `ess` 0.38.0 and its printed output
  is copied from that run; the TypeScript run ends `report: passed, 6 scenario(s)`.
- `cargo xtask docs` passes (the install section names 0.38.0 and nothing older).
- `index.md` describes `site` as HTML.

## Scope

- website/docs/getting-started.md
- website/docs/index.md
