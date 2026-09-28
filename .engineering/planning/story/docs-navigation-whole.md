---
format: aep.planning-md/3
id: story:docs-navigation-whole
kind: story
status: draft
title: Every page is reachable and every link lands on a rendered page
relations:
- decomposes: epic:public-docs-overhaul
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
revision: 2
---
## What

Every guide is reachable from the sidebar, links to the worked example resolve to a rendered page
on both sites, the concept page's "same pattern" section says what it shows, and the worked example
describes the HTML `site` output and the files that exist.

## Acceptance

- `guides/record-realization.md` appears in `sidebars.ts`.
- The example page cites no file absent from `generated/`.
- `task site-build` passes with no broken-link warning.

## Scope

- website/sidebars.ts
- website/docs/concepts/ess.md
- website/docs/guides/generate-artifacts.md
- website/docs/guides/record-realization.md
- website/docs/examples/specification-to-contracts.md
