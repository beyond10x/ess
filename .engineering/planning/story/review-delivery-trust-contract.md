---
format: aep.planning-md/1
id: story:review-delivery-trust-contract
kind: story
status: active
title: Distinguish release consistency from verified evidence
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-persisted-delivery-validation
- depends_on: story:review-report-reader-validation
scope:
- confidence: cited
  path: .github/actions/release-component/action.yml
- confidence: cited
  path: .github/actions/release-component/release.sh
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/release_evidence.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/command_surface.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/delivery_trust.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/support/fake_release_component.rs
- confidence: cited
  path: crates/generate/ess-deployment/src/component.rs
- confidence: cited
  path: crates/generate/ess-deployment/src/release.rs
- confidence: cited
  path: crates/generate/ess-deployment/tests/deployment.rs
- confidence: inferred
  path: docs/design/review-delivery-trust.md
- confidence: cited
  path: website/docs/concepts/component-delivery.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 13
---
## Finding and source

F11 (P1) from `docs/reviews/2026-09-05-architecture-review.md:394`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/generate/ess-deployment/src/release.rs:137`, `.github/actions/release-component/release.sh:46`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Release output and documentation expose the actual evidence verification level so metadata consistency or an arbitrary check log cannot be presented as verified conformance or attestation.

## Implementation boundary

Correct present terminology and define the policy for the evidence kinds already accepted. Validate typed conformance reports where conformance is claimed; classify generic check logs as generic checks. Keep signature verification explicitly unsupported unless an owned verifier/policy is separately designed. Bind claims to artifact/model identity and preserve a conservative unverified result when origin cannot be established.

## Validation

Use matching-digest but unverified/tampered/wrong-suite fixtures and arbitrary logs; prove they cannot reach an evidence-verified status. Document exact supported guarantees and migration of evidence-kind meaning with existing release-action consumers.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No external signer, trust-root invention or production certification. Execution recovery and cache binding have separate owners; new evidence envelopes require a binding design and Atlas coordination.

## Implementation binding and retained history

The original F11 finding, acceptance, implementation boundary, validation and exclusions above are preserved. This body draft keeps story:review-delivery-trust-contract in draft and unselected; it introduces no new story, dependency, model or envelope. Root accepted B01–B07 for future implementation after review-result:delivery-trust-binding-pass1 reported no concrete issue (no executed cases). The full publication draft is intended for docs/design/review-delivery-trust.md; work-order-draft.md retains every T01–T18 variant. The original candidate and complete original story remain separately retained with exact hashes.

Source refresh is complete from d7f12b7a027018fc020fadf8f1d66ca6caac1e68 to 3408bbf049d10215487c50f6f7b5597486b14127. Only browser help changed in main.rs, shifting delivery citations by one line; delivery APIs, command-surface tests and the three reserved public pages are unchanged. The separate recovery publication is not delivery implementation. Root completes browser closure, then runs fresh AEP graph/waves/readiness and selects any next unit; this draft does none of those actions. Future CLI/action interfaces remain unimplemented.

## Scope

- Derived 2026-09-07 against 3408bbf049d10215487c50f6f7b5597486b14127 — cited; complete integrated-source comparison and retained binding direction.
- crates/generate/ess-deployment/src/release.rs — cited; exact unchanged reservation in work-order-draft.md.
- crates/generate/ess-deployment/src/component.rs — cited; exact unchanged reservation in work-order-draft.md.
- crates/generate/ess-deployment/tests/deployment.rs — cited; exact unchanged reservation in work-order-draft.md.
- crates/edge/ess-cli/src/main.rs — cited; exact unchanged reservation in work-order-draft.md.
- crates/edge/ess-cli/src/release_evidence.rs — inferred; exact unchanged reservation in work-order-draft.md.
- crates/edge/ess-cli/tests/delivery_trust.rs — inferred; exact unchanged reservation in work-order-draft.md.
- crates/edge/ess-cli/tests/support/fake_release_component.rs — inferred; exact unchanged reservation in work-order-draft.md.
- crates/edge/ess-cli/tests/command_surface.rs — cited; exact unchanged reservation in work-order-draft.md.
- .github/actions/release-component/action.yml — cited; exact unchanged reservation in work-order-draft.md.
- .github/actions/release-component/release.sh — cited; exact unchanged reservation in work-order-draft.md.
- docs/design/review-delivery-trust.md — inferred; exact unchanged reservation in work-order-draft.md.
- website/docs/concepts/component-delivery.md — cited; exact unchanged reservation in work-order-draft.md.
- website/docs/reference/cli.md — cited; exact unchanged reservation in work-order-draft.md.
- website/docs/reference/formats.md — cited; exact unchanged reservation in work-order-draft.md.
- Confidence: high — cited; integrated release APIs and the 14 owners are established by exact source comparison; planned new files retain inferred marks.
- Would collide with: CLI main/help and command_surface, release action, deployment release/component owners and the three public pages — inferred.
- Read-only dependencies: admitted report/suite/coverage readers, model identity, OCI cache, existing fixture owners and Taskfile — cited.
- Coordinator-only records: lifecycle, journal, scope replacement, post-browser graph/waves/readiness, selection, integration gates and publication — inferred.
- New story/dependency: none — cited; only the original implemented persisted-delivery and report-reader dependencies remain.


## Published browser source and binding adoption

Browser replay is implemented and published at 95ef5be70dbce966056d0484b66db7ed836fa406, whose runtime, tests, lockfiles and public-site bytes equal reviewed integration 3408bbf049d10215487c50f6f7b5597486b14127. Root read the complete refreshed binding/work order/story and independently verified all 74 inputs (1,825,961 bytes) and eight outputs. Root adopts the unchanged B01–B07 and T01–T18 contract at docs/design/review-delivery-trust.md. The earlier draft statements describe their historical stage. The original prerequisite stories remain implemented and no blocker is open. Exact scope replaces all five earlier broad tokens with fourteen reservations; implementation selection and its concrete unit are recorded separately in the wave page. No runtime result is claimed by this adoption.


## Wave 16 implementation selection

Root selects this story alone under the retained standing implementation/publication approval. The accepted binding is docs/design/review-delivery-trust.md, and all fourteen exact reservations plus complete B01–B07/T01–T18 apply. Both prerequisites are implemented, no blocker is open and the current computed draft-wave output is retained in docs/reviews/2026-09-07-review-boundaries-16-opening-waves.md. See docs/plan/2026-09-07-review-boundaries-16.md for exact managed-unit/resource assignment and required gates. Earlier draft statements remain historical; only this selected unit moves proposed then active. Runtime completion remains dependent on actual integrated evidence.
