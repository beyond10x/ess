---
format: aep.planning-md/3
id: story:docs-status-pages-current
kind: story
status: active
title: The status pages report 0.38.0 and the commands that exist
relations:
- decomposes: epic:public-docs-overhaul
- serves: vision:O2
scope:
- confidence: cited
  path: website/docs/status/outlook.md
- confidence: cited
  path: website/docs/status/roadmap.md
- confidence: cited
  path: website/docs/status/where-this-stands.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T14:31:38Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-09-29T14:31:39Z", actor: "human:timo", revision: 5}
---
## Outcome

The status pages report the latest release, the commands that exist and the limits that hold today,
without wave or release-train vocabulary.

## Acceptance

- `where-this-stands.md` names the latest release and no removed command.
- `limitations.md` is reviewed against 0.43.0 and changed where it is wrong.
- `roadmap.md` and `outlook.md` carry no wave or release-train vocabulary.
- `cargo xtask support --check` and `cargo xtask docs` pass.

## Scope

- website/docs/status/*
