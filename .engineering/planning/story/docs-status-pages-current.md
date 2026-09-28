---
format: aep.planning-md/3
id: story:docs-status-pages-current
kind: story
status: draft
title: The status pages report 0.38.0 and the commands that exist
relations:
- decomposes: epic:public-docs-overhaul
scope:
- confidence: cited
  path: website/docs/status/outlook.md
- confidence: cited
  path: website/docs/status/roadmap.md
- confidence: cited
  path: website/docs/status/where-this-stands.md
revision: 2
---
## What

The status pages report 0.38.0 as the latest release, stop listing the removed `ess skill`, and drop
internal planning vocabulary. The generated support matrix block is left to `cargo xtask support`.

## Acceptance

- `where-this-stands.md` names 0.38.0 as the latest release and does not mention `skill`.
- `roadmap.md` and `outlook.md` carry no wave or release-train vocabulary.
- `cargo xtask support --check` and `cargo xtask docs` pass.

## Scope

- website/docs/status/where-this-stands.md
- website/docs/status/roadmap.md
- website/docs/status/outlook.md
