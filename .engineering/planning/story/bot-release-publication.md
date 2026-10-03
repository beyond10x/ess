---
format: aep.planning-md/3
id: story:bot-release-publication
kind: story
status: active
title: Prepare verified ESS release assets for organization-bot publication
relations:
- serves: vision:O2
- derived_from: task:release-0-52-0-20261003
scope:
- confidence: cited
  path: .github/workflows/release-record.yml
- confidence: cited
  path: .github/workflows/release.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: crates/edge/ess-xtask/src/main.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T07:24:17Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-03T07:24:17Z", actor: "human:timo", revision: 4}
---
## Operator decision

On 2026-10-03 the operator selected: Preserve b10x-bot; prepare compliant publishing. Existing CI publishes through github.token as github-actions[bot], contrary to workspace policy. No App credential may be added to this public repository or its candidate-executing CI. Source release and dependency repair are authorized; downstream pins and documentation deployment are excluded.

## Required behavior

Keep the exact-tag gate, WASM/browser checks, native packaging, four target names, checksums, release notes and Linux archive smoke checks. Credential-free CI prepares and retains an exact-tag/exact-commit release artifact only after all those checks pass, with read-only GitHub permissions. Publication then uses the existing protected organization-bot route from the trusted operator environment and verifies the exact tag/run/assets before making a draft public. Ordinary personal gh writes are prohibited. No release is reported complete from workflow preparation alone.

The implementation should minimize executable changes: use existing pinned artifact actions and existing Rust release tooling. New committed executable code, if needed, must be Rust. Adjust the release record workflow and contract tests so an awaiting-publication artifact is not mistaken for a completed release, while failures remain visible. Document the bot publication procedure and exact completion boundary in repository instructions.

## Acceptance

Workflow contract tests fail on current github.token publication and pass on a read-only preparation workflow. Existing tag resolution, green Gate reuse, native prebuild selection and checksum/smoke guarantees continue to pass. Focused xtask tests, formatting, strict lint and ci-lint pass. Final release candidate additionally passes task check and task site-build including site-lab. A real bot-authored Release containing the four verified archives and SHA256SUMS is required to finish the parent release task.

## Scope

Cited: .github/workflows/release.yml, .github/workflows/release-record.yml, crates/edge/ess-xtask/tests/ci_lanes.rs and crates/edge/ess-xtask/src/main.rs hold the release workflow/contracts. Cited: AGENTS.md describes release preparation/publication. No product/runtime/transport implementation belongs to this unit.
