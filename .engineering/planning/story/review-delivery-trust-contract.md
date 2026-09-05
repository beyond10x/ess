---
format: aep.planning-md/1
id: story:review-delivery-trust-contract
kind: story
status: draft
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
  path: .github/actions/release-component
- confidence: inferred
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/generate/ess-deployment
- confidence: inferred
  path: docs/design/review-delivery-trust.md
- confidence: inferred
  path: website/docs
revision: 6
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

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/generate/ess-deployment` — cited; owning implementation or documented surface.
- `crates/edge/ess-cli` — inferred; planned edit surface, verify before dispatch.
- `.github/actions/release-component` — cited; owning implementation or documented surface.
- `docs/design/review-delivery-trust.md` — inferred; planned edit surface, verify before dispatch.
- `website/docs` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

