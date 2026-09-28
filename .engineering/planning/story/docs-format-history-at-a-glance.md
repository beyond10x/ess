---
format: aep.planning-md/3
id: story:docs-format-history-at-a-glance
kind: story
status: draft
title: The format history has one table for every ess/ version and no internal round names
relations:
- decomposes: epic:public-docs-overhaul
scope:
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 2
---
## What

The format history gets one table covering every `ess/` version, so a reader can find the version a
construct needs without reading seventeen paragraphs. The "round-3" and "retrofit issues" wording in
the format history and the formats reference is replaced by what the versions carry.

## Acceptance

- `spec-versions.md` has a table with a row for each of `ess/1` through `ess/17`.
- `grep -n 'round-3\|retrofit' website/docs/reference` prints nothing.
- `cargo xtask docs` passes.

## Scope

- website/docs/reference/spec-versions.md
- website/docs/reference/formats.md
