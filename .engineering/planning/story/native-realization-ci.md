---
format: aep.planning-md/1
id: story:native-realization-ci
kind: story
status: draft
title: Run structural realization compiler checks in CI
relations:
- serves: vision:O2
- informed_by: story:types-only-realizations
scope:
- confidence: cited
  path: .github/workflows/ci.yml
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/generate/schema-contract/Cargo.toml
revision: 2
---
## Evidence

The schema-contract manifest declares go_realization and typescript_typecheck as
required-features test targets (crates/generate/schema-contract/Cargo.toml:17).
The shared CI workflow runs only task check (.github/workflows/ci.yml:65), whose
workspace test command does not select those features (Taskfile.yml).
The test harnesses require ESS_GO_COMPILER pinned to Go 1.26.5 and
ESS_TYPESCRIPT_COMPILER pinned to TypeScript 6.0.3.

During 0.19.0 consolidation, the full optional lane passed with those explicit
compiler paths, including four Go and eight TypeScript tests, four native Rust
tests and two normalization Rust tests. An initial invocation without compiler
paths failed at setup instead of skipping, as intended. This is local release
evidence, not a CI coverage claim.

## Outcome

Every pull request and release candidate executes the existing external-compiler
checks under declared pinned toolchains, with explicit setup failures and no
silent skips. The offline workspace gate remains offline.

## Acceptance

A CI run on an exact candidate reports the existing Go and TypeScript compiler
test targets as executed with their pinned versions; a deliberate generated-code
regression fails that lane. Provisioning is separate from an explicit repository
task, and the release workflow reuses the same lane rather than a weaker copy.

## Scope

The shared CI workflow, repository task definitions and the existing explicit
schema-contract test-lane contract. No new generator behavior, compiler version
upgrade, broader dependency audit or adopter migration belongs to this story.
