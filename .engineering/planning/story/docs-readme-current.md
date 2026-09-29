---
format: aep.planning-md/3
id: story:docs-readme-current
kind: story
status: implemented
title: The README installs the current release and describes site as HTML
relations:
- decomposes: epic:public-docs-overhaul
- serves: vision:O2
scope:
- confidence: cited
  path: README.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T14:37:35Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-09-29T14:37:35Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-09-29T14:46:24Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`README.md` installs the current release, describes `site` as HTML, and links the site's Start
here pages instead of repeating them.

## Acceptance

- `README.md` names no release older than the newest dated `CHANGELOG.md` heading as the one to
  install, and `cargo xtask docs` checks that.
- No sentence in `README.md` says the `site` projection does not emit HTML.
- The command examples `README.md` carries that the site lacks (`project buildkit`, `project
  helm`, `stack resolve`, `deployment compile`, `runtime compile`) are on the site and linked.

## Scope

- README.md
- crates/edge/ess-xtask/src/docs.rs
