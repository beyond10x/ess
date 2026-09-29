---
format: aep.planning-md/3
id: story:docs-guides-current-and-plain
kind: story
status: implemented
title: The guides and CLI reference name current releases and read without internal shorthand
relations:
- decomposes: epic:public-docs-overhaul
- serves: vision:O2
scope:
- confidence: cited
  path: website/docs/guides/synthesize.md
- confidence: cited
  path: website/docs/guides/track-change.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
- confidence: cited
  path: website/docs/reference/cli.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T14:31:37Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-09-29T14:31:38Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-09-29T16:21:02Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
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
