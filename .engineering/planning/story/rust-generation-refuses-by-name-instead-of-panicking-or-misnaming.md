---
format: aep.planning-md/3
id: story:rust-generation-refuses-by-name-instead-of-panicking-or-misnaming
kind: story
status: draft
title: Rust generation refuses by name instead of panicking or allocating a bad identifier
summary: A synthesis panic, `self` becoming `Self`, and colliding view method names (workbench report items 28, 29, 32)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

- A panic: `thread 'main' panicked at crates/generate/ess-synth/src/rust/behaviour.rs:2850:28`, "the
  plan admits `{subject:}` only where a row is held" (item 28, example `28` and the whole model).
- `gap.demo.FinishItem.self: outcome variant allocates invalid Rust identifier \`Self\`` (item 29,
  example `29`).
- `gap.one.Things, gap.two.Things: \`things\` is allocated 2 times in \`port methods:gap-service\`
  (view method, view method)`. Same shape as #415 (closed, commands), now for view methods across
  domains (item 32, example `32`).

Items 28, 29 and 32 of the 2026-11-09 workbench report.

## Acceptance

- The panic is a named refusal, or the construct generates.
- An outcome named `self` (or any name whose identifier is a Rust keyword) is escaped or refused at
  `ess validate`, never at generation.
- Two views of one name in different domains get distinct method names without `naming:`, as
  commands do since #415.

## Scope

- inferred: `crates/generate/ess-synth/src/rust/behaviour.rs`, the Rust identifier allocator.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
