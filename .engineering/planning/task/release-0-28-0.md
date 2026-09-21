---
format: aep.planning-md/1
id: task:release-0-28-0
kind: task
status: active
title: Validate and publish observed conformance semantics in ESS 0.28.0
relations:
- verifies: story:guarded-external-outcomes
- verifies: story:observed-subject-history
- serves: vision:O2
revision: 3
---
## Goal
Integrate the guarded external outcome and observed subject history candidates, validate the release, and publish version 0.28.0 with verified archives.

## Acceptance
- Existing canonical projections and behavioral checks pass; new semantic and emitted runtime mutation witnesses pass.
- The shared security and privacy check passes on the exact integration commit.
- The tagged commit belongs to main, release jobs pass, and all four native archives and checksums are published.
- Required local consumer-accounting refusal remains visible. Using the existing CI profile for local release qualification requires the pending operator decision; no accounting baseline is broadened.

## Scope
Cargo.toml; Cargo.lock; fuzz/Cargo.lock; CHANGELOG.md; changes; WHATS-CHANGED.md; website/docs/getting-started.md; website/docs/reference/spec-versions.md; crates/edge/ess-xtask/src/docs.rs; docs/conformance-core-checkpoint-2026-09-21.md. Integrates the two related stories without concurrent implementation.

## Authorization
The operator requested integration into main and passing checks on 2026-09-21. Release preparation is necessary for the adopter's published-binary pin. A separate decision is pending on the inherited local consumer-accounting refusal versus the already configured CI/release profile.
