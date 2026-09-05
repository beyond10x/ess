---
format: aep.planning-md/1
id: story:review-persisted-delivery-validation
kind: story
status: draft
title: Restore delivery IR invariants at persisted read boundaries
tags:
- P0
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/generate/ess-deployment
revision: 3
---
## Finding and source

F02 (P0) from `docs/reviews/2026-09-05-architecture-review.md:188`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/generate/ess-deployment/src/build.rs:267`, `crates/generate/ess-deployment/src/runtime.rs:230`, `crates/generate/ess-deployment/src/component.rs:70`, `crates/generate/ess-deployment/src/environment.rs:130`, `crates/edge/ess-cli/src/main.rs:1613`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Malformed persisted delivery documents are refused before analysis or external execution while valid compiler-produced documents retain their canonical bytes.

## Implementation boundary

Inventory BuildIr, RuntimeIr, component descriptors, stack locks, deployment plans and nested bundle models. Validate format discriminators, map-key identities, references, cycles, order membership/completeness and domain constraints at DTO-to-validated conversion. Test supported deserialization routes, not only from_json. Route the CLI through the validated entrypoint; prevalidate the complete plan before the first executor call.

## Validation

Mutate each inventoried envelope and nested model, including future/99 plus missing order entries, duplicate/order/cycle/reference defects and a consistently rehashed invalid bundle. Use an injected fake executor to prove zero calls on rejection; valid fixtures round-trip byte-for-byte. Package gates: ess-deployment and ess-cli.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Release evidence authenticity, cache origin and recovery belong to F11 stories; no live Docker/Helm/registry operations are required.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/generate/ess-deployment` — cited; owning implementation or documented surface.
- `crates/edge/ess-cli` — cited; owning implementation or documented surface.
- Confidence: high — cited; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

