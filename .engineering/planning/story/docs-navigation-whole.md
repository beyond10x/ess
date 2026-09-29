---
format: aep.planning-md/3
id: story:docs-navigation-whole
kind: story
status: draft
title: Every page is reachable and every link lands on a rendered page
relations:
- decomposes: epic:public-docs-overhaul
- serves: vision:O2
scope:
- confidence: cited
  path: website/docs/concepts/ess.md
- confidence: cited
  path: website/docs/examples/specification-to-contracts.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/guides/record-realization.md
- confidence: cited
  path: website/sidebars.ts
revision: 3
---
## Outcome

The site is organized for a first-time adopter: Start here (install, two tutorials, use with an
agent), Concepts, Guides split by task, Reference (CLI, diagnostics, formats, format history,
predicates, glossary), Examples, Releases and Status, collapsed by default. Every moved URL still
lands on a rendered page on both `/ess/` and `/docs/ess/`.

## Acceptance

- `website/sidebars.ts` has the categories above; `guides/write-a-specification.md` and
  `guides/verify-conformance.md` are split into task pages of at most 400 lines each.
- Every moved URL has a client redirect and a matching `b10x.docs.yaml` entry; `task site-build`
  passes with `onBrokenLinks: throw`.
- `atlas docs reconcile --workspace . --check` passes from a clean Atlas checkout at remote `main`.

## Scope

- website/sidebars.ts
- website/docusaurus.config.ts
- b10x.docs.yaml
- website/docs/guides/* (split)
