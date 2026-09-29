---
format: aep.planning-md/3
id: story:docs-releases-current
kind: story
status: implemented
title: Every release has a post and a generated what-changed page
relations:
- decomposes: epic:public-docs-overhaul
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T14:28:11Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T14:28:11Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T16:21:14Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The site's Releases section has a post for every release and a What-changed page generated from
`changes/*.yaml`, so an adopter reads what each release changed without opening the repository.

## Acceptance

- A 0.43.0 release post exists; `cargo xtask docs` rule (e) passes.
- `cargo xtask whats-changed` also writes a site page; `--check` fails when it is stale.
- The page is linked from the Releases navigation on both renderings.

## Scope

- crates/edge/ess-xtask/src/whats_changed.rs
- website/blog/
- website/docs/ What-changed page (new)
