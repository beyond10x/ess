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
revision: 3
---
## Outcome

Every format family a public page names is tracked by `FORMAT_RELEASES`, so the check that no page
calls a released format "unreleased" covers all of them; the format history reads without internal
round names and has one table per family.

## Acceptance

- `cargo xtask docs` scans every `<family>/<n>` named under `website/docs` and fails for one that
  `FORMAT_RELEASES` does not track (a test plants one).
- The ~33 families found untracked on 2026-09-29 (ess-build, ess-runtime, ess-realization, ess-docs,
  ess-stack, ess-deployment, ess-environment, ess-release, infra-*, …), `ess-cli*/1` and the
  `ess-execution-*` files behind `deployment reconcile --authority` are tracked and named in
  `formats.md` or `spec-versions.md`.
- `spec-versions.md` has a table row for each of `ess/1` through `ess/18`.
- `grep -n 'round-3\|retrofit' website/docs/reference` prints nothing.

## Scope

- crates/edge/ess-xtask/src/docs.rs
- website/docs/reference/formats.md
- website/docs/reference/spec-versions.md
