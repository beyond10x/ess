---
format: aep.planning-md/3
id: story:rust-recorder-does-not-lose-a-creation
kind: story
status: draft
title: The Rust recorder does not lose a creation it observed
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/sessions.rs
revision: 2
---
# Story: the Rust recorder does not lose a creation it observed

## Outcome

The Rust concurrent-history recorder records a creation the target performed even when the read that would observe it races, as the Go and TypeScript explorers now do.

## Why

Handoff from the closed concurrent-history session (~/.cache/ess-chc/HANDOFF-to-ess.md, item 4): the reads unit (`story:explorers-record-view-reads`, wave 5) fixed a lost-creation exposure in the Go and TypeScript explorers; its implementor reported the same exposure in `crates/verify/ess-conformance/src/sessions.rs`. Not reproduced by the coordinator — inferred from that report.

## Acceptance

- A red-first test shows the Rust recorder losing a creation under the interleaving the reads unit fixed for Go/TS, then green.
- The Go/TS behaviour and the Rust behaviour agree on the same history.

## Scope

- `crates/verify/ess-conformance/src/sessions.rs` — cited (handoff)
