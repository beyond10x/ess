---
format: aep.planning-md/3
id: story:a-duration-member-is-refused-as-its-format-refuses
kind: story
status: draft
title: A Duration member is refused as its served format refuses it
summary: 'Rust reads Duration as any text though format: duration refuses it (W1 GEN1 review, pre-existing, backlog)'
tags:
- backlog
- review-finding
relations:
- informed_by: review-result:adversary-w1-gen1-pass-1
revision: 1
---
## Need

The generated Rust decoder reads `Duration` with `text_at` (`crates/generate/ess-synth/src/rust/wire.rs:1016`,
pre-existing) and admits any string, though the served `format: duration` refuses one. Inferred from the
code by review-result:adversary-w1-gen1-pass-1, not run.

## Acceptance

- A `Duration` member that the served `format: duration` refuses is answered 400 at decode, naming the
  path, by the generated Rust server; a test checks it against the served `openapi.json`.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.
