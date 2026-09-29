---
format: aep.planning-md/3
id: story:docs-first-run-walkthrough
kind: story
status: active
title: The walkthrough goes from an empty directory to a green conformance run
relations:
- decomposes: epic:public-docs-overhaul
- serves: vision:O2
scope:
- confidence: cited
  path: website/docs/getting-started.md
- confidence: cited
  path: website/docs/index.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T14:11:13Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-09-29T14:11:13Z", actor: "human:timo", revision: 5}
---
## Outcome

The site's tutorials take an adopter from an empty directory to a green conformance run, on the
current format with an `ess-inputs.yaml` toolchain pin, and CI runs every command they show and
compares the output they print.

## Acceptance

- Tutorial blocks are fenced with an `ess-tutorial` attribute; `cargo test -p ess-cli --test
  tutorial_page` writes them to a temporary directory, runs each command against the built binary,
  and compares every recorded output line. Changing one expected line makes the test fail.
- The TypeScript step runs where `node` is on PATH and ends `report: passed, <n> scenario(s)`.
- The tutorial uses the current `ess/` format and pins the toolchain with `ess specify toolchain
  install --pin`.
- `index.md` describes `site` as HTML.

## Scope

- crates/edge/ess-cli/tests/tutorial_page.rs (new)
- website/docs/getting-started.md, and the Start here pages Wave B creates
