---
format: aep.planning-md/3
id: story:cli-command-path-deeper-than-two-tokens
kind: story
status: draft
title: A CLI binding declares a command path deeper than two tokens
tags:
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A CLI presentation binding declares a command path of more than two tokens
(`plan store migrate git`), and the generated CLI parses it.

## Evidence

An adopter retrofitting its own CLI (four-level verbs) met on ess 0.56.0:
`error: invalid or conflicting command path `plan store migrate git``. `resolve.rs` in
`crates/specify/ess-cli-contract/src/` refuses any path whose length is outside `1..=2`.

## Acceptance

- A path of up to a stated maximum depth (decide it; at least 4) validates; a path that is a
  prefix of another declared path is still refused as conflicting, naming both.
- `ess generate cli` emits a clap derive tree for the deeper path, and the parse tests cover it.
- Decide whether the change needs a new `ess-cli/N` format (a binding that `ess-cli/2` refuses
  becomes valid); old readers refuse a deeper path by name.
