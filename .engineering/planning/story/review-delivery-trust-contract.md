---
format: aep.planning-md/1
id: story:review-delivery-trust-contract
kind: story
status: implemented
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
- confidence: cited
  path: crates/edge/ess-cli/src/release_evidence.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/command_surface.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/delivery_trust.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/fake_release_component.rs
- confidence: cited
  path: crates/generate/ess-deployment/src/component.rs
- confidence: cited
  path: crates/generate/ess-deployment/src/release.rs
- confidence: cited
  path: crates/generate/ess-deployment/tests/deployment.rs
- confidence: cited
  path: docs/design/review-delivery-trust.md
- confidence: cited
  path: website/docs/concepts/component-delivery.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 16
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

## Scope before implementation (retained history)

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

## Scope

The following confirmation is copied from the implementor’s scope table and checked against the complete correction and final source-review readback. Original inferred marks remain visible above; the source establishes these owners.

- crates/generate/ess-deployment/src/release.rs — cited; originally cited. EvidenceKind, Evidence and verify_release own the declared four-kind map and consistency checks. Documentation corrected; wire and validation untouched.
- crates/generate/ess-deployment/src/component.rs — cited; originally cited. ReleaseBundle and verify_release_bundle own nested admission and canonical bytes. Documentation corrected; API and wire untouched.
- crates/generate/ess-deployment/tests/deployment.rs — cited; originally cited. Existing 21 runner cases cover nested mutable IR, duplicate maps, rehashing and canonical readers. Read and executed unchanged; no new library behavior required.
- crates/edge/ess-cli/src/main.rs — cited; originally cited. ReleaseCommand and release dispatch own five existing routes, canonical publisher staging, and TemporaryDirectory. Added two routes, optional qualified publish, conservative diagnostics, private staging and the unit’s explicit 54-leaf inventory. Integration with the separately published observed-bindings command makes the final count 55; all existing browser help and namespace imports are preserved.
- crates/edge/ess-cli/src/release_evidence.rs — cited; originally inferred. Absent on base; appropriate private edge owner beside coverage.rs and oci_cache.rs. Now owns options, original buffers, existing admission/model/context calls and diagnostics. The correction also requires each component runtime/chart unit to own the existing OCI/Helm build-output kind, while allowing valid mixed and additional outputs. No new cross-crate/persisted model.
- crates/edge/ess-cli/tests/delivery_trust.rs — cited; originally inferred. Absent on base; actual CLI/action boundary cases need this owner. Reuses the read-only bundle_fixture and compiled real ESS; invokes real Bash/jq.
- crates/edge/ess-cli/tests/support/fake_release_component.rs — cited; originally inferred. Absent on base; existing cache/delivery suites establish the Rust process-fixture pattern. Only finite external docker/helm/cosign/syft/ORAS routes are simulated here; unknown operations refuse.
- crates/edge/ess-cli/tests/command_surface.rs — cited; originally cited. Existing observable help/alias suite. Added seven-route help and per-route alias-option equality.
- .github/actions/release-component/action.yml — cited; originally cited. Existing composite input/env contract. Required model/report and XOR expectation inputs added; generic check meaning and conservative description corrected.
- .github/actions/release-component/release.sh — cited; originally cited. Existing real orchestration and four-kind map. Added distinct byte snapshots/pins and all qualification boundaries; reliable child failure propagation; exact build/adopt/chart/summary routes retained.
- docs/design/review-delivery-trust.md — cited; originally inferred. Exists at the accepted binding hash in the baseline source receipt. Its B01–B07/T01–T18 are the implementation contract; only historical/implemented-interface annotations updated.
- website/docs/concepts/component-delivery.md — cited; originally cited. Existing public delivery guide. Added trust limits, exact-selection semantics, breaking input migration and both old/new compatibility directions.
- website/docs/reference/cli.md — cited; originally cited. Existing public CLI surface. Added seven routes, option groups, streams/status, pins and migration link.
- website/docs/reference/formats.md — cited; originally cited. Existing public wire/digest catalog. Clarified unchanged /1 wire, attachment manifest vs raw report hashes and absence of a new envelope.

- Confidence: high — implementor confirmation, complete correction readback and final source review verified by root.
- Shared coordinator records remain outside the unit: planning journal, lifecycle evidence, changelog and wave page.

## Completed source implementation and verification

The implementation is committed as 70c2321d84a499f1b7b5a6ffa34e5db073a2d7a7: 13 files, 2,963 insertions and 50 deletions. Root recorded the review history in 85e334f43e1290ec43d9f91fcacbf3d9619c09f1 and integrated published observed-binding changes bbbe0de65e01ad7dc22fd329bb5f73d70e648d1d into exact combined commit 922dd160263acf3bea5f76e2c97b6eddce1553c6. The sole semantic merge adjustment is the command-leaf count 52+2+1=55. All new delivery evidence code and reviewed test bytes are unchanged by that merge; exact reference-page and CLI additions are recorded separately in the wave evidence.

Review-result:delivery-trust-source-pass1 established one introduced missing component/build-unit check. The same implementor corrected it and added required-kind, missing-chart and mixed-output cases; the affected full package run executed 323 passing tests with strict Clippy and formatting zero. The fixed outcome was recorded after root verified the correction. Review-result:delivery-trust-source-pass2 established no new finding: all 38 delivery and 21 deployment cases passed, with strict Clippy and both formatting checks zero. The complete handed test prefix remains unchanged. Its initial comment-only semantic fixture error and own-test lint refusal are preserved and diagnosed as fixture issues. The AEP comparison contains zero carried, zero new and one resolved finding; no third source attack ran.

The whole gate ran once on clean combined commit 922dd160263acf3bea5f76e2c97b6eddce1553c6. All eleven direct exits were zero: site-build, formatting, strict Clippy, complete workspace tests, rustdoc, command examples, projection consistency, public support checks, release metadata checks, action checks and planning validation. The actual test lane executed 2,205 passing tests in 197 summaries, with no failures or ignored tests. The browser lab separately exercised its 21 claims and 28 deterministic steps. Existing fixture diagnostics and unchanged dependency advisories remain in the raw output; this is not a claim of production execution or security attestation.

Exact gate records are retained under target/review-boundaries-16/gate-922dd160263a-attempt1, including each lane’s direct child status, raw output, all 1,182 tracked source inputs (40,155,926 bytes), frozen tool checks and actual Rust/Go coverage-producer identities and retained binaries. The complete receipt SHA256 is 45b21ac75c512feaca87704b9b47dfebe8df6fc2844921c912f19f76e418c6e4; the results table SHA256 is 61c46c6ab95911798842ca71a54e3ea526a1ab3b1cbf9da258a5685da90ce716. Root’s independent readback is preparation/integration-gate-readback/readback.json, SHA256 50386573650b205eac3d0990e4f4bf635fa08ee1db104a9c1c0973305b79dc47. All recorded lane and coverage processes had exited before completion was recorded.

## Publication and operational follow-through

The verified source is ready for incremental main publication under standing approval. Its actual remote observation, required Website source-lock/snapshot and production delivery, and exact managed cleanup are recorded in docs/plan/2026-09-07-review-boundaries-16.md as they occur. Website task:publish-ess-delivery-trust owns that separate documentation operation. This source completion does not assert that those later actions already ran, and it authorizes no release tag, package version bump or live ESS deployment.
