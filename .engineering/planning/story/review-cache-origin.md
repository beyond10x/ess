---
format: aep.planning-md/1
id: story:review-cache-origin
kind: story
status: active
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
revision: 13
---
## Finding and source

F11 (P1) from `docs/reviews/2026-09-05-architecture-review.md:394`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/main.rs:1457`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A self-consistent cached release bundle or Helm chart substituted under another requested OCI manifest digest is rejected before its bytes are returned, written as the requested output, or passed to Helm. A valid proved hit works without a fetch client. Legacy entries lacking manifest proof cause cold acquisition; corrupt proved entries refuse.

This explicitly includes both cache consumers named by F11; it clarifies the original bundle-only wording rather than treating the chart finding as implicitly closed. The reviewed profile, publication and consumption binding is accepted for wave 12 below.

## Implementation boundary

Both the release-bundle fetch and Helm reconciliation cache must retain and revalidate the requested manifest-to-blob chain before content use. Preserve canonical bundle semantics, persisted-plan validation, dry-run/removal guards and sequential rollout order. Keep external fetch clients injectable. Do not silently upgrade legacy local consistency into registry identity. A concrete manifest/profile, bounded acquisition, cache publication and verified consumption binding precedes implementation.

## Validation

Fake registry/process fixtures cover correct manifest/layers, wrong manifest digest, altered layer, self-consistent replacement bundle and incomplete old cache; test cold and warm paths with no live registry.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

This establishes content binding, not publisher signature trust.

## Scope

Derived 2026-09-07 by story-scoper 0.8.0 from draft revision 10, its implemented prerequisite, the reviewed candidate binding and current integrated source — cited.

- **Owning package:** `crates/edge/ess-cli`; both release-bundle fetch and Helm reconciliation own acquisition, cache admission and external-process dispatch here — cited.
- **Contained implementation:** add the shared checked OCI manifest/blob reader, bounded acquisition, proof-entry publication and verified consumption within the owning package; exact helper and test filenames remain implementation choices — inferred.
- **Internal binding:** `docs/design/review-cache-origin.md`, currently absent; proposed home for the reviewed bundle/Helm profiles, original-byte verification, bounds, cache framing/publication and consumption contract after coordinator acceptance — inferred.
- **Public explanation:** `website/docs/concepts/component-delivery.md`; its digest-verified-cache diagram and complete-chain explanation directly describe this boundary and need the supported profiles, legacy/corrupt behavior and offline-hit limits — cited.
- **Acceptance:** both cache consumers must refuse a self-consistent replacement under another requested OCI manifest digest before returning bytes, writing requested output or invoking Helm; valid proved hits work without a fetch client, legacy entries cause cold acquisition, and corrupt proved entries refuse — cited.
- **Tests:** retain all existing persisted-delivery assertions, replace its placeholder cold-fetch fixture with an independently linked OCI graph, and add actual CLI cold/warm, substitution, profile, bound, interruption, concurrency and verified-snapshot cases inside the owning package — cited.
- **Preservation:** retain canonical bundle/model/plan bytes, checked desired/current-plan admission, dry-run/removal guards, sequential rollout behavior and injectable clients; cache content identity establishes neither signer trust nor applied-state evidence — cited.
- **Dependencies:** existing checked delivery types and strict Digest APIs are sufficient reusable dependencies; this assessment establishes no additional library, release-action, root manifest/lock, public-format or executor-recovery write reservation — inferred.
- **Current-source refresh:** coverage integration changed the CLI file but left the release command, release execution, deployment execution and cache/helper sections byte-identical to the source used by the prior binding review; existing delivery fixtures and the public concept page are also unchanged — cited.
- **Would collide with:** any write anywhere in the owning CLI package, to the proposed internal binding, or to the exact public concept page; broader or nested reservations require manual overlap checking because scope tokens are literal — cited.
- **Delivery:** publication and normal Website/Atlas delivery remain coordinator-owned; no external repository is reserved for this implementation unit — inferred.
- **Confidence:** high for source ownership and reservation sufficiency, because both consumers, their test fixtures and documentation owners are directly established by current source and the unchanged reviewed candidate contract — cited.
- **Pending selection:** the candidate binding remains unaccepted and this story remains draft; this refresh authorizes neither implementation nor a new wave — cited.

## Candidate binding review

The 2026-09-06 independent scope report is SHA256 e6cd5709142582cc406fe8583c23045c71b0fcb3b0ce411119fb0e68ad6da69a. Root verified all 42 scope input and 18 local evidence files. Candidate binding v1 is SHA256 719c29e9f36a67990133c74f33a71b372e4b6ed90c23dd41bfa640af39ef7985. Its independent document review found no binding issue; review-result:cache-origin-binding-pass1 preserves the report verbatim. Root verified all 48 review inputs, both historical story snapshots and all 18 prior local evidence files. The separately acknowledged obsolete Scope acceptance sentence is corrected here; the reviewed binding is unchanged.

The local ORAS evidence establishes an installed synthetic publisher shape, not cache correctness or actual Helm publisher bytes. No implementation or gate has run. Binding acceptance and fresh integrated source/scheduling remain pending; this story is not selected by wave 11.

## Coverage-integrated scope refresh

The independent scope report is retained at
`target/review-boundaries-12/preparation/scopers/cache-report.md`, SHA256
`73c689fd02b3b5530ece77f4dd427e6b8634a52472653dfbd3b24f163cbf23bd`. Root read back every inspected input hash before applying this section
(31 inputs). The report was captured from the original final message without
rewriting it. The machine reservations remain unchanged. This read-only refresh ran no source
tests or build and did not itself select an implementation.

## Accepted implementation binding — wave 12

Root accepts `docs/design/review-cache-origin.md` under the standing implementation approval
for the remediation epic. Original reviewed candidate SHA256 is
`719c29e9f36a67990133c74f33a71b372e4b6ed90c23dd41bfa640af39ef7985`; the recorded independent
document review has no findings, and the fresh source comparison leaves its semantics unchanged.
Only status/source references change when promoting that document. Earlier candidate/preparation
statements, including the scoper's pending-selection line, describe the state before this decision.

Select this story alone for wave 12 with the same three reservations. The implementation owns
both cache consumers and all bound refusal, process, publication and consumption controls.
No manifest, dependency, public wire format, signer trust, execution recovery or release change
is selected. The normal source attack, full gate and exact public delivery remain required.
