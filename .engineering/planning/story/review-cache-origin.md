---
format: aep.planning-md/1
id: story:review-cache-origin
kind: story
status: draft
title: Verify cached bundle bytes against their OCI identity
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-persisted-delivery-validation
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: inferred
  path: docs/design/review-cache-origin.md
- confidence: cited
  path: website/docs/concepts/component-delivery.md
revision: 10
---
## Finding and source

F11 (P1) from `docs/reviews/2026-09-05-architecture-review.md:394`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/main.rs:1457`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A self-consistent cached release bundle or Helm chart substituted under another requested OCI manifest digest is rejected before its bytes are returned, written as the requested output, or passed to Helm. A valid proved hit works without a fetch client. Legacy entries lacking manifest proof cause cold acquisition; corrupt proved entries refuse.

This explicitly includes both cache consumers named by F11; it clarifies the original bundle-only wording rather than treating the chart finding as implicitly closed. The concrete candidate profile, publication and consumption binding remains unaccepted and unselected.

## Implementation boundary

Both the release-bundle fetch and Helm reconciliation cache must retain and revalidate the requested manifest-to-blob chain before content use. Preserve canonical bundle semantics, persisted-plan validation, dry-run/removal guards and sequential rollout order. Keep external fetch clients injectable. Do not silently upgrade legacy local consistency into registry identity. A concrete manifest/profile, bounded acquisition, cache publication and verified consumption binding precedes implementation.

## Validation

Fake registry/process fixtures cover correct manifest/layers, wrong manifest digest, altered layer, self-consistent replacement bundle and incomplete old cache; test cold and warm paths with no live registry.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

This establishes content binding, not publisher signature trust.

## Scope

Derived 2026-09-06 by story-scoper from draft revision 2, F11, implemented persisted-delivery validation and the unchanged coordinator source — cited.

- **Owning package:** crates/edge/ess-cli; release fetch, canonical bundle admission, Helm chart fetch and external-client dispatch all live here. Preserve both cache consumers' existing byte-local semantic checks while adding requested-manifest-to-payload verification — cited.
- **Contained implementation and tests:** keep the shared checked OCI reader/cache helper, new cold/warm substitution regressions and adaptations of the existing persisted-delivery fake-process fixtures inside the owning package; exact new helper/test filenames remain implementation choices — inferred.
- **Internal binding:** docs/design/review-cache-origin.md, currently absent; bind the bundle/Helm manifest profiles, raw-byte and descriptor checks, legacy behavior, bounded acquisition, cache publication/concurrency and the distinction between content identity and signer/execution trust before code — inferred.
- **Public explanation:** website/docs/concepts/component-delivery.md; its existing digest-verified-cache diagram and complete-chain paragraph directly describe this boundary. Explain both supported cache paths, profile limits and legacy/offline behavior there — cited.
- **Acceptance boundary:** F11 explicitly identifies both bundle and chart cache trust. The coordinator clarified Acceptance to include both consumers at revision 3; the earlier bundle-only wording is historical. The concrete binding remains unaccepted and unselected — cited.
- **Preservation:** retain canonical ESS bundle/model/plan bytes, validated desired/current-plan admission before every executor, dry-run/no-removal guards and existing rollout behavior. Local cache proof is neither signature verification nor applied-state evidence — cited.
- **Dependencies:** the checked delivery models and Digest API are usable read-only dependencies; no deployment-library, release-action, root manifest/lock or executor-recovery implementation reservation is established by this assessment — inferred.
- **Compatibility:** no ESS release-bundle or deployment envelope change is required. New private cache layout/proof interpretation must have an explicit compatibility decision; old unproved entries cannot become verified hits merely by adding a checksum — inferred.
- **Would collide with:** every edit within the CLI package, the proposed internal binding, or the exact public concept page. Literal nonmatching scope tokens do not prove disjointness from broader directory reservations. Refresh the final integrated CLI and docs after the active coverage writer closes, before later selection — cited.
- **Delivery:** the public concept-page change requires normal source publication and Website/Atlas delivery under coordinator ownership; no external repository is an implementation reservation here — inferred.
- **Confidence:** medium, because both source owners, the existing test adaptation and actual installed ORAS bundle shape are established, while Helm profile acceptance, legacy handling and atomicity remain unaccepted choices — inferred.

## Candidate binding review

The 2026-09-06 independent scope report is SHA256 e6cd5709142582cc406fe8583c23045c71b0fcb3b0ce411119fb0e68ad6da69a. Root verified all 42 scope input and 18 local evidence files. Candidate binding v1 is SHA256 719c29e9f36a67990133c74f33a71b372e4b6ed90c23dd41bfa640af39ef7985. Its independent document review found no binding issue; review-result:cache-origin-binding-pass1 preserves the report verbatim. Root verified all 48 review inputs, both historical story snapshots and all 18 prior local evidence files. The separately acknowledged obsolete Scope acceptance sentence is corrected here; the reviewed binding is unchanged.

The local ORAS evidence establishes an installed synthetic publisher shape, not cache correctness or actual Helm publisher bytes. No implementation or gate has run. Binding acceptance and fresh integrated source/scheduling remain pending; this story is not selected by wave 11.
