---
format: aep.planning-md/3
id: approval-record:ess-0750-msrv-lock-pins
kind: approval-record
status: approved
title: Pin the ratatui path in the lock instead of raising rust-version
summary: 'D131: instability 0.3.10 and darling 0.20.11 in Cargo.lock; rust-version stays 1.85; the msrv job checks with the declared toolchain.'
tags:
- ess-0.75.0
- non-interactive
relations:
- decides: story:the-workspace-compiles-on-its-declared-rust-version
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:01:19Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Decision

Relayed by the orchestrator on 2026-10-28 (decision D131): "lock-only pins (instability 0.3.10,
darling 0.20.11, fnv 1.0.7), rust-version stays 1.85; raising it would break consumers". Approved
under the orchestrator's grant.

## Conditions

- The `msrv` CI job runs `cargo +<declared toolchain> check --workspace --all-targets --locked`,
  the toolchain read from `rust-version` in `Cargo.toml` (1.85.0 today).
- The CHANGELOG says that a consumer on rustc 1.85 that resolves its own lock may need the same
  two pins, because `Cargo.lock` does not travel with the crates.

## Why

instability 0.3.13 and darling 0.24.1 declare rust-version 1.88. Both reach the workspace only
through ratatui (ess-ui-tui, ess-ui-test, ess-cli). With the pins,
`cargo +1.85.0 check --workspace --all-targets --locked` exits 0.
