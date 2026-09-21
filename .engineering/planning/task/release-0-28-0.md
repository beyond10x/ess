---
format: aep.planning-md/1
id: task:release-0-28-0
kind: task
status: implemented
title: Validate and publish observed conformance semantics in ESS 0.28.0
relations:
- verifies: story:guarded-external-outcomes
- verifies: story:observed-subject-history
- serves: vision:O2
revision: 7
---
## Goal
Integrate the guarded external outcome and observed subject history candidates, validate the release, and publish version 0.28.0 with verified archives.

## Acceptance

- Existing canonical projections and behavioral checks pass; new semantic and emitted runtime mutation witnesses pass.
- The shared security and privacy check passes on the exact integration commit.
- The tagged commit belongs to main, release jobs pass, and all four native archives and checksums are published.
- The operator-approved existing CI profile qualifies this bounded release; the default local consumer-accounting refusal remains visible and no accounting baseline is broadened.

## Scope
Cargo.toml; Cargo.lock; fuzz/Cargo.lock; CHANGELOG.md; changes; WHATS-CHANGED.md; website/docs/getting-started.md; website/docs/reference/spec-versions.md; crates/edge/ess-xtask/src/docs.rs; docs/conformance-core-checkpoint-2026-09-21.md. Integrates the two related stories without concurrent implementation.

## Authorization

The operator explicitly instructed merging and tagging PR #56 on 2026-09-21, after being told that every CI check passes, the full local consumer-accounting gate still refuses, and release under the existing CI profile awaits approval. This authorizes the existing CI/release profile for this bounded 0.28.0 release. Local qualification will use task check SKIP_CONSUMER_CHECKS=true plus task site-lab on the exact tag commit. The ordinary default task check retains consumer accounting; its refusal remains recorded and no accounting baseline or assertion is relaxed. Integration and annotated tagging use the organization bot, followed by verification of the exact tag, required checks, published release and four archives plus checksums.

## Completion

Implemented in PR #56 at 68581d70bb47a048cd399e55c68f225b80303977 and published as ESS 0.28.0. The exact source passed the operator-approved task check SKIP_CONSUMER_CHECKS=true profile, task site-lab, required shared and repository checks, and the release workflow. All four native archives and SHA256SUMS were downloaded and verified; the Linux binary reports ess 0.28.0. The annotated tag belongs to main and task release-status passes. The full default consumer-accounting refusal remains recorded; no accounting baseline was relaxed. See docs/conformance-core-checkpoint-2026-09-21.md and https://github.com/beyond10x/ess/actions/runs/35639713420.
