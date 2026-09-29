---
format: aep.planning-md/3
id: story:docs-hold-your-own-implementation
kind: story
status: active
title: The conformance guide shows how to hold your own implementation to the suite
relations:
- decomposes: epic:public-docs-overhaul
- serves: vision:O2
scope:
- confidence: cited
  path: website/docs/guides/verify-conformance.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T14:29:42Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-09-29T14:29:42Z", actor: "human:timo", revision: 5}
---
## Outcome

An adopter installs `ess` on macOS or Linux, or through an agent plugin, and holds their own
implementation to a generated suite with the Rust, Go or TypeScript runner, following one page per
path.

## Acceptance

- The install page has a macOS (`shasum`) and a Linux (`sha256sum`) verification line, `cargo
  install`, and the agent path (SETUP.md, `/ess:init`).
- One page per runner (Rust, Go, TypeScript) names the generated files and the methods an
  implementation provides exactly as `ess` 0.43.0 writes them; the TypeScript and Go pages are
  `ess-tutorial` blocks the tutorial test runs where the toolchain is present.

## Scope

- website/docs/ Start here install page and the runner pages Wave B creates
