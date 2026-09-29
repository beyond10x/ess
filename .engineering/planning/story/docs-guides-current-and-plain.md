---
format: aep.planning-md/3
id: story:docs-guides-current-and-plain
kind: story
status: draft
title: The guides and CLI reference name current releases and read without internal shorthand
relations:
- decomposes: epic:public-docs-overhaul
scope:
- confidence: cited
  path: website/docs/guides/synthesize.md
- confidence: cited
  path: website/docs/guides/track-change.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
- confidence: cited
  path: website/docs/reference/cli.md
revision: 3
---
## Outcome

Every guide and concept page describes 0.43.0 in plain words: no stale release, no removed
command, no internal wave or round vocabulary, and a page for each shipped command family.

## Acceptance

- `concepts/overview.md` and `concepts/component-delivery.md` are reviewed against 0.43.0 and
  changed where they are wrong.
- Guides exist for `generate project buildkit|helm`, `stack validate`, `deployment
  diff|reconcile` and `verify conform author|select`.
- Adopter guidance from `docs/design/running-a-suite.md` (the test pyramid),
  `error-wire-codes.md` and `mutation-audit-and-model-runner.md` is on the site.
- `grep -rniE 'wave [0-9]|W[0-9]+\.[0-9]|docs/plan/|release train' website/docs` prints nothing.
- `cargo xtask docs` and `task site-build` pass.

## Scope

- website/docs/concepts/*, website/docs/guides/*
