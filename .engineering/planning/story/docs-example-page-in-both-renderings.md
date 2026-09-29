---
format: aep.planning-md/3
id: story:docs-example-page-in-both-renderings
kind: story
status: draft
title: The example page renders on both sites and its snippets are checked
relations:
- decomposes: epic:public-docs-overhaul
- serves: vision:O2
revision: 1
---
## Outcome

"A specification and its contracts" renders on both `/ess/` and the unified `/docs/ess/` site, and
every snippet it copies out of `generated/` is checked byte for byte.

## Acceptance

- The page is plain Markdown; the lab embed lives on its own page that only `/ess/` builds.
- `b10x.docs.yaml` no longer excludes the page.
- A test compares each copied snippet with the file under `generated/` it names and fails on drift.

## Scope

- website/docs/examples/specification-to-contracts.md
- website/src/pages/lab (embed page)
- crates/edge/ess-cli/tests/ (snippet test, new)
